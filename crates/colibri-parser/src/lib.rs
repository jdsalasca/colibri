//! Restricted parser for `.colibri` single-file components.
//!
//! It extracts `script`, `template` and `style` blocks and builds a template
//! AST with byte spans. It never evaluates Rust, renders HTML or treats input
//! as trusted. HTML, Rust and CSS support is intentionally partial.

use std::error::Error;
use std::fmt;

pub use colibri_ir::SourceSpan;

pub mod lower;
pub use lower::{lower, LoweringError};

/// Refuse inputs larger than this before doing any work.
pub const MAX_SOURCE_BYTES: usize = 1024 * 1024;
/// Cap nesting so hostile input cannot exhaust the stack.
pub const MAX_TEMPLATE_DEPTH: usize = 128;

const VOID_ELEMENTS: &[&str] = &[
    "area", "base", "br", "col", "embed", "hr", "img", "input", "link", "meta", "param", "source",
    "track", "wbr",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    SourceTooLarge,
    InvalidTopLevel,
    UnsupportedBlock,
    MissingClosingTag,
    UnexpectedClosingTag,
    DuplicateBlock,
    MissingTemplate,
    InvalidScriptLanguage,
    InvalidStyleLanguage,
    InvalidTagName,
    InvalidAttributeName,
    InvalidAttributeValue,
    DuplicateAttribute,
    UnsafeEventHandler,
    UnclosedComment,
    UnclosedTag,
    UnclosedAttribute,
    UnclosedExpression,
    UnclosedLiteral,
    DepthLimit,
}

impl ErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SourceTooLarge => "source-too-large",
            Self::InvalidTopLevel => "invalid-top-level",
            Self::UnsupportedBlock => "unsupported-block",
            Self::MissingClosingTag => "missing-closing-tag",
            Self::UnexpectedClosingTag => "unexpected-closing-tag",
            Self::DuplicateBlock => "duplicate-block",
            Self::MissingTemplate => "missing-template",
            Self::InvalidScriptLanguage => "invalid-script-language",
            Self::InvalidStyleLanguage => "invalid-style-language",
            Self::InvalidTagName => "invalid-tag-name",
            Self::InvalidAttributeName => "invalid-attribute-name",
            Self::InvalidAttributeValue => "invalid-attribute-value",
            Self::DuplicateAttribute => "duplicate-attribute",
            Self::UnsafeEventHandler => "unsafe-event-handler",
            Self::UnclosedComment => "unclosed-comment",
            Self::UnclosedTag => "unclosed-tag",
            Self::UnclosedAttribute => "unclosed-attribute",
            Self::UnclosedExpression => "unclosed-expression",
            Self::UnclosedLiteral => "unclosed-literal",
            Self::DepthLimit => "depth-limit",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParseError {
    pub code: ErrorCode,
    pub message: String,
    pub span: SourceSpan,
    pub line: usize,
    pub column: usize,
    pub hint: Option<&'static str>,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}:{}: [{}] {}",
            self.span.file,
            self.line,
            self.column,
            self.code.as_str(),
            self.message
        )?;
        if let Some(hint) = self.hint {
            write!(f, " (hint: {hint})")?;
        }
        Ok(())
    }
}

impl Error for ParseError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScriptBlock {
    pub content: String,
    pub lang: Option<String>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StyleBlock {
    pub content: String,
    pub lang: Option<String>,
    pub scoped: bool,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SingleFileComponent {
    pub script: Option<ScriptBlock>,
    pub template: Template,
    pub style: Option<StyleBlock>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Template {
    pub nodes: Vec<TemplateNode>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TemplateNode {
    Element(TemplateElement),
    Text(TemplateText),
    Interpolation(TemplateInterpolation),
    Comment(TemplateComment),
}

impl TemplateNode {
    pub fn span(&self) -> &SourceSpan {
        match self {
            Self::Element(node) => &node.span,
            Self::Text(node) => &node.span,
            Self::Interpolation(node) => &node.span,
            Self::Comment(node) => &node.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateElement {
    pub tag: String,
    pub attributes: Vec<TemplateAttribute>,
    pub children: Vec<TemplateNode>,
    pub span: SourceSpan,
    pub self_closing: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateText {
    pub value: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateInterpolation {
    pub expression: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateComment {
    pub value: String,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TemplateAttribute {
    pub name: String,
    pub value: AttributeValue,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AttributeValue {
    Boolean,
    Literal(String),
    Expression(String),
}

/// Parse a `.colibri` file. `file` is only used for diagnostics and spans.
pub fn parse(file: &str, source: &str) -> Result<SingleFileComponent, ParseError> {
    let parser = Parser {
        file,
        source,
        bytes: source.as_bytes(),
    };

    if source.len() > MAX_SOURCE_BYTES {
        return Err(parser.error(
            ErrorCode::SourceTooLarge,
            format!("source exceeds the {MAX_SOURCE_BYTES}-byte limit"),
            0,
            source.len(),
            None,
        ));
    }

    let mut cursor = 0usize;
    let mut script = None;
    let mut template = None;
    let mut style = None;

    while cursor < source.len() {
        cursor = parser.skip_ws(cursor);
        if cursor >= source.len() {
            break;
        }
        if parser.starts_with(cursor, b"<!--") {
            let end = parser.find(cursor + 4, b"-->").ok_or_else(|| {
                parser.error(
                    ErrorCode::UnclosedComment,
                    "unclosed HTML comment",
                    cursor,
                    source.len(),
                    None,
                )
            })?;
            cursor = end + 3;
            continue;
        }
        if parser.bytes[cursor] != b'<'
            || parser.starts_with(cursor, b"</")
            || parser.starts_with(cursor, b"<!")
        {
            return Err(parser.error(
                ErrorCode::InvalidTopLevel,
                "only script, template and style blocks are allowed at the top level",
                cursor,
                (cursor + 1).min(source.len()),
                None,
            ));
        }

        let block_start = cursor;
        let (open, after_open) = parser.parse_tag_open(cursor)?;
        match open.name.to_ascii_lowercase().as_str() {
            "script" => {
                if script.is_some() {
                    return Err(parser.error(
                        ErrorCode::DuplicateBlock,
                        "duplicate <script> block",
                        block_start,
                        after_open,
                        None,
                    ));
                }
                let lang = literal_attribute(&open.attributes, "lang");
                if lang.as_deref() != Some("rust") {
                    return Err(parser.error(
                        ErrorCode::InvalidScriptLanguage,
                        "script must declare lang=\"rust\"",
                        block_start,
                        after_open,
                        Some("only Rust is supported in the initial phase"),
                    ));
                }
                let close = parser.find_script_close(after_open)?;
                let after_close = parser.parse_close_tag(close, "script")?;
                script = Some(ScriptBlock {
                    content: source[after_open..close].to_owned(),
                    lang,
                    span: parser.span(block_start, after_close),
                });
                cursor = after_close;
            }
            "style" => {
                if style.is_some() {
                    return Err(parser.error(
                        ErrorCode::DuplicateBlock,
                        "duplicate <style> block",
                        block_start,
                        after_open,
                        None,
                    ));
                }
                let lang = literal_attribute(&open.attributes, "lang");
                if !matches!(lang.as_deref(), None | Some("css") | Some("scss")) {
                    return Err(parser.error(
                        ErrorCode::InvalidStyleLanguage,
                        "style language must be css or scss",
                        block_start,
                        after_open,
                        None,
                    ));
                }
                let close = parser.find_style_close(after_open)?;
                let after_close = parser.parse_close_tag(close, "style")?;
                style = Some(StyleBlock {
                    content: source[after_open..close].to_owned(),
                    lang,
                    scoped: open
                        .attributes
                        .iter()
                        .any(|attribute| attribute.name == "scoped"),
                    span: parser.span(block_start, after_close),
                });
                cursor = after_close;
            }
            "template" => {
                if template.is_some() {
                    return Err(parser.error(
                        ErrorCode::DuplicateBlock,
                        "duplicate <template> block",
                        block_start,
                        after_open,
                        None,
                    ));
                }
                let (nodes, _, after_close) = parser.parse_nodes(after_open, "template", 0)?;
                template = Some(Template {
                    nodes,
                    span: parser.span(block_start, after_close),
                });
                cursor = after_close;
            }
            other => {
                return Err(parser.error(
                    ErrorCode::UnsupportedBlock,
                    format!("unsupported top-level block: {other}"),
                    block_start,
                    after_open,
                    None,
                ));
            }
        }
    }

    let template = template.ok_or_else(|| {
        parser.error(
            ErrorCode::MissingTemplate,
            "missing non-empty <template> block",
            source.len(),
            source.len(),
            None,
        )
    })?;
    if template_is_empty(&template.nodes) {
        return Err(parser.error(
            ErrorCode::MissingTemplate,
            "<template> must contain at least one non-whitespace node",
            template.span.start_byte,
            template.span.end_byte,
            None,
        ));
    }

    Ok(SingleFileComponent {
        script,
        template,
        style,
    })
}

fn template_is_empty(nodes: &[TemplateNode]) -> bool {
    nodes
        .iter()
        .all(|node| matches!(node, TemplateNode::Text(text) if text.value.trim().is_empty()))
}

fn literal_attribute(attributes: &[TemplateAttribute], name: &str) -> Option<String> {
    attributes.iter().find_map(|attribute| {
        if attribute.name == name {
            match &attribute.value {
                AttributeValue::Literal(value) => Some(value.clone()),
                _ => None,
            }
        } else {
            None
        }
    })
}

fn is_void(tag: &str) -> bool {
    VOID_ELEMENTS
        .iter()
        .any(|void| tag.eq_ignore_ascii_case(void))
}

fn tags_match(open: &str, close: &str) -> bool {
    if open.bytes().any(|byte| byte.is_ascii_uppercase()) {
        open == close
    } else {
        open.eq_ignore_ascii_case(close)
    }
}

fn is_tag_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':' | b'.')
}

fn is_attribute_name_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b':' | b'.')
}

struct OpenTag {
    name: String,
    attributes: Vec<TemplateAttribute>,
    self_closing: bool,
}

enum RustSpecial {
    None,
    End(usize),
    Unterminated(&'static str),
}

struct Parser<'a> {
    file: &'a str,
    source: &'a str,
    bytes: &'a [u8],
}

impl<'a> Parser<'a> {
    fn error(
        &self,
        code: ErrorCode,
        message: impl Into<String>,
        start: usize,
        end: usize,
        hint: Option<&'static str>,
    ) -> ParseError {
        let start = start.min(self.source.len());
        let end = end.min(self.source.len()).max(start);
        let (line, column) = line_column(self.source, start);
        ParseError {
            code,
            message: message.into(),
            span: self.span(start, end),
            line,
            column,
            hint,
        }
    }

    fn span(&self, start: usize, end: usize) -> SourceSpan {
        SourceSpan {
            file: self.file.to_owned(),
            start_byte: start,
            end_byte: end,
        }
    }

    fn starts_with(&self, at: usize, needle: &[u8]) -> bool {
        self.bytes
            .get(at..)
            .is_some_and(|rest| rest.starts_with(needle))
    }

    fn find(&self, from: usize, needle: &[u8]) -> Option<usize> {
        self.bytes
            .get(from..)?
            .windows(needle.len())
            .position(|window| window == needle)
            .map(|offset| from + offset)
    }

    fn skip_ws(&self, mut cursor: usize) -> usize {
        while cursor < self.source.len() && self.bytes[cursor].is_ascii_whitespace() {
            cursor += 1;
        }
        cursor
    }

    fn parse_tag_open(&self, start: usize) -> Result<(OpenTag, usize), ParseError> {
        let mut cursor = start + 1;
        let name_start = cursor;
        while cursor < self.source.len() && is_tag_name_byte(self.bytes[cursor]) {
            cursor += 1;
        }
        if cursor == name_start || !self.bytes[name_start].is_ascii_alphabetic() {
            return Err(self.error(
                ErrorCode::InvalidTagName,
                "tag name must start with an ASCII letter",
                start,
                (start + 1).min(self.source.len()),
                None,
            ));
        }
        let name = self.source[name_start..cursor].to_owned();
        let mut attributes = Vec::new();

        loop {
            cursor = self.skip_ws(cursor);
            if cursor >= self.source.len() {
                return Err(self.error(
                    ErrorCode::UnclosedTag,
                    format!("unclosed <{name}> tag"),
                    start,
                    self.source.len(),
                    None,
                ));
            }
            match self.bytes[cursor] {
                b'>' => {
                    return Ok((
                        OpenTag {
                            name,
                            attributes,
                            self_closing: false,
                        },
                        cursor + 1,
                    ));
                }
                b'/' => {
                    if self.bytes.get(cursor + 1) == Some(&b'>') {
                        return Ok((
                            OpenTag {
                                name,
                                attributes,
                                self_closing: true,
                            },
                            cursor + 2,
                        ));
                    }
                    return Err(self.error(
                        ErrorCode::InvalidAttributeName,
                        "unexpected '/' in tag",
                        cursor,
                        cursor + 1,
                        None,
                    ));
                }
                _ => {}
            }

            let attribute_start = cursor;
            while cursor < self.source.len() && is_attribute_name_byte(self.bytes[cursor]) {
                cursor += 1;
            }
            if cursor == attribute_start {
                return Err(self.error(
                    ErrorCode::InvalidAttributeName,
                    "invalid attribute name",
                    cursor,
                    (cursor + 1).min(self.source.len()),
                    None,
                ));
            }
            let attribute_name = self.source[attribute_start..cursor].to_owned();
            let after_name = cursor;
            cursor = self.skip_ws(cursor);

            let mut value = AttributeValue::Boolean;
            let mut value_end = after_name;
            if cursor < self.source.len() && self.bytes[cursor] == b'=' {
                cursor = self.skip_ws(cursor + 1);
                if cursor >= self.source.len() {
                    return Err(self.error(
                        ErrorCode::UnclosedAttribute,
                        format!("missing value for attribute `{attribute_name}`"),
                        attribute_start,
                        self.source.len(),
                        None,
                    ));
                }
                match self.bytes[cursor] {
                    quote @ (b'"' | b'\'') => {
                        let content_start = cursor + 1;
                        let close = self.find(content_start, &[quote]).ok_or_else(|| {
                            self.error(
                                ErrorCode::UnclosedAttribute,
                                format!("unclosed value for attribute `{attribute_name}`"),
                                attribute_start,
                                self.source.len(),
                                None,
                            )
                        })?;
                        value =
                            AttributeValue::Literal(self.source[content_start..close].to_owned());
                        value_end = close + 1;
                        cursor = close + 1;
                    }
                    b'{' => {
                        let (close, expression) = self.scan_expression(cursor)?;
                        value = AttributeValue::Expression(expression);
                        value_end = close + 1;
                        cursor = close + 1;
                    }
                    _ => {
                        let value_start = cursor;
                        while cursor < self.source.len()
                            && !self.bytes[cursor].is_ascii_whitespace()
                            && self.bytes[cursor] != b'>'
                        {
                            cursor += 1;
                        }
                        if cursor == value_start {
                            return Err(self.error(
                                ErrorCode::InvalidAttributeValue,
                                format!("missing value for attribute `{attribute_name}`"),
                                attribute_start,
                                self.source.len(),
                                None,
                            ));
                        }
                        value =
                            AttributeValue::Literal(self.source[value_start..cursor].to_owned());
                        value_end = cursor;
                    }
                }
            }

            let lowered = attribute_name.to_ascii_lowercase();
            if lowered.starts_with("on") && !lowered.starts_with("on:") {
                return Err(self.error(
                    ErrorCode::UnsafeEventHandler,
                    format!("event handler `{attribute_name}` must use `on:event={{handler}}`"),
                    attribute_start,
                    value_end,
                    Some("string event handlers and inline scripts are not allowed"),
                ));
            }
            if attributes
                .iter()
                .any(|attribute| attribute.name == attribute_name)
            {
                return Err(self.error(
                    ErrorCode::DuplicateAttribute,
                    format!("duplicate attribute `{attribute_name}`"),
                    attribute_start,
                    value_end,
                    None,
                ));
            }

            attributes.push(TemplateAttribute {
                name: attribute_name,
                value,
                span: self.span(attribute_start, value_end),
            });
        }
    }

    fn parse_close_tag(&self, start: usize, expected: &str) -> Result<usize, ParseError> {
        let mut cursor = start + 2;
        let name_start = cursor;
        while cursor < self.source.len() && is_tag_name_byte(self.bytes[cursor]) {
            cursor += 1;
        }
        if cursor == name_start {
            return Err(self.error(
                ErrorCode::InvalidTagName,
                "missing tag name in closing tag",
                start,
                cursor.max(start + 2),
                None,
            ));
        }
        let name = &self.source[name_start..cursor];
        cursor = self.skip_ws(cursor);
        if cursor >= self.source.len() || self.bytes[cursor] != b'>' {
            return Err(self.error(
                ErrorCode::UnclosedTag,
                format!("missing '>' in closing tag </{name}>"),
                start,
                self.source.len(),
                None,
            ));
        }
        if !tags_match(expected, name) {
            return Err(self.error(
                ErrorCode::UnexpectedClosingTag,
                format!("expected </{expected}>, found </{name}>"),
                start,
                cursor + 1,
                None,
            ));
        }
        Ok(cursor + 1)
    }

    fn parse_nodes(
        &self,
        mut cursor: usize,
        terminator: &str,
        depth: usize,
    ) -> Result<(Vec<TemplateNode>, usize, usize), ParseError> {
        let mut nodes = Vec::new();

        loop {
            if cursor >= self.source.len() {
                return Err(self.error(
                    ErrorCode::MissingClosingTag,
                    format!("missing closing tag </{terminator}>"),
                    self.source.len(),
                    self.source.len(),
                    None,
                ));
            }

            match self.bytes[cursor] {
                b'<' => {
                    if self.starts_with(cursor, b"<!--") {
                        let end = self.find(cursor + 4, b"-->").ok_or_else(|| {
                            self.error(
                                ErrorCode::UnclosedComment,
                                "unclosed HTML comment",
                                cursor,
                                self.source.len(),
                                None,
                            )
                        })?;
                        nodes.push(TemplateNode::Comment(TemplateComment {
                            value: self.source[cursor + 4..end].to_owned(),
                            span: self.span(cursor, end + 3),
                        }));
                        cursor = end + 3;
                    } else if self.starts_with(cursor, b"</") {
                        let after = self.parse_close_tag(cursor, terminator)?;
                        return Ok((nodes, cursor, after));
                    } else if self.starts_with(cursor, b"<!") {
                        return Err(self.error(
                            ErrorCode::InvalidTopLevel,
                            "declarations and processing instructions are not supported inside templates",
                            cursor,
                            (cursor + 2).min(self.source.len()),
                            None,
                        ));
                    } else {
                        let element_start = cursor;
                        let (open, after_open) = self.parse_tag_open(cursor)?;
                        if depth + 1 > MAX_TEMPLATE_DEPTH {
                            return Err(self.error(
                                ErrorCode::DepthLimit,
                                format!("template nesting exceeds {MAX_TEMPLATE_DEPTH} levels"),
                                element_start,
                                after_open,
                                None,
                            ));
                        }

                        if open.self_closing || is_void(&open.name) {
                            nodes.push(TemplateNode::Element(TemplateElement {
                                tag: open.name,
                                attributes: open.attributes,
                                children: Vec::new(),
                                span: self.span(element_start, after_open),
                                self_closing: open.self_closing,
                            }));
                            cursor = after_open;
                        } else {
                            let (children, _, after_close) =
                                self.parse_nodes(after_open, &open.name, depth + 1)?;
                            nodes.push(TemplateNode::Element(TemplateElement {
                                tag: open.name,
                                attributes: open.attributes,
                                children,
                                span: self.span(element_start, after_close),
                                self_closing: false,
                            }));
                            cursor = after_close;
                        }
                    }
                }
                b'{' => {
                    let (close, expression) = self.scan_expression(cursor)?;
                    nodes.push(TemplateNode::Interpolation(TemplateInterpolation {
                        expression,
                        span: self.span(cursor, close + 1),
                    }));
                    cursor = close + 1;
                }
                _ => {
                    let end = self.next_template_delimiter(cursor);
                    nodes.push(TemplateNode::Text(TemplateText {
                        value: self.source[cursor..end].to_owned(),
                        span: self.span(cursor, end),
                    }));
                    cursor = end;
                }
            }
        }
    }

    fn next_template_delimiter(&self, from: usize) -> usize {
        let mut cursor = from;
        while cursor < self.source.len() {
            if self.bytes[cursor] == b'<' || self.bytes[cursor] == b'{' {
                return cursor;
            }
            cursor += 1;
        }
        self.source.len()
    }

    fn scan_expression(&self, open: usize) -> Result<(usize, String), ParseError> {
        let mut cursor = open + 1;
        let mut depth = 1usize;

        while cursor < self.source.len() {
            match self.rust_special(cursor) {
                RustSpecial::End(next) => {
                    cursor = next;
                    continue;
                }
                RustSpecial::Unterminated(kind) => {
                    return Err(self.error(
                        ErrorCode::UnclosedLiteral,
                        format!("unclosed {kind} in template expression"),
                        cursor,
                        self.source.len(),
                        None,
                    ));
                }
                RustSpecial::None => {}
            }

            match self.bytes[cursor] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok((cursor, self.source[open + 1..cursor].to_owned()));
                    }
                }
                _ => {}
            }
            cursor += 1;
        }

        Err(self.error(
            ErrorCode::UnclosedExpression,
            "unclosed `{` in template expression",
            open,
            self.source.len(),
            Some("use `{{` for a literal brace once escape syntax is defined"),
        ))
    }

    fn find_script_close(&self, from: usize) -> Result<usize, ParseError> {
        let mut cursor = from;
        while cursor < self.source.len() {
            match self.rust_special(cursor) {
                RustSpecial::End(next) => {
                    cursor = next;
                    continue;
                }
                RustSpecial::Unterminated(kind) => {
                    return Err(self.error(
                        ErrorCode::UnclosedLiteral,
                        format!("unclosed {kind} in <script> block"),
                        cursor,
                        self.source.len(),
                        None,
                    ));
                }
                RustSpecial::None => {}
            }

            if self.starts_with(cursor, b"</script") {
                let next = cursor + b"</script".len();
                if next >= self.source.len()
                    || self.bytes[next] == b'>'
                    || self.bytes[next].is_ascii_whitespace()
                {
                    return Ok(cursor);
                }
            }
            cursor += 1;
        }

        Err(self.error(
            ErrorCode::MissingClosingTag,
            "missing closing tag </script>",
            from,
            self.source.len(),
            None,
        ))
    }

    fn find_style_close(&self, from: usize) -> Result<usize, ParseError> {
        let mut cursor = from;
        while cursor < self.source.len() {
            if self.starts_with(cursor, b"/*") {
                let end = self.find(cursor + 2, b"*/").ok_or_else(|| {
                    self.error(
                        ErrorCode::UnclosedComment,
                        "unclosed comment in <style> block",
                        cursor,
                        self.source.len(),
                        None,
                    )
                })?;
                cursor = end + 2;
                continue;
            }
            if matches!(self.bytes[cursor], b'"' | b'\'') {
                cursor = self.skip_css_string(cursor)?;
                continue;
            }
            if self.starts_with(cursor, b"</style") {
                let next = cursor + b"</style".len();
                if next >= self.source.len()
                    || self.bytes[next] == b'>'
                    || self.bytes[next].is_ascii_whitespace()
                {
                    return Ok(cursor);
                }
            }
            cursor += 1;
        }

        Err(self.error(
            ErrorCode::MissingClosingTag,
            "missing closing tag </style>",
            from,
            self.source.len(),
            None,
        ))
    }

    fn skip_css_string(&self, start: usize) -> Result<usize, ParseError> {
        let quote = self.bytes[start];
        let mut cursor = start + 1;
        while cursor < self.source.len() {
            if self.bytes[cursor] == b'\\' {
                cursor += 2;
                continue;
            }
            if self.bytes[cursor] == quote {
                return Ok(cursor + 1);
            }
            cursor += 1;
        }
        Err(self.error(
            ErrorCode::UnclosedLiteral,
            "unclosed string in <style> block",
            start,
            self.source.len(),
            None,
        ))
    }

    fn rust_special(&self, start: usize) -> RustSpecial {
        let bytes = self.bytes;
        let len = bytes.len();

        if bytes[start] == b'/' {
            if bytes.get(start + 1) == Some(&b'/') {
                let mut cursor = start + 2;
                while cursor < len && bytes[cursor] != b'\n' {
                    cursor += 1;
                }
                return RustSpecial::End(cursor);
            }
            if bytes.get(start + 1) == Some(&b'*') {
                let mut depth = 1usize;
                let mut cursor = start + 2;
                while cursor < len {
                    if bytes[cursor] == b'/' && bytes.get(cursor + 1) == Some(&b'*') {
                        depth += 1;
                        cursor += 2;
                    } else if bytes[cursor] == b'*' && bytes.get(cursor + 1) == Some(&b'/') {
                        depth -= 1;
                        cursor += 2;
                        if depth == 0 {
                            return RustSpecial::End(cursor);
                        }
                    } else {
                        cursor += 1;
                    }
                }
                return RustSpecial::Unterminated("block comment");
            }
        }

        if bytes[start] == b'"' {
            return match self.skip_rust_string(start, b'"') {
                Some(end) => RustSpecial::End(end),
                None => RustSpecial::Unterminated("string"),
            };
        }

        if bytes[start] == b'b' {
            if bytes.get(start + 1) == Some(&b'"') {
                return match self.skip_rust_string(start + 1, b'"') {
                    Some(end) => RustSpecial::End(end),
                    None => RustSpecial::Unterminated("byte string"),
                };
            }
            if bytes.get(start + 1) == Some(&b'r') {
                if let Some(result) = self.raw_string(start + 1) {
                    return result;
                }
            }
        }

        if bytes[start] == b'r' {
            if let Some(result) = self.raw_string(start) {
                return result;
            }
        }

        if bytes[start] == b'\'' {
            let content = if bytes.get(start + 1) == Some(&b'\\') {
                start + 3
            } else {
                start + 1
            };
            let limit = (content + 4).min(len);
            if let Some(offset) = bytes[content..limit].iter().position(|byte| *byte == b'\'') {
                return RustSpecial::End(content + offset + 1);
            }
        }

        RustSpecial::None
    }

    fn skip_rust_string(&self, start: usize, quote: u8) -> Option<usize> {
        let mut cursor = start + 1;
        while cursor < self.source.len() {
            if self.bytes[cursor] == b'\\' {
                cursor += 2;
                continue;
            }
            if self.bytes[cursor] == quote {
                return Some(cursor + 1);
            }
            cursor += 1;
        }
        None
    }

    fn raw_string(&self, start: usize) -> Option<RustSpecial> {
        let mut cursor = start + 1;
        let mut hashes = 0usize;
        while self.bytes.get(cursor) == Some(&b'#') {
            hashes += 1;
            cursor += 1;
        }
        if self.bytes.get(cursor) != Some(&b'"') {
            return None;
        }
        let content = cursor + 1;
        let mut cursor = content;
        while cursor < self.source.len() {
            if self.bytes[cursor] == b'"' {
                let closed =
                    (0..hashes).all(|offset| self.bytes.get(cursor + 1 + offset) == Some(&b'#'));
                if closed {
                    return Some(RustSpecial::End(cursor + 1 + hashes));
                }
            }
            cursor += 1;
        }
        Some(RustSpecial::Unterminated("raw string"))
    }
}

fn line_column(source: &str, offset: usize) -> (usize, usize) {
    let mut line = 1usize;
    let mut column = 1usize;
    for (index, character) in source.char_indices() {
        if index >= offset {
            break;
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_ok(source: &str) -> SingleFileComponent {
        parse("test.colibri", source).expect("valid component")
    }

    fn parse_err(source: &str) -> ParseError {
        parse("test.colibri", source).expect_err("invalid component")
    }

    fn element(node: &TemplateNode) -> &TemplateElement {
        match node {
            TemplateNode::Element(element) => element,
            other => panic!("expected element, found {other:?}"),
        }
    }

    fn interpolation(node: &TemplateNode) -> &TemplateInterpolation {
        match node {
            TemplateNode::Interpolation(interpolation) => interpolation,
            other => panic!("expected interpolation, found {other:?}"),
        }
    }

    #[test]
    fn parses_sfc_blocks() {
        let component = parse_ok(
            "<style scoped lang=\"scss\">button{color:red}</style>\n\
             <script lang=\"rust\">let x = 1;</script>\n\
             <template><button>{x}</button></template>",
        );
        assert_eq!(
            component
                .script
                .as_ref()
                .map(|script| script.content.trim()),
            Some("let x = 1;")
        );
        assert!(component.style.as_ref().is_some_and(|style| style.scoped));
        assert_eq!(component.template.nodes.len(), 1);
    }

    #[test]
    fn parses_nested_template_with_spans() {
        let source = "<template>\n  <section class=\"card\" hidden><h1>Título</h1><p>Hola {name}</p></section>\n</template>";
        let component = parse_ok(source);
        assert_eq!(component.template.nodes.len(), 3);

        let section = element(&component.template.nodes[1]);
        assert_eq!(section.tag, "section");
        assert_eq!(section.attributes.len(), 2);
        assert_eq!(
            section.attributes[0].value,
            AttributeValue::Literal("card".into())
        );
        assert_eq!(section.attributes[1].value, AttributeValue::Boolean);
        assert_eq!(
            section.span.text(source),
            Some("<section class=\"card\" hidden><h1>Título</h1><p>Hola {name}</p></section>")
        );

        let heading = element(&section.children[0]);
        assert_eq!(heading.tag, "h1");
        assert_eq!(paragraph_expression(&section.children[1]), "name");
    }

    fn paragraph_expression(node: &TemplateNode) -> String {
        let paragraph = element(node);
        assert_eq!(paragraph.tag, "p");
        interpolation(&paragraph.children[1]).expression.clone()
    }

    #[test]
    fn keeps_closing_tags_inside_rust_strings_and_comments() {
        let source = "<script lang=\"rust\">let a = \"</script>\"; let b = r#\"</script>\"#; /* </script> */ let c = b\"</script>\";</script><template><p>x</p></template>";
        let component = parse_ok(source);
        let script = component.script.expect("script");
        assert!(script.content.contains("\"</script>\""));
        assert!(script.content.contains("r#\"</script>\"#"));
        assert!(script.content.contains("/* </script> */"));
        assert!(script.content.contains("b\"</script>\""));
    }

    #[test]
    fn keeps_closing_tags_inside_css_comments_and_strings() {
        let source = "<style>/* </style> */ a::before{content:\"</style>\"}</style><template><p>x</p></template>";
        let component = parse_ok(source);
        let style = component.style.expect("style");
        assert!(style.content.contains("/* </style> */"));
        assert!(style.content.contains("\"</style>\""));
    }

    #[test]
    fn rejects_unbalanced_tags() {
        let error = parse_err("<template><div></span></div></template>");
        assert_eq!(error.code, ErrorCode::UnexpectedClosingTag);
        assert_eq!(error.line, 1);

        let error = parse_err("<template><div></template>");
        assert_eq!(error.code, ErrorCode::UnexpectedClosingTag);
        assert!(error.message.contains("</div>"));

        let error = parse_err("<template><div></div>");
        assert_eq!(error.code, ErrorCode::MissingClosingTag);
        assert_eq!(error.code.as_str(), "missing-closing-tag");
    }

    #[test]
    fn reports_unicode_and_crlf_positions() {
        let source = "<template>\r\n<p>Hola 🌍</p>\r\n</span></template>";
        let error = parse_err(source);
        assert_eq!(error.line, 3);
        assert_eq!(error.column, 1);
        assert_eq!(error.span.text(source), Some("</span>"));

        let source = "<template>\r\n<p>Hola 🌍 { nombre }</p>\r\n</template>";
        let component = parse_ok(source);
        let paragraph = element(&component.template.nodes[1]);
        assert_eq!(interpolation(&paragraph.children[1]).expression, " nombre ");
    }

    #[test]
    fn enforces_depth_limit() {
        let depth = MAX_TEMPLATE_DEPTH + 2;
        let mut source = String::from("<template>");
        source.push_str(&"<div>".repeat(depth));
        source.push_str(&"</div>".repeat(depth));
        source.push_str("</template>");
        assert_eq!(parse_err(&source).code, ErrorCode::DepthLimit);
    }

    #[test]
    fn handles_void_and_self_closing_elements() {
        let source = "<template><input type=\"text\"><br/><img src=\"x.png\"></template>";
        let component = parse_ok(source);
        assert_eq!(component.template.nodes.len(), 3);
        assert!(element(&component.template.nodes[1]).self_closing);
    }

    #[test]
    fn rejects_duplicate_attributes_and_inline_handlers() {
        let error = parse_err("<template><p class=\"a\" class=\"b\">x</p></template>");
        assert_eq!(error.code, ErrorCode::DuplicateAttribute);

        let error = parse_err("<template><button onclick=\"x()\">x</button></template>");
        assert_eq!(error.code, ErrorCode::UnsafeEventHandler);

        parse_ok("<template><button on:click={go}>x</button></template>");
    }

    #[test]
    fn balances_braces_in_expressions() {
        let source = "<template><p>{ if x { \"a\" } else { \"b\" } }</p></template>";
        let component = parse_ok(source);
        let paragraph = element(&component.template.nodes[0]);
        assert_eq!(
            interpolation(&paragraph.children[0]).expression,
            " if x { \"a\" } else { \"b\" } "
        );

        let error = parse_err("<template><p>{value</p></template>");
        assert_eq!(error.code, ErrorCode::UnclosedExpression);
    }

    #[test]
    fn rejects_missing_and_duplicate_blocks() {
        let error = parse_err("<script lang=\"rust\">x</script>");
        assert_eq!(error.code, ErrorCode::MissingTemplate);

        let error = parse_err("<template>a</template><template>b</template>");
        assert_eq!(error.code, ErrorCode::DuplicateBlock);

        let error = parse_err("<template> </template>");
        assert_eq!(error.code, ErrorCode::MissingTemplate);

        let error = parse_err("<main></main><template>x</template>");
        assert_eq!(error.code, ErrorCode::UnsupportedBlock);
    }

    #[test]
    fn rejects_unsupported_languages() {
        let error = parse_err("<script lang=\"js\">x</script><template>ok</template>");
        assert_eq!(error.code, ErrorCode::InvalidScriptLanguage);

        let error = parse_err("<style lang=\"less\">a{}</style><template>ok</template>");
        assert_eq!(error.code, ErrorCode::InvalidStyleLanguage);
    }

    #[test]
    fn rejects_oversized_sources() {
        let mut source = String::from("<template>");
        source.push_str(&"x".repeat(MAX_SOURCE_BYTES));
        source.push_str("</template>");
        assert_eq!(parse_err(&source).code, ErrorCode::SourceTooLarge);
    }

    #[test]
    fn partial_inputs_never_panic() {
        let source = "<style scoped lang=\"scss\">a{color:red}</style><script lang=\"rust\">let s = \"</script>\";</script><template><p>Título {x}</p><!-- c --></template>";
        for (offset, _) in source.char_indices() {
            let _ = parse("partial.colibri", &source[..offset]);
        }
    }
}

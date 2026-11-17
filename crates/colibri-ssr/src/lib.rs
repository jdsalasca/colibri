//! Restricted HTML renderer for the semantic IR.
//!
//! `render_static` rejects dynamic nodes. `render_dynamic` accepts an explicit
//! resolver from expression text to value; the framework still never evaluates
//! Rust and always escapes text and attributes. Every rejection carries the
//! source span of the offending node.

use colibri_ir::{Attribute, Element, Node, SourceSpan};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderErrorKind {
    DynamicContent,
    MissingValue,
    InvalidTag,
    InvalidAttribute,
    UnsafeAttribute,
    InvalidVoidElement,
    UnresolvedComponent,
}

impl RenderErrorKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::DynamicContent => "dynamic-content",
            Self::MissingValue => "missing-value",
            Self::InvalidTag => "invalid-tag",
            Self::InvalidAttribute => "invalid-attribute",
            Self::UnsafeAttribute => "unsafe-attribute",
            Self::InvalidVoidElement => "invalid-void-element",
            Self::UnresolvedComponent => "unresolved-component",
        }
    }

    fn message(self) -> &'static str {
        match self {
            Self::DynamicContent => {
                "static rendering cannot evaluate dynamic content or event handlers"
            }
            Self::MissingValue => "dynamic expression has no value in this render",
            Self::InvalidTag => "tag is not allowed in static rendering",
            Self::InvalidAttribute => "attribute name is not valid",
            Self::UnsafeAttribute => "attribute is unsafe in static rendering",
            Self::InvalidVoidElement => "void elements cannot have children",
            Self::UnresolvedComponent => "component tags must be resolved before static rendering",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderError {
    pub kind: RenderErrorKind,
    pub span: SourceSpan,
}

impl RenderError {
    fn new(kind: RenderErrorKind, span: &SourceSpan) -> Self {
        Self {
            kind,
            span: span.clone(),
        }
    }
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}..{}: [{}] {}",
            self.span.file,
            self.span.start_byte,
            self.span.end_byte,
            self.kind.as_str(),
            self.kind.message()
        )
    }
}

impl Error for RenderError {}

fn valid_name(name: &str) -> bool {
    let mut bytes = name.bytes();
    matches!(bytes.next(), Some(b'a'..=b'z' | b'A'..=b'Z'))
        && bytes.all(|b| b.is_ascii_alphanumeric() || matches!(b, b'-' | b'_' | b':'))
}

fn escape_html(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

fn is_void(tag: &str) -> bool {
    matches!(
        tag,
        "area"
            | "base"
            | "br"
            | "col"
            | "embed"
            | "hr"
            | "img"
            | "input"
            | "link"
            | "meta"
            | "param"
            | "source"
            | "track"
            | "wbr"
    )
}

fn literal_attribute<'a>(element: &'a Element, name: &str) -> Option<&'a str> {
    element
        .attributes
        .iter()
        .find_map(|attribute| match attribute {
            Attribute::Static {
                name: attribute_name,
                value,
                ..
            } if attribute_name.eq_ignore_ascii_case(name) => Some(value.as_str()),
            _ => None,
        })
}

fn has_attribute(element: &Element, name: &str) -> bool {
    element
        .attributes
        .iter()
        .any(|attribute| attribute.name().eq_ignore_ascii_case(name))
}

fn meta_is_safe(element: &Element) -> bool {
    if has_attribute(element, "http-equiv") {
        return false;
    }
    let charset = literal_attribute(element, "charset").is_some();
    let viewport = literal_attribute(element, "name")
        .is_some_and(|name| name.eq_ignore_ascii_case("viewport"))
        && literal_attribute(element, "content").is_some();
    charset || viewport
}

fn link_is_safe(element: &Element) -> bool {
    literal_attribute(element, "rel").is_some_and(|rel| rel.eq_ignore_ascii_case("stylesheet"))
        && literal_attribute(element, "href").is_some()
}

/// Static rendering rejects dynamics; dynamic rendering resolves them.
enum Mode<'a> {
    Static,
    Dynamic(&'a mut dyn FnMut(&str) -> Option<String>),
}

fn resolve(
    mode: &mut Mode<'_>,
    expression: &str,
    span: &SourceSpan,
) -> Result<String, RenderError> {
    match mode {
        Mode::Static => Err(RenderError::new(RenderErrorKind::DynamicContent, span)),
        Mode::Dynamic(resolver) => resolver(expression)
            .ok_or_else(|| RenderError::new(RenderErrorKind::MissingValue, span)),
    }
}

fn render_into(node: &Node, output: &mut String, mode: &mut Mode<'_>) -> Result<(), RenderError> {
    match node {
        Node::Text { value, .. } => output.push_str(&escape_html(value)),
        Node::DynamicText { expression, span } => {
            let value = resolve(mode, expression, span)?;
            output.push_str(&escape_html(&value));
        }
        Node::Fragment { children, .. } => {
            for child in children {
                render_into(child, output, mode)?;
            }
        }
        Node::Element(element) => render_element(element, output, mode)?,
    }
    Ok(())
}

fn render_element(
    element: &Element,
    output: &mut String,
    mode: &mut Mode<'_>,
) -> Result<(), RenderError> {
    let tag = element.tag.to_ascii_lowercase();
    if element.tag.bytes().any(|byte| byte.is_ascii_uppercase()) {
        return Err(RenderError::new(
            RenderErrorKind::UnresolvedComponent,
            &element.span,
        ));
    }
    if !valid_name(&element.tag)
        || matches!(
            tag.as_str(),
            "script" | "style" | "iframe" | "object" | "embed" | "svg" | "math" | "base"
        )
    {
        return Err(RenderError::new(RenderErrorKind::InvalidTag, &element.span));
    }
    if (tag == "meta" && !meta_is_safe(element)) || (tag == "link" && !link_is_safe(element)) {
        return Err(RenderError::new(
            RenderErrorKind::UnsafeAttribute,
            &element.span,
        ));
    }
    if is_void(&tag) && !element.children.is_empty() {
        return Err(RenderError::new(
            RenderErrorKind::InvalidVoidElement,
            &element.span,
        ));
    }

    output.push('<');
    output.push_str(&element.tag);
    for attribute in &element.attributes {
        render_attribute(attribute, output, mode)?;
    }
    output.push('>');
    if !is_void(&tag) {
        for child in &element.children {
            render_into(child, output, mode)?;
        }
        output.push_str("</");
        output.push_str(&element.tag);
        output.push('>');
    }
    Ok(())
}

fn render_attribute(
    attribute: &Attribute,
    output: &mut String,
    mode: &mut Mode<'_>,
) -> Result<(), RenderError> {
    match attribute {
        Attribute::Static { name, value, span } => {
            render_named_attribute(name, Some(value), span, output)
        }
        Attribute::Boolean { name, span } => render_named_attribute(name, None, span, output),
        Attribute::Dynamic {
            name,
            expression,
            span,
        } => {
            let value = resolve(mode, expression, span)?;
            render_named_attribute(name, Some(&value), span, output)
        }
        Attribute::Event { span, .. } => {
            Err(RenderError::new(RenderErrorKind::DynamicContent, span))
        }
    }
}

fn render_named_attribute(
    name: &str,
    value: Option<&str>,
    span: &SourceSpan,
    output: &mut String,
) -> Result<(), RenderError> {
    let lower = name.to_ascii_lowercase();
    if !valid_name(name) {
        return Err(RenderError::new(RenderErrorKind::InvalidAttribute, span));
    }
    if lower.starts_with("on") || lower == "srcdoc" || lower == "style" {
        return Err(RenderError::new(RenderErrorKind::UnsafeAttribute, span));
    }

    if let Some(value) = value {
        if matches!(
            lower.as_str(),
            "href" | "src" | "action" | "formaction" | "xlink:href"
        ) {
            let normalized: String = value
                .chars()
                .filter(|ch| !ch.is_ascii_whitespace() && !ch.is_ascii_control())
                .flat_map(char::to_lowercase)
                .collect();
            if ["javascript:", "vbscript:", "data:"]
                .iter()
                .any(|prefix| normalized.starts_with(prefix))
            {
                return Err(RenderError::new(RenderErrorKind::UnsafeAttribute, span));
            }
        }
        output.push(' ');
        output.push_str(name);
        output.push_str("=\"");
        output.push_str(&escape_html(value));
        output.push('"');
    } else {
        output.push(' ');
        output.push_str(name);
    }
    Ok(())
}

/// Render a static IR tree as safe HTML. Dynamic nodes are rejected.
pub fn render_static(node: &Node) -> Result<String, RenderError> {
    let mut mode = Mode::Static;
    let mut result = String::new();
    render_into(node, &mut result, &mut mode)?;
    Ok(result)
}

/// Render an IR tree resolving dynamic expressions through `resolver`.
///
/// `resolver` receives the expression text stored in the IR (`"user.name"`,
/// `"count"`, ...) and returns the value for this render. Missing values are a
/// `MissingValue` diagnostic, never an empty string. Text and attributes are
/// escaped; URL and attribute safety rules still apply.
pub fn render_dynamic<F>(node: &Node, mut resolver: F) -> Result<String, RenderError>
where
    F: FnMut(&str) -> Option<String>,
{
    let mut mode = Mode::Dynamic(&mut resolver as &mut dyn FnMut(&str) -> Option<String>);
    let mut result = String::new();
    render_into(node, &mut result, &mut mode)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn span(start: usize, end: usize) -> SourceSpan {
        SourceSpan {
            file: "test.colibri".into(),
            start_byte: start,
            end_byte: end,
        }
    }

    fn element(tag: &str, attributes: Vec<Attribute>, children: Vec<Node>) -> Node {
        Node::Element(Element {
            tag: tag.into(),
            attributes,
            children,
            span: span(0, 1),
        })
    }

    fn text(value: &str) -> Node {
        Node::Text {
            value: value.into(),
            span: span(0, 1),
        }
    }

    fn dynamic_text(expression: &str) -> Node {
        Node::DynamicText {
            expression: expression.into(),
            span: span(3, 10),
        }
    }

    fn static_attribute(name: &str, value: &str) -> Attribute {
        Attribute::Static {
            name: name.into(),
            value: value.into(),
            span: span(0, 1),
        }
    }

    #[test]
    fn escapes_content() {
        let node = element(
            "p",
            vec![Attribute::Static {
                name: "title".into(),
                value: "a\"<b".into(),
                span: span(0, 1),
            }],
            vec![text("<hello>&")],
        );
        assert_eq!(
            render_static(&node).unwrap(),
            "<p title=\"a&quot;&lt;b\">&lt;hello&gt;&amp;</p>"
        );
    }

    #[test]
    fn renders_boolean_and_void_attributes() {
        let node = element(
            "input",
            vec![
                Attribute::Static {
                    name: "type".into(),
                    value: "checkbox".into(),
                    span: span(0, 1),
                },
                Attribute::Boolean {
                    name: "checked".into(),
                    span: span(0, 1),
                },
            ],
            vec![],
        );
        assert_eq!(
            render_static(&node).unwrap(),
            "<input type=\"checkbox\" checked>"
        );
    }

    #[test]
    fn rejects_unsafe_attribute_values() {
        let node = element(
            "a",
            vec![Attribute::Static {
                name: "href".into(),
                value: " java\nscript:alert(1)".into(),
                span: span(7, 9),
            }],
            vec![],
        );
        let error = render_static(&node).unwrap_err();
        assert_eq!(error.kind, RenderErrorKind::UnsafeAttribute);
        assert_eq!(error.span, span(7, 9));
    }

    #[test]
    fn does_not_evaluate_dynamic_content() {
        let node = dynamic_text("count");
        let error = render_static(&node).unwrap_err();
        assert_eq!(error.kind, RenderErrorKind::DynamicContent);
        assert_eq!(error.span, span(3, 10));
    }

    #[test]
    fn rejects_unresolved_components_and_void_children() {
        let component = element("Card", vec![], vec![]);
        assert_eq!(
            render_static(&component).unwrap_err().kind,
            RenderErrorKind::UnresolvedComponent
        );

        let void = element("br", vec![], vec![text("x")]);
        assert_eq!(
            render_static(&void).unwrap_err().kind,
            RenderErrorKind::InvalidVoidElement
        );
    }

    #[test]
    fn renders_dynamic_text_and_attributes() {
        let node = element(
            "p",
            vec![Attribute::Dynamic {
                name: "title".into(),
                expression: "label".into(),
                span: span(0, 1),
            }],
            vec![dynamic_text("name")],
        );
        let html = render_dynamic(&node, |expression| match expression {
            "name" => Some("Ada".into()),
            "label" => Some("saludo".into()),
            _ => None,
        })
        .unwrap();
        assert_eq!(html, "<p title=\"saludo\">Ada</p>");
    }

    #[test]
    fn escapes_dynamic_values() {
        let node = element(
            "p",
            vec![Attribute::Dynamic {
                name: "title".into(),
                expression: "label".into(),
                span: span(0, 1),
            }],
            vec![dynamic_text("name")],
        );
        let html = render_dynamic(&node, |_| Some("<script>\"x\"</script>".into())).unwrap();
        assert_eq!(
            html,
            "<p title=\"&lt;script&gt;&quot;x&quot;&lt;/script&gt;\">&lt;script&gt;&quot;x&quot;&lt;/script&gt;</p>"
        );
    }

    #[test]
    fn missing_dynamic_values_are_reported() {
        let error = render_dynamic(&dynamic_text("name"), |_| None).unwrap_err();
        assert_eq!(error.kind, RenderErrorKind::MissingValue);
        assert_eq!(error.span, span(3, 10));
    }

    #[test]
    fn dynamic_urls_are_still_validated() {
        let node = element(
            "a",
            vec![Attribute::Dynamic {
                name: "href".into(),
                expression: "url".into(),
                span: span(5, 9),
            }],
            vec![],
        );
        let error = render_dynamic(&node, |_| Some(" java\nscript:alert(1)".into())).unwrap_err();
        assert_eq!(error.kind, RenderErrorKind::UnsafeAttribute);
        assert_eq!(error.span, span(5, 9));
    }

    #[test]
    fn events_are_not_rendered_server_side() {
        let node = element(
            "button",
            vec![Attribute::Event {
                name: "click".into(),
                handler: "go".into(),
                span: span(2, 4),
            }],
            vec![],
        );
        let error = render_dynamic(&node, |_| Some("x".into())).unwrap_err();
        assert_eq!(error.kind, RenderErrorKind::DynamicContent);
    }

    #[test]
    fn renders_safe_meta_and_stylesheet_link() {
        let head = element(
            "head",
            vec![],
            vec![
                element("meta", vec![static_attribute("charset", "utf-8")], vec![]),
                element(
                    "meta",
                    vec![
                        static_attribute("name", "viewport"),
                        static_attribute("content", "width=device-width, initial-scale=1"),
                    ],
                    vec![],
                ),
                element(
                    "link",
                    vec![
                        static_attribute("rel", "stylesheet"),
                        static_attribute("href", "/static/app.css"),
                    ],
                    vec![],
                ),
            ],
        );
        assert_eq!(
            render_static(&head).unwrap(),
            "<head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><link rel=\"stylesheet\" href=\"/static/app.css\"></head>"
        );
    }

    #[test]
    fn rejects_unsafe_meta_and_link() {
        let refresh = element(
            "meta",
            vec![
                static_attribute("http-equiv", "refresh"),
                static_attribute("content", "0;url=https://evil.example"),
            ],
            vec![],
        );
        assert_eq!(
            render_static(&refresh).unwrap_err().kind,
            RenderErrorKind::UnsafeAttribute
        );

        let preload = element(
            "link",
            vec![
                static_attribute("rel", "preload"),
                static_attribute("href", "/x"),
            ],
            vec![],
        );
        assert_eq!(
            render_static(&preload).unwrap_err().kind,
            RenderErrorKind::UnsafeAttribute
        );

        let script_url = element(
            "link",
            vec![
                static_attribute("rel", "stylesheet"),
                static_attribute("href", "javascript:alert(1)"),
            ],
            vec![],
        );
        assert_eq!(
            render_static(&script_url).unwrap_err().kind,
            RenderErrorKind::UnsafeAttribute
        );
    }
}

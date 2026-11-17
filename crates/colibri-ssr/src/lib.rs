//! Restricted static HTML renderer; dynamic expressions are not evaluated.
//!
//! Every rejected construct returns a `RenderError` with the source span of the
//! offending node so callers can point at the component source.

use colibri_ir::{Attribute, Element, Node, SourceSpan};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderErrorKind {
    DynamicContent,
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

fn render_into(node: &Node, output: &mut String) -> Result<(), RenderError> {
    match node {
        Node::Text { value, .. } => output.push_str(&escape_html(value)),
        Node::DynamicText { span, .. } => {
            return Err(RenderError::new(RenderErrorKind::DynamicContent, span));
        }
        Node::Fragment { children, .. } => {
            for child in children {
                render_into(child, output)?;
            }
        }
        Node::Element(element) => render_element(element, output)?,
    }
    Ok(())
}

fn render_element(element: &Element, output: &mut String) -> Result<(), RenderError> {
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
            "script"
                | "style"
                | "iframe"
                | "object"
                | "embed"
                | "svg"
                | "math"
                | "meta"
                | "base"
                | "link"
        )
    {
        return Err(RenderError::new(RenderErrorKind::InvalidTag, &element.span));
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
        render_attribute(attribute, output)?;
    }
    output.push('>');
    if !is_void(&tag) {
        for child in &element.children {
            render_into(child, output)?;
        }
        output.push_str("</");
        output.push_str(&element.tag);
        output.push('>');
    }
    Ok(())
}

fn render_attribute(attribute: &Attribute, output: &mut String) -> Result<(), RenderError> {
    let (name, value, span) = match attribute {
        Attribute::Static { name, value, span } => (name, Some(value), span),
        Attribute::Boolean { name, span } => (name, None, span),
        Attribute::Dynamic { span, .. } | Attribute::Event { span, .. } => {
            return Err(RenderError::new(RenderErrorKind::DynamicContent, span));
        }
    };

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

/// Render a static IR tree as safe HTML in the supported subset.
pub fn render_static(node: &Node) -> Result<String, RenderError> {
    let mut result = String::new();
    render_into(node, &mut result)?;
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
        let node = Node::DynamicText {
            expression: "count".into(),
            span: span(3, 10),
        };
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
}

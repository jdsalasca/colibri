//! Restricted static HTML renderer; dynamic expressions are not evaluated.
use colibri_ir::{Attribute, Node};
use std::error::Error;
use std::fmt;

#[derive(Debug, PartialEq, Eq)]
pub enum RenderError {
    DynamicContent,
    InvalidTag,
    InvalidAttribute,
    UnsafeAttribute,
    InvalidVoidElement,
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
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
        Node::Text(value) => output.push_str(&escape_html(value)),
        Node::DynamicText { .. } => return Err(RenderError::DynamicContent),
        Node::Fragment(nodes) => {
            for child in nodes {
                render_into(child, output)?;
            }
        }
        Node::Element(element) => {
            let tag = element.tag.to_ascii_lowercase();
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
                return Err(RenderError::InvalidTag);
            }
            if is_void(&tag) && !element.children.is_empty() {
                return Err(RenderError::InvalidVoidElement);
            }
            output.push('<');
            output.push_str(&element.tag);
            for attribute in &element.attributes {
                let Attribute::Static { name, value } = attribute else {
                    return Err(RenderError::DynamicContent);
                };
                let lower = name.to_ascii_lowercase();
                if !valid_name(name) {
                    return Err(RenderError::InvalidAttribute);
                }
                if lower.starts_with("on") || lower == "srcdoc" || lower == "style" {
                    return Err(RenderError::UnsafeAttribute);
                }
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
                        return Err(RenderError::UnsafeAttribute);
                    }
                }
                output.push(' ');
                output.push_str(name);
                output.push_str("=\"");
                output.push_str(&escape_html(value));
                output.push('"');
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
        }
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
    use colibri_ir::Element;

    fn element(tag: &str, attributes: Vec<Attribute>, children: Vec<Node>) -> Node {
        Node::Element(Element {
            tag: tag.into(),
            attributes,
            children,
        })
    }

    #[test]
    fn escapes_content() {
        let node = element(
            "p",
            vec![Attribute::Static {
                name: "title".into(),
                value: "a\"<b".into(),
            }],
            vec![Node::Text("<hello>&".into())],
        );
        assert_eq!(
            render_static(&node).unwrap(),
            "<p title=\"a&quot;&lt;b\">&lt;hello&gt;&amp;</p>"
        );
    }

    #[test]
    fn rejects_unsafe_attribute_values() {
        let node = element(
            "a",
            vec![Attribute::Static {
                name: "href".into(),
                value: " java\nscript:alert(1)".into(),
            }],
            vec![],
        );
        assert_eq!(render_static(&node), Err(RenderError::UnsafeAttribute));
    }

    #[test]
    fn does_not_evaluate_dynamic_content() {
        assert_eq!(
            render_static(&Node::DynamicText {
                expression: "count".into()
            }),
            Err(RenderError::DynamicContent)
        );
    }
}

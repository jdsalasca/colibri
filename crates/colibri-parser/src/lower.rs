//! Lowering from the parsed SFC template AST to the shared semantic IR.
//!
//! Static structure is preserved. Dynamic expressions become typed IR nodes so
//! later stages can evaluate them or reject them with a precise span; they are
//! never dropped silently. HTML comments do not affect rendering and are
//! dropped at this stage.

use crate::{
    AttributeValue, SingleFileComponent, SourceSpan, TemplateAttribute, TemplateElement,
    TemplateNode,
};
use colibri_ir::{Attribute, Component, Element, Node};
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoweringError {
    pub code: &'static str,
    pub message: String,
    pub span: SourceSpan,
}

impl fmt::Display for LoweringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}:{}..{}: [{}] {}",
            self.span.file, self.span.start_byte, self.span.end_byte, self.code, self.message
        )
    }
}

impl Error for LoweringError {}

/// Lower a parsed component to the shared IR. `name` is the component symbol.
pub fn lower(name: &str, component: &SingleFileComponent) -> Result<Component, LoweringError> {
    let span = component.template.span.clone();
    let root = Node::Fragment {
        children: lower_nodes(&component.template.nodes)?,
        span: span.clone(),
    };
    Ok(Component {
        name: name.to_owned(),
        root,
        span,
    })
}

fn lower_nodes(nodes: &[TemplateNode]) -> Result<Vec<Node>, LoweringError> {
    let mut lowered = Vec::with_capacity(nodes.len());
    for node in nodes {
        if let Some(node) = lower_node(node)? {
            lowered.push(node);
        }
    }
    Ok(lowered)
}

fn lower_node(node: &TemplateNode) -> Result<Option<Node>, LoweringError> {
    match node {
        TemplateNode::Text(text) => Ok(Some(Node::Text {
            value: text.value.clone(),
            span: text.span.clone(),
        })),
        TemplateNode::Comment(_) => Ok(None),
        TemplateNode::Interpolation(interpolation) => {
            let expression = non_empty_expression(
                &interpolation.expression,
                &interpolation.span,
                "interpolation",
            )?;
            Ok(Some(Node::DynamicText {
                expression,
                span: interpolation.span.clone(),
            }))
        }
        TemplateNode::Element(element) => Ok(Some(Node::Element(lower_element(element)?))),
    }
}

fn lower_element(element: &TemplateElement) -> Result<Element, LoweringError> {
    let mut attributes = Vec::with_capacity(element.attributes.len());
    for attribute in &element.attributes {
        attributes.push(lower_attribute(attribute)?);
    }
    Ok(Element {
        tag: element.tag.clone(),
        attributes,
        children: lower_nodes(&element.children)?,
        span: element.span.clone(),
    })
}

fn lower_attribute(attribute: &TemplateAttribute) -> Result<Attribute, LoweringError> {
    if let Some(event) = attribute.name.strip_prefix("on:") {
        if event.is_empty() {
            return Err(LoweringError {
                code: "empty-event-name",
                message: "event attribute must name an event after `on:`".into(),
                span: attribute.span.clone(),
            });
        }
        let AttributeValue::Expression(handler) = &attribute.value else {
            return Err(LoweringError {
                code: "event-requires-expression",
                message: format!("event `{event}` must use on:{event}={{handler}}"),
                span: attribute.span.clone(),
            });
        };
        let handler = non_empty_expression(handler, &attribute.span, "event handler")?;
        return Ok(Attribute::Event {
            name: event.to_owned(),
            handler,
            span: attribute.span.clone(),
        });
    }

    match &attribute.value {
        AttributeValue::Boolean => Ok(Attribute::Boolean {
            name: attribute.name.clone(),
            span: attribute.span.clone(),
        }),
        AttributeValue::Literal(value) => Ok(Attribute::Static {
            name: attribute.name.clone(),
            value: value.clone(),
            span: attribute.span.clone(),
        }),
        AttributeValue::Expression(expression) => Ok(Attribute::Dynamic {
            name: attribute.name.clone(),
            expression: non_empty_expression(expression, &attribute.span, "attribute binding")?,
            span: attribute.span.clone(),
        }),
    }
}

fn non_empty_expression(
    expression: &str,
    span: &SourceSpan,
    context: &str,
) -> Result<String, LoweringError> {
    let expression = expression.trim();
    if expression.is_empty() {
        return Err(LoweringError {
            code: "empty-expression",
            message: format!("{context} must contain a Rust expression"),
            span: span.clone(),
        });
    }
    Ok(expression.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    fn lower_source(source: &str) -> Component {
        let sfc = parse("test.colibri", source).expect("valid component");
        lower("Test", &sfc).expect("lowerable component")
    }

    fn element(node: &Node) -> &Element {
        match node {
            Node::Element(element) => element,
            other => panic!("expected element, found {other:?}"),
        }
    }

    #[test]
    fn lowers_static_template_to_ir() {
        let source = "<template><section class=\"card\" hidden><h1>Hola</h1></section></template>";
        let component = lower_source(source);
        let Node::Fragment { children, .. } = &component.root else {
            panic!("root must be a fragment");
        };
        assert_eq!(children.len(), 1);
        let section = element(&children[0]);
        assert_eq!(section.tag, "section");
        assert_eq!(section.attributes.len(), 2);
        let Attribute::Static { name, value, .. } = &section.attributes[0] else {
            panic!("class must lower to a static attribute");
        };
        assert_eq!(name, "class");
        assert_eq!(value, "card");
        assert!(matches!(section.attributes[1], Attribute::Boolean { .. }));
        assert_eq!(
            section.span.text(source),
            Some("<section class=\"card\" hidden><h1>Hola</h1></section>")
        );
    }

    #[test]
    fn lowers_events_without_the_on_prefix() {
        let component =
            lower_source("<template><button on:click={increment}>+</button></template>");
        let Node::Fragment { children, .. } = &component.root else {
            panic!("root must be a fragment");
        };
        let button = element(&children[0]);
        let Attribute::Event { name, handler, .. } = &button.attributes[0] else {
            panic!("on:click must lower to an event attribute");
        };
        assert_eq!(name, "click");
        assert_eq!(handler, "increment");
    }

    #[test]
    fn marks_dynamic_content_instead_of_dropping_it() {
        let component = lower_source("<template><p>Hola {name}</p></template>");
        let Node::Fragment { children, .. } = &component.root else {
            panic!("root must be a fragment");
        };
        let paragraph = element(&children[0]);
        assert!(matches!(paragraph.children[1], Node::DynamicText { .. }));
    }

    #[test]
    fn drops_comments() {
        let component = lower_source("<template><p>a</p><!-- nota --><p>b</p></template>");
        let Node::Fragment { children, .. } = &component.root else {
            panic!("root must be a fragment");
        };
        assert_eq!(children.len(), 2);
    }

    #[test]
    fn rejects_empty_expressions_and_literal_events() {
        let sfc = parse("test.colibri", "<template><p>{}</p></template>").unwrap();
        assert_eq!(lower("Test", &sfc).unwrap_err().code, "empty-expression");

        let sfc = parse(
            "test.colibri",
            "<template><button on:click=\"go()\">x</button></template>",
        )
        .unwrap();
        assert_eq!(
            lower("Test", &sfc).unwrap_err().code,
            "event-requires-expression"
        );
    }
}

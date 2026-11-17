//! Browser-independent view intermediate representation.
//!
//! Nodes carry source spans so later stages can report diagnostics without
//! re-parsing. The IR contains no DOM, browser or file-system APIs.

/// Byte range in the original file. `end_byte` is exclusive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceSpan {
    pub file: String,
    pub start_byte: usize,
    pub end_byte: usize,
}

impl SourceSpan {
    /// Source text covered by this span, or `None` if it is not a valid range.
    pub fn text<'a>(&self, source: &'a str) -> Option<&'a str> {
        source.get(self.start_byte..self.end_byte)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub name: String,
    pub root: Node,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Text {
        value: String,
        span: SourceSpan,
    },
    DynamicText {
        expression: String,
        span: SourceSpan,
    },
    Element(Element),
    Fragment {
        children: Vec<Node>,
        span: SourceSpan,
    },
}

impl Node {
    pub fn span(&self) -> &SourceSpan {
        match self {
            Self::Text { span, .. }
            | Self::DynamicText { span, .. }
            | Self::Fragment { span, .. } => span,
            Self::Element(element) => &element.span,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub tag: String,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
    pub span: SourceSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attribute {
    Static {
        name: String,
        value: String,
        span: SourceSpan,
    },
    Boolean {
        name: String,
        span: SourceSpan,
    },
    Dynamic {
        name: String,
        expression: String,
        span: SourceSpan,
    },
    /// `name` stores the event without the `on:` prefix.
    Event {
        name: String,
        handler: String,
        span: SourceSpan,
    },
}

impl Attribute {
    pub fn name(&self) -> &str {
        match self {
            Self::Static { name, .. }
            | Self::Boolean { name, .. }
            | Self::Dynamic { name, .. }
            | Self::Event { name, .. } => name,
        }
    }

    pub fn span(&self) -> &SourceSpan {
        match self {
            Self::Static { span, .. }
            | Self::Boolean { span, .. }
            | Self::Dynamic { span, .. }
            | Self::Event { span, .. } => span,
        }
    }
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

    #[test]
    fn component_can_hold_dynamic_bindings() {
        let component = Component {
            name: "Counter".into(),
            root: Node::Element(Element {
                tag: "button".into(),
                attributes: vec![Attribute::Event {
                    name: "click".into(),
                    handler: "increment".into(),
                    span: span(0, 1),
                }],
                children: vec![Node::DynamicText {
                    expression: "count".into(),
                    span: span(2, 3),
                }],
                span: span(0, 4),
            }),
            span: span(0, 4),
        };
        assert_eq!(component.name, "Counter");
        assert_eq!(component.root.span().end_byte, 4);
    }

    #[test]
    fn span_text_is_checked() {
        let source = "<p>hola</p>";
        assert_eq!(span(0, 3).text(source), Some("<p>"));
        assert_eq!(span(3, 100).text(source), None);
    }
}

//! A browser-independent view intermediate representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Component {
    pub name: String,
    pub root: Node,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Node {
    Text(String),
    DynamicText { expression: String },
    Element(Element),
    Fragment(Vec<Node>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Element {
    pub tag: String,
    pub attributes: Vec<Attribute>,
    pub children: Vec<Node>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Attribute {
    Static { name: String, value: String },
    Dynamic { name: String, expression: String },
    Event { name: String, handler: String },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn component_can_hold_dynamic_bindings() {
        let component = Component {
            name: "Counter".into(),
            root: Node::Element(Element {
                tag: "button".into(),
                attributes: vec![Attribute::Event {
                    name: "click".into(),
                    handler: "increment".into(),
                }],
                children: vec![Node::DynamicText {
                    expression: "count".into(),
                }],
            }),
        };
        assert_eq!(component.name, "Counter");
    }
}

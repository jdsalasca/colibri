use colibri_core::{use_effect, Effect};
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{Document, Element, Event, EventTarget, Node};

/// A view that can be mounted in the browser.
pub enum View {
    /// Static text.
    Text(String),
    /// Text recomputed whenever a tracked signal changes.
    DynamicText(Rc<dyn Fn() -> String>),
    /// An HTML element with attributes, events and children.
    Element(ElementView),
    /// A group of views rendered in order.
    Fragment(Vec<View>),
}

/// An HTML element in the runtime view tree.
pub struct ElementView {
    /// Tag name, e.g. `button`.
    pub tag: String,
    /// Static, boolean or dynamic attributes.
    pub attributes: Vec<Attribute>,
    /// Event listeners.
    pub events: Vec<EventBinding>,
    /// Child views.
    pub children: Vec<View>,
}

/// Attribute value in the runtime view tree.
pub enum Attribute {
    /// Literal attribute.
    Static { name: String, value: String },
    /// Boolean attribute rendered without a value.
    Boolean { name: String },
    /// Attribute recomputed when a tracked signal changes.
    Dynamic {
        name: String,
        value: Rc<dyn Fn() -> String>,
    },
}

/// An event listener with its Rust handler.
pub struct EventBinding {
    /// Event name without `on:`, e.g. `click`.
    pub name: String,
    /// Handler invoked by the browser.
    pub handler: Rc<dyn Fn(Event)>,
}

struct Listener {
    target: EventTarget,
    event: String,
    closure: Closure<dyn FnMut(Event)>,
}

/// Handle that owns effects and listeners for a mounted view.
pub struct MountHandle {
    container: Element,
    roots: Vec<Node>,
    effects: Vec<Effect>,
    listeners: Vec<Listener>,
}

impl MountHandle {
    /// Remove all nodes, listeners and effects created by `mount`.
    pub fn unmount(self) {
        drop(self);
    }
}

impl Drop for MountHandle {
    fn drop(&mut self) {
        for listener in &self.listeners {
            let _ = listener.target.remove_event_listener_with_callback(
                &listener.event,
                listener.closure.as_ref().unchecked_ref(),
            );
        }
        self.listeners.clear();
        self.effects.clear();
        for root in &self.roots {
            if root.parent_node().as_ref() == Some(&self.container) {
                let _ = self.container.remove_child(root);
            }
        }
    }
}

struct MountState {
    document: Document,
    effects: Vec<Effect>,
    listeners: Vec<Listener>,
}

impl MountState {
    fn render_into(&mut self, view: &View, parent: &Element) -> Result<(), JsValue> {
        match view {
            View::Text(value) => {
                let node = self.document.create_text_node(value);
                parent.append_child(&node)?;
            }
            View::DynamicText(value) => {
                let node = self.document.create_text_node("");
                parent.append_child(&node)?;
                let value = Rc::clone(value);
                let node = node.clone();
                let effect = use_effect(move || {
                    node.set_text_content(Some(&value()));
                });
                self.effects.push(effect);
            }
            View::Element(element) => self.render_element(element, parent)?,
            View::Fragment(children) => {
                for child in children {
                    self.render_into(child, parent)?;
                }
            }
        }
        Ok(())
    }

    fn render_element(&mut self, view: &ElementView, parent: &Element) -> Result<(), JsValue> {
        let element = self.document.create_element(&view.tag)?;
        for attribute in &view.attributes {
            self.render_attribute(&element, attribute)?;
        }
        for event in &view.events {
            self.render_event(&element, event)?;
        }
        for child in &view.children {
            self.render_into(child, &element)?;
        }
        parent.append_child(&element)?;
        Ok(())
    }

    fn render_attribute(
        &mut self,
        element: &Element,
        attribute: &Attribute,
    ) -> Result<(), JsValue> {
        match attribute {
            Attribute::Static { name, value } => {
                element.set_attribute(name, value)?;
            }
            Attribute::Boolean { name } => {
                element.set_attribute(name, "")?;
            }
            Attribute::Dynamic { name, value } => {
                let element = element.clone();
                let name = name.clone();
                let value = Rc::clone(value);
                let effect = use_effect(move || {
                    let _ = element.set_attribute(&name, &value());
                });
                self.effects.push(effect);
            }
        }
        Ok(())
    }

    fn render_event(&mut self, element: &Element, event: &EventBinding) -> Result<(), JsValue> {
        let handler = Rc::clone(&event.handler);
        let closure = Closure::<dyn FnMut(Event)>::new(move |event: Event| handler(event));
        element.add_event_listener_with_callback(&event.name, closure.as_ref().unchecked_ref())?;
        self.listeners.push(Listener {
            target: element.clone().into(),
            event: event.name.clone(),
            closure,
        });
        Ok(())
    }
}

/// Mount a view inside `container`. The returned handle owns effects/listeners.
pub fn mount(view: View, container: Element) -> Result<MountHandle, JsValue> {
    let document = container
        .owner_document()
        .ok_or_else(|| JsValue::from_str("container must belong to a document"))?;
    let container_node: &Node = container.unchecked_ref();
    let roots_before = container_node.child_nodes().length();
    let mut state = MountState {
        document,
        effects: Vec::new(),
        listeners: Vec::new(),
    };
    state.render_into(&view, &container)?;

    let children = container_node.child_nodes();
    let roots = (roots_before..children.length())
        .filter_map(|index| children.item(index))
        .collect();

    Ok(MountHandle {
        container,
        roots,
        effects: state.effects,
        listeners: state.listeners,
    })
}

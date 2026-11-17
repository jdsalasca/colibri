use colibri_core::{use_effect, with_context_scope, without_effect_scope, Effect};
use colibri_ssr::validate_attribute;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::closure::Closure;
use wasm_bindgen::JsCast;
use wasm_bindgen::JsValue;
use web_sys::{Document, Element, Event, EventTarget, Node, Text};

struct KeyedEntry {
    handle: MountHandle,
    roots: Vec<Node>,
}

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
    /// A fragment rebuilt when a tracked signal changes.
    DynamicFragment(Rc<dyn Fn() -> Vec<View>>),
    /// A keyed fragment: existing keys keep their DOM nodes on reorder.
    KeyedFragment(Rc<dyn Fn() -> Vec<KeyedView>>),
    /// A view with component-owned effects that live until unmount.
    Scoped {
        view: Box<View>,
        effects: Rc<RefCell<Option<Vec<Effect>>>>,
    },
}

/// A keyed item for [`View::KeyedFragment`].
pub struct KeyedView {
    /// Stable identity within the list.
    pub key: String,
    /// Rendered item.
    pub view: View,
}

/// A callback that receives the created element once (React-style ref).
pub type ElementRef = Rc<dyn Fn(&Element)>;

/// An HTML element in the runtime view tree.
pub struct ElementView {
    /// Tag name, e.g. `button`.
    pub tag: String,
    /// Static, boolean or dynamic attributes.
    pub attributes: Vec<Attribute>,
    /// Event listeners.
    pub events: Vec<EventBinding>,
    /// Callbacks invoked with the element after creation or hydration.
    pub refs: Vec<ElementRef>,
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

fn clear_listeners(listeners: &mut Vec<Listener>) {
    for listener in listeners.iter() {
        let _ = listener.target.remove_event_listener_with_callback(
            &listener.event,
            listener.closure.as_ref().unchecked_ref(),
        );
    }
    listeners.clear();
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

    /// Move ownership of the roots to another container after reparenting.
    pub(crate) fn adopt(&mut self, container: Element) {
        self.container = container;
    }
}

thread_local! {
    static PENDING_REFS: RefCell<Vec<(ElementRef, Element)>> = const { RefCell::new(Vec::new()) };
}

fn queue_ref(callback: &ElementRef, element: &Element) {
    PENDING_REFS.with(|queue| {
        queue
            .borrow_mut()
            .push((Rc::clone(callback), element.clone()));
    });
}

/// Run queued element refs; root mounts and connected dynamic updates call this.
fn flush_refs() {
    let queued = PENDING_REFS.with(|queue| std::mem::take(&mut *queue.borrow_mut()));
    for (callback, element) in queued {
        callback(&element);
    }
}

fn queued_refs() -> usize {
    PENDING_REFS.with(|queue| queue.borrow().len())
}

fn discard_refs_after(keep: usize) {
    PENDING_REFS.with(|queue| queue.borrow_mut().truncate(keep));
}

impl Drop for MountHandle {
    fn drop(&mut self) {
        clear_listeners(&mut self.listeners);
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
            View::DynamicFragment(render) => self.render_dynamic(render, parent)?,
            View::KeyedFragment(render) => self.render_keyed(render, parent)?,
            View::Scoped { view, effects } => {
                if let Some(effects) = effects.borrow_mut().take() {
                    self.effects.extend(effects);
                }
                self.render_into(view, parent)?;
            }
        }
        Ok(())
    }

    fn render_keyed(
        &mut self,
        render: &Rc<dyn Fn() -> Vec<KeyedView>>,
        parent: &Element,
    ) -> Result<(), JsValue> {
        let start = self.document.create_comment("colibri:keyed:start");
        let end = self.document.create_comment("colibri:keyed:end");
        let start_node: &Node = start.unchecked_ref();
        let end_node: &Node = end.unchecked_ref();
        parent.append_child(start_node)?;
        parent.append_child(end_node)?;

        let render = Rc::clone(render);
        let parent = parent.clone();
        let end_anchor: Node = end.unchecked_ref::<Node>().clone();
        let current: Rc<RefCell<HashMap<String, KeyedEntry>>> =
            Rc::new(RefCell::new(HashMap::new()));
        let slot = Rc::clone(&current);
        let effect = use_effect(move || {
            let Some(document) = parent.owner_document() else {
                return;
            };
            let active = document.active_element();
            let mut old = slot.borrow_mut();
            let mut next = HashMap::new();
            for keyed in with_context_scope(|| render()) {
                let KeyedView { key, view } = keyed;
                if next.contains_key(&key) {
                    web_sys::console::error_1(&JsValue::from_str(&format!("duplicate key: {key}")));
                    continue;
                }
                if let Some(entry) = old.remove(&key) {
                    for node in &entry.roots {
                        let _ = parent.insert_before(node, Some(&end_anchor));
                    }
                    next.insert(key, entry);
                    continue;
                }
                let Ok(temp) = document.create_element("div") else {
                    continue;
                };
                match without_effect_scope(|| mount_inner(view, temp.clone(), false)) {
                    Ok(mut handle) => {
                        let children = temp.child_nodes();
                        let mut roots = Vec::new();
                        for index in 0..children.length() {
                            if let Some(node) = children.item(index) {
                                roots.push(node);
                            }
                        }
                        for node in &roots {
                            let _ = parent.insert_before(node, Some(&end_anchor));
                        }
                        handle.adopt(parent.clone());
                        next.insert(key, KeyedEntry { handle, roots });
                    }
                    Err(error) => web_sys::console::error_1(&error),
                }
            }
            for (_, entry) in old.drain() {
                entry.handle.unmount();
            }
            *old = next;
            if parent.is_connected() {
                flush_refs();
            }
            if let Some(active) = active {
                let node: &Node = active.unchecked_ref();
                if node.is_connected() {
                    if let Some(element) = active.dyn_ref::<web_sys::HtmlElement>() {
                        let _ = element.focus();
                    }
                }
            }
        });
        self.effects.push(effect);
        Ok(())
    }

    fn render_dynamic(
        &mut self,
        render: &Rc<dyn Fn() -> Vec<View>>,
        parent: &Element,
    ) -> Result<(), JsValue> {
        let start = self.document.create_comment("colibri:start");
        let end = self.document.create_comment("colibri:end");
        let start_node: &Node = start.unchecked_ref();
        let end_node: &Node = end.unchecked_ref();
        parent.append_child(start_node)?;
        parent.append_child(end_node)?;

        let render = Rc::clone(render);
        let parent = parent.clone();
        let end_anchor: Node = end.unchecked_ref::<Node>().clone();
        let current: Rc<RefCell<Option<MountHandle>>> = Rc::new(RefCell::new(None));
        let slot = Rc::clone(&current);
        let effect = use_effect(move || {
            let Some(document) = parent.owner_document() else {
                return;
            };
            let Ok(temp) = document.create_element("div") else {
                return;
            };
            match without_effect_scope(|| {
                mount_inner(
                    View::Fragment(with_context_scope(|| render())),
                    temp.clone(),
                    false,
                )
            }) {
                Ok(mut handle) => {
                    if let Some(old) = slot.borrow_mut().take() {
                        old.unmount();
                    }
                    while let Some(child) = temp.first_child() {
                        if parent.insert_before(&child, Some(&end_anchor)).is_err() {
                            break;
                        }
                    }
                    handle.adopt(parent.clone());
                    *slot.borrow_mut() = Some(handle);
                    if parent.is_connected() {
                        flush_refs();
                    }
                }
                Err(error) => web_sys::console::error_1(&error),
            }
        });
        self.effects.push(effect);
        Ok(())
    }

    fn rollback(&mut self, container: &Element, roots_before: usize) {
        clear_listeners(&mut self.listeners);
        self.effects.clear();
        let container_node: &Node = container.unchecked_ref();
        let children = container_node.child_nodes();
        while children.length() as usize > roots_before {
            let Some(child) = children.item(roots_before as u32) else {
                break;
            };
            let _ = container.remove_child(&child);
        }
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
        for callback in &view.refs {
            queue_ref(callback, &element);
        }
        Ok(())
    }

    fn render_attribute(
        &mut self,
        element: &Element,
        attribute: &Attribute,
    ) -> Result<(), JsValue> {
        match attribute {
            Attribute::Static { name, value } => {
                validate_dom_attribute(name, Some(value))?;
                element.set_attribute(name, value)?;
            }
            Attribute::Boolean { name } => {
                validate_dom_attribute(name, None)?;
                element.set_attribute(name, "")?;
            }
            Attribute::Dynamic { name, value } => {
                let element = element.clone();
                let name = name.clone();
                let value = Rc::clone(value);
                let effect = use_effect(move || {
                    let next = value();
                    if let Err(error) = validate_dom_attribute(&name, Some(&next)) {
                        web_sys::console::error_1(&error);
                        return;
                    }
                    let _ = element.set_attribute(&name, &next);
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

    fn hydrate_into(&mut self, view: &View, cursor: &mut ChildCursor) -> Result<(), JsValue> {
        match view {
            View::Text(value) => {
                if !value.is_empty() {
                    let node = cursor.next_text(value, &self.document)?;
                    node.set_text_content(Some(value));
                } else {
                    cursor.take_empty_text();
                }
            }
            View::DynamicText(value) => {
                let current = value();
                let node = cursor.next_text(&current, &self.document)?;
                let value = Rc::clone(value);
                let node: Node = node.unchecked_ref::<Node>().clone();
                let effect = use_effect(move || {
                    node.set_text_content(Some(&value()));
                });
                self.effects.push(effect);
            }
            View::Element(element) => {
                let node = cursor.next(&format!("<{}>", element.tag))?;
                let existing: Element = node.dyn_into().map_err(|_| {
                    JsValue::from_str(&format!("hydration mismatch: expected <{}>", element.tag))
                })?;
                if !existing.tag_name().eq_ignore_ascii_case(&element.tag) {
                    return Err(JsValue::from_str(&format!(
                        "hydration mismatch: expected <{}>, found <{}>",
                        element.tag,
                        existing.tag_name()
                    )));
                }
                for attribute in &element.attributes {
                    self.render_attribute(&existing, attribute)?;
                }
                for event in &element.events {
                    self.render_event(&existing, event)?;
                }
                let mut children = ChildCursor::new(&existing);
                for child in &element.children {
                    self.hydrate_into(child, &mut children)?;
                }
                children.finish()?;
                for callback in &element.refs {
                    queue_ref(callback, &existing);
                }
            }
            View::Fragment(children) => {
                for child in children {
                    self.hydrate_into(child, cursor)?;
                }
            }
            View::Scoped { view, effects } => {
                if let Some(effects) = effects.borrow_mut().take() {
                    self.effects.extend(effects);
                }
                self.hydrate_into(view, cursor)?;
            }
            View::DynamicFragment(_) | View::KeyedFragment(_) => {
                return Err(JsValue::from_str(
                    "hydration does not support Dynamic or Keyed fragments yet",
                ));
            }
        }
        Ok(())
    }
}

/// Walks existing DOM children in order while hydrating a view.
struct ChildCursor {
    parent: Element,
    nodes: Vec<Node>,
    index: usize,
}

impl ChildCursor {
    fn new(parent: &Element) -> Self {
        let children = parent.child_nodes();
        let mut nodes = Vec::with_capacity(children.length() as usize);
        for index in 0..children.length() {
            if let Some(node) = children.item(index) {
                nodes.push(node);
            }
        }
        Self {
            parent: parent.clone(),
            nodes,
            index: 0,
        }
    }

    fn next(&mut self, expected: &str) -> Result<Node, JsValue> {
        let node =
            self.nodes.get(self.index).cloned().ok_or_else(|| {
                JsValue::from_str(&format!("hydration mismatch: missing {expected}"))
            })?;
        self.index += 1;
        Ok(node)
    }

    fn finish(&self) -> Result<(), JsValue> {
        if self.index == self.nodes.len() {
            return Ok(());
        }
        Err(JsValue::from_str(&format!(
            "hydration mismatch: {} extra node(s)",
            self.nodes.len() - self.index
        )))
    }

    /// Consume (or split) the next text node so this view piece owns exactly
    /// `expected` UTF-16 units. Browsers coalesce adjacent SSR text into one node.
    fn next_text(&mut self, expected: &str, document: &Document) -> Result<Text, JsValue> {
        let wanted = expected.encode_utf16().count() as u32;
        if !self.peek_is_text() {
            if wanted != 0 {
                return Err(JsValue::from_str(
                    "hydration mismatch: expected a text node",
                ));
            }
            let node = document.create_text_node("");
            let node_ref: &Node = node.unchecked_ref();
            match self.nodes.get(self.index) {
                Some(anchor) => {
                    self.parent.insert_before(node_ref, Some(anchor))?;
                }
                None => {
                    self.parent.append_child(node_ref)?;
                }
            }
            self.nodes.insert(self.index, node_ref.clone());
        }

        let node = self.next("text")?;
        expect_text(&node)?;
        let text: Text = node.unchecked_ref::<Text>().clone();
        if text.length() > wanted {
            let rest = text.split_text(wanted)?;
            let rest_node: Node = rest.unchecked_ref::<Node>().clone();
            self.nodes.insert(self.index, rest_node);
        }
        Ok(text)
    }

    fn peek_is_text(&self) -> bool {
        self.nodes
            .get(self.index)
            .is_some_and(|node| node.node_type() == Node::TEXT_NODE)
    }

    /// SSR emits empty text as nothing, so an empty static text is optional.
    fn take_empty_text(&mut self) {
        let empty = self
            .nodes
            .get(self.index)
            .filter(|node| node.node_type() == Node::TEXT_NODE)
            .and_then(|node| node.text_content())
            .is_some_and(|text| text.is_empty());
        if empty {
            self.index += 1;
        }
    }
}

fn expect_text(node: &Node) -> Result<(), JsValue> {
    if node.node_type() == Node::TEXT_NODE {
        Ok(())
    } else {
        Err(JsValue::from_str(
            "hydration mismatch: expected a text node",
        ))
    }
}

fn validate_dom_attribute(name: &str, value: Option<&str>) -> Result<(), JsValue> {
    validate_attribute(name, value)
        .map_err(|kind| JsValue::from_str(&format!("unsafe attribute `{name}`: {}", kind.as_str())))
}

/// Attach component effects so the mount owns their lifetime.
pub fn attach_effects(view: View, effects: Vec<Effect>) -> View {
    if effects.is_empty() {
        view
    } else {
        View::Scoped {
            view: Box::new(view),
            effects: Rc::new(RefCell::new(Some(effects))),
        }
    }
}

/// Mount a view inside `container`. The returned handle owns effects/listeners.
pub fn mount(view: View, container: Element) -> Result<MountHandle, JsValue> {
    without_effect_scope(|| mount_inner(view, container, true))
}

fn mount_inner(view: View, container: Element, flush: bool) -> Result<MountHandle, JsValue> {
    let document = container
        .owner_document()
        .ok_or_else(|| JsValue::from_str("container must belong to a document"))?;
    let container_node: &Node = container.unchecked_ref();
    let roots_before = container_node.child_nodes().length();
    let refs_before = queued_refs();
    let mut state = MountState {
        document,
        effects: Vec::new(),
        listeners: Vec::new(),
    };
    if let Err(error) = state.render_into(&view, &container) {
        state.rollback(&container, roots_before as usize);
        discard_refs_after(refs_before);
        return Err(error);
    }

    let children = container_node.child_nodes();
    let roots = (roots_before..children.length())
        .filter_map(|index| children.item(index))
        .collect();

    let handle = MountHandle {
        container,
        roots,
        effects: state.effects,
        listeners: state.listeners,
    };
    if flush {
        flush_refs();
    }
    Ok(handle)
}

/// Adopt server-rendered DOM instead of rebuilding it.
///
/// The existing children of `container` must match `view` element by element;
/// text and attributes are corrected in place, so node identity survives.
/// `Dynamic` and `Keyed` fragments are not hydratable yet and return an error.
pub fn hydrate(view: View, container: Element) -> Result<MountHandle, JsValue> {
    without_effect_scope(|| {
        let document = container
            .owner_document()
            .ok_or_else(|| JsValue::from_str("container must belong to a document"))?;
        let container_node: &Node = container.unchecked_ref();
        let refs_before = queued_refs();
        let mut state = MountState {
            document,
            effects: Vec::new(),
            listeners: Vec::new(),
        };
        let mut cursor = ChildCursor::new(&container);
        if let Err(error) = state.hydrate_into(&view, &mut cursor) {
            drop(state);
            discard_refs_after(refs_before);
            return Err(error);
        }
        if let Err(error) = cursor.finish() {
            drop(state);
            discard_refs_after(refs_before);
            return Err(error);
        }

        let children = container_node.child_nodes();
        let roots = (0..children.length())
            .filter_map(|index| children.item(index))
            .collect();

        let handle = MountHandle {
            container,
            roots,
            effects: state.effects,
            listeners: state.listeners,
        };
        flush_refs();
        Ok(handle)
    })
}

/// Hydrate an island found by `selector`; `None` when the page has no such node.
///
/// Islands are independent interactive roots inside a server-rendered page: each
/// one hydrates its own subtree and is torn down by its own [`MountHandle`].
pub fn hydrate_island(
    selector: &str,
    view: impl FnOnce() -> View,
) -> Result<Option<MountHandle>, JsValue> {
    let document = web_sys::window()
        .and_then(|window| window.document())
        .ok_or_else(|| JsValue::from_str("no document"))?;
    let Some(container) = document.query_selector(selector)? else {
        return Ok(None);
    };
    hydrate(view(), container).map(Some)
}

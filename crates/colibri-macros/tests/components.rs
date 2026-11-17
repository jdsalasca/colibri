//! Functional components, typed props, null safety and composition.

use colibri_ir::Node;
use colibri_macros::{component, view};
use colibri_ssr::render_static;

#[component]
fn Greeting(name: String) -> Node {
    view! { <p>"Hola " {name}</p> }
}

#[component]
fn Card(title: String, count: i32) -> Node {
    view! { <article><h1>{title}</h1><span>{count}</span></article> }
}

#[component]
fn MaybeName(name: Option<String>) -> Node {
    view! { <p>{name.unwrap_or_else(|| String::from("sin nombre"))}</p> }
}

#[component]
fn Panel(title: String, children: Vec<Node>) -> Node {
    view! {
        <section>
            <h2>{title}</h2>
            <Fragment nodes={children}/>
        </section>
    }
}

#[component]
fn ThemeLabel() -> Node {
    let theme = colibri_core::use_context::<&'static str>().unwrap_or("light");
    view! { <span>{theme}</span> }
}

#[component]
fn ThemedSection() -> Node {
    colibri_core::provide_context("dark");
    view! { <section><ThemeLabel/></section> }
}

#[component]
fn UnscopedSection() -> Node {
    view! { <div><ThemeLabel/></div> }
}

#[test]
fn renders_component_with_typed_props() {
    let node = Greeting(GreetingProps {
        name: String::from("Ada"),
    });
    assert_eq!(render_static(&node).unwrap(), "<p>Hola Ada</p>");
}

#[test]
fn composes_components_and_dynamic_values() {
    let node = view! { <section><Card title="Panel" count={3}/></section> };
    assert_eq!(
        render_static(&node).unwrap(),
        "<section><article><h1>Panel</h1><span>3</span></article></section>"
    );
}

#[test]
fn option_props_are_null_safe() {
    assert_eq!(
        render_static(&MaybeName(MaybeNameProps {
            name: Some(String::from("Ada"))
        }))
        .unwrap(),
        "<p>Ada</p>"
    );
    assert_eq!(
        render_static(&MaybeName(MaybeNameProps { name: None })).unwrap(),
        "<p>sin nombre</p>"
    );
}

#[test]
fn context_flows_to_children_and_is_scoped() {
    assert_eq!(
        render_static(&ThemedSection(ThemedSectionProps {})).unwrap(),
        "<section><span>dark</span></section>"
    );
    assert_eq!(
        render_static(&UnscopedSection(UnscopedSectionProps {})).unwrap(),
        "<div><span>light</span></div>"
    );
}

#[test]
fn components_accept_children_and_keep_order() {
    let node = view! {
        <Panel title="Datos">
            <p>"uno"</p>
            <p>"dos"</p>
        </Panel>
    };
    assert_eq!(
        render_static(&node).unwrap(),
        "<section><h2>Datos</h2><p>uno</p><p>dos</p></section>"
    );
}

//! Parity between the two source syntaxes for the static subset (M04).
//!
//! IR spans are ignored: the Rust macro cannot read byte offsets on stable.

use colibri_ir::{Attribute, Node};
use colibri_macros::view;
use colibri_parser::{lower, parse};
use colibri_ssr::render_static;

fn shape(node: &Node) -> String {
    match node {
        Node::Text { value, .. } => format!("text({value:?})"),
        Node::DynamicText { expression, .. } => format!("dynamic({expression:?})"),
        Node::Fragment { children, .. } => format!(
            "fragment[{}]",
            children.iter().map(shape).collect::<Vec<_>>().join(",")
        ),
        Node::Element(element) => {
            let attributes = element
                .attributes
                .iter()
                .map(|attribute| match attribute {
                    Attribute::Static { name, value, .. } => format!("{name}={value:?}"),
                    Attribute::Boolean { name, .. } => name.clone(),
                    Attribute::Dynamic {
                        name, expression, ..
                    } => {
                        format!("{name}?={expression:?}")
                    }
                    Attribute::Event { name, handler, .. } => {
                        format!("on:{name}={handler:?}")
                    }
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "element({},[{}],[{}])",
                element.tag,
                attributes,
                element
                    .children
                    .iter()
                    .map(shape)
                    .collect::<Vec<_>>()
                    .join(",")
            )
        }
    }
}

fn sfc(source: &str) -> (String, String) {
    let component = parse("parity.colibri", source).expect("valid SFC");
    let ir = lower("Parity", &component).expect("lowerable SFC");
    let html = render_static(&ir.root).expect("static SFC");
    (shape(&ir.root), html)
}

fn rust(node: &Node) -> (String, String) {
    let html = render_static(node).expect("static Rust view");
    (shape(node), html)
}

#[test]
fn static_text_attributes_and_unicode_match() {
    let sfc_source = "<template><article class=\"card\" hidden><h1>Título</h1><p>Hola & adiós</p></article></template>";
    let rust_node = view! {
        <article class="card" hidden>
            <h1>"Título"</h1>
            <p>"Hola & adiós"</p>
        </article>
    };

    let (sfc_shape, sfc_html) = sfc(sfc_source);
    let (rust_shape, rust_html) = rust(&rust_node);
    assert_eq!(sfc_shape, rust_shape);
    assert_eq!(sfc_html, rust_html);
}

#[test]
fn nested_void_and_boolean_elements_match() {
    let sfc_source =
        "<template><section><h2>Sección</h2><input type=\"text\"><br></section></template>";
    let rust_node = view! {
        <section>
            <h2>"Sección"</h2>
            <input type="text">
            <br>
        </section>
    };

    let (sfc_shape, sfc_html) = sfc(sfc_source);
    let (rust_shape, rust_html) = rust(&rust_node);
    assert_eq!(sfc_shape, rust_shape);
    assert_eq!(sfc_html, rust_html);
}

#[test]
fn hostile_attribute_values_escape_identically() {
    let sfc_source = "<template><p title=\"<script>alert(1)</script>\">x & y</p></template>";
    let rust_node = view! { <p title="<script>alert(1)</script>">"x & y"</p> };

    let (sfc_shape, sfc_html) = sfc(sfc_source);
    let (rust_shape, rust_html) = rust(&rust_node);
    assert_eq!(sfc_shape, rust_shape);
    assert_eq!(sfc_html, rust_html);
    assert!(!rust_html.contains("<script>"));
}

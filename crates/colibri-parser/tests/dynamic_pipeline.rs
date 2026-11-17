//! Dynamic pipeline: `.colibri` -> AST -> IR -> HTML with an explicit resolver.

use colibri_parser::{lower, parse};
use colibri_ssr::{render_dynamic, RenderErrorKind};
use std::collections::HashMap;

fn render(
    source: &str,
    values: &HashMap<&str, String>,
) -> Result<String, colibri_ssr::RenderError> {
    let component = parse("dynamic.colibri", source).expect("valid SFC");
    let ir = lower("Dynamic", &component).expect("lowerable SFC");
    render_dynamic(&ir.root, |expression| values.get(expression).cloned())
}

#[test]
fn renders_dynamic_sfc_with_resolved_values() {
    let source = "<template><article><h1>{title}</h1><p>Hola {name}</p><a href={url}>perfil</a></article></template>";
    let values = HashMap::from([
        ("title", "Colibrí".to_string()),
        ("name", "Ada".to_string()),
        ("url", "https://example.com".to_string()),
    ]);
    assert_eq!(
        render(source, &values).unwrap(),
        "<article><h1>Colibrí</h1><p>Hola Ada</p><a href=\"https://example.com\">perfil</a></article>"
    );
}

#[test]
fn missing_values_report_the_expression_span() {
    let source = "<template><p>Hola {name}</p></template>";
    let error = render(source, &HashMap::new()).unwrap_err();
    assert_eq!(error.kind, RenderErrorKind::MissingValue);
    assert_eq!(error.span.text(source), Some("{name}"));
}

#[test]
fn resolved_values_are_escaped() {
    let source = "<template><p>{name}</p></template>";
    let values = HashMap::from([("name", "<script>alert(1)</script>".to_string())]);
    let html = render(source, &values).unwrap();
    assert_eq!(html, "<p>&lt;script&gt;alert(1)&lt;/script&gt;</p>");
}

#[test]
fn resolved_urls_are_still_validated() {
    let source = "<template><a href={url}>x</a></template>";
    let values = HashMap::from([("url", " javascript:alert(1)".to_string())]);
    let error = render(source, &values).unwrap_err();
    assert_eq!(error.kind, RenderErrorKind::UnsafeAttribute);
    assert_eq!(error.span.text(source), Some("href={url}"));
}

#[test]
fn events_are_not_server_rendered() {
    let source = "<template><button on:click={go}>x</button></template>";
    let error = render(source, &HashMap::new()).unwrap_err();
    assert_eq!(error.kind, RenderErrorKind::DynamicContent);
    assert_eq!(error.span.text(source), Some("on:click={go}"));
}

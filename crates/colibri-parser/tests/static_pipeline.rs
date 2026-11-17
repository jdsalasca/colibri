//! End-to-end static pipeline: `.colibri` source -> AST -> IR -> HTML.

use colibri_parser::{lower, parse};
use colibri_ssr::{render_static, RenderErrorKind};

fn render(source: &str) -> Result<String, colibri_ssr::RenderError> {
    let component = parse("pipeline.colibri", source).expect("valid SFC");
    let ir = lower("Pipeline", &component).expect("lowerable SFC");
    render_static(&ir.root)
}

#[test]
fn renders_static_sfc_to_html() {
    let html = render(
        "<template><article class=\"card\" hidden><h1>Título</h1><p>Hola & adiós</p><input type=\"text\"></article></template>",
    )
    .unwrap();
    assert_eq!(
        html,
        "<article class=\"card\" hidden><h1>Título</h1><p>Hola &amp; adiós</p><input type=\"text\"></article>"
    );
}

#[test]
fn dynamic_interpolation_reports_its_span() {
    let source = "<template><p>Hola {name}</p></template>";
    let error = render(source).unwrap_err();
    assert_eq!(error.kind, RenderErrorKind::DynamicContent);
    assert_eq!(error.span.text(source), Some("{name}"));
}

#[test]
fn dynamic_attributes_and_events_report_their_spans() {
    let source = "<template><a href={url}>x</a></template>";
    let error = render(source).unwrap_err();
    assert_eq!(error.kind, RenderErrorKind::DynamicContent);
    assert_eq!(error.span.text(source), Some("href={url}"));

    let source = "<template><button on:click={go}>x</button></template>";
    let error = render(source).unwrap_err();
    assert_eq!(error.kind, RenderErrorKind::DynamicContent);
    assert_eq!(error.span.text(source), Some("on:click={go}"));
}

#[test]
fn drops_comments_and_preserves_text() {
    let html = render("<template><p>a</p><!-- nota --></template>").unwrap();
    assert_eq!(html, "<p>a</p>");
}

#[test]
fn rejects_unsafe_urls_end_to_end() {
    let source = "<template><a href=\"javascript:alert(1)\">x</a></template>";
    let error = render(source).unwrap_err();
    assert_eq!(error.kind, RenderErrorKind::UnsafeAttribute);
    assert_eq!(
        error.span.text(source),
        Some("href=\"javascript:alert(1)\"")
    );
}

#[test]
fn unresolved_components_are_not_emitted_as_html() {
    let error = render("<template><Card title=\"x\"/></template>").unwrap_err();
    assert_eq!(error.kind, RenderErrorKind::UnresolvedComponent);
}

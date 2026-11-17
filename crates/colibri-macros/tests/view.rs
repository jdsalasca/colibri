//! Compile-pass coverage for `view!` and the static renderer.

use colibri_macros::view;
use colibri_ssr::render_static;

#[test]
fn renders_static_view() {
    let node = view! {
        <article class="card" hidden>
            "Título"
            <p>"Hola & adiós"</p>
            <input type="text">
        </article>
    };
    assert_eq!(
        render_static(&node).unwrap(),
        "<article class=\"card\" hidden>Título<p>Hola &amp; adiós</p><input type=\"text\"></article>"
    );
}

#[test]
fn multiple_roots_render_in_order() {
    let node = view! { <p>"a"</p> <p>"b"</p> };
    assert_eq!(render_static(&node).unwrap(), "<p>a</p><p>b</p>");
}

#[test]
fn void_elements_do_not_consume_following_nodes() {
    let node = view! { <div><br><span>"x"</span></div> };
    assert_eq!(
        render_static(&node).unwrap(),
        "<div><br><span>x</span></div>"
    );
}

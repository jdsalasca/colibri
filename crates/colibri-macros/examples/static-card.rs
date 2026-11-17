//! Static Rust view rendered to HTML: `cargo run -p colibri-macros --example static-card`.
use colibri_macros::view;
use colibri_ssr::render_static;

fn main() {
    let node = view! {
        <article class="card">
            <h1>"Colibrí"</h1>
            <p>"HTML estático desde la ruta Rust."</p>
        </article>
    };
    match render_static(&node) {
        Ok(html) => println!("{html}"),
        Err(error) => eprintln!("{error}"),
    }
}

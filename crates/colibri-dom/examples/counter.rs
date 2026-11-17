//! WASM counter: `cargo build --target wasm32-unknown-unknown --example counter --release`.
#[cfg(target_arch = "wasm32")]
mod wasm {
    use colibri_core::{use_memo, use_state, Resource};
    use colibri_dom::{mount, use_async, View};
    use colibri_macros::{component, dom};
    use gloo_timers::future::TimeoutFuture;
    use wasm_bindgen::prelude::*;

    #[component]
    pub fn Panel(title: String, children: Vec<View>) -> View {
        dom! {
            <section class="panel">
                <h2>{title}</h2>
                <Fragment nodes={children}/>
            </section>
        }
    }

    #[wasm_bindgen(start)]
    pub fn start() -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("no document"))?;
        let container = document
            .get_element_by_id("app")
            .ok_or_else(|| JsValue::from_str("missing #app"))?;

        let count = use_state(0_i32);
        let count_text = count.clone();
        let count_click = count.clone();
        let doubled = use_memo({
            let count = count.clone();
            move || count.get() * 2
        });
        let manuals = use_async(async {
            TimeoutFuture::new(500).await;
            Ok::<_, String>(42_i32)
        });
        let manuals_text = manuals.clone();
        let visible = use_state(true);
        let visible_click = visible.clone();
        let visible_render = visible.clone();
        let conditional = move || {
            if visible_render.get() {
                vec![dom! { <p class="conditional">"bloque condicional visible"</p> }]
            } else {
                Vec::new()
            }
        };

        let view = dom! {
            <main class="counter">
                <Panel title="Contador reactivo">
                    <p class="eyebrow">"COLIBRÍ · WASM"</p>
                    <p class="value">{count_text.get()}</p>
                    <p class="double">"doble: " {doubled.get()}</p>
                    <p class="async">{match manuals_text.get() {
                        Resource::Loading => String::from("cargando catálogo…"),
                        Resource::Ready(total) => format!("{total} manuales listos"),
                        Resource::Failed(error) => format!("error: {error}"),
                    }}</p>
                    <Dynamic nodes={conditional}/>
                    <button type="button" class="toggle" on:click={move |_event| visible_click.update(|v| *v = !*v)}>
                        "toggle bloque"
                    </button>
                    <button type="button" on:click={move |_event| count_click.update(|n| *n += 1)}>
                        "+1"
                    </button>
                </Panel>
            </main>
        };

        let handle = mount(view, container)?;
        std::mem::forget(handle);
        Ok(())
    }
}

fn main() {}

//! WASM counter: `cargo build --target wasm32-unknown-unknown --example counter --release`.
#[cfg(target_arch = "wasm32")]
mod wasm {
    use colibri_core::{
        provide_context, use_context, use_effect, use_memo, use_reducer, use_state, Resource,
        Signal,
    };
    use colibri_dom::{hydrate_island, mount, use_async, use_hydrated_state, KeyedView, View};
    use colibri_macros::{component, dom};
    use gloo_timers::future::TimeoutFuture;
    use wasm_bindgen::prelude::*;
    use wasm_bindgen::JsCast;

    #[component]
    pub fn Panel(title: String, children: Vec<View>) -> View {
        dom! {
            <section class="panel">
                <h2>{title}</h2>
                <Fragment nodes={children}/>
            </section>
        }
    }

    #[component]
    pub fn TitleEffect(count: Signal<i32>) -> View {
        use_effect({
            let count = count.clone();
            move || {
                if let Some(document) = web_sys::window().and_then(|window| window.document()) {
                    document.set_title(&format!("Contador {}", count.get()));
                }
            }
        });
        View::Fragment(Vec::new())
    }

    #[component]
    pub fn HydratedCounter() -> View {
        let count = use_hydrated_state("counter", 0_i32);
        let value = count.clone();
        let label = count.clone();
        let click = count.clone();
        dom! {
            <section class="hydrated">
                <p class="label">"contador: " {label.get()}</p>
                <p class="value">{value.get()}</p>
                <button type="button" class="increment" on:click={move |_event| click.update(|n| *n += 1)}>
                    "+1"
                </button>
            </section>
        }
    }

    #[component]
    pub fn HydratedToggle() -> View {
        let on = use_hydrated_state("toggle", false);
        let label = on.clone();
        let class = on.clone();
        let click = on.clone();
        dom! {
            <button
                type="button"
                class={if class.get() { "toggle-island on" } else { "toggle-island off" }}
                on:click={move |_event| click.update(|value| *value = !*value)}
            >
                {if label.get() { "ON" } else { "OFF" }}
            </button>
        }
    }

    #[wasm_bindgen]
    pub fn hydrate_islands_demo() -> Result<(), JsValue> {
        let mut handles = Vec::new();
        if let Some(handle) = hydrate_island("#island-counter", || {
            dom! { <HydratedCounter/> }
        })? {
            handles.push(handle);
        }
        if let Some(handle) = hydrate_island("#island-toggle", || {
            dom! { <HydratedToggle/> }
        })? {
            handles.push(handle);
        }
        let old = ISLANDS.with(|slot| std::mem::replace(&mut *slot.borrow_mut(), handles));
        drop(old);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn unmount_islands() {
        let old = ISLANDS.with(|slot| std::mem::take(&mut *slot.borrow_mut()));
        drop(old);
    }

    #[derive(Clone, Copy, PartialEq)]
    enum Theme {
        Light,
        Dark,
    }

    #[component]
    pub fn ThemeBadge() -> View {
        let theme = use_context::<Theme>().unwrap_or(Theme::Light);
        let label = match theme {
            Theme::Light => "tema claro",
            Theme::Dark => "tema oscuro",
        };
        dom! { <span class="theme-label">{label.to_string()}</span> }
    }

    #[component]
    pub fn ThemePanel() -> View {
        let theme = use_state(Theme::Light);
        let toggle = theme.clone();
        let render = {
            let theme = theme.clone();
            move || {
                let current = theme.get();
                provide_context(current);
                let toggle = toggle.clone();
                let class = match current {
                    Theme::Light => "theme-panel light",
                    Theme::Dark => "theme-panel dark",
                };
                vec![dom! {
                    <div class={class}>
                        <ThemeBadge/>
                        <button
                            type="button"
                            class="theme-toggle"
                            on:click={move |_event| toggle.update(|value| {
                                *value = if *value == Theme::Dark { Theme::Light } else { Theme::Dark };
                            })}
                        >
                            "cambiar tema"
                        </button>
                    </div>
                }]
            }
        };
        dom! { <Dynamic nodes={render}/> }
    }

    #[component]
    pub fn SearchFocus() -> View {
        dom! {
            <input
                class="search"
                type="search"
                placeholder="Buscar manual…"
                ref={move |element| {
                    let input: &web_sys::HtmlElement = element.unchecked_ref();
                    let _ = input.focus();
                }}
            />
        }
    }

    #[wasm_bindgen(start)]
    pub fn start() -> Result<(), JsValue> {
        if web_sys::window()
            .and_then(|window| window.document())
            .and_then(|document| document.get_element_by_id("app"))
            .is_some()
        {
            mount_app()?;
        }
        Ok(())
    }

    #[wasm_bindgen]
    pub fn hydrate_demo() -> Result<(), JsValue> {
        let document = web_sys::window()
            .and_then(|window| window.document())
            .ok_or_else(|| JsValue::from_str("no document"))?;
        let container = document
            .get_element_by_id("hydrated")
            .ok_or_else(|| JsValue::from_str("missing #hydrated"))?;
        let handle = colibri_dom::hydrate(dom! { <HydratedCounter/> }, container)?;
        let old = ROOT.with(|slot| slot.borrow_mut().replace(handle));
        drop(old);
        Ok(())
    }

    thread_local! {
        static ROOT: std::cell::RefCell<Option<colibri_dom::MountHandle>> =
            const { std::cell::RefCell::new(None) };
        static ISLANDS: std::cell::RefCell<Vec<colibri_dom::MountHandle>> =
            const { std::cell::RefCell::new(Vec::new()) };
    }

    #[wasm_bindgen]
    pub fn mount_app() -> Result<(), JsValue> {
        let window = web_sys::window().ok_or_else(|| JsValue::from_str("no window"))?;
        let document = window
            .document()
            .ok_or_else(|| JsValue::from_str("no document"))?;
        let container = document
            .get_element_by_id("app")
            .ok_or_else(|| JsValue::from_str("missing #app"))?;
        let handle = mount(app(), container)?;
        let old = ROOT.with(|slot| slot.borrow_mut().replace(handle));
        drop(old);
        Ok(())
    }

    #[wasm_bindgen]
    pub fn unmount_app() {
        let old = ROOT.with(|slot| slot.borrow_mut().take());
        drop(old);
    }

    #[wasm_bindgen]
    pub fn stats_json() -> String {
        let stats = colibri_core::stats();
        format!(
            "{{\"signals\":{},\"subscriptions\":{},\"observers\":{},\"queued\":{}}}",
            stats.signals, stats.subscriptions, stats.observers, stats.queued
        )
    }

    #[derive(Clone, Copy)]
    enum CounterAction {
        Increment,
    }

    fn app() -> View {
        let counter = use_reducer(0_i32, |value: &i32, _: CounterAction| value + 1);
        let counter_text = counter.clone();
        let counter_click = counter.clone();
        let doubled = use_memo({
            let counter = counter.clone();
            move || counter.get() * 2
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
        let items = use_state(vec![
            String::from("A"),
            String::from("B"),
            String::from("C"),
        ]);
        let reorder = items.clone();
        let keyed_items = items.clone();
        let keyed = move || {
            keyed_items
                .get()
                .into_iter()
                .map(|label| KeyedView {
                    key: label.clone(),
                    view: dom! {
                        <li class="keyed-item">
                            <span class="key-label">{label.clone()}</span>
                            <button type="button" class="item-button">"focus"</button>
                        </li>
                    },
                })
                .collect::<Vec<_>>()
        };

        let view = dom! {
            <main class="counter">
                <TitleEffect count={counter.signal().clone()}/>
                <SearchFocus/>
                <p class="eyebrow">"COLIBRÍ · WASM"</p>
                <Panel title="Contador reactivo">
                    <p class="value">{counter_text.get()}</p>
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
                    <ul class="keyed-list"><Keyed nodes={keyed}/></ul>
                    <button type="button" class="reorder" on:click={move |_event| reorder.update(|v| v.reverse())}>
                        "reordenar"
                    </button>
                    <button type="button" class="increment" on:click={move |_event| counter_click.dispatch(CounterAction::Increment)}>
                        "+1"
                    </button>
                </Panel>
                <ThemePanel/>
            </main>
        };
        view
    }
}

fn main() {}

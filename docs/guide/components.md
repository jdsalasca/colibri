# Componentes funcionales y SSR

API para construir una app Colibrí hoy: funciones Rust con props tipadas, `view!` y render SSR. No hay runtime cliente todavía.

## Definir un componente

```rust
use colibri_ir::Node;
use colibri_macros::{component, view};

#[component]
pub fn Greeting(name: String) -> Node {
    view! { <p>"Hola " {name}</p> }
}
```

`#[component]` genera:

```rust
pub struct GreetingProps { pub name: String }
pub fn Greeting(__colibri_props: GreetingProps) -> Node { /* ... */ }
```

- El nombre del componente debe ser `PascalCase`.
- Los props son los parámetros de la función; se pasan por valor.
- Se admiten tipos owned, incluidos `Option<T>`, `Vec<T>`, structs y enums.
- No se admiten referencias, genéricos, `async`, `const` ni `unsafe` todavía.
- El error de tipo lo da rustc: si un prop es `String`, `1` no compila.

## Componer

```rust
#[component]
pub fn Card(title: String, count: i32) -> Node {
    view! {
        <article class="card">
            <h1>{title}</h1>
            <span>{count}</span>
        </article>
    }
}

#[component]
pub fn Dashboard(total: i32) -> Node {
    view! {
        <main>
            <Card title="Productos" count={total}/>
            <p>"Total: " {total}</p>
        </main>
    }
}
```

Reglas de `view!`:

- Texto literal entre comillas: `"Hola"`.
- Expresión dinámica: `{total}`. Se formatea con `Display`; no hay evaluación de strings.
- Atributos estáticos: `class="card"`.
- Atributos dinámicos: `title={name}`.
- Booleanos: `hidden`.
- Componentes: `<Card title="Productos" count={total}/>`.
- Children: `<Layout title="Panel"><p>"contenido"</p></Layout>`.
- Para insertar `Vec<Node>` dentro de un componente, usa el componente reservado `<Fragment nodes={children}/>`.

### Layout con children

```rust
#[component]
pub fn Layout(title: String, children: Vec<Node>) -> Node {
    view! {
        <html>
            <head><title>{title}</title></head>
            <body>
                <nav>"ManualOps"</nav>
                <Fragment nodes={children}/>
            </body>
        </html>
    }
}

let page = view! {
    <Layout title="Dashboard">
        <h1>"Hola"</h1>
    </Layout>
};
```

`view!` añade automáticamente `children: vec![...]` cuando un componente tiene contenido. El struct de props debe declarar `children: Vec<Node>`. Sin children nombrados/slots todavía.

## Servir con Axum

```rust
use axum::{response::Html, routing::get, Router};
use colibri_ssr::render_static;

async fn index() -> Html<String> {
    let page = Dashboard(DashboardProps { total: 3 });
    Html(render_static(&page).expect("valid page"))
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/", get(index));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```

## Cliente WASM

Los componentes también pueden devolver `View` y montarse con `colibri-dom`:

```rust
use colibri_dom::{mount, View};
use colibri_macros::{component, dom};

#[component]
pub fn Panel(title: String, children: Vec<View>) -> View {
    dom! {
        <section>
            <h2>{title}</h2>
            <Fragment nodes={children}/>
        </section>
    }
}
```

`dom!` acepta `on:click={move |_| ...}` y valores dinámicos. Los children de un componente DOM son `Vec<View>`, no `Vec<Node>`. Build:

```sh
cargo build -p colibri-dom --example counter --target wasm32-unknown-unknown --release
wasm-bindgen --target web --out-dir crates/colibri-dom/demo/pkg \
  target/wasm32-unknown-unknown/release/examples/counter.wasm
```

### Listas y condicionales

`<Dynamic nodes={closure}/>` reconstruye un subárbol cuando cambian las señales leídas:

```rust
let visible = use_state(true);
let render = {
    let visible = visible.clone();
    move || {
        if visible.get() {
            vec![dom! { <p>"visible"</p> }]
        } else {
            Vec::new()
        }
    }
};
let view = dom! { <div><Dynamic nodes={render}/></div> };
```

Es reconciliación **coarse**: reconstruye el subárbol, no preserva foco/estado local y usa un wrapper `display: contents`. Para listas keyed estables hace falta una fase posterior.

La validación de atributos/URL del DOM comparte política con SSR (`colibri_ssr::validate_attribute`).

Demo completo: [`colibri-dom/demo`](../../crates/colibri-dom/demo/README.md).

Dependencias del app:

```toml
[dependencies]
axum = "0.8"
tokio = { version = "1", features = ["full"] }
colibri-ir = { path = "../colibri/crates/colibri-ir" }
colibri-macros = { path = "../colibri/crates/colibri-macros" }
colibri-ssr = { path = "../colibri/crates/colibri-ssr" }
```

## Seguridad

- `render_static` escapa texto y atributos, valida URLs y rechaza tags/atributos peligrosos.
- No construyas HTML concatenando strings fuera de `view!`/`colibri-ssr`.
- Los props `Option<T>` son la vía de null safety; no hay `null`.

## Límites actuales

- Sin cliente interactivo, eventos ni estado en navegador.
- Sin componentes con children/slots.
- Sin router de framework: usa Axum.
- Sin SCSS integrado: compila Sass a CSS en el app o sirve CSS.
- Sin hydration, islands ni streaming.

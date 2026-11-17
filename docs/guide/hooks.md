# Hooks y estado (estado actual)

Colibrí expone primitivas con nombres React, pero **no son hooks de orden de renderizado**. Son handles de Rust con ownership y scope explícitos.

## Disponible hoy en `colibri-core`

```rust
use colibri_core::{batch, effect, memo, on_cleanup, Signal};

let count = Signal::new(0);
let doubled = memo({
    let count = count.clone();
    move || count.get() * 2
});

let _effect = effect({
    let doubled = doubled.clone();
    move || println!("{}", doubled.get())
});

batch(|| {
    count.set(1);
    count.set(2);
});
```

- `use_state(initial) -> Signal<T>`: estado con lectura/escritura.
- `use_memo(compute) -> Memo<T>`: derivación eager con igualdad (`PartialEq`); no notifica si el valor no cambió.
- `use_effect(fn)`: efecto síncrono; se re-ejecuta cuando cambian las señales leídas. Dentro de un `#[component]` que devuelve `View`, el montaje es dueño del efecto aunque ignores el handle.
- `use_effect_with_cleanup(fn -> cleanup)`: cleanup antes de re-run y al desmontar.
- `with_effect_scope(fn) -> (R, Vec<Effect>)`: ejecuta `fn` y recoge los efectos creados dentro; es la base del ownership por componente.
- `batch(fn)`: agrupa cambios para que los efectos vean el estado final.
- `untrack(fn)`: lee señales sin registrar dependencia.
- `take_error()`: devuelve `ReactiveError::CycleLimit` tras 4.096 ejecuciones.
- `stats()`: señales, suscripciones, observadores y cola.

## Hooks en cliente (WASM)

`colibri-dom` conecta las primitivas al DOM. Un componente que devuelve `View` puede usar `use_state`, `use_memo`, `use_effect` y eventos:

```rust
use colibri_core::{use_memo, use_state};
use colibri_dom::{mount, View};
use colibri_macros::{component, dom};

#[component]
pub fn Counter() -> View {
    let count = use_state(0);
    let text = count.clone();
    let click = count.clone();
    let doubled = use_memo({
        let count = count.clone();
        move || count.get() * 2
    });
    dom! {
        <div>
            <p>{text.get()}</p>
            <p>"doble: " {doubled.get()}</p>
            <button on:click={move |_| click.update(|n| *n += 1)}>"+1"</button>
        </div>
    }
}
```

`mount(view, container)` construye el DOM, suscribe los bindings y limpia listeners/effects al soltar el `MountHandle`. Demo reproducible: [`colibri-dom/demo`](../../crates/colibri-dom/demo/README.md).

Los effects creados por un componente que devuelve `View` quedan **ligados a su montaje**: `#[component]` los recoge con `with_effect_scope` y `mount` los limpia al desmontar, sin que tengas que guardar el `Effect`:

```rust
#[component]
pub fn TitleEffect(count: Signal<i32>) -> View {
    use_effect(move || set_document_title(&format!("Contador {}", count.get())));
    View::Fragment(Vec::new())
}
```

Límite: la detección es sintáctica. Si el retorno es un alias o wrapper de `View`, conserva el handle o anota el retorno como `View`. Ver [ADR 0015](../adr/0015-component-effect-ownership.md). No llames hooks (`use_async`, `use_effect`) dentro del callback de otro effect: van en el cuerpo del componente.

### Estado SSR

Para que la hidratación no reinicie el estado, el servidor emite un script JSON y el cliente lo lee con `use_hydrated_state`:

```rust
// servidor (Axum u otro): el JSON lo produce tu app
let script = colibri_ssr::state_script("colibri-state", &json)?;
// <script type="application/json" id="colibri-state">{"counter":7}</script>

// cliente
let count = colibri_dom::use_hydrated_state("counter", 0_i32);
```

- Tipos soportados: `i32`, `f64`, `bool`, `String`.
- Clave ausente o tipo distinto: usa el valor inicial y lo reporta por consola.
- El script se lee una vez y es público: nunca pongas secretos.
- Ver [ADR 0018](../adr/0018-hydration-state.md).

## Hidratación

`colibri_dom::hydrate(view, container)` adopta el HTML servido por SSR en vez de reconstruirlo:

```rust
let handle = colibri_dom::hydrate(dom! { <HydratedCounter/> }, container)?;
```

- Exige que el árbol coincida etiqueta por etiqueta (los atributos se reaplican y el texto se corrige; el texto adyacente del SSR se parte para que `"label: " {valor}` funcione).
- Conserva identidad de nodos y foco; los eventos y effects se registran sobre los nodos existentes.
- Los hooks se ejecutan de nuevo: **no hay estado serializado**, el estado arranca en sus valores iniciales.
- `Dynamic` y `Keyed` todavía no son hidratables: `hydrate` devuelve error.
- Si la estructura no coincide, devuelve error sin borrar el DOM SSR. Ver [ADR 0016](../adr/0016-hydration.md).

### Islas

Una página SSR puede tener varias regiones interactivas independientes:

```rust
let mut handles = Vec::new();
if let Some(handle) = colibri_dom::hydrate_island("#island-counter", || dom! { <Counter/> })? {
    handles.push(handle);
}
if let Some(handle) = colibri_dom::hydrate_island("#island-toggle", || dom! { <Toggle/> })? {
    handles.push(handle);
}
```

Cada isla hidrata su subárbol y se desmonta con su propio `MountHandle`; un selector inexistente devuelve `None`. Un solo bundle WASM sirve todas las islas: todavía no hay manifiesto ni chunking. Ver [ADR 0017](../adr/0017-islands.md).

## Async y lazy

```rust
use colibri_core::Resource;
use colibri_dom::use_async;
use gloo_timers::future::TimeoutFuture;

let manuals = use_async(async {
    TimeoutFuture::new(500).await;
    Ok::<_, String>(42)
});

match manuals.get() {
    Resource::Loading => "cargando…",
    Resource::Ready(total) => "listo",
    Resource::Failed(error) => "error",
}
```

- `use_async` sólo existe en `wasm32`; crea `Signal<Resource<T>>` y actualiza al completar.
- Dentro de un componente que devuelve `View`, al desmontar se cancela **cooperativamente**: el resultado tardío se descarta y no toca el DOM. El future no se aborta. Fuera de un componente corre hasta completar.
- `use_lazy(compute)` calcula una vez al primer `get` y cachea; no se invalida. Para derivaciones que cambian usa `use_memo`.

## Lo que todavía no existe

- **Reconciliación keyed**: `<Keyed>` conserva identidad/foco al reordenar, pero no re-renderiza el contenido de una key existente; falta diff/estado por key.
- **SFC cliente**: `dom!` es la ruta Rust; `.colibri` aún no compila a WASM.
- **Hidratación de `Dynamic`/`Keyed`** y serialización automática de estado (la siembra explícita ya existe; solo primitivos planos).
- **Suspense/streaming**: `Resource` no suspende el render ni hace streaming.
- **Aborto real de futures** (la cancelación es cooperativa) y code splitting/lazy components.
- **Scope de terceros**: sólo los componentes `-> View` reciben effects con ownership automático; SSR descarta los efectos.

## Semántica recomendada para el futuro

1. Los hooks se resuelven por **scope de componente**, no por orden de llamadas.
2. `Resource<T>` con `Loading`, `Ready(T)`, `Error(E)` y cancelación al desmontar.
3. `Lazy` como componente que no ejecuta su cómputo hasta el primer render real.
4. Un `Scope` de componente que limpie effects, listeners y tareas.
5. Reconciliación estable para listas keyed y condicionales.

Hasta entonces, usa `Signal`/`Memo`/`Effect` como primitivas de servidor o en procesos Rust; no esperes comportamiento React en el navegador.

Ver [reactividad](../spec/reactivity.md) y [M07](../tasks/M07-browser-runtime.md).

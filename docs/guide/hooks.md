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
- `use_effect(fn)`: efecto síncrono; se re-ejecuta cuando cambian las señales leídas.
- `use_effect_with_cleanup(fn -> cleanup)`: cleanup antes de re-run y al desmontar.
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
- `use_lazy(compute)` calcula una vez al primer `get` y cachea; no se invalida. Para derivaciones que cambian usa `use_memo`.

## Lo que todavía no existe

- **Reconciliación estructural**: la forma del árbol se fija al montar; listas/condicionales que cambian requieren remount o diff (pendiente).
- **SFC cliente**: `dom!` es la ruta Rust; `.colibri` aún no compila a WASM.
- **Hidratación** SSR → cliente.
- **Suspense/streaming**: `Resource` no suspende el render ni hace streaming.
- **Cancelación de futures** al desmontar y code splitting/lazy components.

## Semántica recomendada para el futuro

1. Los hooks se resuelven por **scope de componente**, no por orden de llamadas.
2. `Resource<T>` con `Loading`, `Ready(T)`, `Error(E)` y cancelación al desmontar.
3. `Lazy` como componente que no ejecuta su cómputo hasta el primer render real.
4. Un `Scope` de componente que limpie effects, listeners y tareas.
5. Reconciliación estable para listas keyed y condicionales.

Hasta entonces, usa `Signal`/`Memo`/`Effect` como primitivas de servidor o en procesos Rust; no esperes comportamiento React en el navegador.

Ver [reactividad](../spec/reactivity.md) y [M07](../tasks/M07-browser-runtime.md).

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

## Lo que todavía no existe

- **Render loop de componente**: `use_state` cambia el valor, pero un componente SSR no se vuelve a ejecutar ni el HTML se repinta.
- **DOM/eventos**: no hay runtime cliente WASM; `on:click` no compila.
- **Async**: no existe `use_resource`/`use_async` ni estados `Loading/Ready/Error`.
- **Lazy**: `use_memo` es eager; no hay componentes lazy ni code splitting.
- **Hydration/SSR cliente**: pendiente M07/M09.

## Semántica recomendada para el futuro

1. Los hooks se resuelven por **scope de componente**, no por orden de llamadas.
2. `Resource<T>` con `Loading`, `Ready(T)`, `Error(E)` y cancelación al desmontar.
3. `Lazy` como componente que no ejecuta su cómputo hasta el primer render real.
4. Un `Scope` de componente que limpie effects, listeners y tareas.
5. El cliente re-renderiza sólo los bindings afectados, no el árbol completo.

Hasta entonces, usa `Signal`/`Memo`/`Effect` como primitivas de servidor o en procesos Rust; no esperes comportamiento React en el navegador.

Ver [reactividad](../spec/reactivity.md) y [M07](../tasks/M07-browser-runtime.md).

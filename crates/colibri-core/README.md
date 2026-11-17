# Colibrí Core — API experimental de reactividad

Crate Rust **single-thread**, sin DOM ni WebAssembly requerido. API actual en desarrollo; no son Hooks de React asociados al orden de renderizado.

## Estado, memo y efecto
~~~rust
use colibri_core::{batch, use_effect, use_memo, use_state, Scope};

let count = use_state(1);
let double = {
    let count = count.clone();
    use_memo(move || count.get() * 2)
};
let mut scope = Scope::new();
let effect = {
    let double = double.clone();
    use_effect(move || println!("double = {}", double.get()))
};
scope.track_effect(effect);
batch(|| {
    count.set(2);
    count.set(3);
});
assert_eq!(double.get(), 6);
drop(scope); // unregister
~~~

## Contratos reales (prototipo)
- `use_state` retorna `Signal<T>` con `get()/with()/set()/update()`.
- `use_memo` retorna `Memo<T>` read-only, requiere `T: Clone + PartialEq + 'static`. Eager, cache del resultado, no notifica si resultado igual.
- `use_effect` se ejecuta inmediatamente y observa señales leídas **sincrónicamente**. La suscripción dura mientras viva el handle `Effect`.
- `use_effect_with_cleanup` devuelve una función de cleanup, ejecutada antes del siguiente run y al soltar Effect.
- `batch` agrupa cambios para deduplicar efectos; subscribers manuales aún son síncronos.
- `Scope::track_effect` administra lifetime y desuscribe al desmontar; `Scope::dispose` ejecuta cleanups LIFO y `Scope::own` apropia cualquier valor.
- `take_error()` devuelve `ReactiveError::CycleLimit` si un ciclo superó 4.096 ejecuciones; el motor vacía la cola y sigue usable.
- `stats()` expone señales, suscripciones, observadores y tareas en cola por hilo (instrumentación de tests).

## Lo que no existe todavía
- Un backend navegador, DOM o hidratación, scheduler concurrente, tracking async, efecto post-paint o React-like rerender.
- Valores derivados lazy, comparación custom, estados thread-safe, panic boundary para código de usuario, transacciones atómicas, profiling.
- Las APIs no garantizan rendimiento superior a Solid o Leptos.

## Riesgos que investigar
- `use_memo` es eager: evaluaciones innecesarias si no hay lectores.
- Writes dentro de effects: pueden producir ciclos; el motor corta a 4.096 ejecuciones y reporta `CycleLimit`, pero el grafo sigue suscrito hasta corregirlo.
- `get()` clona `T`; para estructuras grandes usar `with()` o explorar políticas de acceso con benchmarks.
- Se hace tracking automático sólo en reads sincrónicos, no a través de operaciones async.
- Cada dependencia está representada por un observer/subscription que puede consumir memoria.

## Validación
~~~sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo test -p colibri-core
cargo run -p colibri-core --example reactive_basics
~~~

Ver `docs/spec/reactivity.md` y `docs/research/2026-11-rust-web-performance.md`.

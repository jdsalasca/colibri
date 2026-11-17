# ADR 0012 - Async `Resource` y valores lazy

Fecha: 2026-11-16
Estado: **Aceptada** para M07/M05 extensión.

## Contexto
Faltaban dos semánticas React pedidas explícitamente: datos asíncronos con estados y valores calculados de forma perezosa. El runtime DOM ya tenía `use_state`/`use_memo`/`use_effect` y eventos, pero no podía representar `Loading/Ready/Error` ni posponer un cálculo caro.

## Decisión

1. `colibri-core::Resource<T>` con `Loading`, `Ready(T)` y `Failed(String)`.
2. `colibri_dom::use_async(future)` (sólo `wasm32`):
   - crea `Signal<Resource<T>>` en `Loading`;
   - `wasm_bindgen_futures::spawn_local` ejecuta el future;
   - al terminar hace `set(Ready/​Failed)`;
   - los efectos del DOM actualizan el texto/atributo.
3. `colibri-core::Lazy<T>` + `use_lazy(compute)`:
   - cálculo al primer `get`, cacheado para siempre;
   - handle clonable (`Rc`), comparte el valor;
   - no depende de señales ni se invalida.
4. El demo usa `use_async` con `gloo-timers` (500 ms) y muestra `cargando catálogo… → 42 manuales listos`.

## Alternativas
- `Future` en cada render: repetiría peticiones y perdería estado.
- Hooks con orden de llamadas tipo React: innecesario en Rust; los handles tienen ownership explícito.
- `Lazy` reactivo: sería `Memo`; `use_memo` ya cubre derivaciones con invalidación.
- Async en SSR: exige un runtime/planificador de servidor y un contrato de streaming; fuera de esta rebanada.

## Consecuencias
- `Resource` es genérico y sin dependencias de browser; `use_async` sólo existe en `colibri-dom` wasm.
- No hay cancelación al desmontar: el future sigue y hace `set` sobre una señal que puede quedar sin observadores. No rompe memoria, pero no es cancelación cooperativa.
- No hay `Suspense` ni streaming; el estado inicial es `Loading` y el DOM se actualiza al completar.
- `Lazy` no se invalida; si el valor depende de estado cambiante, usa `use_memo`.
- `Lazy` es single-thread y no promete `Send + Sync`.

## Validación
- Unit test de `Lazy`: cómputo una vez, handles clonados comparten.
- Chrome/WASM: texto inicial `cargando catálogo…`, luego `42 manuales listos`; click `0→1`, doble `0→2`; cero errores de consola.
- 83 pruebas de host; CI wasm compila el ejemplo.

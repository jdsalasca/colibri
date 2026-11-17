# M07 - DOM/browser runtime mínimo

**Prioridad:** P1. **Depende de:** M04 + M05. **Área:** `colibri-dom`. **Specs:** [`docs/spec/rendering.md`](../spec/rendering.md), [`docs/spec/reactivity.md`](../spec/reactivity.md). **ADR:** [0011](../adr/0011-dom-runtime-mvp.md). **Estado:** parcial (ruta Rust WASM; SFC cliente pendiente).

## Objetivo
Montar, actualizar y desmontar un contador real en Chromium desde componentes escritos en ambas sintaxis, sin runtime duplicado.

## Pasos
1. ADR del binding browser: `web-sys/wasm-bindgen` o alternativa medida.
2. Adaptor de nodos/texto/atributos con operadores DOM que eviten reconstrucciones cuando cambia sólo un binding.
3. Event bindings con scopes y cleanup; propagation y focus consistentes.
4. Integrar Signal/Scheduler sin mantener borrows durante callbacks del navegador.
5. Añadir harness E2E Chromium con event count, mutation count y memoria/logs.
6. Probar ciclo mount/unmount repetido y validar que no quedan listeners.

## No hacer
No intentar reconciliación general, resumabilidad, multi-threading Wasm, automatismo de chunking o preservar estado HMR.

## Aceptación
- [x] Ambos contadores reaccionan a click y actualizan sólo binding esperado (Rust/WASM sí; SFC cliente pendiente).
- [x] No se acumulan event listeners ni subscriptions tras teardown (`MountHandle` los limpia; E2E de 25 ciclos con `stats()` a cero).
- [x] Tests con teclado/focus y accesibilidad básica (E2E: Enter/Space activan, `Shift+Tab` mueve foco).
- [x] Se mide el coste del puente JS↔WASM con una línea base simple (~20–30 µs/click, 7–10× vanilla; ver `docs/benchmarks.md`).
- [x] Un servidor de prueba puede ejecutar el demo sin depender de Vite aún.

## Handoff
API de montaje, semántica de teardown, perfil de dependencia DOM y demo reproducible.

Implementado en `colibri-dom` (wasm32):

- `View::{Text, DynamicText, Element, Fragment}` y `dom!`.
- `mount(view, container)`: DOM una vez, bindings con `use_effect`, eventos con `Closure`.
- `MountHandle`: cleanup de listeners/effects/nodos.
- `hydrate(view, container)`: adopta HTML SSR estático (ver [ADR 0016](../adr/0016-hydration.md)); E2E en `/hydrate.html`.
- `#[component]` funciona con retorno `View`; children vía `<Fragment nodes={children}/>`.
- Demo `crates/colibri-dom/demo`: `Panel` + `use_state` + `use_memo` + `on:click`; Chrome: 0→1, doble 0→2, sin errores.

Pendiente: SFC cliente, reconciliación estructural, hidratación, Suspense y accesibilidad automatizada ampliada.

## Extensión async/lazy (ADR 0012)

- `colibri_core::Resource<T>` con `Loading/Ready/Failed`.
- `colibri_dom::use_async`: `spawn_local` + `Signal<Resource<T>>`; el demo pasa de `cargando` a `42 manuales listos`.
- `colibri_core::use_lazy`: cálculo único al primer `get`, cacheado y clonable.
- Sin Suspense/streaming ni cancelación de futures.

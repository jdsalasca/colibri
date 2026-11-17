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
- [ ] Ambos contadores reaccionan a click y actualizan sólo binding esperado (Rust/WASM sí; SFC cliente pendiente).
- [ ] No se acumulan event listeners ni subscriptions tras teardown (`MountHandle` los limpia, falta test E2E).
- [ ] Tests con teclado/focus y accesibilidad básica (botón nativo y focus CSS; falta test automatizado).
- [ ] Se mide el coste del puente JS↔WASM con una línea base simple.
- [x] Un servidor de prueba puede ejecutar el demo sin depender de Vite aún.

## Handoff
API de montaje, semántica de teardown, perfil de dependencia DOM y demo reproducible.

Implementado en `colibri-dom` (wasm32):

- `View::{Text, DynamicText, Element, Fragment}` y `dom!`.
- `mount(view, container)`: DOM una vez, bindings con `use_effect`, eventos con `Closure`.
- `MountHandle`: cleanup de listeners/effects/nodos.
- `#[component]` funciona con retorno `View`; children vía `<Fragment nodes={children}/>`.
- Demo `crates/colibri-dom/demo`: `Panel` + `use_state` + `use_memo` + `on:click`; Chrome: 0→1, doble 0→2, sin errores.

Pendiente: SFC cliente, reconciliación estructural, tests E2E en CI, medición JS↔WASM y accesibilidad automatizada.

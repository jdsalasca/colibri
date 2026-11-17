# M07 — DOM/browser runtime mínimo

**Prioridad:** P1. **Depende de:** M04 + M05. **Área:** futuro `colibri-dom`. **Specs:** `docs/spec/rendering.md`, `docs/spec/reactivity.md`.

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
- [ ] Ambos contadores reaccionan a click y actualizan sólo binding esperado.
- [ ] No se acumulan event listeners ni subscriptions tras teardown.
- [ ] Tests con teclado/focus y accesibilidad básica.
- [ ] Se mide el coste del puente JS↔WASM con una línea base simple.
- [ ] Un servidor de prueba puede ejecutar el demo sin depender de Vite aún.

## Handoff
API de montaje, semántica de teardown, perfil de dependencia DOM y demo reproducible.

# M05 — Reactive core + lifecycle

**Prioridad:** P0. **Depende de:** M00. **Área:** `colibri-core`. **Spec:** `docs/spec/reactivity.md`.

## Objetivo
Evolucionar el prototipo de `Signal<T>` y `Scope` hacia semántica determinista, segura ante reentrancia y comprobable, sin acoplarlo al DOM.

## Plan incremental
1. Definir y probar notificación, subscriptions, borrow safety y actualización anidada. Primero corregir fallos de reentrancia.
2. Definir Cleanup/Scope y unsubscribe idempotente: lifetimes, orden y Drop.
3. Añadir tracking dinámico de dependencias bajo un contrato explícito.
4. Añadir Memo y Effect con invalidación/cleanup sin glitches.
5. Scheduler/Batching cooperativo con deduplicación; que las actualizaciones reactivas no dependan de un browser scheduler.
6. Instrumentación opcional de cantidad de nodos/subscriptions/ejecuciones para tests.

## No hacer
No construir VDOM, DOM bindings, arquitectura multi-thread Wasm o optimizaciones de memoria sin medidas y pruebas.

## Aceptación
- [ ] Tests de reentrancia, unsubscribe durante callback y fuga de referencias pasan.
- [ ] Branch switching limpia dependencias antiguas.
- [ ] Invalidación de memo/effect no observa valores inconsistentes.
- [ ] 10.000 ciclos mount/update/unmount quedan sin observers residuales.
- [ ] Núcleo sigue sin depender de web-sys/wasm-bindgen/Vite.

## Handoff
Semántica de orden y batching escrita, benchmark de overhead de suscripción y riesgos para el adaptador M07.

## Avance parcial en PR #2

Implementación experimental aislada: `use_state`, `use_memo` eager con `PartialEq`, `use_effect` sincrónico, cleanup, batching, branch-sensitive dependency tracking, cola con memos antes que effects, pruebas unitarias. **No marcar M05 finalizado**: falta revisión profunda de reentrancia, recuperación de pánicos/ciclos, scheduler browser, evaluación de costes memoria y APIs estables. 

Investigación: [Rust/WASM performance](../research/2026-11-rust-web-performance.md).

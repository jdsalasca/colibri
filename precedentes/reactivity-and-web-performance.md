# Precedentes de arquitectura reactiva — insumos para agentes

Fecha de revisión: 2026-11-16. **No es un ranking de benchmarks**. Ver [investigación completa](../docs/research/2026-11-rust-web-performance.md).

| Framework | Acierto concreto | Riesgo relevante | Propuesta Colibrí |
|---|---|---|---|
| React | Componentes, DevTools, scheduler, React Compiler | Re-render/effects y reglas de hooks en ciertos casos | Señales independientes del orden; effects solo externos |
| Solid | Tracking fino de señales, memos y dynamic branches | Memoria de subscriptions/observer graphs | Contrastar índices generacionales y estabilidad |
| Svelte 5 | Compilador y runes `$state/$derived/$effect` | Código generado, efectos demasiado usados | IR compartida + memo puro vs effect |
| Leptos 0.8 | Rust reactive_graph, Owners, lazy Memo | Coste de signals/arena/WASM startup | Contrastar eager vs lazy memo; cleanup determinista |
| Dioxus 0.7 | Ergonomía RSX, hot reload, targets | Bundle WASM, reconciliation | No copiar VDOM sin medir |
| Qwik | Resumabilidad y carga diferida | Serialización closures / RTT y chunking | Investigar después de SSR/islas |
| Astro | HTML-first, islands | Costo integración y límites de islands | 0 client runtime en static |
| Vue 3 | Refs/computed/watch, SFCs | Proxies/memoria y parsing | Contractos SFC y Rust sin duplicar motor |
| Yew | VDOM y modelado de componente Rust | Diff/updates de DOM y runtime | Contrastar sin prejuicio contra virtual DOM |

## Archivos a leer en fuentes primarias
- React: `packages/react-reconciler/`, `packages/react-debug-tools/`; docs React Compiler/effects.
- Solid: `packages/solid/src/reactive/signal.ts`; comprobar versión de commit upstream.
- Svelte: `packages/svelte/src/internal/client/reactivity/`; comparar compiled vs runtime.
- Leptos: crates `reactive_graph` / `tachys`; owner lifetime.
- Dioxus: crates `core`, `web`; RSX macro y runtime.
- Qwik: `packages/qwik/src/core/`; serialización/resume (no duplicar implementación).
- Astro: compiler + islands scripts; distinción estático/interactivo.
- Yew: `packages/yew/src/`; reconciliación y hooks.

## Investigación comparativa entregable
Cada agente que estudie un precedente debe entregar:
1. Commit/versión exacta revisados, ruta de archivo real y licencia.
2. Diagrama o descripción del mecanismo (registro → invalidación → recomputación → limpieza).
3. Un escenario donde el enfoque sobresale y otro donde probablemente tenga coste.
4. Test/perf experiment replicable, condiciones y métricas sin resultados inventados.
5. Qué adoptar, qué descartar y por qué, con ADR si altera API de Colibrí.

**No vendorizar código upstream** ni reclamar rendimiento sin pruebas homogéneas.

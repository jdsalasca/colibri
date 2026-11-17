# Estado real del proyecto

Actualizado: 2026-11-16. Baseline M00 verificado en local sobre `main` (Windows 11, rustc 1.99.0 stable, cargo 1.99.0): formato, Clippy y 9 pruebas pasan. La ejecución de CI en GitHub para el nuevo workflow (que ahora usa `cargo fmt --all --check`) queda pendiente de push.

## Existente en la rama de fundación
| Módulo | Implementado (limitado) | NO implementado |
|---|---|---|
| `colibri-core` | `Signal<T>` con suscripciones explícitas; `Scope` con callbacks de limpieza | autotracking, Memo, Effect, Scheduler, concurrencia, DOM |
| `colibri-parser` | scanner SFC que ignora cierres dentro de strings/comentarios; AST de template con spans, atributos e interpolaciones; límites de tamaño/profundidad (M01) | lowering a IR, validación de expresiones Rust, HTML5 completo, entidades, SCSS |
| `colibri-ir` | tipos preliminares `Component`/`Node`/`Attribute` | IR tipada, lowering, análisis semántico, grafo de dependencias |
| `colibri-ssr` | escape HTML para subconjunto estático y rechazo de expresiones dinámicas | SSR dinámico, streaming, islas, hidratación |
| `colibri-cli` | `check` de estructura SFC | compilación, `dev`, `build`, `preview`, `benchmark` |
| `packages/vite-plugin-colibri` | README de intención | plugin ejecutable |
| `examples/` | bosquejos de sintaxis | aplicación compilable y ejecutable |

## Limitaciones y riesgos conocidos
- El parser actual busca cierres de etiquetas de forma simple: no es seguro para HTML arbitrario ni etiquetas embebidas, comentarios complejos o bloques Rust que incluyan textos coincidentes.
- El renderer está deliberadamente restringido. No ampliar tags/atributos sin revisar XSS, URL schemes, atributos booleanos y reglas HTML/ARIA.
- El runtime usa notificaciones síncronas; reentrancia, dependencias cíclicas y orden de entrega necesitan un contrato verificable.
- La configuración de CI existe, pero el estado de sus ejecuciones debe comprobarse en GitHub.
- Los archivos de ejemplo `Counter.rs` y `Counter.colibri` son **sintaxis objetivo**, no una implementación compilable.
- No existe todavía evidencia de ventajas respecto a React, Solid, Svelte, Leptos o JS.

## Registro de verificación
| Comprobación | Estado |
|---|---|
| `cargo fmt --all --check` | Pasa en local (M00) |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pasa en local (M00) |
| `cargo test --workspace` | Pasa en local: 23 pruebas (M00: 9, M01: 14) |
| `colibri check examples/counter.colibri` | Pasa en local; sólo valida bloques y estructura del template, no compila |
| CLI con SFC inválido | Reporta `archivo:línea:columna` y código de error (M01) |
| CI de GitHub con `--check` | Pendiente: requiere push |
| Browser/WASM | No implementado |
| Benchmarks reproducibles | No implementados |

Actualizar **únicamente** con resultados y enlaces al commit/CI. La sección es una protección contra declaraciones optimistas.

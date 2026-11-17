# Estado real del proyecto

Actualizado: 2026-11-16. Los workflows de GitHub Actions asociados al commit `801f768811e8e9379fceba92d03fb18846e1fc63` terminaron en **success** ([run 3](https://github.com/jdsalasca/colibri/actions/runs/43986730971) y [run 4](https://github.com/jdsalasca/colibri/actions/runs/43986737277)). Esta verificación no demuestra funcionalidad de framework completo. Cualquier agente debe revisar nuevamente la rama y sus resultados.

## Existente en la rama de fundación
| Módulo | Implementado (limitado) | NO implementado |
|---|---|---|
| `colibri-core` | `Signal<T>` con suscripciones explícitas; `Scope` con callbacks de limpieza | autotracking, Memo, Effect, Scheduler, concurrencia, DOM |
| `colibri-parser` | extracción lineal preliminar de bloques `script/template/style`; validación básica de lang | parser robusto de HTML, Rust y SCSS, spans, comentarios complejos, anidamiento |
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
| Integridad documental en GitHub | Archivos del PR consultados mediante GitHub; verificar tras fusionar |
| `cargo fmt --all --check` | No verificado aquí |
| `cargo clippy --workspace --all-targets -- -D warnings` | No verificado aquí |
| `cargo test --workspace` | No verificado aquí |
| Browser/WASM | No implementado |
| Benchmarks reproducibles | No implementados |

Actualizar **únicamente** con resultados y enlaces al commit/CI. La sección es una protección contra declaraciones optimistas.

# Estado real del proyecto

Actualizado: 2026-11-16. Baseline M00 verificado en local sobre `main` (Windows 11, rustc 1.99.0 stable, cargo 1.99.0): formato, Clippy y 9 pruebas pasan. La ejecución de CI en GitHub para el nuevo workflow (que ahora usa `cargo fmt --all --check`) queda pendiente de push.

## Existente en la rama de fundación
| Módulo | Implementado (limitado) | NO implementado |
|---|---|---|
| `colibri-core` | `Signal<T>` con suscripciones explícitas; `Scope` con callbacks de limpieza | autotracking, Memo, Effect, Scheduler, concurrencia, DOM |
| `colibri-parser` | scanner SFC que ignora cierres dentro de strings/comentarios; AST de template con spans, atributos e interpolaciones; límites de tamaño/profundidad (M01) | lowering a IR, validación de expresiones Rust, HTML5 completo, entidades, SCSS |
| `colibri-ir` | IR con spans: texto, texto dinámico, elementos, fragmentos, atributos y eventos; lowering SFC→IR (M02) | componentes/calls, props, control de flujo, análisis de dependencias, codegen |
| `colibri-ssr` | render estático seguro desde IR; booleanos, void, allowlist de URLs y errores con span (M02) | SSR dinámico, streaming, islas, hidratación |
| `colibri-cli` | `check` estructural y `render` HTML estático | compilación Rust/WASM, dev/build, preview, benchmark |
| `packages/vite-plugin-colibri` | README de intención | plugin ejecutable |
| `examples/` | bosquejos de sintaxis | aplicación compilable y ejecutable |

## Limitaciones y riesgos conocidos
- El parser actual busca cierres de etiquetas de forma simple: no es seguro para HTML arbitrario ni etiquetas embebidas, comentarios complejos o bloques Rust que incluyan textos coincidentes.
- El renderer está deliberadamente restringido. No ampliar tags/atributos sin revisar XSS, URL schemes, atributos booleanos y reglas HTML/ARIA.
- El runtime usa notificaciones síncronas; reentrancia, dependencias cíclicas y orden de entrega necesitan un contrato verificable.
- La configuración de CI existe, pero el estado de sus ejecuciones debe comprobarse en GitHub.
- Los archivos de ejemplo `Counter.rs` y `Counter.colibri` son **sintaxis objetivo**, no una implementación compilable.
- No existe todavía evidencia de ventajas respecto a React, Solid, Svelte, Leptos o JS.
- El pipeline estático no evalúa expresiones ni eventos y no decodifica entidades HTML de entrada (`&amp;` se trata como texto literal). Los comentarios HTML se descartan en lowering.

## Registro de verificación
| Comprobación | Estado |
|---|---|
| `cargo fmt --all --check` | Pasa en local (M00) |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pasa en local (M00) |
| `cargo test --workspace` | Pasa en local: 34 pruebas (parser 19, IR 2, SSR 5, pipeline 6, core 2) |
| `colibri check examples/counter.colibri` | Pasa en local; sólo valida bloques y estructura del template, no compila |
| `colibri render examples/static-card.colibri` | Emite HTML estático (M02); rechaza contenido dinámico con span |
| CLI con SFC inválido | Reporta `archivo:línea:columna` y código de error (M01) |
| CI de GitHub con `--check` | Pendiente: requiere push |
| Browser/WASM | No implementado |
| Benchmarks reproducibles | No implementados |

Actualizar **únicamente** con resultados y enlaces al commit/CI. La sección es una protección contra declaraciones optimistas.

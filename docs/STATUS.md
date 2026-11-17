# Estado real del proyecto

Actualizado: 2026-11-16. Baseline M00–M04 verificado en local sobre `main` (Windows 11, rustc 1.99.0 stable, cargo 1.99.0): formato, Clippy y 47 pruebas pasan, incluidas paridad Rust/SFC y render HTML estático. La ejecución de CI en GitHub queda pendiente de push.

## Existente en main
| Módulo | Implementado (limitado) | NO implementado |
|---|---|---|
| `colibri-core` | `Signal<T>`, `Scope`; en la integración M05: tracking sincrónico, `use_state`, `use_memo` eager, `use_effect`, cleanup y `batch` (**prototipo**) | browser scheduler, tracking async, memo lazy, concurrencia, DOM |
| `colibri-parser` | scanner SFC que ignora cierres dentro de strings/comentarios; AST de template con spans, atributos e interpolaciones; límites de tamaño/profundidad (M01) | lowering a IR, validación de expresiones Rust, HTML5 completo, entidades, SCSS |
| `colibri-ir` | IR con spans: texto, texto dinámico, elementos, fragmentos, atributos y eventos; lowering SFC→IR (M02) | componentes/calls, props, control de flujo, análisis de dependencias, codegen |
| `colibri-ssr` | render estático seguro desde IR; booleanos, void, allowlist de URLs y errores con span (M02) | SSR dinámico, streaming, islas, hidratación |
| `colibri-cli` | `check` estructural y `render` HTML estático | compilación Rust/WASM, dev/build, preview, benchmark |
| `colibri-macros` | `view!` estático con errores de compilación para dinámico/eventos (M03) | `#[component]`, props, children, eventos, reactividad, codegen |
| `packages/vite-plugin-colibri` | README de intención | plugin ejecutable |
| `examples/` | bosquejos de sintaxis | aplicación compilable y ejecutable |

## Limitaciones y riesgos conocidos
- El parser actual busca cierres de etiquetas de forma simple: no es seguro para HTML arbitrario ni etiquetas embebidas, comentarios complejos o bloques Rust que incluyan textos coincidentes.
- El renderer está deliberadamente restringido. No ampliar tags/atributos sin revisar XSS, URL schemes, atributos booleanos y reglas HTML/ARIA.
- El núcleo reactivo nuevo usa reactividad síncrona con cola de memos antes de effects y límite de ejecuciones por ciclo. Suscripciones manuales todavía son síncronas; reentrancia, posibles ciclos y orden deben probarse en casos complejos. Los memos son eager y no existe scheduler del navegador.
- La configuración de CI existe, pero el estado de sus ejecuciones debe comprobarse en GitHub.
- Los archivos de ejemplo `Counter.rs` y `Counter.colibri` son **sintaxis objetivo**, no una implementación compilable.
- No existe todavía evidencia de ventajas respecto a React, Solid, Svelte, Leptos o JS.
- El pipeline estático no evalúa expresiones ni eventos y no decodifica entidades HTML de entrada (`&amp;` se trata como texto literal). Los comentarios HTML se descartan en lowering.
- La ruta Rust usa spans sintéticos (`file!()`, `0..0`) porque los proc macros estables no exponen offsets de byte; los errores de sintaxis sí apuntan al token real.

## Registro de verificación
| Comprobación | Estado |
|---|---|
| `cargo fmt --all --check` | Pasa en local (M00) |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pasa en local (M00) |
| `cargo test --workspace` | Pasa en local: 47 pruebas (parser 19, pipeline 6, macros 13, IR 2, SSR 5, core 2) |
| `colibri check examples/counter.colibri` | Pasa en local; sólo valida bloques y estructura del template, no compila |
| `colibri render examples/static-card.colibri` | Emite HTML estático (M02); rechaza contenido dinámico con span |
| CLI con SFC inválido | Reporta `archivo:línea:columna` y código de error (M01) |
| CI de GitHub con `--check` | Pendiente: requiere push |
| Browser/WASM | No implementado |
| Benchmarks reproducibles | No implementados |

Actualizar **únicamente** con resultados y enlaces al commit/CI. La sección es una protección contra declaraciones optimistas.

## Avance M05 — motor reactivo experimental

Se integra un subsistema de estado/derivaciones/efectos independiente del navegador, con pruebas de tracking dinámico, igualdad de Memo, batch, cleanup y 10.000 scopes. **No cerrar M05:** falta validar manejo seguro de ciclos/pánicos, reconciliación con el DOM, memory profiling y benchmark nativo/WASM.

Investigación: [Rust web performance](research/2026-11-rust-web-performance.md); [comparación de precedentes](../precedentes/reactivity-and-web-performance.md). Las cifras de rendimiento aún no existen.

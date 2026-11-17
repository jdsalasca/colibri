# Estado real del proyecto

Actualizado: 2026-11-16. M00–M05 se validaron en CI (`8eb70c2`, 58 pruebas; PR #3, 55). La primera rebanada de M06 (SSR dinámico con resolver) pasó CI en `main` y `develop` para `93c7e5a`: [main run 43992063285](https://github.com/jdsalasca/colibri/actions/runs/43992063285), [develop run 43992063264](https://github.com/jdsalasca/colibri/actions/runs/43992063264), **68 pruebas**. No se ha probado navegador ni rendimiento comparativo.

## Existente en main
| Módulo | Implementado (limitado) | NO implementado |
|---|---|---|
| `colibri-core` | `Signal<T>`, `Scope`; tracking sincrónico, `use_state`, `use_memo` eager, `use_effect`, cleanup, `batch`; ciclos recuperables con `take_error()`, `stats()` y `Scope::dispose` (M05) | browser scheduler, tracking async, memo lazy, concurrencia, DOM |
| `colibri-parser` | scanner SFC, AST con spans, atributos e interpolaciones; límites de tamaño/profundidad; lowering estático a IR (M01–M02) | validación integral de expresiones Rust, HTML5 completo, entidades, SCSS |
| `colibri-ir` | IR con spans: texto, texto dinámico, elementos, fragmentos, atributos y eventos; lowering SFC→IR (M02) | componentes/calls, props, control de flujo, análisis de dependencias, codegen |
| `colibri-ssr` | render estático seguro y SSR dinámico con resolver explícito; escape, URL allowlist, booleanos, void y `MissingValue` con span (M02/M06a) | evaluación Rust server, eventos SSR, streaming, islas, hidratación |
| `colibri-cli` | `check`, `render` estático y `render --bind key=value` para expresiones | compilación Rust/WASM, dev/build, preview, benchmark |
| `colibri-macros` | `view!` estático con errores de compilación para dinámico/eventos (M03) | `#[component]`, props, children, eventos, reactividad, codegen |
| `packages/vite-plugin-colibri` | README de intención | plugin ejecutable |
| `examples/` | SFC estático, SFC dinámico con `--bind` y macro Rust ejecutables; ejemplo nativo de reactividad en `colibri-core/examples/` | UI browser/WASM ejecutable |

## Limitaciones y riesgos conocidos
- El parser actual busca cierres de etiquetas de forma simple: no es seguro para HTML arbitrario ni etiquetas embebidas, comentarios complejos o bloques Rust que incluyan textos coincidentes.
- El renderer está deliberadamente restringido. No ampliar tags/atributos sin revisar XSS, URL schemes, atributos booleanos y reglas HTML/ARIA.
- El núcleo reactivo usa reactividad síncrona con cola de memos antes de effects. Los ciclos se cortan tras 4.096 ejecuciones y dejan un `ReactiveError::CycleLimit` recuperable con `take_error()`; no hay panic automático. Los memos son eager y no existe scheduler del navegador.
- `stats()` es instrumentación thread-local para tests; no es un perfil de memoria ni una métrica de producción.
- CI del PR #3 comprobado con 55 tests y checks de formato/Clippy; cualquier commit futuro debe volver a pasar los checks.
- Los archivos de ejemplo `Counter.rs` y `Counter.colibri` son **sintaxis objetivo**, no una implementación compilable.
- No existe todavía evidencia de ventajas respecto a React, Solid, Svelte, Leptos o JS.
- El pipeline estático no evalúa expresiones ni eventos y no decodifica entidades HTML de entrada (`&amp;` se trata como texto literal). Los comentarios HTML se descartan en lowering.
- El SSR dinámico usa un resolver del host; no evalúa Rust, no renderiza eventos y no hace streaming ni hidratación ([ADR 0007](adr/0007-dynamic-ssr-resolver.md)).
- La ruta Rust usa spans sintéticos (`file!()`, `0..0`) porque los proc macros estables no exponen offsets de byte; los errores de sintaxis sí apuntan al token real.

## Registro de verificación
| Comprobación | Estado |
|---|---|
| `cargo fmt --all --check` | Pasa en CI del PR #3 (run 43989925541) |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pasa en CI del PR #3 |
| `cargo test --workspace` | CI `93c7e5a`: 68 pruebas, 0 fallos. Local: 68 |
| `colibri check examples/counter.colibri` | Pasa en local; sólo valida bloques y estructura del template, no compila |
| `colibri render examples/static-card.colibri` | Emite HTML estático (M02); rechaza contenido dinámico con span |
| `colibri render examples/greeting.colibri --bind name=Ada --bind message=Bienvenida` | Emite HTML con valores resueltos (M06a); falla con `MissingValue` si falta un binding |
| CLI con SFC inválido | Reporta `archivo:línea:columna` y código de error (M01) |
| CI de GitHub con `--check` | **Verificado** en `main`/`develop` para `93c7e5a`: [run 43992063285](https://github.com/jdsalasca/colibri/actions/runs/43992063285) / [run 43992063264](https://github.com/jdsalasca/colibri/actions/runs/43992063264) |
| Browser/WASM | No implementado |
| Benchmarks reproducibles | No implementados |

Actualizar **únicamente** con resultados y enlaces al commit/CI. La sección es una protección contra declaraciones optimistas.

## Avance M05 — motor reactivo experimental

Se integra un subsistema de estado/derivaciones/efectos independiente del navegador, con pruebas de tracking dinámico, igualdad de Memo, batch, cleanup, 10.000 scopes y 10.000 ciclos signal+memo+effect. El contrato de M05 queda cubierto, incluidos ciclos recuperables (`take_error`), `stats()` y `Scope::dispose`. **No cerrar el ecosistema:** quedan riesgos de M07/M10 (reconciliación DOM, panic boundary, memoria por suscripción, benchmark WASM/Chromium).

Investigación: [Rust web performance](research/2026-11-rust-web-performance.md); [comparación de precedentes](../precedentes/reactivity-and-web-performance.md). Las cifras de rendimiento aún no existen.

## Avance M06a — SSR dinámico con resolver

`render_dynamic` acepta un resolver del host para `{expr}` y atributos dinámicos. Escapa texto/atributos, mantiene URL allowlist y devuelve `MissingValue` con span si falta un valor. Los eventos siguen siendo error de SSR. El CLI expone `--bind key=value`. **No cierra M06:** falta evaluación Rust real, paridad dinámica con `view!`, streaming e hidratación. Ver [ADR 0007](adr/0007-dynamic-ssr-resolver.md).

### Entrega entre agentes

Los siguientes problemas se registraron en [issue #4](https://github.com/jdsalasca/colibri/issues/4): recuperación de pánicos/ciclos, reentrancia, memo eager frente a lazy, memoria por subscription y mediciones con WASM/Chromium. No cerrar M05 hasta resolver o acotar técnicamente estos riesgos.

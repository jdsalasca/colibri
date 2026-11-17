# Estado real del proyecto

Actualizado: 2026-11-17. M00-M07 parcial (DOM/WASM, async, lazy, Dynamic, Keyed, ownership de effects, scope suspendido, cancelación cooperativa, teardown y latencia medidos) pasaron CI en `70fa4cd`; los ajustes de Terra #2 y el endurecimiento M07 están **en local con 91 pruebas + 4 E2E** y pendientes de CI. E2E Playwright/Chromium: `0→1`, doble `0→2`, async `42`, condicional `1→0→1`, keyed `A,B,C→C,B,A` con foco, título `Contador 0→1`, teardown a cero y teclado. Click→DOM ~20–30 µs (7–10× vanilla). Terra: 8/8 de la #1; 3/6 de la #2 corregidos y 3 aceptados. ManualOps (Axum + Colibrí) sigue validado. Sin comparativa con otros frameworks ni memoria.

## Existente en main
| Módulo | Implementado (limitado) | NO implementado |
|---|---|---|
| `colibri-core` | `Signal<T>`, `Scope`; tracking sincrónico, `use_state`, `use_memo` eager, `use_effect`, `use_lazy`, `Resource`, cleanup, `batch`; ciclos recuperables con `take_error()`, `stats()` y `Scope::dispose` (M05) | browser scheduler, tracking async, memo lazy, concurrencia, DOM |
| `colibri-dom` | runtime DOM/WASM: `View`, `dom!`, `mount`, bindings con effects, eventos, `use_async`/`Resource`, `Dynamic` y `Keyed` con anclas (M07 parcial) | diff/estado por key, SFC cliente, hidratación, Suspense |
| `colibri-parser` | scanner SFC, AST con spans, atributos e interpolaciones; límites de tamaño/profundidad; lowering estático a IR (M01–M02) | validación integral de expresiones Rust, HTML5 completo, entidades, SCSS |
| `colibri-ir` | IR con spans: texto, texto dinámico, elementos, fragmentos, atributos y eventos; lowering SFC→IR (M02) | componentes/calls, props, control de flujo, análisis de dependencias, codegen |
| `colibri-codegen` | SFC → programa Rust con script, expresiones y atributos dinámicos; `build`/`eval` ejecutan con rustc (M06b) | control de flujo, componentes, eventos, SCSS, optimización |
| `colibri-ssr` | render estático seguro y SSR dinámico con resolver explícito; escape, URL allowlist, booleanos, void, `meta charset`/viewport y `link stylesheet` validados; `MissingValue` con span (M02/M06a) | eventos SSR, streaming, islas, hidratación |
| `colibri-cli` | `check`, `render` (estático/`--bind`), `build` (Rust generado) y `eval` (compila y ejecuta) | compilación WASM, dev server, preview, benchmark |
| `colibri-macros` | `view!` dinámico y `#[component]` con props tipadas, children vía `Fragment`, composición y `Option`/null safety; eventos bloqueados (M03/M09) | slots nombrados, eventos, WASM, componentes en SFC |
| `packages/vite-plugin-colibri` | README de intención | plugin ejecutable |
| `examples/` | SFC estático, SFC dinámico con `--bind`, `eval` con script Rust, macro Rust y contador WASM con hooks/eventos; ejemplo nativo de reactividad en `colibri-core/examples/` | UI browser/WASM ejecutable con SFC/hidratación |

## Limitaciones y riesgos conocidos
- El parser actual busca cierres de etiquetas de forma simple: no es seguro para HTML arbitrario ni etiquetas embebidas, comentarios complejos o bloques Rust que incluyan textos coincidentes.
- El renderer está deliberadamente restringido. No ampliar tags/atributos sin revisar XSS, URL schemes, atributos booleanos y reglas HTML/ARIA.
- El núcleo reactivo usa reactividad síncrona con cola de memos antes de effects. Los ciclos se cortan tras 4.096 ejecuciones y dejan un `ReactiveError::CycleLimit` recuperable con `take_error()`; no hay panic automático. Los memos son eager y no existe scheduler del navegador.
- `use_state`/`use_memo`/`use_effect` existen en `colibri-core`; el runtime DOM/WASM los conecta a texto/atributos en componentes que devuelven `View`. Los efectos de esos componentes quedan ligados al montaje (`with_effect_scope` + `View::Scoped`). Siguen pendientes: SFC cliente, hidratación y cancelación de `use_async`. Ver [hooks](guide/hooks.md).
- `stats()` es instrumentación thread-local para tests; no es un perfil de memoria ni una métrica de producción.
- CI del PR #3 comprobado con 55 tests y checks de formato/Clippy; cualquier commit futuro debe volver a pasar los checks.
- Los archivos de ejemplo `Counter.rs` y `Counter.colibri` son **sintaxis objetivo**, no una implementación compilable.
- No existe todavía evidencia de ventajas respecto a React, Solid, Svelte, Leptos o JS.
- El pipeline estático no evalúa expresiones ni eventos y no decodifica entidades HTML de entrada (`&amp;` se trata como texto literal). Los comentarios HTML se descartan en lowering.
- El SSR dinámico tiene dos vías: resolver del host (`render --bind`, [ADR 0007](adr/0007-dynamic-ssr-resolver.md)) y codegen Rust (`build`/`eval`, [ADR 0008](adr/0008-sfc-rust-codegen.md)). La segunda compila el script real con rustc, pero no es sandbox, no admite control de flujo/componentes/eventos/SCSS y los errores de tipo apuntan al Rust generado.
- El runtime DOM/WASM (`dom!`) actualiza texto/atributos/eventos y `Dynamic` reconstruye listas/condicionales con anclas; la reconciliación es coarse (sin keyed/foco/estado local) y no hay hidratación SSR ([ADR 0011](adr/0011-dom-runtime-mvp.md), [0013](adr/0013-dynamic-fragments.md)).
- El audit portátil de `teachers` pasa; el baseline estricto (`missing_docs`, `unwrap_used`, cobertura 60%) sólo se aplica por ahora a `colibri-codegen`. Ver [calidad](quality.md).
- La ruta Rust usa spans sintéticos (`file!()`, `0..0`) porque los proc macros estables no exponen offsets de byte; los errores de sintaxis sí apuntan al token real.

## Registro de verificación
| Comprobación | Estado |
|---|---|
| `cargo fmt --all --check` | Pasa en CI del PR #3 (run 43989925541) |
| `cargo clippy --workspace --all-targets -- -D warnings` | Pasa en CI del PR #3 |
| `cargo test --workspace` | Local: 91 pruebas, 0 fallos. CI `70fa4cd`: 88 |
| `colibri check examples/counter.colibri` | Pasa en local; sólo valida bloques y estructura del template, no compila |
| `colibri render examples/static-card.colibri` | Emite HTML estático (M02); rechaza contenido dinámico con span |
| `colibri render examples/greeting.colibri --bind name=Ada --bind message=Bienvenida` | Emite HTML con valores resueltos (M06a); falla con `MissingValue` si falta un binding |
| `colibri eval examples/greeting.colibri` | Compila el script con rustc y emite HTML con valores calculados (M06b) |
| `cargo test -p colibri-ssr` | 12 pruebas: escape, URL allowlist, dinámicos, meta/link seguros y rechazos |
| `cargo test -p colibri-macros` | Componentes, props tipadas, composición, children y `Option`/null safety (M03b) |
| Build WASM + wasm-bindgen | CI `f2ff960` compila `wasm32-unknown-unknown`; demo local con wasm-bindgen |
| Chrome/Playwright | Contador `0→1`, doble `0→2`, async `42`, condicional `1→0→1`, keyed `A,B,C→C,B,A` con foco; sin wrappers ni errores |
| E2E en CI | Job `e2e` con wasm-bindgen + Playwright/Chromium; pasa en `bd8311a` |
| Seguridad DOM/SSR | `validate_attribute` compartido; allowlist `http/https/mailto/tel/relativas`; pruebas de `ftp`/`data`/`file`/`javascript` |
| Revisión Terra | 8 hallazgos; 6 aplicados, 2 diferidos en [review](reviews/2026-11-17-terra.md) |
| Audit portátil de `teachers` | fmt + clippy + README + `expect` limpios; quedan 4 archivos >400 líneas (deuda documentada en [quality](quality.md)) |
| CLI con SFC inválido | Reporta `archivo:línea:columna` y código de error (M01) |
| CI de GitHub con `--check` | **Verificado** en `main`/`develop` para `70fa4cd`: [run 44124353971](https://github.com/jdsalasca/colibri/actions/runs/44124353971) / [run 44124416847](https://github.com/jdsalasca/colibri/actions/runs/44124416847) (validate + wasm + e2e) |
| Browser/WASM | No implementado |
| Benchmarks reproducibles | No implementados |

Actualizar **únicamente** con resultados y enlaces al commit/CI. La sección es una protección contra declaraciones optimistas.

## Avance M05 — motor reactivo experimental

Se integra un subsistema de estado/derivaciones/efectos independiente del navegador, con pruebas de tracking dinámico, igualdad de Memo, batch, cleanup, 10.000 scopes y 10.000 ciclos signal+memo+effect. El contrato de M05 queda cubierto, incluidos ciclos recuperables (`take_error`), `stats()` y `Scope::dispose`. **No cerrar el ecosistema:** quedan riesgos de M07/M10 (reconciliación DOM, panic boundary, memoria por suscripción, benchmark WASM/Chromium).

Investigación: [Rust web performance](research/2026-11-rust-web-performance.md); [comparación de precedentes](../precedentes/reactivity-and-web-performance.md). Las cifras de rendimiento aún no existen.

## Avance M06 — SSR dinámico y codegen Rust

**M06a:** `render_dynamic` acepta un resolver del host para `{expr}` y atributos dinámicos. Escapa texto/atributos, mantiene URL allowlist y devuelve `MissingValue` con span si falta un valor. Los eventos siguen siendo error de SSR. El CLI expone `--bind key=value`. Ver [ADR 0007](adr/0007-dynamic-ssr-resolver.md).

**M06b:** `colibri-codegen` genera un programa Rust con el `<script>` del SFC dentro de `main`, `format!` para cada expresión y atributos dinámicos, y renderizado final con `colibri-ssr`. `build` emite el `.rs`; `eval` crea un proyecto Cargo temporal y lo ejecuta. Los errores de tipo los da rustc. Ver [ADR 0008](adr/0008-sfc-rust-codegen.md).

**No cierra M06:** falta paridad dinámica con `view!`, eventos/DOM, streaming, islas e hidratación.

## App de validación — ManualOps (`../colibri-dashboard`)

Dashboard de manuales internos con Axum + componentes Colibrí, sin JavaScript:

- `cargo fmt --check`, `clippy -D warnings` y 5 tests limpios.
- HTTP real: `/health` 200, `/` 200, `/manuals` 200, `/manuals/new` 200, `/manuals/1` 200, `/manuals/9999` 404, `POST /manuals` 303.
- Capturas: `dashboard.png` (desktop) y `dashboard-mobile.png` (390×844).
- Sirve `/static/app.css` compilado desde SCSS.
- Limitaciones: sin persistencia, autenticación, edición ni eventos cliente.

## Avance M07 — runtime DOM/WASM

`colibri-dom` (wasm32) monta un `View`, actualiza bindings con effects y registra eventos. Los effects de un componente `-> View` los posee el montaje: `#[component]` los captura con `with_effect_scope` y `View::Scoped` los libera al desmontar. `Dynamic` reconstruye condicionales; `Keyed` reordena preservando nodos/foco. `use_async` añade `Resource<T>`; `use_lazy` cachea un cálculo. El demo usa `use_state`, `use_memo`, `use_async`, `Dynamic`, `Keyed`, `TitleEffect`, `Panel` y eventos; E2E CI verifica contador, async, condicional `1→0→1`, keyed `A,B,C→C,B,A`, foco y título `Contador 0→1`. Ver [ADR 0011](adr/0011-dom-runtime-mvp.md), [0012](adr/0012-async-resource-lazy.md), [0013](adr/0013-dynamic-fragments.md), [0014](adr/0014-keyed-views.md), [0015](adr/0015-component-effect-ownership.md) y [revisiones Terra](reviews/2026-11-17-terra.md) ([#2](reviews/2026-11-17-terra-2.md)).

Pendiente: contenido/estado por key, aborto real de futures (hoy cancelación cooperativa), SFC cliente, hidratación, Suspense/streaming y comparativa de rendimiento con otros frameworks.

Teardown: 25 ciclos `mount_app`/`unmount_app` dejan `stats()` en `{signals:0, subscriptions:0, observers:0, queued:0}` y `#app` vacío. Teclado: Enter/Space activan, `Shift+Tab` mueve el foco. Click→DOM: ~20–30 µs por click (7–10× el baseline JS) en la máquina de desarrollo; datos y límites en [benchmarks](benchmarks.md).

### Entrega entre agentes

Los siguientes problemas se registraron en [issue #4](https://github.com/jdsalasca/colibri/issues/4): recuperación de pánicos/ciclos, reentrancia, memo eager frente a lazy, memoria por subscription y mediciones con WASM/Chromium. No cerrar M05 hasta resolver o acotar técnicamente estos riesgos.

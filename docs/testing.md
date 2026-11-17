# Estrategia de pruebas y CI

**Regla:** no existe "done" sin prueba observable reproducible. La configuración actual de CI necesita comprobación; registrar evidencia en `docs/STATUS.md`.

## Pirámide
1. **Unitarios**: lexer/template parser, escapes, valores/reactive graph, ids y cleanup.
2. **Golden tests**: SFC→AST→IR y macro Rust→IR con spans y diagnósticos; snapshots versionados, revisión de cambios.
3. **Propiedades**: no-panic con entradas arbitrarias, roundtrip semántico para subconjuntos válidos y invariantes de grapho.
4. **Integración**: dos sintaxis de mismo componente, igualdad de HTML y eventos/estado/cleanup.
5. **Browser E2E**: Chromium real con clics, focus, teclado, navegación, reordenamiento keyed y cambios de layout.
6. **Seguridad**: XSS vectors, URL encodings, atributos maliciosos, HTML mal formado.
7. **Rendimiento/estabilidad**: baseline versionado, long sessions, loops de mount/unmount, consumo de memoria y latencia.

## Mínimos por crate
| Crate | Pruebas obligatorias |
|---|---|
| parser | nesting, Unicode, comentarios, strings con closing tags, atributos, offsets, límites |
| ir | equivalencia semántica, atributos, component calls, key uniqueness, spans |
| core | reentrancia, orden, unsubscribe en callback, drop exact-once, graph cycles |
| ssr | escape por contexto, URL allowlist, elementos void, hidratación, atributos ARIA |
| macros/codegen (futuro) | compile-pass, compile-fail, mensajes con spans, SFC parity |
| dom (futuro) | lifecycle, focus, event propagation, diff de mutaciones |
| vite (futuro) | import/rebuild, CSS HMR, error mapping, WASM chunks |

## Comandos de calidad
~~~sh
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo run -p colibri-cli -- check examples/counter.colibri
~~~
**Advertencia:** el último comando sólo comprueba bloques SFC preliminares; no compila el contador. Si CI actual ejecuta formato en vez de comprobarlo, corregirlo en M00.

## Matriz de ejemplo canónica
| Caso | Rust component | .colibri | Expectativa |
|---|---|---|---|
| texto estático/UTF-8 | sí | sí | HTML semántico igual |
| contador y evento click | sí | sí | mismo texto, orden, cleanup |
| prop inicializada | sí | sí | mismo valor y error de tipo |
| lista keyed reorder | sí | sí | identidad y focus correctos |
| unmount/remount 10k | sí | sí | sin listeners ni subscriptions colgados |
| contenido hostil | sí | sí | escape/diagnóstico seguro |

## CI gradual
- Ahora: formatting, Clippy, tests workspace, CLI check.
- Con macros: compiletests y snapshots de diagnósticos.
- Con DOM: E2E Chromium en entorno fijo + web platform test subset.
- Con benchmarks: job no-blocking para mediciones históricas y gates de regresión después de estabilizar máquina e incertidumbre.
- Fuzzing: corpus minimizado en repositorio, nunca introducir input malicioso en producción.

## Definition of Done
Tests pasan en el commit exacto, no sólo en máquina del agente; las limitaciones están documentadas y el PR incluye los comandos ejecutados. Si las herramientas no están disponibles, etiquetar `implemented-unverified`.

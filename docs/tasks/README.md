# Backlog ejecutable para agentes — Colibrí

**Lee primero** `AGENTS.md`, `docs/STATUS.md`, `docs/architecture.md` y `docs/agent-workflow.md`.

Cada archivo Mxx es un **contrato de entrega**, no una orden de implementarlo todo de una vez. Selecciona un ID, confirma dependencias, identifica ownership y abre un PR pequeño. El estado definitivo requiere evidencia en CI.

## Grafo de dependencias
~~~text
M00 (CI + baseline)
├── M01 (template AST) ── M02 (SFC → IR) ─┬── M03 (Rust macro → IR) ── M04 (paridad)
│                                          └── M06 (SSR estático/dinámico)
├── M05 (reactividad / scopes) ───────────────────────────────┐
└── M10 (benchmark baseline parcial)                          │
                                   M04 + M05 ── M07 (DOM/WASM)│
                                                   │          │
                                                   M08 (CLI/Vite/SCSS)
                                                     │
                                M06 + M07 + M08 ── M09 (islas/SSR)
                                                        │
                                                   M10 (comparativas)
~~~
M11 seguridad/accesibilidad aplica como gate transversal a **todos** los hitos y no puede posponerse.

## Catálogo
| ID | Entregable | Depende de | Permite |
|---|---|---|---|
| [M00](M00-ci-baseline.md) | CI, formato, comandos, pruebas verdes | — | M01, M05 |
| [M01](M01-template-ast.md) | Lexer/template AST, errores con spans | M00 | M02 |
| [M02](M02-sfc-ir.md) | SFC estático a IR común | M01 | M03, M04, M06 |
| [M03](M03-rust-macro.md) | Ruta Rust proc macro a mismo IR | M02 contrato IR | M04 |
| [M04](M04-equivalence.md) | Paridad real de sintaxis | M02, M03 | M07 |
| [M05](M05-reactivity.md) | Signals/Memo/Effect/Scope/Scheduler | M00 | M06, M07 |
| [M06](M06-ssr.md) | Render seguro SSR/SSG | M02; M05 para dinámico | M09 |
| [M07](M07-browser-runtime.md) | Runtime browser y teardown | M04, M05 | M08, M09 |
| [M08](M08-tooling.md) | Cargo CLI, SCSS, Vite | M07 | M09 |
| [M09](M09-islands.md) | Static-first/SSR/islas selectivas | M06, M07, M08 | M10 final |
| [M10](M10-benchmarks.md) | benchmark reproducible y resultados | baseline desde M00; final M09 | validar hipótesis |
| [M11](M11-accessibility-security.md) | gates seguridad/accesibilidad | transversal | cualquier release |

## Campos obligatorios al reclamar tarea
- ID, nombre de agente, base SHA y branch, rutas objetivo, bloqueos/decisiones pendientes, pruebas y resultado esperado.
- No reclamar una tarea de la que otro agente ya sea dueño sin coordinación.
- No marcar completado por generar archivos: verificar test + documentación + review + SHA.

## Ruta crítica sugerida hoy
1. Un agente **M00**, corrigiendo formato de CI y reproduciendo baseline.
2. Otro puede **diseñar fixtures de M01**, sin pisar el parser hasta cerrarse M00.
3. Un tercero investiga **M05** en una rama aislada tras M00.
4. El resto trabaja en spec, pruebas y propuestas de ADR sin diseñar API incompatibles.

## DoD transversal
Sin tests, sin evidencia de ejecución y sin actualización de STATUS, la tarea sigue `implemented-unverified`. Nunca reclamar superioridad frente a React sin pruebas.

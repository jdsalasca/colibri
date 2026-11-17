# START HERE — agente implementador

**Misión:** construir Colibrí, un framework web Rust-first, accesible y eficiente en dispositivos modestos. **Dos sintaxis, una semántica; calcula solo lo que cambia, carga solo lo necesario.**

## En 10 minutos
1. Leer `AGENTS.md`, `docs/STATUS.md` y `docs/architecture.md`. Revisa los ADR aplicables.
2. Escoger **una** tarea sin dependencia bloqueada en `docs/tasks/README.md`. No empezar por Vite/HMR, resumabilidad ni un navegador completo.
3. Registrar en el PR: objetivo, rutas a tocar, pruebas, riesgos y decisiones. Si hay varios agentes, acordar dueño de cada archivo antes de editar.
4. Ejecutar `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings` y `cargo test --workspace`. Si faltan herramientas o falla algo previo, indicarlo sin inventar un pase.
5. Hacer PR pequeño con tests y una sección de *handoff*. Actualizar `docs/STATUS.md` y el estado de la tarea **solo tras evidencia**.

## Invariantes no negociables
- Los componentes Rust y SFC deben llegar al mismo IR semántico. No crear dos runtimes.
- `colibri-core` no conoce navegador, DOM, Vite, Node ni Sass.
- Compilación ≠ runtime; ningún parser o compilador se envía al navegador.
- SSR/SSG escapan datos por defecto. No hay HTML sin sanitizar por atajos.
- Los scopes limpian suscripciones/listeners/tareas; nunca "optimizar" perdiendo corrección.
- Priorizar accesibilidad, rutas seguras, errores legibles y tamaño de salida.
- No afirmar que *WASM es siempre más rápido*, ni que los objetivos de rendimiento ya se alcanzaron.

## Primer camino crítico
`M01 template AST` → `M02 SFC→IR` → `M03 Rust→IR` → `M04 parity`. Puede correr en paralelo la especificación/pruebas de lifecycle de `M05` una vez estabilizado el contrato. No comenzar browser tooling hasta completar esas pruebas semánticas.

## Salidas esperadas de cada PR
- Código pequeño y probado.
- Pruebas de errores y casos límite.
- Justificación de dependencias nuevas y coste en runtime.
- Docs y ejemplos actualizados.
- Estado real: implementado / parcial / bloqueado.
- Enlaces a issues/ADR y explicación reproducible de la validación.

**No combines varios hitos en un PR; no hagas cambios especulativos en toda la arquitectura.**

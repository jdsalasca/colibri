# Contratos técnicos (propuestas)

**Ninguna API se considera estable hasta que exista implementación, tests y ADR.** Estos documentos orientan el trabajo de agentes y deben evolucionar junto al código, nunca por separado.

1. [sfc.md](sfc.md): sintaxis, parsing y estilos de `.colibri`.
2. [components.md](components.md): props, children, state, eventos e interoperabilidad con Rust.
3. [ir.md](ir.md): representación semántica común.
4. [reactivity.md](reactivity.md): Signals, Memo, Effect, Scope, Scheduler.
5. [rendering.md](rendering.md): HTML, SSR/SSG, DOM, islas, hidratación.
6. [toolchain.md](toolchain.md): Cargo, Vite, SCSS, CLI y empaquetado.

Contrato implementado: [template-ast.md](template-ast.md) (M01).

## Jerarquía para resolver dudas
`AGENTS.md` y ADR aceptados → contrato de spec → implementación verificada → documentación aspiracional. Cuando exista desacuerdo, abrir ADR y no crear compatibilidad falsa.

## Primer contract test
`.colibri` estático y vista Rust equivalente deben producir el mismo resultado observable sin copiar dos renderers. Cambios a IR requieren pruebas diferenciales y revisión coordinada.

# Documentación de Colibrí — índice canónico

> **Estado:** fundación experimental; los contratos escritos aquí son propuestas de implementación salvo que se indique expresamente lo contrario. La fuente de verdad sobre código ejecutable es `docs/STATUS.md`.

## Ruta para agentes nuevos
1. [START_HERE.md](START_HERE.md): cómo empezar sin alterar invariantes.
2. [STATUS.md](STATUS.md): exactamente qué existe, qué falta y qué se ha verificado.
3. [vision.md](vision.md): usuarios, objetivos y límites.
4. [architecture.md](architecture.md): componentes y dependencias permitidas.
5. [spec/](spec/): contratos de lenguaje y runtime que preceden al código.
6. [tasks/README.md](tasks/README.md): backlog ordenado con criterios de aceptación.
7. [agent-workflow.md](agent-workflow.md): coordinación de agentes, PRs y handoffs.
8. [testing.md](testing.md), [security.md](security.md), [benchmarks.md](benchmarks.md): puertas de calidad.
9. [adr/](adr/): decisiones y asuntos pendientes.
10. [precedentes](../precedentes/README.md): repositorios de referencia para estudio, no dependencias vendorizadas.

## Estado normativo
- **Accepted ADR:** decisión arquitectónica adoptada, aún puede estar sin implementar.
- **Proposed specification:** contrato para discusión; no garantizar API pública todavía.
- **Implemented:** respaldado por código + test que pasa.
- **Verified:** comprobado en CI reproducible y documentado.
- No atribuir a Colibrí características de otros frameworks.
- Las páginas de `spec/` son diseño, no evidencia de implementación.

## Mapa rápido
| Pregunta | Documento |
|---|---|
| ¿Qué queremos construir? | [Visión](vision.md) |
| ¿Dónde pongo código? | [Arquitectura](architecture.md) |
| ¿Cómo defino un `.colibri`? | [Sintaxis SFC](spec/sfc.md) |
| ¿Qué comparten `.rs` y `.colibri`? | [Contrato de componentes](spec/components.md), [IR](spec/ir.md) |
| ¿Cómo funcionan señales y efectos? | [Reactividad](spec/reactivity.md) |
| ¿Cómo se renderiza? | [Renderizado](spec/rendering.md) |
| ¿Cómo integro Rust, Vite y Sass? | [Toolchain](spec/toolchain.md) |
| ¿Cómo construyo componentes y SSR? | [Componentes](guide/components.md) |
| ¿Cómo funcionan hooks, async y lazy? | [Hooks](guide/hooks.md) |
| ¿Qué estándar de calidad aplico? | [Calidad](quality.md) |
| ¿Qué hago a continuación? | [Backlog](tasks/README.md) |

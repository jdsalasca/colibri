# Arquitectura de referencia — Colibrí v0.1

> **Diseño objetivo, NO estado de implementación.** Estado verificado: [STATUS.md](STATUS.md). ADR base: [0001](adr/0001-dual-syntax-single-ir.md), [0002](adr/0002-static-first.md).

## Contrato fundamental
Dos sintaxis, **una sola semántica observable**. Rust `#[component]`/`view!` y archivos `.colibri` deben converger en un IR compartido o en una forma compilada demostrablemente equivalente. La IR es una estructura de **compilación**, no un árbol obligatorio residente en el cliente. Ninguna sintaxis tiene privilegios ocultos en runtime.

~~~text
Rust component (.rs)          Colibrí SFC (.colibri)
  proc macro / Rust AST          SFC blocks / template AST / Rust script
             \                    /
               normalized, typed component model
                         |
                  semantic IR + spans
                         |
             analyses / constant folding / bindings
                         |
         +---------------+---------------+
         |               |               |
   Static HTML/CSS     Server Rust    Client Rust/WASM
   (zero client RT)       SSR          (selective islands)
                                         |
                                 thin Web APIs bridge
~~~

## Paquetes actuales y dependencias
| Crate | Propósito | Puede depender de | Nunca debe depender de |
|---|---|---|---|
| `colibri-ir` | modelo semántico / vistas | biblioteca estándar inicialmente | core, DOM, browser, Vite |
| `colibri-parser` | SFC, templates, posiciones | ir en lowering futuro; bibliotecas de parser cuando se justifiquen | DOM, SSR |
| `colibri-core` | señales, scopes, scheduler | std, abstractions explicit | parser, Vite, DOM |
| `colibri-ssr` | render seguro y streaming futuro | ir, core cuando SSR dinámico llegue | Vite, browser bindings |
| `colibri-cli` | herramientas orquestadoras | parser, codegen, Cargo, sistemas de build | client runtime |
| `vite-plugin-colibri` | integración JS/TS sólo de desarrollo | Vite, puentes a CLI | core runtime |

## Paquetes planificados, NO crear sin tarea/ADR
`colibri-macros` (proc macros), `colibri-codegen` (IR a Rust), `colibri-dom` (adaptador WASM/browser), `colibri-router` (rutas), `colibri-test` (harness). Paquetes separados sólo cuando exista un contrato y caso funcional que justifique la división.

## Flujos compilación
1. Leer el SFC y extraer bloques conservando offsets y ubicaciones. Nunca interpretar código Rust con expresiones regulares.
2. Parsear template a AST estructurado, seguro para anidamiento y texto UTF-8.
3. Validar sintaxis Rust con rustc/syn en su contexto; emitir errores con span de SFC.
4. Normalizar props, children, eventos, bindings, expresiones y atributos en la IR.
5. Identificar subárboles estáticos, dependencias, identidad de nodos y necesidades de cliente.
6. Generar código Rust tipado y manifiestos de assets (Sass compilado fuera del navegador).
7. Cargo compila servidor/WASM; adaptador JS mínimo enlaza APIs DOM si hacen falta.
8. Verificar equivalencia estática y dinámica entre origen Rust y SFC.

## Flujos runtime
- **Static**: emitir HTML/CSS; 0 bytes de código Colibrí cliente.
- **SSR**: computar HTML seguro en servidor Rust; opcional streaming futuro.
- **Islands**: markup inicial + activación de componentes interactivos; medir chunks/arranque.
- **Client**: app completa Rust/WASM si tiene justificación funcional.
- El estado se vincula a scopes, que determinan vida útil de effects/listeners/tareas.
- El DOM backend usa Web APIs, con batching sólo donde preserve comportamiento observable.
- Scheduling cooperativo: no pretender interrumpir arbitrariamente WASM síncrono.

## Límites de responsabilidad
- `parser` no ejecuta Rust, ni genera HTML de usuario directamente.
- `IR` representa semántica y referencias; no contiene APIs del DOM ni asignadores del navegador.
- `core` no renderiza ni conoce HTML.
- `ssr` escapa contenido y rechaza operaciones inseguras.
- `dom` (futuro) manipula nodos a través de APIs browser y limpia listeners al desmontar.
- `cli/tooling` nunca se distribuye en bundle de producción.

## Compatibilidad y evolución
- Evitar publicar una API estable antes de tests de equivalencia.
- Cada cambio incompatible a SFC/IR requiere nota de migración y ADR.
- Los ejemplos conceptuales deben llevar etiqueta "no implementado".
- En caso de ambigüedad: preferir corrección y estándares web antes que atajos de rendimiento.

## Contratos detallados
- [SFC](spec/sfc.md), [componentes](spec/components.md), [IR](spec/ir.md), [reactividad](spec/reactivity.md), [renderizado](spec/rendering.md), [toolchain](spec/toolchain.md).
- [Plan de ejecución](tasks/README.md), [seguridad](security.md), [pruebas](testing.md), [benchmarks](benchmarks.md).

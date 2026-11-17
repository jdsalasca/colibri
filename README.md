# 🐦 Colibrí

**Built in Rust. Light by design.**  
**Compute only what changes. Load only what matters.**

Colibrí es un proyecto de investigación y desarrollo de un framework web *Rust-first*, orientado a construir aplicaciones accesibles para equipos de recursos limitados. Su objetivo es minimizar trabajo computacional, memoria adicional, latencia y bytes enviados al navegador **sin sacrificar accesibilidad ni experiencia de desarrollo**.

> **Estado: fundación experimental (v0.0.1). No es aún un framework de producción.**
> Este repositorio contiene prototipos comprobables de un núcleo de señales, parser SFC, representación intermedia y renderizado HTML estático. Aún **no** compila componentes `.colibri` a WASM ni implementa `#[component]`, `view!`, reactividad automática, Vite/HMR, hidratación o enrutamiento.

## La idea: dos sintaxis, un solo motor

- **Rust puro:** componentes `.rs` con macros `#[component]` y `view!` (API objetivo, no implementada).
- **Colibrí SFC:** archivos `.colibri` con `<script lang="rust">`, `<template>` y `<style scoped lang="scss">` (parser inicial implementado; compilación pendiente).
- **Un IR compartido** para representar componentes; el objetivo es que ambas sintaxis generen el mismo comportamiento y coste de ejecución.
- **HTML/CSS primero:** las páginas que no requieren interactividad no deberían descargar un runtime cliente.
- **WASM selectivo, no dogmático:** JavaScript mínimo cuando sea la opción de mejor rendimiento.

## Lo que sí funciona hoy

| Paquete | Estado | Función |
|---|---|---|
| `colibri-core` | Prototipo | `Signal<T>`, suscripciones y limpieza por `Scope` (notificación síncrona; sin seguimiento automático) |
| `colibri-parser` | Prototipo | Extrae y valida bloques básicos de archivos `.colibri` |
| `colibri-ir` | Modelo inicial | Nodos compartidos para vistas estáticas y enlaces dinámicos |
| `colibri-ssr` | Prototipo | Genera HTML seguro para nodos **estáticos** del IR |
| `colibri-cli` | Prototipo | Comando `check` para validar estructura SFC |

## Primeros pasos

Necesitas [Rust estable](https://rustup.rs/).

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p colibri-cli -- check examples/counter.colibri
```

> `colibri check` valida **estructura de bloques**, no tipos Rust, atributos HTML, bindings ni interactividad. El compilador y el plugin Vite todavía no están implementados.

## Proyecto

```text
crates/
  colibri-core/       Estado, suscripciones, ownership básico
  colibri-ir/         Modelo de representación de componentes
  colibri-parser/     Lectura de componentes de archivo único
  colibri-ssr/        Renderizador seguro de HTML estático
  colibri-cli/        Utilidades para el desarrollador
packages/
  vite-plugin-colibri/  Contrato del futuro plugin (documentación)
docs/
  architecture.md
  roadmap.md
  benchmarks.md
  publishing.md
  adr/
examples/
  counter.colibri       Ejemplo de sintaxis propuesta (no ejecutable)
precedentes/
  README.md, frameworks.tsv, fetch.sh, notes/
AGENTS.md                  Instrucciones técnicas para colaboradores y agentes
```

## Compromisos de diseño

1. **Eficiencia medida, no prometida:** baselines, hardware, repetición y registros de las pruebas.
2. **Correctitud, seguridad y accesibilidad:** escapar HTML, validar enlaces, ciclo de vida de recursos, semántica de DOM y teclado.
3. **Interoperabilidad:** estándares web, SCSS, Vite y módulos JS cuando corresponda.
4. **Separación de compilador y runtime:** la herramienta de desarrollo no debe convertirse en coste del usuario final.
5. **No crear otro React por inercia:** revisar React, Solid, Svelte, Qwik, Leptos, Dioxus, Astro y Yew con espíritu crítico.

## Implementación por agentes

**Empieza por [START HERE](docs/START_HERE.md)** y [el índice de documentación](docs/README.md). Los contratos de sintaxis, IR, reactividad, renderizado y toolchain están en [`docs/spec/`](docs/spec/). El [backlog M00–M11](docs/tasks/README.md) contiene dependencias, ownership sugerido, criterios de aceptación y handoffs. También hay [protocolo multiagente](docs/agent-workflow.md), [estado real](docs/STATUS.md), [modelo de amenazas](docs/security.md) y [pruebas](docs/testing.md).

Consulta [arquitectura](docs/architecture.md), [roadmap](docs/roadmap.md), [plan de benchmarks](docs/benchmarks.md), [decisiones arquitectónicas](docs/adr/) y [precedentes](precedentes/).

## Publicación

El repositorio oficial es [jdsalasca/colibri](https://github.com/jdsalasca/colibri). Los cambios se revisan en ramas y pull requests; consulta [contribución/publicación](docs/publishing.md).

## Contribución y licencia

Proyecto inicial. Antes de aceptar código de terceros o distribuir un framework público, decidir explícitamente licencia, política de contribuciones y política de seguridad. **No se ha elegido una licencia aún.** No copiar código de los frameworks estudiados sin revisar licencias, avisos y obligaciones.
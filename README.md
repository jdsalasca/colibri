# 🐦 Colibrí

**Built in Rust. Light by design.**  
**Compute only what changes. Load only what matters.**

Colibrí es un proyecto de investigación y desarrollo de un framework web *Rust-first*, orientado a construir aplicaciones accesibles para equipos de recursos limitados. Su objetivo es minimizar trabajo computacional, memoria adicional, latencia y bytes enviados al navegador **sin sacrificar accesibilidad ni experiencia de desarrollo**.

> **Estado: fundación experimental (v0.0.1). No es aún un framework de producción.**
> El subconjunto estático ya se renderiza a HTML desde `.colibri` y desde la macro `view!`. Aún **no** hay compilación a WASM, `#[component]`, integración reactiva con DOM, Vite/HMR, hidratación ni enrutamiento.

## La idea: dos sintaxis, un solo motor

- **Rust puro:** `view!` estático implementado; `#[component]`, props y eventos son API objetivo.
- **Colibrí SFC:** archivos `.colibri` con `<script lang="rust">`, `<template>` y `<style scoped lang="scss">` (parser y render estático implementados; compilación pendiente).
- **Un IR compartido** para representar componentes; el objetivo es que ambas sintaxis generen el mismo comportamiento y coste de ejecución.
- **HTML/CSS primero:** las páginas que no requieren interactividad no deberían descargar un runtime cliente.
- **WASM selectivo, no dogmático:** JavaScript mínimo cuando sea la opción de mejor rendimiento.

## Lo que sí funciona hoy

| Paquete | Estado | Función |
|---|---|---|
| `colibri-core` | Prototipo | Signals, scopes y motor con `use_state`, `use_memo`, `use_effect`, `use_lazy`, `Resource`, `batch`, ciclos recuperables (`take_error`), `stats()` y `Scope::dispose` |
| `colibri-dom` | Prototipo | Runtime DOM/WASM: `View`, `dom!`, `mount`, effects, eventos, `use_async`/`Resource`, `Dynamic` y `Keyed` (M07 parcial) |
| `colibri-parser` | Prototipo | Scanner SFC y AST de template con spans (M01); no compila |
| `colibri-ir` | Prototipo | IR semántica con spans y lowering SFC→IR (M02) |
| `colibri-codegen` | Prototipo | SFC → programa Rust; `build`/`eval` con rustc real (M06b) |
| `colibri-macros` | Prototipo | `#[component]` con props tipadas y `view!` dinámico/composición (M03/M09) |
| `colibri-ssr` | Prototipo | Render HTML estático seguro y SSR dinámico con resolver explícito y errores por span (M02/M06a) |
| `colibri-cli` | Prototipo | `check`, `render` (estático/`--bind`), `build` y `eval` |

## Dos sintaxis, el mismo IR

```rust
use colibri_macros::view;

let node = view! { <article class="card"><h1>"Título"</h1></article> };
```

```html
<template><article class="card"><h1>Título</h1></article></template>
```

Ambas rutas producen el mismo IR normalizado y el mismo HTML estático; la suite de paridad lo verifica en cada ejecución de tests.

## Componentes funcionales

```rust
use colibri_ir::Node;
use colibri_macros::{component, view};

#[component]
pub fn Greeting(name: String) -> Node {
    view! { <p>"Hola " {name}</p> }
}

let node = view! { <Greeting name="Ada"/> };
```

Props tipadas generadas por el macro, composición y `Option<T>` para null safety. Guía: [componentes y SSR](docs/guide/components.md).

## Cliente WASM

```rust
use colibri_core::{use_memo, use_state};
use colibri_dom::{mount, View};
use colibri_macros::{component, dom};

#[component]
pub fn Counter() -> View {
    let count = use_state(0);
    let text = count.clone();
    let click = count.clone();
    let doubled = use_memo({
        let count = count.clone();
        move || count.get() * 2
    });
    dom! {
        <div>
            <p>{text.get()}</p>
            <p>"doble: " {doubled.get()}</p>
            <button on:click={move |_| click.update(|n| *n += 1)}>"+1"</button>
        </div>
    }
}
```

Build y demo: [`colibri-dom/demo`](crates/colibri-dom/demo/README.md). `dom!` acepta eventos y actualiza bindings sin reconstruir el árbol. `use_async` devuelve `Signal<Resource<T>>`, `use_lazy` cachea un cálculo, `Dynamic` reconstruye condicionales y `Keyed` reordena preservando identidad y foco.

## Primeros pasos

Necesitas [Rust estable](https://rustup.rs/).

```bash
cargo fmt --all --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo run -p colibri-cli -- check examples/counter.colibri
cargo run -p colibri-cli -- render examples/static-card.colibri
cargo run -p colibri-cli -- render examples/greeting.colibri --bind name=Ada --bind message=Bienvenida
cargo run -p colibri-cli -- eval examples/greeting.colibri
```

> `colibri check` valida **estructura de bloques y template**, no tipos Rust. `colibri render` emite HTML estático o resuelve `{expr}` con `--bind key=value`; no evalúa Rust. `colibri build` emite el programa Rust y `colibri eval` lo compila y ejecuta con rustc (no es sandbox; los eventos siguen fuera del SSR). El plugin Vite todavía no está implementado.

## Proyecto

```text
crates/
  colibri-core/       Signals, memo, effects, scheduler y scopes
  colibri-dom/        Runtime DOM/WASM con `dom!` y `mount`
  colibri-codegen/    SFC a programa Rust ejecutable
  colibri-ir/         Modelo de representación de componentes
  colibri-macros/     Macro proc `view!` para la ruta Rust
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
  static-card.colibri   SFC estático que renderiza hoy
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

## Calidad

El código nuevo sigue el baseline de Rust-teacher (repo local `teachers`) y se audita con su runner portátil. Detalles y deuda pendiente en [quality.md](docs/quality.md).

## Implementación por agentes

**Empieza por [START HERE](docs/START_HERE.md)** y [el índice de documentación](docs/README.md). Los contratos de sintaxis, IR, reactividad, renderizado y toolchain están en [`docs/spec/`](docs/spec/). El [backlog M00–M11](docs/tasks/README.md) contiene dependencias, ownership sugerido, criterios de aceptación y handoffs. También hay [protocolo multiagente](docs/agent-workflow.md), [estado real](docs/STATUS.md), [modelo de amenazas](docs/security.md) y [pruebas](docs/testing.md).

Consulta [arquitectura](docs/architecture.md), [roadmap](docs/roadmap.md), [plan de benchmarks](docs/benchmarks.md), [decisiones arquitectónicas](docs/adr/) y [precedentes](precedentes/).

## Publicación

El repositorio oficial es [jdsalasca/colibri](https://github.com/jdsalasca/colibri). Los cambios se revisan en ramas y pull requests; consulta [contribución/publicación](docs/publishing.md).

## Contribución y licencia

Proyecto inicial. Antes de aceptar código de terceros o distribuir un framework público, decidir explícitamente licencia, política de contribuciones y política de seguridad. **No se ha elegido una licencia aún.** No copiar código de los frameworks estudiados sin revisar licencias, avisos y obligaciones.
## Investigación y motor reactivo

- [API experimental del core](crates/colibri-core/README.md): `use_state`, `use_memo`, `use_effect`, cleanup, `batch`, `take_error()` y `stats()`.
- [Investigación sobre rendimiento web Rust/WASM](docs/research/2026-11-rust-web-performance.md): fuentes primarias, hipótesis falsables y diseño de benchmarks.
- [Precedentes reactivos](precedentes/reactivity-and-web-performance.md): React, Solid, Svelte, Leptos, Dioxus y otros.

Esto es una fundación reactiva, no una prueba de superioridad de rendimiento ni un motor de navegador operativo.

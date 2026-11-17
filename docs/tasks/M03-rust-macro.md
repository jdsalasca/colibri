# M03 — Camino Rust `view!`/`#[component]` → IR

**Prioridad:** P0. **Depende de:** contrato IR estabilizado M02. **Área:** nuevo `colibri-macros` bajo [ADR 0005](../adr/0005-rust-view-proc-macro.md). **Specs:** [`docs/spec/components.md`](../spec/components.md), [`docs/spec/ir.md`](../spec/ir.md). **Estado:** implementado en local (subconjunto estático); CI pendiente de push.

## Objetivo
Desarrollar la primera ruta de fuente Rust que expresa **el mismo subconjunto semántico estático** del SFC. Resolver higiene de macros y diagnósticos con span.

## Trabajo
1. Evaluar proc-macro crate vs macro_rules, registrar decisión en ADR.
2. Añadir API mínima compilable `view!` y, sólo si es necesario para parity, `#[component]`.
3. Reutilizar especificación de nodos/props validada en M02. No crear un renderer alternativo.
4. Generar salida Rust tipada que consuma IR o un plan semántico equivalente; documentar si parte del IR existe sólo en compile-time.
5. Añadir compile-pass/fail con referencias a errores y tests de spans de macro.

## No hacer
No introducir hooks tipo React, HMR, Vite, runtime DOM, SSR dinámico ni APIs de props avanzadas sin ADR.

## Aceptación
- [x] Una vista estática en Rust compila y renderiza vía contrato compartido.
- [x] Existen compile-fail tests para errores conocidos.
- [x] La macro no añade un segundo sistema de estado/eventos.
- [x] Documentación no promete sintaxis antes de verificar.
- [x] M04 dispone de dos rutas de entrada funcionales.

## Handoff
Ejemplo `.rs` que compila, salida IR/plan y limitaciones de Rust token syntax.

Implementado en `colibri-macros` con `syn`/`quote` (ADR 0005):

```rust
use colibri_macros::view;

let node = view! {
    <article class="card" hidden>
        <h1>"Título"</h1>
        <input type="text">
    </article>
};
```

La raíz es `Node::Fragment`, igual que el lowering SFC. Texto se escribe como literal (`"Título"`), no como HTML crudo. Se aceptan tags anidados, void/autocierre, atributos estáticos/booleanos, nombres con guion (`data-*`, `aria-*`) y Unicode en literales.

Bloqueado a propósito con error de compilación: texto dinámico `{expr}`, atributos `{expr}`, `on:evento` y atributos con namespace. `#[component]`, props y children no se implementaron porque M04 no los necesita todavía. Los spans del IR de la ruta Rust son sintéticos (`file!()`, `0..0`); los errores de sintaxis sí usan el span real del token.

Verificado: 5 pruebas unitarias de parsing, 3 de render, 3 de paridad, 2 doc-tests (uno `compile_fail`) y `cargo clippy -D warnings`.

Ejemplo ejecutable: `cargo run -p colibri-macros --example static-card`.

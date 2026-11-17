# M03 — Camino Rust `view!`/`#[component]` → IR

**Prioridad:** P0. **Depende de:** contrato IR estabilizado M02. **Área:** nuevo `colibri-macros` bajo ADR. **Specs:** `docs/spec/components.md`, `docs/spec/ir.md`.

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
- [ ] Una vista estática en Rust compila y renderiza vía contrato compartido.
- [ ] Existen compile-fail tests para errores conocidos.
- [ ] La macro no añade un segundo sistema de estado/eventos.
- [ ] Documentación no promete sintaxis antes de verificar.
- [ ] M04 dispone de dos rutas de entrada funcionales.

## Handoff
Ejemplo `.rs` que compila, salida IR/plan y limitaciones de Rust token syntax.

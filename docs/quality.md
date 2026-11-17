# Calidad de código — baseline `teachers`

Colibrí adopta el baseline de **Rust-teacher** (`C:\Users\jdsal\Documents\Programming-personal\teachers`)
para el código nuevo y lo audita con su runner portátil:

```powershell
cd C:\Users\jdsal\Documents\Programming-personal\teachers
python tools/lint.py --project "C:\Users\jdsal\Documents\Programming-personal\colibri"
```

## Aplicado hoy

- `rustfmt.toml`: `max_width = 100`.
- `clippy.toml`: complejidad 8, 4 parámetros, 40 líneas por función.
- `colibri-codegen` activa `missing_docs`, `unsafe_code = forbid`, `unwrap_used`,
  `expect_used`, `panic`, `too_many_arguments`, `too_many_lines` y
  `cognitive_complexity` como `deny`.
- Los tests permiten `unwrap`/`expect`/`panic` con un `#![allow]` local y motivo.
- CI ejecuta `cargo fmt --all --check`, `clippy -D warnings` y tests.

## Resultado del audit

- `cargo fmt --check`, Clippy y 80 pruebas: limpios.
- `python tools/lint.py --project ...` ya no encuentra `unwrap/expect` en librería ni falta de README.
- Hallazgo pendiente: 4 archivos superan 400 líneas:
  - `crates/colibri-parser/src/lib.rs` (1290)
  - `crates/colibri-core/src/reactive.rs` (706)
  - `crates/colibri-macros/src/lib.rs` (581)
  - `crates/colibri-ssr/src/lib.rs` (491)

Plan: dividir parser SFC/template, motor reactivo, macros y tests en módulos en una ronda de calidad dedicada. No es un problema de correctitud, pero el audit estricto seguirá en rojo hasta hacerlo.

## Todavía no aplicado al repo completo

- `missing_docs` y `unwrap_used` en todos los crates: `colibri-core` mantiene dos `expect` justificados en `Memo` (invariante de inicialización) y saturación de ids.
- Cobertura con `cargo llvm-cov --fail-under-lines 60` (Rust-teacher).
- Mutation testing, OSV y auditoría de dependencias en CI.

## Regla para código nuevo

1. Tests primero cuando sea posible.
2. Funciones ≤40 líneas, complejidad ≤8, ≤4 parámetros.
3. API pública documentada; sin `unwrap`/`expect`/`panic` fuera de tests.
4. Si se necesita una excepción, `#[allow(..., reason = "...")]` y justificarla
   en el PR/handoff.
5. El audit de `teachers` debe pasar antes de publicar.

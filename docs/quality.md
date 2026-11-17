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

## Todavía no aplicado al repo completo

- `missing_docs` y `unwrap_used` en los crates existentes: hay deuda de docs y
  tests que usan `unwrap`; activarlo exige una ronda de refactor y allowances
  justificadas por test.
- Cobertura con `cargo llvm-cov --fail-under-lines 60` (Rust-teacher).
- Mutation testing, OSV y auditoría de dependencias en CI.
- Límite de 400 líneas por archivo: `reactive.rs` y `parser` superan el umbral.

## Regla para código nuevo

1. Tests primero cuando sea posible.
2. Funciones ≤40 líneas, complejidad ≤8, ≤4 parámetros.
3. API pública documentada; sin `unwrap`/`expect`/`panic` fuera de tests.
4. Si se necesita una excepción, `#[allow(..., reason = "...")]` y justificarla
   en el PR/handoff.
5. El audit de `teachers` debe pasar antes de publicar.

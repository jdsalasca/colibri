# M00 — CI y baseline reproducible

**Prioridad:** P0. **Depende de:** nada. **Área/owner:** CI, workspace, tests. **Estado:** parcial (local verde; CI pendiente de push).

## Problema
El repositorio contiene prototipos, pero no se ha demostrado una ejecución de CI verde. La fundación debe volverse verificable antes de hacer crecer el compilador.

## Trabajo delimitado
1. Ejecutar workspace con toolchain estable, registrar versión Rust y commit.
2. Corregir errores de formato/Clippy/compilación sin cambiar semántica pública a menos que un test lo exija.
3. Garantizar que CI use `cargo fmt --all --check`, no `cargo fmt --all` que cambia archivos de runner y esconde problemas.
4. Confirmar `cargo test --workspace`; registrar outputs y resultados.
5. Ejecutar CLI sobre `examples/counter.colibri` sólo como check preliminar.
6. Actualizar `docs/STATUS.md` y dejar CI URL.

## No hacer
No introducir macros nuevas, Vite, optimizaciones, dependencias pesadas o benchmarks que comparen frameworks.

## Aceptación
- [ ] Workflow CI se ejecuta en PR real y termina verde.
- [x] Formato check, Clippy y tests pasan sobre el mismo SHA.
- [x] README contiene comandos exactos que pasan.
- [x] El `check` no se anuncia como compilación.
- [x] Se documentan limitaciones detectadas y cualquier deuda que no pueda cerrarse aquí.

## Handoff
Entregar SHA de baseline, captura textual/resumen de CI y lista de condiciones que habilitan M01/M05.

Verificado en local (2026-11-16, Windows 11, rustc 1.99.0): `cargo fmt --all --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace` (9 pruebas) y `colibri check examples/counter.colibri`. CI actualizado a `--check` y con paso de `check`; falta la ejecución en GitHub tras el push.

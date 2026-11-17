# M02 — Lowering de SFC a IR

**Prioridad:** P0. **Depende de:** M01. **Área:** `colibri-parser` / `colibri-ir`. **Specs:** `docs/spec/sfc.md`, `docs/spec/ir.md`.

## Objetivo
Tomar el AST de `.colibri` y convertir un subconjunto estático de templates a IR semántica validada y renderizable.

## Trabajo
1. Documentar/afinar tipos IR para `Element`, `StaticText`, `Attribute`, `Fragment` y `SourceSpan`.
2. Mapear AST→IR sin perder whitespace significativo, UTF-8, ruta/spans.
3. Marcar y rechazar de forma tipada expresiones dinámicas no implementadas; nunca descartarlas silenciosamente.
4. Validar tags/attrs según subconjunto explícito y separar escaping de parser (renderer es responsable del contexto).
5. Añadir golden tests de lowering + errores, incluida nested markup y props futuras claramente bloqueadas.

## No hacer
No crear runtime SFC propio, no implementar macros Rust en este ticket, no añadir Wasm/Node, no interpretar Rust strings arbitrarias.

## Aceptación
- [ ] Un SFC estático produce IR y HTML semántico esperado mediante `colibri-ssr`.
- [ ] Un binding dinámico no admitido causa diagnóstico preciso, no HTML incompleto.
- [ ] Tests de atributos, texto, anidamiento, entidades, Unicode y spans pasan.
- [ ] IR tiene contrato documentado para consumo de M03.
- [ ] No se copian reglas de escape HTML en varias capas.

## Handoff
Modelo IR acordado, tests de referencia y superficie pendiente de props/children.

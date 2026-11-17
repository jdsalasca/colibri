# M02 — Lowering de SFC a IR

**Prioridad:** P0. **Depende de:** M01. **Área:** `colibri-parser` / `colibri-ir`. **Specs:** [`docs/spec/ir.md`](../spec/ir.md), [`docs/spec/sfc.md`](../spec/sfc.md). **Estado:** implementado en local (2026-11-16); CI pendiente de push. La decodificación de entidades HTML de entrada queda pendiente.

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
- [x] Un SFC estático produce IR y HTML semántico esperado mediante `colibri-ssr`.
- [x] Un binding dinámico no admitido causa diagnóstico preciso, no HTML incompleto.
- [x] Tests de atributos, texto, anidamiento, entidades de salida, Unicode y spans pasan.
- [x] IR tiene contrato documentado para consumo de M03.
- [x] No se copian reglas de escape HTML en varias capas.

## Handoff
Modelo IR acordado, tests de referencia y superficie pendiente de props/children.

Verificado en local: 34 pruebas de workspace, incluidas 6 de pipeline `.colibri → AST → IR → HTML` en `crates/colibri-parser/tests/static_pipeline.rs`. `colibri-cli render examples/static-card.colibri` emite HTML; `render examples/counter.colibri` falla con span sobre `{count}`.

IR actual: `SourceSpan`, `Component`, `Node::{Text, DynamicText, Element, Fragment}`, `Element`, `Attribute::{Static, Boolean, Dynamic, Event}`. Los eventos guardan el nombre sin `on:`; los comentarios se descartan en lowering; el autocierre es sintáctico y no viaja al IR.

Pendiente para M03/M06: componentes y props, control de flujo, decodificación de entidades HTML, evaluación de expresiones, eventos en cliente y línea/columna en errores de render.

# M01 — Lexer + AST de template SFC

**Prioridad:** P0. **Depende de:** M00. **Área:** `colibri-parser`. **Spec:** `docs/spec/sfc.md`.

## Objetivo
Reemplazar la extracción superficial por tokens y AST estructurado de HTML/template, preservando fuente y spans. No generar Rust todavía.

## Salidas API propuestas
- `SourceSpan { file, start_byte, end_byte }`.
- `TemplateNode` con `Element`, `Text`, `Interpolation`, `Fragment` y `Comment` si conviene para spans.
- `ParseDiagnostic { code, message, span, hint }`.
- Mantener una capa separada de extracción de `script/template/style`, sin confundir cierres dentro de strings/comentarios.

## Trabajo
1. Definir gramática limitada y errores explícitos; no intentar toda HTML5 si el parser no la soporta.
2. Implementar scanner SFC con manejo de strings/comentarios, tags anidados y Unicode.
3. Añadir AST con atributos literales y bindings `{expr}` sin ejecutar expresiones.
4. Añadir snapshots de AST/errores y fixtures pequeños con nested tags, strings que contengan cierres, líneas CRLF y Unicode.
5. Introducir límites de profundidad/tamaño razonables, sin panic por input adversarial.

## No hacer
No ejecutar Rust, no añadir DOM, CSS processor, Vue syntax avanzada, AST a IR ni scheduling.

## Aceptación
- [ ] Parser produce AST anidada con spans precisos.
- [ ] Casos malformed, duplicate, closing tag in literal y Unicode fallan sin panic.
- [ ] Todos los tests previos siguen verdes; API de CLI `check` reporta ruta/línea.
- [ ] Contrato de AST documentado para M02.
- [ ] El parser no evalúa código ni inyecta HTML por sí mismo.

## Handoff
API de AST y ejemplos golden, lista de sintaxis admitida/rechazada y decisiones no tomadas.

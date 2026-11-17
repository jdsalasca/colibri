# M06 — SSR/SSG seguro

**Prioridad:** P1. **Depende de:** M02 para vistas estáticas; M05 para bindings dinámicos. **Área:** `colibri-ssr`. **Specs:** [`docs/spec/rendering.md`](../spec/rendering.md), [`docs/security.md`](../security.md). **ADR:** [0007](../adr/0007-dynamic-ssr-resolver.md). **Estado:** parcial (M06a: resolver explícito, sin evaluación Rust).

## Objetivo
Emitir HTML seguro desde IR compartida. El resultado estático debe tener 0 runtime Colibrí cliente. SSR dinámico se aborda después de fijar evaluación de expresiones y scopes.

## Pasos
1. Auditoría del renderer restrictivo existente: tags, atributos, escapes, URLs.
2. Integración AST/SFC→IR→SSR static; fixtures de HTML con Unicode y nested attrs.
3. Mejorar semántica de atributos booleanos, form controls y ARIA dentro de subconjunto definido.
4. Evaluar expresiones Rust del lado server desde código compilado; no hacer `eval` de strings.
5. Investigar SSR streaming e hidratación en ADR separado (no introducir al MVP).

## Seguridad
URLs por contexto; HTML/attr escaping; sin insert raw HTML; límites de profundidad; sin tags peligrosos por accidente.

## Aceptación
- [x] Pruebas XSS y HTML semántico pasan.
- [x] Página completamente estática descarga cero JS/WASM propio.
- [x] SSR ante expresión no soportada falla claramente, nunca descarta binding.
- [ ] Rust/SFC producen mismo resultado conforme M04 (estático sí; dinámico SFC pendiente en la macro).
- [x] Código de renderer no importa Vite o APIs browser.

## Handoff
Tabla de elementos/atributos soportados, límites, test hostiles y decisión sobre extensibilidad.

Implementado en esta rebanada (ADR 0007):

- `render_static(node)` sin cambios: rechaza dinámicos con span.
- `render_dynamic(node, resolver)`: resuelve `{expr}` y atributos dinámicos; escapa texto/atributos; valida URLs y nombres; `MissingValue` si falta el valor; eventos siguen siendo error de SSR.
- CLI: `colibri render examples/greeting.colibri --bind name=Ada --bind message=Bienvenida`.
- 5 pruebas de SSR + 5 de pipeline dinámico.

Pendiente para cerrar M06: codegen/evaluación Rust real del lado servidor, paridad dinámica con `view!`, streaming, islas e hidratación. El resolver actual es un contrato de host, no un intérprete de Rust.

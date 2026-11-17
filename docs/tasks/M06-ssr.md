# M06 — SSR/SSG seguro

**Prioridad:** P1. **Depende de:** M02 para vistas estáticas; M05 para bindings dinámicos. **Área:** `colibri-ssr` / `colibri-codegen`. **Specs:** [`docs/spec/rendering.md`](../spec/rendering.md), [`docs/spec/toolchain.md`](../spec/toolchain.md), [`docs/security.md`](../security.md). **ADR:** [0007](../adr/0007-dynamic-ssr-resolver.md), [0008](../adr/0008-sfc-rust-codegen.md). **Estado:** M06a (resolver) y M06b (codegen/evaluación Rust) implementados; falta paridad dinámica con `view!`, streaming e hidratación.

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

## M06b — codegen y evaluación Rust (ADR 0008)

`colibri-codegen` genera un programa Rust completo:

- El `<script lang="rust">` se inserta dentro de `fn main`; el compilador valida tipos.
- `{expr}` se convierte en `format!("{}", (expr))`; atributos dinámicos se convierten en valores estáticos ya formateados.
- El árbol se construye en runtime y usa `colibri_ssr::render_static`, reutilizando escape y validación.
- `on:evento` y componentes sin resolver fallan en codegen con span.
- CLI: `build` emite el `.rs`; `eval` crea un proyecto Cargo temporal, compila y ejecuta.
- Ejemplo: `cargo run -p colibri-cli -- eval examples/greeting.colibri`.

Pruebas: 4 unitarias de codegen y 3 de integración que invocan `rustc` real y ejecutan el binario (estático, dinámico y escape XSS). Limitaciones: sin control de flujo, componentes, props, eventos ni SCSS; el script generado no es sandbox; los errores de rustc apuntan al archivo generado.

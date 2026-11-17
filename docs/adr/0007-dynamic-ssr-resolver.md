# ADR 0007 - SSR dinámico con resolver explícito

Fecha: 2026-11-16
Estado: **Aceptada** como primera rebanada de M06.

## Contexto
M02 dejó un IR con `DynamicText`, `Attribute::Dynamic` y `Attribute::Event`, pero el renderer sólo aceptaba nodos estáticos. M05 aporta estado reactivo, no evaluación de expresiones Rust. Evaluar strings de plantilla con un intérprete o `eval` está prohibido por seguridad y por el contrato "las expresiones son Rust".

## Decisión
Añadir `render_dynamic(node, resolver)` donde el llamador entrega `FnMut(&str) -> Option<String>`:

- El resolver recibe el texto de la expresión tal como está en el IR (`"count"`, `"user.name"`).
- Un valor ausente produce `RenderErrorKind::MissingValue` con el span; nunca se sustituye por cadena vacía.
- Todo valor resuelto se escapa como texto o atributo; se mantienen las reglas de nombres, `on*`, `srcdoc`, `style` y allowlist de URLs.
- Los eventos (`on:click`) siguen siendo error de SSR: pertenecen al runtime cliente de M07.
- `render_static` conserva su comportamiento: cualquier nodo dinámico es `DynamicContent`.

El CLI expone el mismo contrato con `render <archivo> --bind key=value`. La evaluación real de expresiones Rust requiere codegen (futuro), no un intérprete.

## Alternativas
- `eval`/intérprete de expresiones: inseguro, divergente de Rust y prohibido por AGENTS.
- Codegen Rust inmediato: solución final, pero exige compiler/codegen y otra ADR; bloquea el resto de M06.
- Renderer que ignora dinámicos: descarta bindings silenciosamente; viola el contrato.
- Resolver obligatorio en todas las APIs: rompe `render_static` y la paridad estática existente.

## Consecuencias
- El llamador decide cómo obtener valores; no hay magia de estado ni evaluación Rust.
- SFC dinámico ya se puede renderizar si el host conoce los valores (CLI, server, tests).
- La paridad dinámica Rust/SFC queda pendiente: la macro `view!` aún rechaza `{expr}`.
- No hay streaming, islas ni hidratación; el HTML dinámico sigue siendo una cadena completa.

## Validación
- 5 pruebas nuevas de `colibri-ssr` (texto/atributo, escape XSS, URL peligrosa, valor ausente, evento).
- 5 pruebas de pipeline `.colibri → IR → render_dynamic` en `crates/colibri-parser/tests/dynamic_pipeline.rs`.
- 68 pruebas de workspace; `clippy -D warnings` limpio.

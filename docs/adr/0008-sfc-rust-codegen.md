# ADR 0008 - Codegen SFC → Rust para evaluación real

Fecha: 2026-11-16
Estado: **Aceptada** como M06b.

## Contexto
M06a resolvía expresiones con un mapa del host. Eso desbloquea SSR dinámico, pero no evalúa Rust: el framework seguía sin ejecutar el `<script lang="rust">` de un SFC ni las expresiones `{expr}` con la semántica del lenguaje. Un intérprete o `eval` de strings está prohibido por seguridad y por el contrato "las expresiones son Rust".

## Decisión
Crear `colibri-codegen` y generar un programa Rust completo:

- El bloque `<script lang="rust">` se inserta dentro de `fn main` (los `use`, `fn`, `struct` e `impl` locales son válidos en Rust).
- Cada `{expr}` se convierte en `format!("{}", (expr))`; el compilador aplica tipos y `Display`.
- Cada atributo dinámico se convierte en `Attribute::Static` con el valor formateado.
- El árbol se construye en runtime y se renderiza con `colibri_ssr::render_static`, reutilizando escape, URL allowlist y validación de tags/atributos.
- Eventos (`on:evento`) y componentes sin resolver se rechazan en codegen con span.
- El CLI expone `build` (emite el `.rs`) y `eval` (crea un proyecto Cargo temporal, compila y ejecuta).

## Alternativas
- Intérprete de expresiones: inseguro, divergente de Rust y prohibido.
- Resolver de host (M06a): útil, pero no evalúa Rust.
- Generar directamente HTML en codegen sin IR: duplicaría las reglas de escape y validación.
- Compilar a WASM en esta fase: prematuro; M07/M09 lo necesitan, M06b no.

## Consecuencias
- El script del SFC se ejecuta como código de desarrollador; `eval` no es un sandbox.
- Errores de tipo/compilación aparecen en el archivo Rust generado, no en el `.colibri`; los errores de render sí llevan spans del SFC.
- Limitaciones: expresiones deben implementar `Display`; no hay control de flujo, componentes, props, eventos ni SCSS; un `#![inner]` o `fn main` en el script choca con el wrapper.
- `eval` compila dependencias en un target temporal (segundos); `build` es la vía barata para inspeccionar el código.
- El IR generado es runtime y no se optimiza en compile-time; el objetivo es corrección y evaluación real, no rendimiento todavía.

## Validación
- 4 pruebas unitarias de codegen (nodos, script, literales, rechazos).
- 3 pruebas de integración que invocan `rustc` real y ejecutan el binario: estático, texto dinámico y escape XSS de texto/atributo.
- `colibri eval examples/greeting.colibri` imprime el HTML con valores calculados por Rust.
- 75 pruebas de workspace; `clippy -D warnings` y audit portátil de `teachers` limpios.

# ADR 0005 - Proc macro `view!` para la ruta Rust

Fecha: 2026-11-16
Estado: **Aceptada** para el subconjunto estático (M03).

## Contexto
La ruta Rust necesita expresar el mismo subconjunto semántico que un template SFC. El código de entrada es una secuencia de tokens Rust, no HTML en texto. Parsearlo con expresiones regulares o un scanner casero reproduce los errores que el proyecto quiere evitar: spans imprecisos, strings/expresiones mal delimitadas y diagnósticos engañosos.

## Decisión
Crear `colibri-macros` como proc-macro crate y usar `syn` + `quote` + `proc-macro2` para parsear la entrada y generar construcción de `colibri_ir`.

- `view!` acepta elementos, texto como literal de cadena, atributos estáticos y booleanos, tags anidados y void/autocierre.
- Texto interpolado, atributos dinámicos y `on:evento` se rechazan en compilación con un mensaje que nombra el hito que los habilita (M05/M06/M07).
- La raíz generada es siempre `Node::Fragment`, igual que el lowering SFC.
- Los spans del IR generado son sintéticos (`file!()`, `0..0`): los proc macros estables no exponen offsets de byte del archivo. Los errores de sintaxis sí usan el span real del token y rustc muestra línea/columna.

## Alternativas
- `macro_rules!`: sin dependencias, pero no puede analizar expresiones Rust ni dar spans útiles; obligaría a una sintaxis más pobre y a duplicar el parser.
- Proc macro con parser propio de tokens: evita `syn`, pero reimplementa Rust parcialmente y falla con casos que `syn` ya cubre.
- DSL de constructores (`el("p").text(...)`): compila hoy sin macros, pero rompe la paridad de sintaxis con SFC y no es la API objetivo.

## Consecuencias
- Dependencias nuevas de build (`syn`, `quote`, `proc-macro2`); no llegan al bundle del navegador.
- El crate que use `view!` debe depender de `colibri-ir`.
- Sin offset de byte en el IR de la ruta Rust; M04 compara IR ignorando spans y M07/M09 pueden añadir un source map si hace falta.
- `#[component]`, props, children, eventos y reactividad quedan fuera de M03 y requieren ADR/contrato propios.

## Validación
- Tests de expansión feliz y de errores en `colibri-macros`.
- `cargo test --workspace` compila y renderiza la ruta Rust con `colibri-ssr`.
- M04 compara IR normalizado y HTML de ambos caminos.

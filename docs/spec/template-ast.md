# Contrato del AST de template (M01)

**Estado:** implementado en `colibri-parser` y verificado en local. Es el contrato de entrada para M02 (lowering a IR). No describe compilación, evaluación de Rust ni renderizado.

## Punto de entrada

```rust
pub fn parse(file: &str, source: &str) -> Result<SingleFileComponent, ParseError>;
```

`file` sólo se usa en spans y diagnósticos. El parser no lee archivos ni ejecuta código.

```rust
pub struct SingleFileComponent {
    pub script: Option<ScriptBlock>,
    pub template: Template,
    pub style: Option<StyleBlock>,
}
```

- Exactamente un `<template>` no vacío.
- `script` y `style` son opcionales; los duplicados son error.
- Comentarios HTML en el nivel superior se ignoran.

## Tipos

| Tipo | Campos | Notas |
|---|---|---|
| `SourceSpan` | `file`, `start_byte`, `end_byte` | Fin exclusivo; `span.text(source)` devuelve el fragmento o `None`. |
| `Template` | `nodes`, `span` | `span` cubre desde `<template>` hasta `</template>`. |
| `TemplateNode` | `Element`, `Text`, `Interpolation`, `Comment` | `node.span()` en todos los casos. |
| `TemplateElement` | `tag`, `attributes`, `children`, `span`, `self_closing` | `span` cubre el elemento completo. |
| `TemplateText` | `value`, `span` | Whitespace significativo, sin recortar. |
| `TemplateInterpolation` | `expression`, `span` | Texto Rust entre llaves, sin evaluar. |
| `TemplateComment` | `value`, `span` | Sólo `<!-- ... -->`. |
| `TemplateAttribute` | `name`, `value`, `span` | Nombre con mayúsculas original. |
| `AttributeValue` | `Boolean`, `Literal(String)`, `Expression(String)` | `{expr}` se guarda como expresión, no se evalúa. |

Los offsets son bytes UTF-8 sobre el archivo original. `end_byte` es exclusivo y siempre cae en un límite de carácter.

## Reglas del template

- Tags y atributos anidados; elementos void y autocierre (`<br>`, `<br/>`).
- Las etiquetas de cierre deben coincidir; los tags en minúscula se comparan sin distinguir mayúsculas y los componentes con mayúsculas de forma exacta.
- `{ ... }` es una interpolación. Las llaves internas se equilibran y se ignoran las que aparecen en strings, chars, raw strings y comentarios Rust.
- El texto se conserva tal cual, incluidos CRLF y Unicode.
- `<template>` raíz puede contener varios nodos hermanos.

## Fronteras de bloques

- `</script>` no cierra si aparece dentro de strings (`"…"`, `b"…"`, `r#"…"#`), chars o comentarios Rust (incluidos bloques anidados).
- `</style>` no cierra si aparece dentro de comentarios CSS o strings entre comillas.
- Los cierres de template se resuelven con anidamiento real, no con una búsqueda textual.

## Límites y seguridad

- Tamaño máximo: `MAX_SOURCE_BYTES` (1 MiB); por encima se rechaza antes de parsear.
- Profundidad máxima: `MAX_TEMPLATE_DEPTH` (128); no hay recursión sin cota.
- No se evalúa Rust, no se genera HTML y no se decodifican entidades.
- Se rechazan atributos de evento en forma de string (`onclick="…"`); la forma admitida es `on:evento={handler}`.
- Se rechazan atributos duplicados para evitar ambigüedad.
- El parser no debe entrar en pánico con entradas hostiles; hay una prueba que alimenta todos los prefijos de un SFC válido.

## Diagnósticos

`ParseError` expone `code`, `message`, `span`, `line`, `column` y `hint`. `line` y `column` son 1-based. `Display` produce:

```text
archivo.colibri:3:1: [unexpected-closing-tag] expected </div>, found </span>
```

Códigos: `source-too-large`, `invalid-top-level`, `unsupported-block`, `missing-closing-tag`, `unexpected-closing-tag`, `duplicate-block`, `missing-template`, `invalid-script-language`, `invalid-style-language`, `invalid-tag-name`, `invalid-attribute-name`, `invalid-attribute-value`, `duplicate-attribute`, `unsafe-event-handler`, `unclosed-comment`, `unclosed-tag`, `unclosed-attribute`, `unclosed-expression`, `unclosed-literal`, `depth-limit`.

## Límites conocidos (no implementado)

- No es un parser HTML5: no infiere tags, no decodifica entidades, no admite doctype, CDATA ni contenido crudo dentro de `<template>`.
- Una `{` literal en texto se interpreta como interpolación; falta definir una sintaxis de escape.
- Las expresiones se delimitan, no se validan como Rust; eso corresponde a M02/M03 con `syn`/rustc.
- No hay control de flujo (`if`, `for`), componentes, props, slots ni CSS.
- Un solo bloque `script` y uno `style`.
- La detección de chars se apoya en una ventana corta; es suficiente para expresiones normales, no es un lexer Rust completo.

## Consumo previsto por M02

1. Recorrer `Template.nodes` preservando el orden y los spans.
2. Convertir `Element`/`Text`/`Comment` a IR estática y `Interpolation`/`Expression` a nodos dinámicos o a un error tipado si aún no se soportan.
3. No volver a parsear HTML ni reinterpretar strings; el AST es la única fuente.
4. Propagar `SourceSpan` a los diagnósticos de lowering.

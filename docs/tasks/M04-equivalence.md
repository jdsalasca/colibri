# M04 — Suite de paridad Rust / `.colibri`

**Prioridad:** P0. **Depende de:** M02 + M03. **Área:** integration tests, ejemplos. **Specs:** [`docs/spec/components.md`](../spec/components.md) y [`docs/testing.md`](../testing.md). **Estado:** implementado para el subconjunto estático; CI pendiente de push.

## Objetivo
Probar que las dos sintaxis no producen dos motores: sus componentes equivalentes comparten semántica observable.

## Primer conjunto de fixtures
- Texto estático, atributos y Unicode.
- Composición de componentes y props primitivas si ya existe contrato.
- Escape de texto y atributos con datos hostiles.
- Diagnósticos equivalentes para inputs inválidos.
- Subconjunto reactivo (se añadirán fixtures después de M05/M07).

## Método
- Comparar árboles IR normalizados, ignorando sólo metadatos de span/formato.
- Comparar DOM/HTML *semánticamente*, no por comparación ingenua de espacios donde HTML no los preserve.
- Prohibir snapshots que normalicen diferencias importantes de identidad, attrs, estado o eventos.
- Documentar todo caso que todavía no puede probarse y crear deuda concreta.

## Aceptación
- [x] Rust y SFC producen misma representación estática para fixtures admitidos.
- [x] Tests corren en CI y son no-intermitentes (verificados en local; falta ejecución en GitHub).
- [x] Errores muestran ruta fuente correcta.
- [x] Ningún runtime específico de SFC o Rust fue introducido.
- [x] Existe plan de ampliación a evento/cleanup cuando M07 esté disponible.

## Handoff
Matriz de cobertura con fixture, versión, comportamiento, excepciones y próximo test fallido útil.

| Fixture | SFC | Rust | Resultado |
|---|---|---|---|
| Texto, atributos, Unicode | `crates/colibri-macros/tests/parity.rs` | idem | IR normalizado y HTML idénticos |
| Anidamiento, void, booleanos | idem | idem | idénticos |
| Atributo hostil `<script>` y `&` | idem | idem | idénticos y escapados |

Se compara una forma normalizada del IR (tag, atributos, texto y orden) ignorando spans, más el HTML de `colibri-ssr`. Todos los tests son deterministas y no necesitan navegador.

No cubierto aún: entidades HTML de entrada (`&lt;`) porque el parser no las decodifica; texto con `<` literal en SFC (requiere entidades); componentes/props; contenido dinámico y eventos. El plan para M07 es añadir fixtures de click, estado y teardown comparando comportamiento observable y limpieza, no sólo HTML.

Errores: SFC reporta `archivo:línea:columna`; la macro usa spans de token de rustc. Los errores de `render_static` de la ruta Rust llevan `file!()` pero offsets sintéticos `0..0`, porque los proc macros estables no exponen offsets de byte.

# M04 — Suite de paridad Rust / `.colibri`

**Prioridad:** P0. **Depende de:** M02 + M03. **Área:** integration tests, ejemplos. **Specs:** `docs/spec/components.md` y `docs/testing.md`.

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
- [ ] Rust y SFC producen misma representación estática para fixtures admitidos.
- [ ] Tests corren en CI y son no-intermitentes.
- [ ] Errores muestran ruta fuente correcta.
- [ ] Ningún runtime específico de SFC o Rust fue introducido.
- [ ] Existe plan de ampliación a evento/cleanup cuando M07 esté disponible.

## Handoff
Matriz de cobertura con fixture, versión, comportamiento, excepciones y próximo test fallido útil.

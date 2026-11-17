# M06 — SSR/SSG seguro

**Prioridad:** P1. **Depende de:** M02 para vistas estáticas; M05 para bindings dinámicos. **Área:** `colibri-ssr`. **Specs:** `docs/spec/rendering.md`, `docs/security.md`.

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
- [ ] Pruebas XSS y HTML semántico pasan.
- [ ] Página completamente estática descarga cero JS/WASM propio.
- [ ] SSR ante expresión no soportada falla claramente, nunca descarta binding.
- [ ] Rust/SFC producen mismo resultado conforme M04.
- [ ] Código de renderer no importa Vite o APIs browser.

## Handoff
Tabla de elementos/atributos soportados, límites, test hostiles y decisión sobre extensibilidad.

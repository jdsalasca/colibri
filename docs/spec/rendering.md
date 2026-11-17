# Renderizado, DOM, SSR y activación cliente

**Estado:** MVP estático implementado y primera rebanada de SSR dinámico con resolver explícito (M06a, [ADR 0007](../adr/0007-dynamic-ssr-resolver.md)). Evaluación Rust real, streaming e hidratación siguen pendientes.

## Modos objetivo
| Modo | Transferencia cliente | Uso típico |
|---|---|---|
| Static/SSG | HTML/CSS sin runtime Colibrí | blog, docs, landing |
| SSR | HTML generado por Rust; JS/WASM sólo si es necesario | contenido dinámico server |
| Islands | HTML + runtime específico de cada región interactiva | dashboards ligeros |
| Client | bundle Rust/WASM para UI intensamente interactiva | editores y apps |

Ningún modo obliga a cargar WASM. El coste de instanciar módulos y el puente JS↔WASM deben compararse con una salida JavaScript pequeña cuando corresponda.

## Contrato de renderer
- Elementos y atributos semánticos HTML y ARIA se representan conforme al estándar.
- Texto interpolado y valores de atributo se escapan según contexto; nunca `innerHTML` de fuente no confiable.
- URLs se validan por esquema y contexto; bloquear `javascript:` y otras formas peligrosas tras normalización.
- Form inputs controlados deben conservar selección/caret cuando sea posible.
- Eventos se agregan/retiran con ownership del scope; delegación sólo donde preserve orden, captura y propagación.
- Focus, teclado, accesibilidad y navegación deben funcionar; enlaces/acciones con mejora progresiva.
- Dom updates se agrupan si no cambian el orden observable ni disparan efectos inesperados.
- IDs para hidratación deben ser deterministas y evitar colisiones; la divergencia SSR/cliente emite diagnóstico.

## Etapas
1. **MVP estático:** AST/IR → string HTML seguro, validando tags, escape y atributos permitidos.
2. **SSR dinámico:** evaluar expresiones válidas en Rust servidor y añadir soporte de streaming incremental mediante ADR.
3. **DOM mínimo:** crear/montar/desmontar nodos, textos y atributos desde bindings; observabilidad del número de mutaciones.
4. **Islas:** manifest de rutas/símbolos y chunking medido; activación por interacción o visibilidad según estrategia explícita.
5. **Resumabilidad:** línea experimental separada; no prometer serialización arbitraria de closures/objetos Rust.

## SSR dinámico con resolver (M06a)
~~~rust
let html = render_dynamic(&ir.root, |expression| match expression {
    "name" => Some("Ada".to_string()),
    _ => None,
})?;
~~~
- El resolver recibe el texto de la expresión del IR; no se evalúa Rust.
- Un valor ausente es `MissingValue`, nunca cadena vacía.
- Texto y atributos se escapan; URLs y atributos peligrosos se validan igual que en estático.
- `on:evento` sigue siendo error de SSR hasta M07.
- El CLI ofrece `--bind key=value` para probar el contrato.

## Seguridad mínima
- Separar `SafeHtml` revisado y literal de `String` ordinario, si en el futuro existe una API de HTML sin escape.
- Para atributos `href/src/action/formaction` aplicar reglas de URL. Rechazar handlers inline estilo `onclick` string.
- No imprimir valores dinámicos dentro de tags `script`, `style` ni URLs sin tratamiento especializado.
- `meta`/`link` sólo por [allowlist](../adr/0010-head-tags-allowlist.md): charset, viewport y stylesheet; nada de `http-equiv`, `preload` ni atributos dinámicos.
- Probar scripts inyectados, entidades, Unicode, URL codificadas, cierre malicioso, atributos duplicados y SSR mismatch.

## Criterios de aceptación
- Snapshot HTML semánticamente equivalente para componentes Rust y SFC.
- No JS/WASM generado para página 100 % estática; medir transferencia, no sólo import graph.
- Tests DOM: mutación granular sin destruir nodos no afectados y cleanup de listeners.
- Tests SSR/client: mismo estado inicial, markup consistente, errored hydration reproducible.
- Cualquier nuevo tag/atributo seguro entra acompañado de tests XSS/ARIA aplicables.

# Contrato común de componentes — Rust y SFC

**Estado:** propuesta para diseño/implementación; macros todavía inexistentes.

## Objetivo central
Un componente con la misma interfaz tiene idéntico contrato de props, children, eventos, estado, lifecycle e identidad si se define en `Counter.rs` o en `Counter.colibri`. Los backends pueden emitir artefactos distintos, pero no cambiar el comportamiento observable.

## Modelo conceptual
~~~rust
// Pseudocódigo de API objetivo; aún no compila.
#[component]
fn Counter(initial: i64) -> impl View {
    let count = signal(initial);
    view! {
        <button type="button" on:click=move |_| count.update(|n| *n += 1)>
            "Count: " {count}
        </button>
    }
}
~~~
El SFC equivalente declara props tipadas y usa `<template>`. La convención exacta de declaración de props en SFC se decidirá en un ADR; no introducir dos mecanismos incompatibles.

## Semántica necesaria
| Área | Regla |
|---|---|
| Props | Tipadas mediante Rust, sin coerción silenciosa; errores de compilación legibles |
| Children/slots | Orden y ownership explícitos; definir API mínima antes de slots nombrados |
| Identidad | Un componente montado tiene una identidad estable asociada a su parent y key |
| Estado | Se crea una vez al montar la instancia; no se reinicia por update de un binding |
| Efectos | Se activan bajo condiciones documentadas; cleanup al desmontar |
| Eventos | Tipados si es posible; listeners vinculados al scope; evitar globals |
| Listas | Key estable cuando hay reordenamiento y estado local; avisar/rechazar key duplicada |
| Atributos | Normalizar camelCase/hyphen según HTML y definir props de componentes explícitamente |
| Fallos | No panicar por entradas externas; distinguir compile error, render error y runtime failure |
| Asíncrono | Cancelación o invalidación al desmontar y ante respuestas tardías |

## Prioridad de implementación
1. Renderizar texto + elementos + componentes anidados estáticos.
2. Props tipadas primitivas y children.
3. Binding reactivo de texto/atributos.
4. Eventos y scopes.
5. Listas keyed y condicionales.
6. Slots, contexto, composición avanzada y async resources.

## Ejemplo de equivalencia
Al montar, ambos componentes producen el mismo HTML semántico. Al invocar un click, ambos actualizan exactamente el mismo texto; al desmontar dejan cero listeners y subscriptions residuales. La prueba compara **resultado observable y desmontaje**, no sólo formato idéntico de Rust generado.

## Fallos a evitar
- ABI privada distinta por sintaxis, que obligue a incluir dos runtimes.
- Capturas de closures no válidas en Rust; el generador debe producir Rust real compilable.
- Renderizar automáticamente cada componente entero en actualizaciones pequeñas.
- Reconstruir el DOM cuando el binding puede actualizar un text node.
- Aceptar atributos desconocidos como instrucciones ejecutables del cliente.

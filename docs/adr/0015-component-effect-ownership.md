# ADR 0015 - Effects con ownership de componente

Fecha: 2026-11-17
Estado: **Aceptada** para el runtime DOM. Cierra el hallazgo #3 de Terra.

## Contexto
`use_effect` devolvía un `Effect` que se desmontaba al soltar el handle. Si un componente lo ignoraba (`use_effect(..);` sin binding), el effect se destruía al instante o quedaba vivo para siempre: no había forma de que el componente fuera dueño de sus efectos. Terra lo marcó como riesgo alto.

## Decisión

1. `with_effect_scope(f)` apila un scope TLS, ejecuta `f` y devuelve `(resultado, Vec<Effect>)`.
2. `use_effect` y `use_effect_with_cleanup` registran una **copia** del efecto en el scope más interno, si existe.
3. `Effect` es `Clone`: comparte observer y cleanup. `Drop` solo desmonta cuando cae el **último** handle (`Rc::strong_count == 1`).
4. `#[component]`:
   - envuelve el cuerpo en `with_effect_scope`;
   - si el tipo de retorno termina sintácticamente en `View`, devuelve `colibri_dom::attach_effects(view, effects)`;
   - si el retorno es `Node` u otro, descarta los efectos (SSR no tiene ciclo de vida).
5. `View::Scoped { view, effects }` entrega los efectos a `MountState` al montar; se limpian al soltar el `MountHandle`, igual que listeners.
6. `without_effect_scope(f)` suspende el scope ambiental. `flush` y `mount` lo usan: los effects internos del renderer pertenecen al montaje, y un callback no hereda el scope del componente que casualmente se esté construyendo.

## Alternativas
- Handle obligatorio: seguro, pero ruidoso y fácil de olvidar.
- Contexto de componente completo (árbol + hooks indexados): correcto a largo plazo, demasiado para esta fase.
- Detección por trait `IntoView`: más robusta que el nombre sintáctico, requiere cambiar la firma de todos los componentes.

## Consecuencias
- Un componente puede llamar `use_effect(..)` sin conservar el handle; el montaje lo mantiene vivo y lo limpia al desmontar.
- Efectos creados fuera de un scope conservan el comportamiento anterior (viven mientras exista el handle).
- Efecto observado en el demo: `TitleEffect` actualiza `document.title` (`Contador 0 → Contador 1`) sin guardar el `Effect` en ningún sitio.
- **Límite**: la detección es sintáctica (`-> View`). Un alias (`type MyView = View`) o un wrapper no recibe scope automático; en ese caso hay que conservar el handle o anotar el retorno como `View`.
- **Riesgo aceptado**: si un effect suelta su propio último handle durante su ejecución, el cleanup de esa pasada puede perderse (la cola del motor guarda `Weak`; solo ocurre en autodestrucción).
- **No soportado**: llamar `use_async`/`use_effect` dentro del callback de otro effect. Los hooks van en el cuerpo del componente.

## Validación
- `cargo test --workspace`: 91 pruebas, incluidas cleanup con panic, scope suspendido y effects fuera del scope ambiental.
- E2E Playwright/Chromium: título `Contador 0` al cargar y `Contador 1` tras incrementar; resto de la suite sigue verde.

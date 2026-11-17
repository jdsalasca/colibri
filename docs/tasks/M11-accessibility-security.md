# M11 — Gates transversales de accesibilidad y seguridad

**Prioridad:** continua (P0). **Depende de:** cada funcionalidad a validar. **Áreas:** parser, SSR, DOM, assets, tooling.

## Objetivo
Impedir que mejoras de rendimiento degraden corrección, seguridad o uso por teclado/lectores de pantalla.

## Seguridad
- [ ] escape contextual de texto/atributos; URLs seguras; no handlers `on*=` inline como strings.
- [ ] tests XSS para Unicode, entidades, URL obfuscadas y HTML mal formado.
- [ ] ningún modo de compilación introduce `eval`, `innerHTML` con datos hostiles o serialización de estado insegura.
- [ ] parser/codegen no panican o desbordan memoria frente a entradas hostiles dentro de límites declarados.
- [ ] dependencias y licencias revisadas.

## Accesibilidad
- [ ] HTML semántico real y roles correctos.
- [ ] tab order, focus, links y button keyboard activation funcionan.
- [ ] inputs conservan estado/caret cuando corresponda.
- [ ] `prefers-reduced-motion`, CSS fallback y contenido static-first son respetados.
- [ ] se testean controles con herramientas automáticas y al menos escenarios manuales con teclado.

## Aceptación
Cada PR enlaza pruebas de esta matriz aplicables a sus cambios. Vulnerabilidades conocidas o regresiones de teclado bloquean merge. No afirmar auditoría de seguridad o conformidad WCAG completa sin evaluación independiente.

## Handoff
Registro de vectores considerados, cobertura, fallos/falsos positivos y mitigaciones.

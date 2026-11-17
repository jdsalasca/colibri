# ADR 0010 - Allowlist de `meta` y `link` en SSR

Fecha: 2026-11-16
Estado: **Aceptada** para M06.

## Contexto
El SSR rechazaba todos los `meta` y `link`. Una app real necesita `charset`, viewport y hoja de estilos; el dashboard de validación tuvo que inyectarlos después del render, lo que rompe la garantía de que el HTML sale seguro del renderer.

## Decisión
Permitir un subconjunto estático y validado:

- `meta charset="…"` sin `http-equiv`.
- `meta name="viewport" content="…"` con ambos atributos estáticos.
- `link rel="stylesheet" href="…"` con `href` validado por la allowlist de URLs.

Todo lo demás se rechaza con `UnsafeAttribute`:

- `meta http-equiv` (incluido `refresh`).
- `link` con `rel` distinto de `stylesheet` (`preload`, `icon`, `canonical`, …).
- Atributos dinámicos en `meta`/`link`; deben ser literales de confianza.
- `on*`, `srcdoc`, `style` siguen bloqueados por la validación general.

## Alternativas
- Mantener el rechazo e inyectar head tags después del render: funciona, pero el HTML final ya no lo produce el renderer y se pierde la auditoría.
- Permitir todos los `meta`/`link`: `http-equiv=refresh` y `link rel=preload` abren vectores y comportamientos no deseados.
- Añadir CSP completa por `meta`: posponer; requiere otra allowlist y pruebas.

## Consecuencias
- Las apps pueden declarar charset, viewport y CSS sin parches.
- Los head tags son estáticos; los valores dinámicos se rechazan.
- La allowlist es deliberadamente pequeña; ampliarla exige tests XSS y actualizar este ADR.
- `render_static` y `render_dynamic` comparten la validación.

## Validación
- `renders_safe_meta_and_stylesheet_link`.
- `rejects_unsafe_meta_and_link` (refresh, preload, `javascript:` en href).
- Suite SSR: 12 pruebas; workspace: 82.

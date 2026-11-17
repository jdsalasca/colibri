# Modelo de amenazas y requisitos de seguridad

**Prioridad 0:** seguridad de renderizado, compilación y extensiones. Esto es guía de diseño, no auditoría de la implementación.

## Fronteras de confianza
- **Contenido no confiable**: texto/atributos/URL recibidos de usuarios, APIs y archivos de datos.
- **Código fuente de la app**: Rust/JS/Sass escritos por el desarrollador; se ejecutan con los permisos del build y **no son sandbox**.
- **Compilador**: procesa templates/expresiones y genera HTML, Rust o JS de puente.
- **Navegador**: aplica CSP/sandbox/Web APIs, pero no arregla HTML inyectado por el framework.
- **Dependencias**: crates, npm, assets y fuentes upstream pueden introducir supply-chain risk.

## Obligaciones
1. Texto interpolado va como text node o escapado contextual; nunca convertir `String` en `SafeHtml` implícitamente.
2. Atributos HTML escapan comillas/entidades; booleanos y URL tienen reglas propias. `on*=` como string y `srcdoc` sin sanitización están prohibidos.
3. Validar URLs según ubicación (href/src/action/formaction, namespaces SVG); normalizar entidades/control chars y evitar bypass de `javascript:`, `data:` peligrosos o esquemas desconocidos.
4. Evitar `eval`, creación de funciones mediante strings e inyección HTML. No insertar Rust o JS basado en inputs no confiables.
5. SSR usa escape contextual; streaming debe preservar límites de tags/atributos y prevenir HTML parcial inseguro.
6. Seguridad del parser: tamaño/profundidad limitados, overflow/panic-free con entradas hostiles, Unicode válido y reportes sin volcado accidental de secretos.
7. Reconstrucción/hidratación: datos serializados no deben romper `<script>` o permitir prototype pollution en adaptador JS.
8. DOM lifecycle: remover listeners y abortar tareas para prevenir retención de datos/filtraciones.
9. CSP/Trusted Types cuando se integre browser; preferir APIs DOM tipadas y política de seguridad restrictiva por defecto.
10. Dependency hygiene: `cargo audit` o equivalente, lockfile para apps, provenance/review de versiones y revisión de licencias upstream; nunca vendorizar React/Solid/etc como dependencia de runtime.

## Casos de prueba adversariales
- Texto `<img src=x onerror=...>` debe mostrarse como texto.
- URL con espacios/entidades/control chars más `javascript:` debe rechazarse.
- Comillas y `&` en atributos; tags mal cerrados; nombres desconocidos de evento.
- Template recursivo, profundidad extrema y tamaños enormes no deben colgar el proceso.
- SSR y cliente deben tratar igual el mismo valor y markup.
- Enlaces externos, foco y navegación accesible deben seguir funcionando.

## Proceso de incidentes
No publicar vulnerabilidades sin confirmar. Para reporte responsable, documentar reproducción mínima, versiones afectadas, evaluación de impacto, mitigación y test de regresión. Política pública de disclosure se creará antes de release estable.

## Límites explícitos
El prototipo `colibri-ssr` rechaza muchas construcciones peligrosas, pero **no está auditado**, no soporta todo HTML/URL y no garantiza protección frente a todos los vectores. Nunca etiquetarlo como "XSS-proof".

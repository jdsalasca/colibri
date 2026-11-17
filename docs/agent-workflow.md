# Protocolo multiagente — cómo implementar Colibrí

Este documento rige el trabajo de agentes autónomos/colaboradores para reducir conflictos y decisiones incompatibles. `AGENTS.md` es el resumen obligatorio; aquí está el procedimiento.

## Antes de abrir una tarea
1. Leer `docs/START_HERE.md` y `docs/STATUS.md` y confirmar qué código hay en la **rama actual**, no sólo en el README.
2. Elegir un work item de `docs/tasks/README.md` con dependencias finalizadas.
3. Formular una intención de cambio: *ID, objetivo, dueño, dependencias, rutas bajo edición, criterio de aceptación y límites explícitos*. Compartirla en issue/PR si hay agentes paralelos.
4. Reservar ownership de archivos de escritura. Varios agentes pueden investigar a la vez, pero un archivo compartido tiene un único dueño por bloque de trabajo.

## Convenciones de ramas y PR
- Rama desde base actual: `feat/M01-template-ast`, `fix/M05-scope-cleanup`, `docs/M00-...`.
- Un PR = una responsabilidad verificable. No introducir arquitectura adyacente fuera del ticket.
- Rebase/merge base con cuidado; nunca force-push sobre ramas ajenas ni borrar trabajos de otro agente.
- No fusionar si CI falla; anotar flakiness con evidencia y no ocultarlo.
- Revisar las decisiones normativas antes de alterar IR, APIs o crate graph. Cambios incompatibles requieren ADR.

## Plantilla de handoff
~~~text
ID y resultado:
SHA/PR:
Archivos tocados:
Contrato / ADR:
Qué funciona (pruebas exactas):
Qué NO funciona:
Riesgos y bordes:
Dependencias / próximos pasos:
Estado real en STATUS.md:
~~~

## Comunicación entre agentes
- El dueño de M01 define template AST/spans; M02 consume un contrato versionado. Evitar que M02 invente un parser propio.
- M03 usa el mismo modelo de IR que M02. Cambiar IR requiere coordinación con M04 y SSR.
- M05 define semántica de scopes; M07 DOM no crea un scheduler paralelo.
- M08 delega parser/codegen al CLI y no duplica análisis Rust dentro del plugin.
- M10 define métodos de benchmark; ningún agente declara más velocidad por intuición.

## Etiquetas de estado
- `not-started`: sin implementación;
- `in-progress`: PR abierto;
- `blocked`: dependencia, API o decisión pendiente;
- `implemented-unverified`: código escrito sin pruebas en CI;
- `verified`: pruebas reproducibles y aceptación cumplida.
No marcar progreso sin registro de evidencias. La lista de tareas no sustituye a los tests.

## Condiciones de parada
Detener y proponer ADR en vez de codificar si surge un cambio no previsto en: identidad de componentes, serialización de closures, seguridad HTML, firma pública de signals, esquema IR, ciclo de vida o interoperabilidad Rust/SFC.

## Revisión obligatoria
- ¿Semántica Rust y SFC sigue alineada?
- ¿Hay fuga potencial de subscriptions/listeners?
- ¿Se preserva semántica HTML y accesibilidad?
- ¿El resultado de compilación está desacoplado de tooling de desarrollo?
- ¿La mejora de rendimiento tiene benchmark controlado?
- ¿La documentación diferencia planeado/implementado/verificado?

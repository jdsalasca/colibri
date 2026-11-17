# ADR 0003 — Separar IR de compilación y runtime

Fecha: 2026-11-16
Estado: **Propuesta**, pendiente de revisión.

## Contexto
Los componentes SFC y Rust requieren un modelo común para analizar dependencias y producir HTML / Rust / WASM. Enviar esa misma estructura completa al navegador puede convertirse en un Virtual DOM obligatorio y perjudicar memoria.

## Decisión propuesta
La IR semántica vive en compilación; el generador produce instrucciones especializadas por destino. El cliente utiliza estructuras compactas de referencias/estado necesarias para interacción, no necesariamente el árbol original.

## Alternativas
- IR completa en browser: simplifica inspección, potencial coste de memoria/CPU.
- Especialización total: mejor oportunidad de reducir trabajo, complejidad codegen mayor.
- Camino híbrido: especialización cuando se conoce estructura, fallback explícito para casos dinámicos.

## Consecuencias
Necesitamos tests de equivalencia por backend, soporte a debug info/spans y medición de tamaños. No prohibimos un VDOM opcional en casos futuros; sólo evitamos imponerlo universalmente.

## Validación
M02–M04 generan casos de prueba; M07 demuestra bindings y teardown; M10 mide memoria y latencia. No adoptar como realidad implementada antes de esas pruebas.

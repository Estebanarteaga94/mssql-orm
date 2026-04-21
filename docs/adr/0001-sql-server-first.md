# ADR 0001: SQL Server como único objetivo inicial

## Estado

Aprobado

## Contexto

El ORM nace para SQL Server y depende de decisiones concretas de dialecto: quoting con corchetes, parámetros `@P1..@Pn`, `IDENTITY`, `ROWVERSION`, `OUTPUT INSERTED.*` y tipos propios del motor.

Intentar soportar múltiples bases de datos desde el inicio introduciría abstracciones prematuras y debilitaría el diseño del compilador SQL y del sistema de migraciones.

## Decisión

La primera versión del ORM soporta únicamente SQL Server.

## Consecuencias

- El compilador SQL puede modelar directamente el dialecto de SQL Server.
- Las migraciones pueden priorizar operaciones y tipos reales del motor objetivo.
- No se introducirán capas genéricas multi-database en Etapa 0 ni Etapa 1.
- Si en el futuro se evalúa soporte adicional, deberá justificarse con nuevos ADRs y sin degradar el diseño existente.

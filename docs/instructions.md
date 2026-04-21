# Instrucciones Operativas del Agente

## Propósito

Este repositorio debe ser trabajado por un agente de desarrollo de forma autónoma, ordenada y trazable. Estas instrucciones son la fuente operativa principal para cualquier sesión futura y deben seguirse incluso si no existe contexto adicional en la conversación.

## Principios de Operación

1. Trabajar primero con comprensión del estado actual del repositorio.
2. Mantener trazabilidad de decisiones, cambios y pendientes en `docs/`.
3. Priorizar cambios pequeños, verificables y coherentes con la arquitectura definida.
4. No asumir funcionalidades no documentadas como ya decididas.
5. Antes de modificar código, revisar siempre `docs/tasks.md`, `docs/worklog.md` y `docs/context.md` si existe.
6. Toda sesión debe dejar el repositorio en un estado más claro que al inicio.

## Flujo de Trabajo Obligatorio

Seguir este flujo en cada sesión:

1. Análisis
   - Leer `docs/tasks.md`.
   - Leer `docs/worklog.md`.
   - Leer `docs/context.md` si existe.
   - Revisar el estado actual del código y los archivos relacionados con la tarea.
   - Identificar restricciones, dependencias y riesgos.
2. Plan
   - Definir el objetivo concreto de la sesión.
   - Seleccionar una sola tarea o un bloque pequeño de trabajo relacionado.
   - Establecer criterios de validación antes de editar.
3. Ejecución
   - Implementar cambios mínimos y consistentes.
   - Respetar límites arquitectónicos y evitar mezclar responsabilidades.
   - No introducir trabajo no solicitado fuera del alcance activo.
4. Validación
   - Ejecutar validaciones locales razonables para el cambio realizado.
   - Verificar compilación, pruebas, lint o consistencia documental según aplique.
   - Si algo no puede validarse, dejarlo registrado explícitamente.
5. Documentación
   - Actualizar `docs/tasks.md`.
   - Registrar en `docs/worklog.md` lo hecho, la fecha y el resultado.
   - Actualizar `docs/context.md` si cambió el entendimiento del sistema, la arquitectura o las decisiones vigentes.
6. Commit
   - Al terminar una tarea completada y validada, realizar un commit con el trabajo realizado.
   - Si la tarea no puede validarse o queda parcial, dejarlo registrado antes de decidir si corresponde commit.

## Reglas de Modificación de Código

1. Leer antes de editar.
2. Hacer cambios alineados con el diseño existente o con el plan arquitectónico vigente.
3. No reescribir módulos completos si un cambio acotado resuelve el objetivo.
4. No introducir dependencias nuevas sin justificarlo en `docs/worklog.md`.
5. No mezclar refactor, nuevas funciones y cambios cosméticos en la misma sesión salvo que sea estrictamente necesario.
6. Preservar compatibilidad de API pública salvo instrucción explícita en contra.
7. Mantener nombres claros, tipado consistente y límites de módulo razonables.
8. Si un archivo contiene cambios previos no hechos por el agente, no revertirlos sin instrucción explícita.
9. Mantener un único repositorio Git en la raíz del proyecto; no crear repositorios anidados dentro de crates o submódulos de trabajo local salvo instrucción explícita.

## Reglas de Documentación

1. `docs/tasks.md` es la fuente única de verdad del trabajo pendiente.
2. `docs/worklog.md` debe registrar cada sesión de trabajo relevante.
3. `docs/context.md` debe resumir estado actual, arquitectura activa, decisiones vigentes y próximos focos.
4. La documentación debe ser concreta, accionable y mantenerse sincronizada con el código.
5. Si una decisión arquitectónica cambia, reflejarla el mismo día en la documentación.

## Restricciones

El agente no debe:

1. Implementar funcionalidades no priorizadas en `docs/tasks.md` salvo correcciones necesarias para completar la tarea activa.
2. Hacer cambios destructivos amplios sin necesidad técnica clara.
3. Borrar documentación operativa.
4. Inventar comportamiento del sistema sin respaldo en código o documentación.
5. Marcar tareas como completadas sin evidencia verificable.
6. Dejar trabajo parcialmente hecho sin registrarlo en `docs/worklog.md` y `docs/tasks.md`.

## Gestión de Tareas

Reglas para `docs/tasks.md`:

1. Mantener exactamente tres secciones: `Pendientes`, `En Progreso`, `Completadas`.
2. Mover una tarea a `En Progreso` antes de comenzar trabajo real sobre ella.
3. Mover una tarea a `Completadas` solo después de validar el resultado.
4. Si una tarea grande aparece, dividirla en subtareas concretas antes de ejecutarla.
5. Añadir nuevas tareas cuando se descubra trabajo necesario no registrado.
6. Redactar tareas como entregables verificables, no como intenciones vagas.

## Criterios de Calidad del Código

Todo cambio debe buscar:

1. Claridad por encima de ingenio innecesario.
2. Cohesión de módulos y separación de responsabilidades.
3. Seguridad en errores, tipos y flujos críticos.
4. Validación suficiente para el alcance del cambio.
5. Ausencia de deuda técnica evitable introducida por prisa.
6. Documentación actualizada cuando el comportamiento o la arquitectura cambien.

## Cierre Obligatorio de Sesión

Antes de terminar una sesión:

1. Confirmar qué tarea quedó completada, en progreso o pendiente.
2. Registrar el trabajo en `docs/worklog.md`.
3. Registrar bloqueos o riesgos si existen.
4. Dejar el siguiente paso recomendado en `docs/worklog.md` o `docs/context.md`.
5. Realizar un commit cuando la tarea haya quedado completada y validada.

# Colaboración con IA

## Objetivo

Esta carpeta define cómo debe colaborar un agente de IA con el repositorio sin perder trazabilidad, límites arquitectónicos ni continuidad entre sesiones.

## Fuente de verdad

Antes de trabajar, el agente debe tratar estos archivos como obligatorios:

- `docs/instructions.md`
- `docs/tasks.md`
- `docs/worklog.md`
- `docs/context.md`
- `docs/plan_orm_sqlserver_tiberius_code_first.md`

## Resultado esperado por sesión

Cada sesión debe dejar:

- una tarea concreta tomada desde `docs/tasks.md`
- implementación real o documentación operativa real
- validación ejecutada o limitación registrada
- actualización de `docs/tasks.md`
- registro en `docs/worklog.md`
- actualización de `docs/context.md` si cambió el estado operativo

## Límites de actuación

- No improvisar arquitectura fuera del plan vigente.
- No adelantar features de etapas posteriores salvo soporte mínimo para completar la tarea activa.
- No mezclar generación SQL en `mssql-orm-query`.
- No mover ejecución fuera de `mssql-orm-tiberius`.
- No introducir dependencias nuevas sin dejar constancia en `docs/worklog.md`.
- No marcar tareas como completadas sin validación razonable.

## Política de continuidad

- Si el plan maestro no está en la raíz, registrar brevemente la ruta real usada.
- Si aparece trabajo demasiado grande, dividirlo en `docs/tasks.md` antes de implementarlo.
- Si se detectan cambios ajenos, no revertirlos; trabajar alrededor de ellos o registrarlos como riesgo si bloquean.
- Si no se puede validar algo, dejar la razón y el impacto documentados.
- Si una tarea queda parcial, mantenerla en `En Progreso` con redacción precisa.

## Validación mínima sugerida

Aplicar solo lo que corresponda al cambio real:

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

Para cambios exclusivamente documentales, al menos verificar consistencia del repositorio y registrar por qué no se requirieron validaciones más profundas si aplica.

## Cierre

Al completar una tarea validada:

- moverla a `Completadas`
- registrar la sesión en `docs/worklog.md`
- actualizar `docs/context.md` si cambió el foco o el entendimiento del proyecto
- realizar commit limpio sin arrastrar archivos no relacionados

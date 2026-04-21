# Plantilla de sesión para agentes

## 1. Lectura inicial

Leer en este orden:

1. `docs/instructions.md`
2. `docs/tasks.md`
3. `docs/worklog.md`
4. `docs/context.md`
5. `docs/plan_orm_sqlserver_tiberius_code_first.md`

## 2. Selección de tarea

- Tomar la tarea prioritaria más inmediata.
- Moverla a `En Progreso` antes de editar.
- Si es demasiado grande, dividirla en subtareas verificables.

## 3. Revisión del repositorio

- Revisar estructura real del workspace.
- Confirmar rutas existentes antes de referenciarlas.
- Verificar si hay cambios locales no relacionados.

## 4. Ejecución

- Implementar solo la tarea activa y su soporte mínimo.
- Respetar límites por crate:
  - `core`: contratos, metadata, tipos, errores
  - `macros`: derives y parsing de atributos
  - `query`: AST y query builder sin SQL
  - `sqlserver`: compilación a SQL Server
  - `tiberius`: ejecución y adaptación al driver
  - `migrate`: migraciones code-first
  - `mssql-orm`: API pública consolidada

## 5. Validación

- Ejecutar las validaciones que realmente apliquen.
- Si una validación no puede correr, documentarlo.

## 6. Cierre documental

- Actualizar `docs/tasks.md`
- Actualizar `docs/worklog.md`
- Actualizar `docs/context.md` si corresponde

## 7. Commit

- Hacer commit solo si la tarea quedó terminada y validada.
- Excluir archivos no relacionados detectados en el árbol de trabajo.

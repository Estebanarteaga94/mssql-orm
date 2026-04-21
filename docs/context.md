# Contexto del Proyecto

## Estado Actual

El repositorio contiene como base principal el documento `docs/plan_orm_sqlserver_tiberius_code_first.md`, que describe la visión y roadmap para construir un ORM code-first en Rust para SQL Server usando Tiberius.

El backlog operativo de `docs/tasks.md` ya fue alineado con ese plan maestro y ahora representa la secuencia de trabajo recomendada por etapas.

Ya existe un workspace Rust inicial con crates separadas para `mssql-orm`, `core`, `macros`, `query`, `sqlserver`, `tiberius`, `migrate` y `cli`.
El control de versiones quedó consolidado en un único repositorio Git en la raíz; no deben existir repositorios anidados dentro de `crates/`.

## Objetivo Técnico Actual

Consolidar la Etapa 0 sobre la base ya creada: automatización de validaciones, documentación arquitectónica mínima y preparación del terreno para implementar metadata en Etapa 1.

## Dirección Arquitectónica Vigente

- El proyecto apunta a un workspace Rust con múltiples crates.
- La arquitectura propuesta separa `core`, `macros`, `query`, `sqlserver`, `tiberius`, `migrate` y `cli`.
- SQL Server es el objetivo inicial único.
- Tiberius debe quedar encapsulado como adaptador de infraestructura, no como núcleo del ORM.
- El MVP debe enfocarse en metadata, macros de entidad, CRUD básico, query builder simple, `DbContext`, `DbSet` y migraciones básicas.
- La crate pública `mssql-orm` centraliza la API expuesta y reexporta internals seleccionados.
- `mssql-orm-macros` quedó creada como crate `proc-macro`, pero sus derives siguen siendo placeholders sin generación real.
- La operación del proyecto ahora exige realizar commit al cerrar una tarea completada y validada.

## Fuente de Verdad

- Plan maestro: `docs/plan_orm_sqlserver_tiberius_code_first.md`
- Operación del agente: `docs/instructions.md`
- Trabajo pendiente: `docs/tasks.md`
- Historial de sesiones: `docs/worklog.md`

## Riesgos Inmediatos

- La base del workspace todavía es solo estructural; no existe metadata real, AST útil ni integración con SQL Server/Tiberius.
- Los derives actuales en `mssql-orm-macros` son stubs y no deben considerarse funcionalidad implementada.
- Si futuras sesiones empiezan a programar sin revisar `docs/`, se pierde trazabilidad.
- Como el repositorio raíz es nuevo, cualquier archivo ajeno al trabajo técnico debe revisarse antes de incluirlo en commits iniciales.

## Próximo Enfoque Recomendado

1. Configurar CI base con `cargo check`, `cargo test`, `cargo fmt --check` y `cargo clippy`.
2. Crear README principal y documentación arquitectónica mínima por crate para fijar límites antes de Etapa 1.
3. Empezar `Entity` trait y metadata base en `mssql-orm-core` solo después de cerrar la Etapa 0.

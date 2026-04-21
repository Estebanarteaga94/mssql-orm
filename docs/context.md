# Contexto del Proyecto

## Estado Actual

El repositorio contiene como base principal el documento `docs/plan_orm_sqlserver_tiberius_code_first.md`, que describe la visión y roadmap para construir un ORM code-first en Rust para SQL Server usando Tiberius.

El backlog operativo de `docs/tasks.md` ya fue alineado con ese plan maestro y ahora representa la secuencia de trabajo recomendada por etapas.

Ya existe un workspace Rust inicial con crates separadas para `mssql-orm`, `core`, `macros`, `query`, `sqlserver`, `tiberius`, `migrate` y `cli`.
El control de versiones quedó consolidado en un único repositorio Git en la raíz; no deben existir repositorios anidados dentro de `crates/`.
También existe CI base en GitHub Actions para validar formato, compilación, pruebas y lint del workspace.
Ya existe documentación pública mínima en `README.md`, documentación arquitectónica en `docs/architecture/overview.md` y ADRs iniciales en `docs/adr/`.
Ya existe `docs/ai/` con guía de colaboración, plantilla de sesión y checklist de handoff para futuras sesiones autónomas.
`mssql-orm-core` ya contiene el contrato `Entity` y la metadata base de entidades, columnas, índices y foreign keys.

## Objetivo Técnico Actual

Continuar la Etapa 1 implementando `#[derive(Entity)]` en `mssql-orm-macros` sobre la metadata base ya definida en `mssql-orm-core`.

## Dirección Arquitectónica Vigente

- El proyecto apunta a un workspace Rust con múltiples crates.
- La arquitectura propuesta separa `core`, `macros`, `query`, `sqlserver`, `tiberius`, `migrate` y `cli`.
- SQL Server es el objetivo inicial único.
- Tiberius debe quedar encapsulado como adaptador de infraestructura, no como núcleo del ORM.
- El MVP debe enfocarse en metadata, macros de entidad, CRUD básico, query builder simple, `DbContext`, `DbSet` y migraciones básicas.
- La crate pública `mssql-orm` centraliza la API expuesta y reexporta internals seleccionados.
- `mssql-orm-core` ya define `Entity`, `EntityMetadata`, `ColumnMetadata`, `IndexMetadata`, `ForeignKeyMetadata`, `SqlServerType` y tipos auxiliares.
- `mssql-orm-macros` sigue siendo una crate `proc-macro` con derives placeholder sin generación real.
- La operación del proyecto ahora exige realizar commit al cerrar una tarea completada y validada.
- El workflow `.github/workflows/ci.yml` es la automatización mínima vigente y replica las validaciones locales base del workspace.
- La arquitectura ya quedó documentada y respaldada por ADRs para SQL Server primero, separación estricta por crates y API pública concentrada en `mssql-orm`.
- La colaboración autónoma ya quedó formalizada en `docs/ai/`, por lo que las siguientes sesiones deben apoyarse en esa guía además de `docs/instructions.md`.

## Fuente de Verdad

- Plan maestro: `docs/plan_orm_sqlserver_tiberius_code_first.md`
- Operación del agente: `docs/instructions.md`
- Colaboración con IA: `docs/ai/`
- Trabajo pendiente: `docs/tasks.md`
- Historial de sesiones: `docs/worklog.md`
- Arquitectura y decisiones: `README.md`, `docs/architecture/overview.md`, `docs/adr/`

## Riesgos Inmediatos

- Aunque ya existe metadata base en `core`, todavía no se genera automáticamente desde modelos Rust.
- Los derives actuales en `mssql-orm-macros` son stubs y no deben considerarse funcionalidad implementada.
- Aún no existe AST útil ni integración con SQL Server/Tiberius.
- Si futuras sesiones empiezan a programar sin revisar `docs/`, se pierde trazabilidad.
- Como el repositorio raíz es nuevo, cualquier archivo ajeno al trabajo técnico debe revisarse antes de incluirlo en commits iniciales.

## Próximo Enfoque Recomendado

1. Implementar `#[derive(Entity)]` en `mssql-orm-macros`.
2. Hacer que el derive genere `EntityMetadata` estática y respete los límites ya definidos en `core`.
3. Mantener `README`, arquitectura, ADRs y `docs/ai/` sincronizados si cambia el proceso operativo o algún límite entre crates.

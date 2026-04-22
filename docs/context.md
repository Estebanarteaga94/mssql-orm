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
La metadata base fue re-alineada contra el plan maestro para preservar el orden de PK compuesto y evitar helpers con semántica no definida por el plan.

## Objetivo Técnico Actual

Continuar la Etapa 2 fortaleciendo las pruebas de mapping de filas y extracción de valores persistibles, ahora que `#[derive(Insertable)]` y `#[derive(Changeset)]` ya están implementados sobre los contratos base de `core`.

## Dirección Arquitectónica Vigente

- El proyecto apunta a un workspace Rust con múltiples crates.
- La arquitectura propuesta separa `core`, `macros`, `query`, `sqlserver`, `tiberius`, `migrate` y `cli`.
- SQL Server es el objetivo inicial único.
- Tiberius debe quedar encapsulado como adaptador de infraestructura, no como núcleo del ORM.
- El MVP debe enfocarse en metadata, macros de entidad, CRUD básico, query builder simple, `DbContext`, `DbSet` y migraciones básicas.
- La crate pública `mssql-orm` centraliza la API expuesta y reexporta internals seleccionados.
- `mssql-orm-core` ya define `Entity`, `EntityMetadata`, `ColumnMetadata`, `IndexMetadata`, `ForeignKeyMetadata`, `SqlServerType` y tipos auxiliares.
- El plan maestro prevalece explícitamente sobre helpers o inferencias locales cuando se definan contratos, campos de metadata o responsabilidades entre crates.
- `mssql-orm-macros` ya implementa un `#[derive(Entity)]` funcional sobre structs con campos nombrados, generando `EntityMetadata` estática e implementación del trait `Entity`.
- El derive soporta al menos los atributos base ya priorizados en la Etapa 1: `table`, `schema`, `primary_key`, `identity`, `length`, `nullable`, `default_sql`, `index` y `unique`.
- El derive también cubre soporte directo para `column`, `sql_type`, `precision`, `scale`, `computed_sql` y `rowversion`, en línea con el shape de metadata ya definido en `core`.
- `mssql-orm-core` ya define `EntityColumn<E>` como símbolo estático de columna, y `#[derive(Entity)]` genera asociados como `Customer::email` para el query builder futuro.
- La crate pública `mssql-orm` ya contiene pruebas `trybuild` que cubren un caso válido de entidad y errores de compilación esperados para ausencia de PK, `identity` inválido y `rowversion` inválido.
- `mssql-orm-core` ya define `SqlValue`, `ColumnValue`, `Row`, `FromRow`, `Insertable<E>` y `Changeset<E>` como contratos base de mapping y persistencia.
- `mssql-orm-core` ya define `SqlTypeMapping` con implementaciones base para `bool`, `i32`, `i64`, `f64`, `String`, `Vec<u8>`, `Uuid`, `Decimal`, `NaiveDate`, `NaiveDateTime` y `Option<T>`, alineadas con las convenciones actuales del plan.
- `mssql-orm-macros` ya implementa `#[derive(Insertable)]` y `#[derive(Changeset)]` para structs con campos nombrados usando `#[orm(entity = MiEntidad)]`.
- `Insertable` soporta `#[orm(column = "...")]` por campo y produce `Vec<ColumnValue>` resolviendo el nombre final de columna contra la metadata de la entidad objetivo.
- `Changeset` exige `Option<T>` en el nivel externo de cada campo para mantener la semántica de omisión de cambios; esto permite también `Option<Option<T>>` para representar actualizaciones a `NULL`.
- La crate pública `mssql-orm` declara `extern crate self as mssql_orm` para que los macros puedan apuntar a una ruta estable tanto dentro del workspace como desde crates consumidoras.
- La `prelude` pública ya reexporta los derives `Entity`, `Insertable` y `Changeset`, por lo que los tests de integración usan la misma superficie que usará un consumidor real.
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

- La Etapa 2 ya tiene derives reales de `Insertable` y `Changeset`, pero todavía falta ampliar la cobertura de pruebas alrededor de `FromRow` y de la extracción de valores persistibles en escenarios más cercanos al uso real.
- Aún no existe AST útil ni integración con SQL Server/Tiberius.
- Si futuras sesiones empiezan a programar sin revisar `docs/`, se pierde trazabilidad.
- Como el repositorio raíz es nuevo, cualquier archivo ajeno al trabajo técnico debe revisarse antes de incluirlo en commits iniciales.

## Próximo Enfoque Recomendado

1. Ejecutar `Etapa 2: Crear pruebas de mapping de filas y extracción de valores persistibles`, ampliando cobertura sobre modelos derivados y casos nulos.
2. Mantener las convenciones de `EntityColumn`, `Entity::campo`, `Insertable` y `Changeset` estables mientras se prepara la futura integración con query builder y capa de ejecución.
3. Mantener `README`, arquitectura, ADRs y `docs/ai/` sincronizados si cambia el proceso operativo o algún límite entre crates.

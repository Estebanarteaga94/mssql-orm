# mssql-orm

ORM `code-first` en Rust para SQL Server con Tiberius como driver de bajo nivel.

## Estado

El repositorio está en fase fundacional. La Etapa 0 deja preparado el workspace, la trazabilidad operativa y la automatización base antes de entrar en metadata, proc macros y ejecución real contra SQL Server.

La fuente de verdad del roadmap técnico es [docs/plan_orm_sqlserver_tiberius_code_first.md](docs/plan_orm_sqlserver_tiberius_code_first.md).

## Objetivo

Construir una librería reusable inspirada en `Entity Framework Core` y `Eloquent`, pero idiomática en Rust:

- modelos `code-first` definidos en Rust
- metadata generada en compilación con `proc_macros`
- query builder tipado con AST separada de la generación SQL
- compilación específica para SQL Server
- ejecución aislada en un adaptador sobre Tiberius
- migraciones `code-first`
- API pública concentrada en una crate principal

## Arquitectura del Workspace

El workspace está dividido en crates con responsabilidades explícitas:

- `mssql-orm-core`: contratos, metadata, tipos compartidos y errores
- `mssql-orm-macros`: derives y generación de metadata en compilación
- `mssql-orm-query`: AST y primitivas del query builder, sin emitir SQL
- `mssql-orm-sqlserver`: compilación de AST a SQL Server
- `mssql-orm-tiberius`: conexión, ejecución, filas y transacciones sobre Tiberius
- `mssql-orm-migrate`: snapshots, diffs y operaciones de migración
- `mssql-orm-cli`: interfaz de línea de comandos para migraciones y soporte operativo
- `mssql-orm`: crate pública que reexporta la API soportada

Más detalle en [docs/architecture/overview.md](docs/architecture/overview.md).

## Restricciones Arquitectónicas

- `core` no depende de Tiberius
- `query` no genera SQL directo
- la generación SQL vive solo en `mssql-orm-sqlserver`
- la ejecución vive solo en `mssql-orm-tiberius`
- la API pública se concentra en `mssql-orm`
- SQL Server es el único motor objetivo en esta fase

## Estado Actual del Código

- El workspace compila y tiene CI base en GitHub Actions.
- Los crates exponen solo marcadores y límites de responsabilidad.
- Los derives de `mssql-orm-macros` siguen siendo placeholders.
- Aún no existe metadata real de entidades, AST funcional ni integración con SQL Server.

## Validación Base

El pipeline vigente ejecuta:

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

## Documentación Operativa

- Instrucciones del agente: [docs/instructions.md](docs/instructions.md)
- Backlog: [docs/tasks.md](docs/tasks.md)
- Historial de sesiones: [docs/worklog.md](docs/worklog.md)
- Contexto vigente: [docs/context.md](docs/context.md)
- ADRs: [docs/adr](docs/adr)

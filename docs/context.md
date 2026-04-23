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

Continuar la Etapa 5 implementando las operaciones CRUD restantes sobre `DbSet<T>`, ahora que `DbContext`, `DbSet<T>`, `#[derive(DbContext)]` y el runner base `DbSet::query()` ya existen en la superficie pública.

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
- La crate pública `mssql-orm` ya incluye una prueba de integración de Etapa 2 sobre un `Row` neutral, con cobertura de lectura tipada exitosa, ausencia de columna requerida, mismatch de tipo, `NULL` opcional y extracción de `ColumnValue` desde modelos `Insertable` y `Changeset`.
- `mssql-orm-query` ya dejó de ser un placeholder y ahora expone `Expr`, `Predicate`, `SelectQuery`, `CountQuery`, `InsertQuery`, `UpdateQuery`, `DeleteQuery`, `OrderBy`, `Pagination`, `TableRef`, `ColumnRef` y `CompiledQuery`.
- El AST de `mssql-orm-query` reutiliza `EntityColumn<E>` y metadata estática de `core` para construir referencias de tabla y columna sin generar SQL directo.
- `InsertQuery` y `UpdateQuery` ya se pueden construir desde `Insertable<E>` y `Changeset<E>`, conectando persistencia estructural con la futura compilación SQL Server.
- `mssql-orm-sqlserver` ya implementa quoting seguro de identificadores mediante `quote_identifier`, `quote_qualified_identifier`, `quote_table_ref` y `quote_column_ref`.
- El quoting actual usa corchetes SQL Server, escapa `]` como `]]` y rechaza identificadores vacíos, con caracteres de control o multipartes pasados como una sola cadena.
- `mssql-orm-sqlserver` ya implementa compilación de `select`, `insert`, `update`, `delete` y `count` a `CompiledQuery`, incluyendo placeholders `@P1..@Pn` y preservación de orden de parámetros.
- La compilación actual emite `OUTPUT INSERTED.*` para `insert` y `update`, usa `*` cuando `select` no tiene proyección explícita y exige `ORDER BY` antes de `OFFSET/FETCH`.
- `mssql-orm-sqlserver` ya cuenta con snapshots versionados para `select`, `insert`, `update`, `delete` y `count`, fijando el SQL generado y la secuencia observable de parámetros.
- La crate `mssql-orm-sqlserver` ahora usa `insta` solo como `dev-dependency` para congelar el contrato del compilador sin introducir dependencia runtime nueva.
- `mssql-orm-tiberius` ya integra la dependencia real `tiberius` y expone `MssqlConnectionConfig`, `MssqlConnection` y `TokioConnectionStream`.
- `MssqlConnectionConfig` ya parsea ADO connection strings mediante `tiberius::Config`, conserva el string original y rechaza entradas vacías o sin host usable.
- `MssqlConnection::connect` ya abre `TcpStream`, configura `TCP_NODELAY` e inicializa `tiberius::Client`, sin adelantar todavía ejecución de `CompiledQuery` ni mapeo de filas.
- `mssql-orm-tiberius` ya expone `ExecuteResult`, el trait `Executor` y los métodos `execute`/`query_raw` sobre `MssqlConnection<S>`.
- El adaptador ya prepara `CompiledQuery`, valida conteo de placeholders y realiza binding real de `SqlValue` hacia `tiberius::Query`.
- El binding de `Decimal` ya se resuelve a `tiberius::numeric::Numeric`; el caso `SqlValue::Null` sigue siendo provisional y hoy se envía como `Option::<String>::None`.
- `mssql-orm-tiberius` ya expone `MssqlRow<'a>` como wrapper sobre `tiberius::Row`, implementa el contrato neutral `Row` del core y convierte tipos soportados de SQL Server a `SqlValue`.
- El adaptador ya encapsula errores de Tiberius en `OrmError` mediante una capa interna de mapeo contextual, incluyendo lectura de filas y detección básica de deadlock.
- `MssqlConnection<S>` ya implementa `fetch_one<T: FromRow>` y `fetch_all<T: FromRow>` apoyándose en `query_raw`, `MssqlRow` y el contrato `FromRow` del core.
- `mssql-orm-tiberius` ya cuenta con pruebas de integración reales en `crates/mssql-orm-tiberius/tests/sqlserver_integration.rs`, activables mediante `MSSQL_ORM_TEST_CONNECTION_STRING`.
- Las pruebas reales usan tablas efímeras únicas en `tempdb.dbo` en lugar de `#temp tables`, porque la ejecución RPC usada por Tiberius no preserva tablas temporales locales entre llamadas separadas.
- La validación manual de esta sesión confirmó conectividad real con SQL Server local usando el login `sa`; la cadena original con `Database=test` no fue usable porque esa base no estaba accesible, así que la verificación se ejecutó contra `master`.
- La crate pública `mssql-orm` declara `extern crate self as mssql_orm` para que los macros puedan apuntar a una ruta estable tanto dentro del workspace como desde crates consumidoras.
- La crate pública `mssql-orm` ya expone `DbContext`, `DbSet`, `DbSetQuery`, `SharedConnection`, `connect_shared` y reexporta `tokio`, permitiendo que `#[derive(DbContext)]` genere métodos `connect`, `from_connection` y `from_shared_connection` sin depender de imports adicionales en el consumidor.
- `DbSet<T>` ya encapsula una conexión compartida sobre `tokio::sync::Mutex<MssqlConnection<_>>`, expone metadata de entidad y ahora también expone `query()` y `query_with(SelectQuery)` como base pública para ejecución de queries por entidad.
- `DbSetQuery<T>` ya encapsula un `SelectQuery` y soporta `all`, `first` y `count`, reutilizando `SqlServerCompiler`, `fetch_one` y `fetch_all` sin mover ejecución ni generación SQL fuera de sus crates.
- `mssql-orm-sqlserver` ahora compila `CountQuery` con alias estable `AS [count]`, habilitando materialización consistente del conteo desde la crate pública.
- `mssql-orm-macros` ya implementa `#[derive(DbContext)]` para structs con campos `DbSet<Entidad>`, validando en compilación que el shape del contexto siga el contrato previsto.
- La `prelude` pública ya reexporta los derives `Entity`, `Insertable`, `Changeset` y `DbContext`, por lo que los tests de integración usan la misma superficie que usará un consumidor real.
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

- `SqlValue::Null` sigue siendo no tipado en el core, por lo que su binding actual en Tiberius es provisional y conviene revisarlo cuando exista suficiente contexto de tipo.
- `DbSet<T>` ya sostiene una conexión compartida y un runner base de query, pero todavía faltan `find`, `insert`, `update` y `delete`; las siguientes tareas deben reutilizar ese handle sin introducir lógica SQL fuera de `mssql-orm-sqlserver`.
- Las pruebas reales dependen de un connection string válido en `MSSQL_ORM_TEST_CONNECTION_STRING`; si apunta a una base inexistente, la validación falla antes de probar el adaptador.
- Si futuras sesiones empiezan a programar sin revisar `docs/`, se pierde trazabilidad.
- Como el repositorio raíz es nuevo, cualquier archivo ajeno al trabajo técnico debe revisarse antes de incluirlo en commits iniciales.

## Próximo Enfoque Recomendado

1. Implementar `Etapa 5: Implementar DbSet::find por primary key simple`.
2. Reutilizar `DbSetQuery`, `query_with`, metadata de primary key, `MssqlConnection`, `fetch_one` y `SqlServerCompiler` como base inmediata.
3. Mantener estable el contrato de `#[derive(DbContext)]` y la nueva superficie `DbSetQuery<T>` mientras entran `insert`, `update` y `delete`.

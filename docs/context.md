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

Continuar la Etapa 7 implementando la generación SQL de migraciones y la base de historial `__mssql_orm_migrations`, ahora que snapshots, operaciones y diff engine básico ya quedaron cubiertos con pruebas unitarias dedicadas.

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
- La crate pública `mssql-orm` ahora también expone el trait `EntityColumnPredicateExt` en su `prelude`, habilitando `eq`, `ne`, `gt`, `gte`, `lt`, `lte`, `is_null` e `is_not_null` directamente sobre `EntityColumn`.
- La misma extensión pública ahora también expone `contains`, `starts_with` y `ends_with`, reutilizando `Predicate::Like` con parámetros generados en la crate pública.
- La crate pública `mssql-orm` ahora también expone `EntityColumnOrderExt`, habilitando `asc()` y `desc()` directamente sobre `EntityColumn` y produciendo `OrderBy` del AST existente.
- `DbSet<T>` ya encapsula una conexión compartida sobre `tokio::sync::Mutex<MssqlConnection<_>>`, expone metadata de entidad y ahora también expone `query()` y `query_with(SelectQuery)` como base pública para ejecución de queries por entidad.
- `DbSet<T>` ahora también expone `find<K>()` para primary key simple, construyendo un `SelectQuery` filtrado desde la metadata de entidad y delegando la ejecución al runner base.
- `DbSet<T>` ahora también expone `insert<I>()`, compilando un `InsertQuery` desde `Insertable<E>` y materializando la entidad devuelta por `OUTPUT INSERTED.*`.
- `DbSet<T>` ahora también expone `update<K, C>() -> Result<Option<E>, OrmError>`, compilando un `UpdateQuery` desde `Changeset<E>` y materializando la fila actualizada cuando existe.
- `DbSet<T>` ahora también expone `delete<K>() -> Result<bool, OrmError>`, compilando un `DeleteQuery` por primary key simple y devolviendo si hubo al menos una fila afectada.
- La crate pública `mssql-orm` ahora también expone `PageRequest` como contrato estable de paginación explícita.
- La crate pública `mssql-orm` ahora también expone `PredicateCompositionExt`, habilitando `and`, `or` y `not` sobre `Predicate` sin introducir un DSL lógico paralelo.
- `DbSetQuery<T>` ya encapsula un `SelectQuery` y soporta `filter`, `order_by`, `limit`, `take`, `paginate`, `all`, `first` y `count`, reutilizando `SqlServerCompiler`, `fetch_one` y `fetch_all` sin mover ejecución ni generación SQL fuera de sus crates.
- La crate pública `mssql-orm` ahora también cuenta con una batería específica de pruebas públicas del query builder: una prueba de integración sobre la forma del AST y un caso `trybuild` que valida el encadenamiento desde código consumidor.
- La crate pública `mssql-orm` ahora también cuenta con snapshots del SQL generado desde el query builder público y con una prueba explícita de seguridad para confirmar que valores no confiables quedan parametrizados y no se interpolan en el SQL.
- `mssql-orm-migrate` ya dejó de ser solo un marker crate y ahora expone `ModelSnapshot`, `SchemaSnapshot`, `TableSnapshot`, `ColumnSnapshot`, `IndexSnapshot` e `IndexColumnSnapshot` como base del sistema de migraciones.
- El snapshot actual usa `String` y `Vec<_>` para ser persistible fuera de metadata estática, pero conserva el shape relevante de SQL Server (`SqlServerType`, `IdentityMetadata`, nullability, defaults, computed, rowversion, longitudes, precisión/escala, PK e índices).
- `TableSnapshot` conserva nombre de PK y columnas de PK además de columnas e índices, permitiendo que la siguiente subtarea convierta metadata code-first a snapshot sin redefinir el contrato base.
- `mssql-orm-migrate` ahora también implementa conversión directa desde metadata estática: `ColumnSnapshot: From<&ColumnMetadata>`, `IndexColumnSnapshot: From<&IndexColumnMetadata>`, `IndexSnapshot: From<&IndexMetadata>` y `TableSnapshot: From<&EntityMetadata>`.
- `ModelSnapshot::from_entities(&[&EntityMetadata])` ya agrupa entidades por schema usando orden determinista y ordena tablas por nombre dentro de cada schema, dejando una base estable para snapshots persistidos y futuros diffs.
- La conversión actual conserva el orden original de columnas, el nombre y columnas de primary key, y los índices declarados en metadata; foreign keys siguen fuera de alcance hasta etapas posteriores.
- `mssql-orm-migrate` ahora también expone `MigrationOperation` en un módulo separado, con payloads mínimos para `CreateSchema`, `DropSchema`, `CreateTable`, `DropTable`, `AddColumn`, `DropColumn` y `AlterColumn`.
- Las operaciones de tabla reutilizan `TableSnapshot` completo y las de columna reutilizan `ColumnSnapshot`, evitando duplicar contratos antes de implementar el diff engine.
- `MigrationOperation` ya expone helpers de lectura para `schema_name()` y `table_name()`, lo que simplifica ordenamiento y aserciones del futuro diff básico sin introducir aún generación SQL.
- `mssql-orm-migrate` ahora también expone `diff_schema_and_table_operations(previous, current)`, que compara `ModelSnapshot` y emite operaciones deterministas para `CreateSchema`, `CreateTable`, `DropTable` y `DropSchema`.
- El orden del diff actual es intencionalmente seguro para este alcance: primero crea schemas, luego tablas nuevas; después elimina tablas sobrantes y al final schemas vacíos/eliminados.
- El diff de schemas/tablas no intenta todavía detectar renombres ni cambios internos de columnas; esas responsabilidades quedan explícitamente para las siguientes subtareas de Etapa 7.
- `mssql-orm-migrate` ahora también expone `diff_column_operations(previous, current)`, limitado a tablas compartidas entre ambos snapshots.
- El diff de columnas ya detecta `AddColumn`, `DropColumn` y `AlterColumn` comparando `ColumnSnapshot` completo y usando orden determinista por nombre de columna.
- El diff de columnas ignora intencionalmente tablas nuevas o eliminadas, para no duplicar trabajo ya cubierto por `CreateTable`/`DropTable`; renombres de columna siguen fuera de alcance en este MVP.
- La cobertura del diff engine ya quedó centralizada en pruebas unitarias dedicadas dentro de `crates/mssql-orm-migrate/src/diff.rs`, en lugar de estar dispersa en `lib.rs`.
- Esa batería ya fija casos mínimos de orden seguro, no-op sobre snapshots iguales, altas/bajas de tablas, altas/bajas de columnas, alteraciones básicas y una composición completa de diff sobre snapshots mínimos.
- `lib.rs` quedó otra vez enfocado en reexports, boundaries y shape base de snapshots/operaciones, reduciendo ruido y duplicación en la capa pública de la crate.
- La crate pública `mssql-orm` ya cuenta con una prueba de integración real en `crates/mssql-orm/tests/stage5_public_crud.rs` que valida `insert`, `find`, `query`, `update` y `delete` contra SQL Server.
- Esa prueba crea y limpia `dbo.mssql_orm_public_crud` dentro de la base activa del connection string y usa `MSSQL_ORM_TEST_CONNECTION_STRING` con skip limpio cuando no existe configuración.
- La misma prueba pública ahora acepta `KEEP_TEST_TABLES=1` para conservar `dbo.mssql_orm_public_crud` y facilitar inspección manual posterior en SQL Server.
- La misma prueba pública ahora también acepta `KEEP_TEST_ROWS=1` para conservar la tabla y dejar una fila final persistida, facilitando inspección manual con datos reales.
- El repositorio ahora también incluye `examples/basic-crud/` como ejemplo ejecutable fuera del workspace principal, validado con `cargo run --manifest-path`.
- Ese ejemplo usa `DATABASE_URL`, prepara `dbo.basic_crud_users`, recorre `insert`, `find`, `query`, `update` y `delete`, y limpia la tabla al final.
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

## Configuración Operativa Local

- Connection string actualmente usada para validaciones reales e integraciones locales:
  `Server=localhost;Database=tempdb;User Id=SA;Password=Ea.930318;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;`
- Usarla para `MSSQL_ORM_TEST_CONNECTION_STRING` en pruebas reales y para `DATABASE_URL` en el ejemplo `examples/basic-crud/` mientras el entorno local siga siendo el mismo.
- Esta configuración es específica del entorno local actual; si SQL Server, credenciales o base cambian, debe actualizarse el mismo día en esta sección y en el `worklog`.

## Riesgos Inmediatos

- `SqlValue::Null` sigue siendo no tipado en el core, por lo que su binding actual en Tiberius es provisional y conviene revisarlo cuando exista suficiente contexto de tipo.
- La base CRUD pública y el ejemplo ejecutable ya existen; el siguiente riesgo inmediato es introducir un query builder público que duplique o contradiga el AST y runner ya presentes.
- `find` todavía no soporta primary key compuesta; hoy falla explícitamente en ese caso y ese límite debe mantenerse documentado hasta que exista soporte dedicado.
- `update` tampoco soporta primary key compuesta en esta etapa y retorna `Option<E>` para representar ausencia de fila, reservando semánticas de conflicto más fuertes para la Etapa 11.
- `delete` tampoco soporta primary key compuesta en esta etapa y retorna `bool` para distinguir entre fila eliminada y ausencia de fila, reservando conflictos de concurrencia para la Etapa 11.
- Las pruebas reales dependen de un connection string válido en `MSSQL_ORM_TEST_CONNECTION_STRING`; si apunta a una base inexistente, la validación falla antes de probar el adaptador.
- Si futuras sesiones empiezan a programar sin revisar `docs/`, se pierde trazabilidad.
- Como el repositorio raíz es nuevo, cualquier archivo ajeno al trabajo técnico debe revisarse antes de incluirlo en commits iniciales.

## Próximo Enfoque Recomendado

1. Implementar `Etapa 7: Implementar generación SQL y tabla __mssql_orm_migrations`.
2. Mantener la generación SQL exclusivamente en la capa SQL Server y usar `MigrationOperation` como entrada, sin mezclar CLI todavía.
3. Dejar la validación real contra SQL Server para después de contar con SQL generado y registro de historial básico.

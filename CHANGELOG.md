# Changelog

Todos los cambios relevantes de `mssql-orm` se documentan en este archivo.

El proyecto sigue una estrategia de release incremental. Este changelog describe la surface disponible en el release inicial `0.1.0` del workspace y sus exclusiones explicitas.

## [0.1.0] - Unreleased

Release inicial del ORM `code-first` para Rust y SQL Server basado en Tiberius.

### Disponible

- Workspace modular con crates separadas:
  - `mssql-orm-core`
  - `mssql-orm-macros`
  - `mssql-orm-query`
  - `mssql-orm-sqlserver`
  - `mssql-orm-tiberius`
  - `mssql-orm-migrate`
  - `mssql-orm-cli`
  - `mssql-orm`
- Crate publica `mssql-orm` con `prelude` y reexports avanzados seleccionados.
- `#[derive(Entity)]` con metadata estatica, `FromRow` generado y simbolos de columna.
- Atributos code-first para tablas, schema, columnas, primary key, identity, tipos SQL Server, longitud, precision, escala, nullability, defaults, computed columns, rowversion, indices, foreign keys y hints explicitos de rename.
- `#[derive(Insertable)]` y `#[derive(Changeset)]` para payloads de escritura.
- `#[derive(DbContext)]` con `DbSet<T>`, conexion directa, health check, transacciones runtime y fuente de metadata para migraciones.
- CRUD publico base sobre `DbSet<T>`:
  - `find`
  - `insert`
  - `update`
  - `delete`
- Query builder publico con:
  - predicados tipados (`eq`, `ne`, `gt`, `gte`, `lt`, `lte`, `is_null`, `is_not_null`)
  - predicados string (`contains`, `starts_with`, `ends_with`)
  - composicion logica (`and`, `or`, `not`)
  - `order_by`
  - `limit`
  - `take`
  - `paginate`
  - `all`
  - `first`
  - `count`
  - joins explicitos (`inner_join`, `left_join`)
- Proyecciones tipadas publicas sobre `DbSetQuery` con `select(...)`, `all_as::<T>()` y `first_as::<T>()` para DTOs que implementan `FromRow`.
- Raw SQL tipado con `DbContext::raw<T>(...)`, `DbContext::raw_exec(...)`, parametros `@P1..@Pn`, materializacion `FromRow` y ejecucion de comandos.
- AST en `mssql-orm-query` sin generacion SQL directa.
- Compilador SQL Server en `mssql-orm-sqlserver` para queries y DDL de migraciones, con parametros `@P1..@Pn`.
- Adaptador Tiberius con conexion, ejecucion, mapping de filas, transacciones, timeouts, tracing, slow query logs, retry acotado, health checks y pool opcional bajo feature `pool-bb8`.
- Active Record base:
  - `Entity::query(&db)`
  - `Entity::find(&db, id)`
  - `entity.save(&db)`
  - `entity.delete(&db)`
- Concurrencia optimista con `rowversion` y `OrmError::ConcurrencyConflict`.
- Change tracking experimental con `Tracked<T>`, `EntityState`, `find_tracked`, `add_tracked`, `remove_tracked` y `save_changes`.
- Migraciones code-first:
  - `ModelSnapshot`
  - serializacion/deserializacion JSON
  - diff de schemas, tablas, columnas, indices y foreign keys
  - renombres explicitos de tabla y columna
  - DDL SQL Server para operaciones soportadas
  - `down.sql` generado cuando el plan es reversible con payload suficiente
  - bloqueo por defecto de cambios destructivos en `migration add`
  - script idempotente de `database update` con historial, checksum y transaccion por migracion
- CLI `mssql-orm-cli` con:
  - `migration add`
  - `migration list`
  - `database update`
  - `database update --execute`
  - `--model-snapshot`
  - `--snapshot-bin`
- Ejemplo `examples/todo-app` con dominio relacional, query builder publico, endpoints HTTP minimos, health check, pool opcional y smoke reproducible contra SQL Server real.
- Documentacion publica inicial:
  - `README.md`
  - `docs/quickstart.md`
  - `docs/code-first.md`
  - `docs/api.md`
  - `docs/query-builder.md`
  - `docs/relationships.md`
  - `docs/transactions.md`
  - `docs/migrations.md`
  - `docs/entity-policies.md`

### Entity Policies

- Se introdujo el concepto de `Entity Policies`.
- `#[derive(AuditFields)]` permite definir columnas reutilizables de auditoria.
- `#[orm(audit = Audit)]` expande columnas auditables dentro de `EntityMetadata.columns`.
- Las columnas auditables participan como metadata/schema normal en snapshots, diff y DDL.
- `#[derive(SoftDeleteFields)]` y `#[orm(soft_delete = SoftDelete)]` agregan borrado logico runtime, visibilidad de lectura por defecto y schema como columnas ordinarias.
- `#[derive(TenantContext)]` y `#[orm(tenant = CurrentTenant)]` agregan tenant opt-in con filtros obligatorios sobre la entidad raiz y autollenado/validacion de inserts tenant-scoped.

### Exclusiones explicitas

- SQL Server es el unico backend soportado.
- No hay soporte multi-base de datos.
- No hay navigation properties.
- No hay lazy loading ni eager loading automatico.
- No hay aliases de tabla en joins.
- No hay agregaciones tipadas de alto nivel ni aliases automaticos para self-joins.
- `count()` no conserva joins en esta etapa.
- CRUD publico, Active Record y tracking siguen orientados a primary key simple.
- `save_changes()` y `Tracked<T>` son experimentales.
- No hay savepoints.
- `db.transaction(...)` no debe considerarse soportado sobre contextos creados desde `from_pool(...)` hasta pinnear una conexion fisica durante todo el closure.
- No hay autollenado runtime de campos auditables.
- `audit = Audit` no agrega campos Rust visibles ni simbolos de columna sobre la entidad.
- `AuditProvider`, `timestamps`, autollenado runtime de auditoria y filtros automaticos de `soft_delete`/`tenant` sobre entidades unidas manualmente quedan diferidos.
- `raw<T>()` y `raw_exec()` no aplican automaticamente filtros ORM de `tenant` ni `soft_delete`.
- `down.sql` no se ejecuta automaticamente.
- No existe `database downgrade`.
- `migration.rs` queda fuera del MVP actual.

### Validacion conocida

- El workspace tiene validaciones locales y CI para formato, compilacion, tests y clippy.
- Hay cobertura `trybuild` para derives publicos y errores de macros.
- Hay snapshots de SQL compilado para queries y migraciones.
- Las pruebas reales contra SQL Server dependen de `MSSQL_ORM_TEST_CONNECTION_STRING`.
- El ejemplo `todo-app` cuenta con smoke reproducible usando `DATABASE_URL`.

### Documentacion de referencia

- API publica: [docs/api.md](docs/api.md)
- Quickstart: [docs/quickstart.md](docs/quickstart.md)
- Code-first: [docs/code-first.md](docs/code-first.md)
- Query builder: [docs/query-builder.md](docs/query-builder.md)
- Relaciones y joins: [docs/relationships.md](docs/relationships.md)
- Transacciones: [docs/transactions.md](docs/transactions.md)
- Migraciones: [docs/migrations.md](docs/migrations.md)
- Entity Policies: [docs/entity-policies.md](docs/entity-policies.md)

# Changelog

All relevant `mssql-orm` changes are documented in this file.

The project follows an incremental release strategy. This changelog describes the API surface available in the initial `0.1.0` workspace release and its explicit exclusions.

## [0.1.0] - Unreleased

Initial code-first ORM release for Rust and SQL Server, built on top of Tiberius.

### Available

- Modular workspace with separate crates:
  - `mssql-orm-core`
  - `mssql-orm-macros`
  - `mssql-orm-query`
  - `mssql-orm-sqlserver`
  - `mssql-orm-tiberius`
  - `mssql-orm-migrate`
  - `mssql-orm-cli`
  - `mssql-orm`
- Public `mssql-orm` crate with a `prelude` and selected advanced reexports.
- `#[derive(Entity)]` with static metadata, generated `FromRow`, and column symbols.
- Code-first attributes for tables, schemas, columns, primary keys, identity columns, SQL Server types, length, precision, scale, nullability, defaults, computed columns, rowversion, indexes, foreign keys, and explicit rename hints.
- `#[derive(Insertable)]` and `#[derive(Changeset)]` for write payloads.
- `#[derive(DbContext)]` with `DbSet<T>`, direct connections, health checks, runtime transactions, and a metadata source for migrations.
- Basic public CRUD over `DbSet<T>`:
  - `find`
  - `insert`
  - `update`
  - `delete`
- Public query builder with:
  - typed predicates (`eq`, `ne`, `gt`, `gte`, `lt`, `lte`, `is_null`, `is_not_null`)
  - string predicates (`contains`, `starts_with`, `ends_with`)
  - logical composition (`and`, `or`, `not`)
  - `order_by`
  - `limit`
  - `take`
  - `paginate`
  - `all`
  - `first`
  - `count`
  - explicit joins (`inner_join`, `left_join`)
- Public typed projections on `DbSetQuery` with `select(...)`, `all_as::<T>()`, and `first_as::<T>()` for DTOs that implement `FromRow`.
- Typed raw SQL with `DbContext::raw<T>(...)`, `DbContext::raw_exec(...)`, `@P1..@Pn` parameters, `FromRow` materialization, and command execution.
- AST in `mssql-orm-query` without direct SQL generation.
- SQL Server compiler in `mssql-orm-sqlserver` for queries and migration DDL, using `@P1..@Pn` parameters.
- Tiberius adapter with connection handling, execution, row mapping, transactions, timeouts, tracing, slow query logs, bounded retries, health checks, and optional pooling behind the `pool-bb8` feature.
- Basic Active Record:
  - `Entity::query(&db)`
  - `Entity::find(&db, id)`
  - `entity.save(&db)`
  - `entity.delete(&db)`
- Optimistic concurrency with `rowversion` and `OrmError::ConcurrencyConflict`.
- Experimental change tracking with `Tracked<T>`, `EntityState`, `find_tracked`, `add_tracked`, `remove_tracked`, and `save_changes`.
- Code-first migrations:
  - `ModelSnapshot`
  - JSON serialization/deserialization
  - schema, table, column, index, and foreign-key diffs
  - explicit table and column renames
  - SQL Server DDL for supported operations
  - generated `down.sql` when the full plan is reversible with the available payload
  - destructive-change blocking by default in `migration add`
  - idempotent `database update` script with migration history, checksums, and one transaction per migration
- `mssql-orm-cli` commands:
  - `migration add`
  - `migration list`
  - `database update`
  - `database update --execute`
  - `--model-snapshot`
  - `--snapshot-bin`
- `examples/todo-app` with a relational domain, public query builder usage, minimal HTTP endpoints, health check, optional pooling, and a reproducible smoke flow against real SQL Server.
- Initial public documentation:
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

- The release introduces the `Entity Policies` concept.
- `#[derive(AuditFields)]` defines reusable audit columns.
- `#[orm(audit = Audit)]` expands audit columns into `EntityMetadata.columns`.
- `AuditEntity::audit_policy()` exposes audit-owned columns for audited entities without changing snapshots, diffs, or DDL.
- Audit columns participate as normal metadata/schema columns in snapshots, diffs, and DDL.
- `#[derive(SoftDeleteFields)]` and `#[orm(soft_delete = SoftDelete)]` add runtime soft-delete behavior, default read visibility, and schema columns through the normal column pipeline.
- `#[derive(TenantContext)]` and `#[orm(tenant = CurrentTenant)]` add opt-in tenant scoping with mandatory filters on the root entity and tenant insert auto-fill/validation.

### Explicit Exclusions

- SQL Server is the only supported backend.
- Multi-database support is not available.
- Navigation properties are not available.
- Lazy loading and automatic eager loading are not available.
- Table aliases in joins are not available.
- High-level typed aggregations and automatic aliases for self-joins are not available.
- `count()` does not preserve joins in this release.
- Public CRUD, Active Record, and tracking are still oriented around simple primary keys.
- `save_changes()` and `Tracked<T>` are experimental.
- Savepoints are not available.
- `db.transaction(...)` must not be treated as supported on contexts created from `from_pool(...)` until a physical connection can be pinned for the full closure.
- `AuditProvider` has a public runtime contract and audit-owned column metadata, but is not transported through `DbContext`/`SharedConnection` and is not applied to insert/update paths yet.
- Runtime audit-field auto-fill is not implemented.
- `audit = Audit` does not add visible Rust fields or entity column symbols.
- `timestamps`, runtime audit auto-fill, and automatic `soft_delete`/`tenant` filters over manually joined entities remain deferred.
- `raw<T>()` and `raw_exec()` do not automatically apply ORM `tenant` or `soft_delete` filters.
- `down.sql` is not executed automatically.
- `database downgrade` does not exist.
- `migration.rs` is outside the current MVP.

### Known Validation

- The workspace has local validation and CI for formatting, compilation, tests, and clippy.
- `trybuild` covers public derives and macro errors.
- SQL snapshots cover compiled queries and migrations.
- Real SQL Server tests depend on `MSSQL_ORM_TEST_CONNECTION_STRING`.
- The `todo-app` example has a reproducible smoke flow using `DATABASE_URL`.

### Reference Documentation

- Public API: [docs/api.md](docs/api.md)
- Quickstart: [docs/quickstart.md](docs/quickstart.md)
- Code-first: [docs/code-first.md](docs/code-first.md)
- Query builder: [docs/query-builder.md](docs/query-builder.md)
- Relationships and joins: [docs/relationships.md](docs/relationships.md)
- Transactions: [docs/transactions.md](docs/transactions.md)
- Migrations: [docs/migrations.md](docs/migrations.md)
- Entity Policies: [docs/entity-policies.md](docs/entity-policies.md)

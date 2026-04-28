# Public API

This document is a compact inventory of the current public surface exposed by the root crate `mssql-orm`.

For consumer code, the recommended entry point is:

```rust
use mssql_orm::prelude::*;
```

The root crate concentrates the user API and reexports selected internals for tests, tooling, and advanced cases. Responsibilities remain separated by crate: `query` builds ASTs, `sqlserver` compiles SQL, `tiberius` executes, `migrate` manages snapshots/diffs/migrations, and `core` defines shared contracts.

See also [Core concepts](core-concepts.md).

## Public Derives

The following derives are available from the public crate:

- `#[derive(Entity)]`
- `#[derive(Insertable)]`
- `#[derive(Changeset)]`
- `#[derive(DbContext)]`
- `#[derive(AuditFields)]`
- `#[derive(SoftDeleteFields)]`
- `#[derive(TenantContext)]`

Basic example:

```rust
use mssql_orm::prelude::*;

#[derive(Entity, Debug, Clone)]
#[orm(table = "users", schema = "dbo")]
struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(length = 180)]
    email: String,
}

#[derive(Insertable)]
#[orm(entity = User)]
struct NewUser {
    email: String,
}

#[derive(Changeset)]
#[orm(entity = User)]
struct UpdateUser {
    email: Option<String>,
}

#[derive(DbContext)]
struct AppDb {
    pub users: DbSet<User>,
}
```

## Model Contracts

The prelude exposes the main metadata and mapping contracts:

- `Entity`
- `EntityMetadata`
- `EntityColumn`
- `ColumnMetadata`
- `PrimaryKeyMetadata`
- `IdentityMetadata`
- `IndexMetadata`
- `IndexColumnMetadata`
- `ForeignKeyMetadata`
- `ReferentialAction`
- `EntityPolicy`
- `EntityPolicyMetadata`
- `SqlServerType`
- `SqlTypeMapping`
- `SqlValue`
- `ColumnValue`
- `Row`
- `FromRow`
- `Insertable`
- `Changeset`
- `OrmError`

Typical use:

```rust
let metadata = User::metadata();
let email_column = User::email;

assert_eq!(metadata.table, "users");
assert_eq!(email_column.column_name(), "email");
```

## DbContext and DbSet

The main data-access API is:

- `DbContext`
- `DbSet<T>`
- `DbSetQuery<T>`
- `DbContextEntitySet<T>`
- `SharedConnection`
- `connect_shared(...)`
- `connect_shared_with_options(...)`
- `connect_shared_with_config(...)`

`#[derive(DbContext)]` generates inherent methods on your context:

- `connect(...)`
- `connect_with_options(...)`
- `connect_with_config(...)`
- `from_connection(...)`
- `from_shared_connection(...)`
- `health_check().await`
- `transaction(|tx| async move { ... }).await`
- `save_changes().await`
- `from_pool(...)` when `pool-bb8` is enabled

`DbSet<T>` exposes CRUD and query operations:

- `find(key).await`
- `insert(model).await`
- `update(key, changeset).await`
- `delete(key).await`
- `query()`
- `query_with(select_query)`
- `entity_metadata()`
- `find_tracked(key).await`
- `add_tracked(entity)`
- `remove_tracked(&mut tracked)`

Relevant limits:

- `find`, `update`, `delete`, Active Record, and public tracking remain oriented around simple primary keys.
- `save_changes()` and `Tracked<T>` are experimental.
- `db.transaction(...)` is blocked for contexts created from pools until one physical connection can be pinned for the full closure.

## Query Builder

The public query extensions include:

- `EntityColumnPredicateExt`
- `PredicateCompositionExt`
- `EntityColumnOrderExt`
- `PageRequest`
- `SelectProjections`

Common query methods:

- `filter(...)`
- `order_by(...)`
- `limit(...)`
- `take(...)`
- `paginate(...)`
- `inner_join::<T>(...)`
- `left_join::<T>(...)`
- `select(...)`
- `all().await`
- `first().await`
- `count().await`
- `all_as::<T>().await`
- `first_as::<T>().await`

The query builder produces AST values. SQL generation belongs to `mssql-orm-sqlserver`.

## Raw SQL

Raw SQL is exposed through:

- `DbContext::raw<T>(sql)`
- `DbContext::raw_exec(sql)`
- `RawQuery<T>`
- `RawCommand`
- `RawParam`
- `RawParams`
- `QueryHint`

Raw SQL uses `@P1..@Pn` parameters and materializes query rows through `FromRow`. It does not automatically apply tenant or soft-delete filters.
`RawQuery<T>::query_hint(QueryHint::Recompile)` can append SQL Server `OPTION (RECOMPILE)` for parametrized raw queries that need per-execution plan compilation.

## Entity Policies

Public policy-related contracts and derives include:

- `EntityPolicy`
- `EntityPolicyMetadata`
- `AuditFields`
- `SoftDeleteFields`
- `TenantContext`
- `SoftDeleteEntity`
- `TenantScopedEntity`
- `SoftDeleteProvider`
- `SoftDeleteContext`
- `SoftDeleteOperation`
- `SoftDeleteRequestValues`
- `ActiveTenant`

Implemented behavior:

- `audit = Audit` contributes metadata/schema columns only.
- `soft_delete = SoftDelete` changes delete behavior and read visibility for the root entity.
- `tenant = CurrentTenant` adds fail-closed tenant filtering and tenant insert validation/fill for opt-in entities.

Deferred behavior:

- runtime audit auto-fill through `AuditProvider`;
- automatic policy filters over manually joined entities;
- global tenant conventions without a user-defined tenant type.

## Migrations

Migration-related public helpers include:

- `MigrationModelSource`
- `model_snapshot_from_source::<C>()`
- `model_snapshot_json_from_source::<C>()`

Advanced migration types are reexported through the `migrate` module for tooling.

## Operational Types

The public crate reexports Tiberius adapter configuration types such as:

- `MssqlConnectionConfig`
- `MssqlOperationalOptions`
- `MssqlTimeoutOptions`
- `MssqlRetryOptions`
- `MssqlTracingOptions`
- `MssqlSlowQueryOptions`
- `MssqlHealthCheckOptions`
- `MssqlHealthCheckQuery`
- `MssqlParameterLogMode`
- `MssqlPoolOptions`
- `MssqlPoolBackend`
- `MssqlPool`, `MssqlPoolBuilder`, and `MssqlPooledConnection` when `pool-bb8` is enabled

## Advanced Reexports

The root crate also reexports selected internal crates:

- `mssql_orm::core`
- `mssql_orm::query`
- `mssql_orm::sqlserver`
- `mssql_orm::tiberius`
- `mssql_orm::migrate`
- `mssql_orm::macros`

These are useful for tests, tooling, snapshots, and advanced diagnostics. Normal application code should prefer the prelude.

## Current Exclusions

- SQL Server is the only backend.
- Navigation properties are not available.
- Lazy loading and automatic eager loading are not available.
- Table aliases in joins are not available.
- High-level typed aggregations are not available.
- Composite primary-key persistence is not complete across public CRUD and Active Record.
- `migration.rs` is not generated.

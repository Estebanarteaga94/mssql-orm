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
- `#[derive(FromRow)]`
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

#[derive(FromRow)]
struct UserSummary {
    id: i64,
    #[orm(column = "email_address")]
    email: String,
    display_name: Option<String>,
}

#[derive(DbContext)]
struct AppDb {
    pub users: DbSet<User>,
}
```

`#[derive(FromRow)]` is available for DTOs used by typed projections and typed raw SQL. It supports structs with named fields, default aliases from field names, explicit field aliases with `#[orm(column = "...")]`, and nullable or missing projected columns through `Option<T>`.

## Model Contracts

The prelude exposes the main metadata and mapping contracts:

- `Entity`
- `EntityMetadata`
- `EntityColumn`
- `Navigation<T>`
- `Collection<T>`
- `ColumnMetadata`
- `PrimaryKeyMetadata`
- `IdentityMetadata`
- `IndexMetadata`
- `IndexColumnMetadata`
- `ForeignKeyMetadata`
- `NavigationMetadata`
- `NavigationKind`
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

Navigation fields declared with `Navigation<T>` or `Collection<T>` are metadata-only in the current cut. `#[derive(Entity)]` accepts `belongs_to`, `has_one` and `has_many` attributes, excludes those fields from `ColumnMetadata`, and initializes the wrappers empty when materializing an entity.

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
- `try_inner_join_navigation::<T>(...)`
- `try_left_join_navigation::<T>(...)`
- `try_inner_join_navigation_as::<T>(...)`
- `try_left_join_navigation_as::<T>(...)`
- `include::<T>(...)`
- `include_as::<T>(...)`
- `select(...)`
- `all().await`
- `first().await`
- `count().await`
- `all_as::<T>().await`
- `first_as::<T>().await`

The query builder produces AST values. SQL generation belongs to `mssql-orm-sqlserver`.

`include::<T>(...)` and `include_as::<T>(...)` are limited to one
`belongs_to` / `has_one` navigation. Root policies are applied to the effective
query predicate, while included-entity `tenant` and default `soft_delete`
policies are applied to the include join predicate.

Projection DTOs can derive `FromRow`:

```rust
use mssql_orm::prelude::*;

#[derive(Debug, FromRow)]
struct UserSummary {
    id: i64,
    #[orm(column = "email_address")]
    email: String,
}

let summaries = db
    .users
    .query()
    .select((
        User::id,
        SelectProjection::expr_as(mssql_orm::query::Expr::from(User::email), "email_address"),
    ))
    .all_as::<UserSummary>()
    .await?;
```

The derive is intentionally limited to named-field DTOs. Tuple structs, unit structs, and field-level `#[orm(...)]` attributes other than `column = "..."` are rejected at compile time.

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
- `AuditEntity`
- `SoftDeleteEntity`
- `TenantScopedEntity`
- `AuditProvider`
- `AuditContext`
- `AuditOperation`
- `AuditRequestValues`
- `AuditValues`
- `SoftDeleteProvider`
- `SoftDeleteContext`
- `SoftDeleteOperation`
- `SoftDeleteRequestValues`
- `SoftDeleteValues`
- `ActiveTenant`

Implemented behavior:

- `audit = Audit` contributes metadata/schema columns only.
- `#[derive(Entity)]` implements `AuditEntity`; `audit_policy()` returns the audit-owned columns for audited entities and `None` for entities without `audit`.
- The runtime `AuditProvider` contract exists in the public crate, including operation, request values, context, and precedence rules for resolving `ColumnValue`s.
- `#[derive(AuditFields)]` implements `AuditValues`, so the same audit policy struct can be passed as typed request values with `with_audit_values(Audit { ... })`.
- `SharedConnection` and derived `DbContext`s transport `AuditProvider`, `AuditRequestValues`, and typed `AuditValues`; transaction contexts inherit the same shared runtime.
- Insert/update paths consume that runtime: `DbSet::insert`, `DbSet::update`, Active Record `save`, and `save_changes()` for `Added`/`Modified` complete missing audit columns declared by `AuditEntity::audit_policy()`.
- `soft_delete = SoftDelete` changes delete behavior and read visibility for the root entity.
- `#[derive(SoftDeleteFields)]` implements `SoftDeleteValues`, so the same soft-delete policy struct can be passed as typed request values with `with_soft_delete_values(SoftDelete { ... })`.
- `SharedConnection` and derived `DbContext`s transport `SoftDeleteProvider`, `SoftDeleteRequestValues`, and typed `SoftDeleteValues`; typed values are converted into the existing request-values path.
- `tenant = CurrentTenant` adds fail-closed tenant filtering and tenant insert validation/fill for opt-in entities.

Deferred behavior:

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
- Navigation properties currently expose metadata, explicit join inference, and single-navigation eager loading for `belongs_to` / `has_one`.
- Lazy loading and collection eager loading are not available.
- High-level typed aggregations are not available.
- Composite primary-key persistence is not complete across public CRUD and Active Record.
- `migration.rs` is not generated.

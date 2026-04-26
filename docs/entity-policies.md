# Entity Policies

This document describes the public concept, architectural boundaries, implemented behavior, and deferred work for `Entity Policies` in `mssql-orm`.

The initial MVP implemented audit as metadata/schema through `#[derive(AuditFields)]` and `#[orm(audit = Audit)]`. Later cuts added runtime behavior for `soft_delete` and mandatory tenant filters for `tenant`. Runtime `AuditProvider` behavior remains deferred.

See also [Core concepts](core-concepts.md).

## Goal

An `Entity Policy` is a reusable code-first model component that an entity can declare to add cross-cutting columns and, when explicitly designed, related runtime behavior.

The feature avoids repeating the same structural fields in many entities, for example audit columns, soft-delete columns, or tenant columns. A policy does not replace the entity model; it extends it declaratively.

```rust
use mssql_orm::prelude::*;

#[derive(AuditFields)]
struct Audit {
    #[orm(default_sql = "SYSUTCDATETIME()")]
    #[orm(sql_type = "datetime2")]
    created_at: String,

    #[orm(nullable)]
    #[orm(sql_type = "datetime2")]
    updated_at: Option<String>,
}

#[derive(Entity, Debug, Clone)]
#[orm(table = "todos", schema = "todo", audit = Audit)]
struct Todo {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(length = 200)]
    title: String,
}
```

## Core Rule

Columns contributed by a policy must become normal `ColumnMetadata` entries inside `EntityMetadata.columns`.

This prevents a second schema pipeline. The rest of the system continues to use the same pieces:

- `ModelSnapshot::from_entities(...)` reads columns from `EntityMetadata`.
- The diff engine compares `ColumnSnapshot` values without caring whether a column came from an entity field or from a policy.
- `mssql-orm-sqlserver` compiles DDL from normal snapshots and operations.
- `DbContext` and the migration CLI consume normal entity metadata.

Policy-specific metadata can exist for validation and ergonomics, but it must not become a parallel path for snapshots, diffs, or DDL.

## Metadata Contract

The neutral contract lives in `mssql-orm-core` and does not know Tiberius, executable SQL, or migrations:

```rust
pub struct EntityPolicyMetadata {
    pub name: &'static str,
    pub columns: &'static [ColumnMetadata],
}

pub trait EntityPolicy: Sized + Send + Sync + 'static {
    const POLICY_NAME: &'static str;
    const COLUMN_NAMES: &'static [&'static str] = &[];

    fn columns() -> &'static [ColumnMetadata];

    fn metadata() -> EntityPolicyMetadata {
        EntityPolicyMetadata::new(Self::POLICY_NAME, Self::columns())
    }
}
```

The contract exposes a stable name, a static column-name slice for compile-time validation, and a static `ColumnMetadata` slice. Expansion into an entity remains the responsibility of `mssql-orm-macros`.

`EntityMetadata` does not currently keep a separate list of policies. The data that must flow through snapshots, diffs, and DDL is the resulting column.

## Current Policy Matrix

| Policy | Status | Scope |
| --- | --- | --- |
| `audit = Audit` | Implemented | Metadata/schema columns only. No runtime auto-fill. |
| `soft_delete = SoftDelete` | Implemented | Runtime logical delete, default read visibility, and schema columns through the normal column pipeline. |
| `tenant = CurrentTenant` | Implemented | Opt-in tenant scope, active tenant runtime state, fail-closed filters on the root entity, and insert fill/validation. |
| `AuditProvider` | Deferred | Future runtime audit auto-fill for insert/update paths. |
| `timestamps` | Deferred | Not implemented as a separate policy. |

## Audit Fields

`#[derive(AuditFields)]` implements `EntityPolicy` for a user-defined struct. Its fields become reusable audit columns.

Supported field attributes include:

- `column`
- `length`
- `nullable`
- `default_sql`
- `sql_type`
- `precision`
- `scale`
- `renamed_from`
- `insertable`
- `updatable`

Unsupported audit-field attributes include:

- `primary_key`
- `identity`
- `computed_sql`
- `rowversion`
- `index`
- `unique`
- `foreign_key`
- `on_delete`

The derive validates:

- only structs with named fields are accepted;
- field types must implement `SqlTypeMapping`;
- column names must not be empty;
- duplicate columns are rejected;
- unsupported attributes produce compile-time errors.

## `#[orm(audit = Audit)]`

The entity attribute references a Rust type visible from the derive site:

```rust
#[derive(Entity)]
#[orm(table = "orders", schema = "sales", audit = Audit)]
struct Order {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,
}
```

The macro:

- requires `Audit` to implement `EntityPolicy`;
- rejects duplicate `audit` declarations;
- expands audit columns after the entity's own fields in stable order;
- rejects collisions between entity fields and audit columns;
- exposes the final columns through `EntityMetadata.columns`.

Audit columns are metadata/schema columns in the current release. They do not become visible Rust fields and do not generate symbols such as `Todo::created_at`. `FromRow` for the entity materializes only real Rust fields.

## Soft Delete

`soft_delete` is not a metadata-only feature. It changes runtime semantics for delete and read visibility.

Public shape:

```rust
#[derive(SoftDeleteFields)]
struct SoftDelete {
    #[orm(sql_type = "datetime2")]
    deleted_at: Option<String>,

    #[orm(nullable)]
    #[orm(length = 120)]
    deleted_by: Option<String>,
}

#[derive(Entity)]
#[orm(table = "todos", schema = "todo", soft_delete = SoftDelete)]
struct Todo {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    title: String,
}
```

Implemented behavior:

- `#[derive(SoftDeleteFields)]` implements an `EntityPolicy` named `soft_delete`.
- By default, soft-delete columns are `insertable = false` and `updatable = true`.
- `#[derive(Entity)]` accepts `#[orm(soft_delete = SoftDelete)]`.
- Soft-delete columns enter metadata and snapshots as normal columns.
- `DbSet::delete(...)`, Active Record `delete`, and deleted tracked entities use `UpdateQuery` instead of physical `DELETE`.
- Normal entities still use physical `DELETE`.
- `rowversion` and `OrmError::ConcurrencyConflict` remain respected.
- Public reads default to active-only visibility for the root entity.
- `with_deleted()` includes deleted rows.
- `only_deleted()` returns only logically deleted rows.

Visibility convention:

- the first soft-delete policy column controls visibility;
- nullable columns use `IS NULL` / `IS NOT NULL`;
- `BIT` columns use `false` / `true`.

Current limit: automatic soft-delete filtering applies only to the root entity of `DbSetQuery<E>`, not to every manually joined entity.

## Tenant

Tenant is a security feature, not just a schema convenience. It is opt-in per entity and fail-closed.

Public shape:

```rust
#[derive(TenantContext)]
struct CurrentTenant {
    #[orm(column = "tenant_id")]
    id: i64,
}

#[derive(Entity)]
#[orm(table = "orders", schema = "sales", tenant = CurrentTenant)]
struct Order {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    amount: i64,
}
```

Implemented behavior:

- `#[derive(TenantContext)]` accepts a struct with exactly one non-optional tenant field.
- The tenant context implements `EntityPolicy` with `POLICY_NAME = "tenant"`.
- `#[derive(Entity)]` accepts `#[orm(tenant = CurrentTenant)]`.
- Tenant columns enter metadata as normal columns.
- Entities without `tenant` are cross-tenant even when the context has an active tenant.
- `SharedConnection` transports `ActiveTenant { column_name, value }`.
- Derived contexts expose `with_tenant(...)` and `clear_tenant()`.
- Reads on tenant-scoped root entities add mandatory tenant predicates.
- Writes add mandatory tenant predicates.
- Inserts auto-fill the tenant column when missing, accept matching explicit values, and reject mismatched values.
- Internal existence checks for concurrency preserve tenant filtering.

Current limit: automatic tenant filtering applies to the root entity only. Filters for tenant-scoped manually joined entities must be written explicitly until the AST has a stronger alias and per-join metadata design.

## Runtime Audit Provider Design

Runtime audit auto-fill remains deferred because it affects insert, update, Active Record, change tracking, request context, and transactions.

The intended direction is:

- `audit = Audit` remains the compile-time source of columns;
- a future `AuditProvider` supplies runtime values such as `now`, user id, or request values;
- mutation happens in the public `mssql-orm` crate over normalized `Vec<ColumnValue>`;
- `core`, `query`, `sqlserver`, and `tiberius` do not learn request context;
- values are not inferred globally from column names.

Any future implementation must preserve:

- explicit opt-in through `#[orm(audit = Audit)]`;
- no silent overwrite of user-provided values without a clear rule;
- deterministic handling inside transactions;
- compatibility with `Insertable`, `Changeset`, Active Record, and `save_changes()`.

Implementation must be split before code changes. The current backlog tracks it as Etapa 19:

- define the public-crate runtime contract first (`AuditOperation`, request values, context, and precedence rules);
- expose which columns came from `audit = Audit` through an auxiliary runtime contract generated by macros, without changing schema metadata;
- transport the provider through `SharedConnection` and derived `DbContext` helpers;
- apply the provider to insert paths and update paths separately;
- validate with focused unit tests, public `trybuild` coverage, runtime tests, and documentation.

The first implementation step must not auto-fill by matching names such as `created_at` or `updated_by`. Until the auxiliary audit-column contract exists, runtime audit cannot safely know which `ColumnMetadata` entries are audit-owned because `EntityMetadata.columns` intentionally keeps all schema columns flattened.

## Interactions

Policies can coexist, but collisions are rejected:

- entity field vs. audit column;
- entity field vs. soft-delete column;
- entity field vs. tenant column;
- audit vs. soft-delete;
- audit vs. tenant;
- soft-delete vs. tenant.

Runtime behavior is layered:

- audit currently contributes schema only;
- soft delete decides whether delete compiles as `DELETE` or `UPDATE`;
- tenant decides the mandatory security boundary;
- rowversion still controls optimistic concurrency.

## Validation

Coverage includes:

- public `trybuild` fixtures for valid policy usage;
- compile-fail fixtures for invalid audit, soft-delete, and tenant shapes;
- metadata tests for column order, defaults, nullability, insertable/updatable flags, and collisions;
- migration tests proving policy columns enter snapshots, diffs, and DDL as normal columns;
- runtime tests for soft-delete behavior;
- runtime and compiled-SQL tests for tenant filters and insert validation.

## Deferred Work

- Runtime `AuditProvider`.
- `timestamps` as a separate policy or alias.
- Visible Rust fields for generated audit columns.
- Generated entity column symbols for policy-only columns.
- Automatic policy filters over all manually joined entities.
- Predefined global tenant conventions without a user-defined tenant context.

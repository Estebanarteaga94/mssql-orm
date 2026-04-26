# Repository Audit

This audit records the repository state before writing broader conceptual documentation. It is based on the workspace manifests, public crate exports, implementation modules, tests, examples and existing docs reviewed on 2026-04-26.

The master plan requested as `plan_orm_sqlserver_tiberius_code_first.md` is not present at the repository root. The active plan file is `docs/plan_orm_sqlserver_tiberius_code_first.md`.

## Workspace Crates

The workspace currently contains all target crates defined by the architecture:

- `mssql-orm-core`: core contracts, metadata, SQL Server type mapping, neutral row/value abstractions and shared errors.
- `mssql-orm-macros`: procedural derives for entities, contexts, persistence models and entity policies.
- `mssql-orm-query`: query AST and builder primitives. This crate carries query structure and does not emit SQL strings.
- `mssql-orm-sqlserver`: SQL Server-specific quoting, query compilation and migration DDL generation.
- `mssql-orm-tiberius`: Tiberius connection, execution, row adaptation, transactions, operational options and optional `bb8` pooling.
- `mssql-orm-migrate`: model snapshots, diff operations, migration filesystem helpers and update script assembly.
- `mssql-orm-cli`: command-line tooling for `migration add`, `migration list` and `database update`.
- `mssql-orm`: public facade crate that reexports the consumer API and selected advanced internals.

The separation still matches the intended boundaries: `core` does not depend on Tiberius, `query` does not generate SQL, SQL emission belongs to `sqlserver`, execution belongs to `tiberius`, and the normal consumer entry point is the root crate.

## Public API Verified In Code

The root crate exposes the normal consumer surface through `mssql_orm::prelude::*`.

Verified public derives:

- `Entity`
- `DbContext`
- `Insertable`
- `Changeset`
- `AuditFields`
- `SoftDeleteFields`
- `TenantContext`

Verified model and metadata contracts:

- `Entity`, `EntityMetadata`, `ColumnMetadata`, `EntityColumn`
- `PrimaryKeyMetadata`, `IdentityMetadata`, `IndexMetadata`, `ForeignKeyMetadata`
- `EntityPolicy`, `EntityPolicyMetadata`
- `SqlServerType`, `SqlTypeMapping`, `SqlValue`, `ColumnValue`
- `Row`, `FromRow`, `Insertable`, `Changeset`, `OrmError`

Verified runtime/query API in the root crate:

- `DbContext`, `DbSet<T>`, `DbSetQuery<T>`, `DbContextEntitySet<T>`, `SharedConnection`
- `connect_shared`, `connect_shared_with_config`, `connect_shared_with_options`
- `DbContext::raw<T>(...)`, `DbContext::raw_exec(...)`
- `DbContext::health_check()`
- `DbContext::transaction(...)`
- `DbContext::save_changes()`
- `DbSet::find`, `insert`, `update`, `delete`, `query`, `query_with`
- `DbSet::find_tracked`, `add_tracked`, `remove_tracked`
- `DbSetQuery::filter`, `order_by`, `limit`, `take`, `paginate`, `count`
- `DbSetQuery::inner_join`, `left_join`, `join`
- `DbSetQuery::select`, `all_as::<T>()`, `first_as::<T>()`
- `DbSetQuery::with_deleted`, `only_deleted`
- `ActiveRecord`, `EntityPersist`, `EntityPrimaryKey`
- `MigrationModelSource`, `model_snapshot_from_source`, `model_snapshot_json_from_source`
- `SoftDeleteProvider`, `SoftDeleteContext`, `SoftDeleteRequestValues`
- `TenantScopedEntity`, `ActiveTenant`

Verified advanced module exports:

- `mssql_orm::core`
- `mssql_orm::query`
- `mssql_orm::sqlserver`
- `mssql_orm::tiberius`
- `mssql_orm::migrate`
- `mssql_orm::macros`

## Implemented Features

The following features are implemented in code and have tests or implementation modules backing them:

- Code-first entity metadata via `#[derive(Entity)]`, including table/schema names, primary keys, identity columns, nullability, length, defaults, explicit SQL type hints, precision/scale, indexes, unique indexes, computed columns, rowversion and rename hints.
- Generated static column symbols such as `User::email` for typed predicates, ordering and projections.
- Generated `FromRow` for entities.
- Generated `Insertable` and `Changeset` models.
- `DbContext` and typed `DbSet<T>` access from the public crate.
- CRUD over simple primary keys: `find`, `insert`, `update`, `delete`.
- Query builder over an AST with filters, logical predicate composition, ordering, pagination, limits, joins and counts.
- SQL Server compiler for select, insert, update, delete, count, joins, projection aliases, parameters and migration operations.
- Tiberius execution adapter with connection-string parsing, parameter binding, row mapping, health checks, transactions, timeouts, tracing, slow-query options, retry options and optional pooling.
- Raw SQL typed queries and commands through `raw<T>()` and `raw_exec()`.
- Typed projections through `select(...)`, `all_as::<T>()` and `first_as::<T>()`.
- Active Record convenience methods built over `DbSet`.
- Experimental change tracking with `Tracked<T>` and `save_changes()`.
- Optimistic concurrency conflict reporting through `OrmError::ConcurrencyConflict` for rowversion-aware routes.
- Entity policies for audit metadata/schema, soft-delete runtime behavior and opt-in tenant filtering.
- Migration snapshots, diff operations, SQL Server DDL generation, scaffold filesystem helpers and CLI commands.
- `examples/todo-app` as a real external example crate with domain, context, HTTP wiring, snapshot exporter and migration artifacts.

## Incomplete Or Explicitly Limited Features

These items exist only with explicit limits or partial scope:

- Public CRUD, Active Record and tracking routes are oriented to simple primary keys. Composite primary keys exist in metadata but are not a complete public persistence workflow.
- `save_changes()` and `Tracked<T>` are experimental and snapshot based, not a full EF-style unit of work.
- `db.transaction(...)` is blocked for contexts created from a pool until a physical connection can be pinned for the whole closure.
- `raw<T>()` and `raw_exec()` do not apply ORM filters for `tenant` or `soft_delete`. The caller must write those predicates manually.
- Soft-delete automatic read filters apply to the root entity of `DbSetQuery<E>`, not to every manually joined entity.
- Query aliases for multiple references to the same table are not supported; SQL Server join compilation requires unique joined tables until alias support exists.
- High-level aggregate projection APIs are not implemented. Low-level expression functions exist, but typed aggregate ergonomics are not a public feature.
- Audit policy columns are metadata/schema only. They are not visible entity fields, do not generate column symbols and are not automatically filled during insert/update.
- `AuditProvider` runtime behavior is designed conceptually but not implemented.
- Migration rollback generation is available only when operation payloads are reversible. Some destructive operations still require manual `down.sql`.
- `migration.rs` is explicitly deferred from the migration artifact MVP.
- Multi-database support is intentionally out of scope.

## Planned-Only Or Deferred Features

The following should not be documented as available behavior:

- Runtime `AuditProvider` autoloading of `created_at`, `created_by`, `updated_at` or `updated_by`.
- Field-level access or generated symbols for policy-contributed audit columns such as `Todo::created_at`.
- Navigation properties and inferred joins.
- Automatic table aliases for self-joins or repeated table joins.
- High-level typed aggregate builder APIs.
- Complete composite-primary-key persistence across all public CRUD, Active Record and tracking paths.
- A Rust `migration.rs` migration API parallel to the current SQL/snapshot artifact flow.
- Database backends other than SQL Server.

## Documentation Implications

`docs/core-concepts.md` should describe the real implemented flow:

`Entity -> EntityMetadata -> Query AST -> SQL Server SQL -> Tiberius execution -> Row -> Entity or DTO`

It should avoid presenting planned-only behavior as shipped. Claims about audit autoloading, navigation properties, inferred joins, multi-database abstractions, table alias support, aggregate builders and composite primary key CRUD should be marked as unavailable or pending verification unless the implementation changes first.

The public README can safely link to this audit and to `docs/core-concepts.md` once created, but should not duplicate the full inventory.

## Verification Commands Used

- `rg --files`
- `sed -n ... docs/instructions.md docs/tasks.md docs/worklog.md docs/context.md docs/plan_orm_sqlserver_tiberius_code_first.md`
- `sed -n ... Cargo.toml crates/*/Cargo.toml`
- `rg -n "^pub (use|trait|struct|enum|fn|mod)|^pub\\([^)]+\\)" crates/...`
- `rg -n "TODO|todo!|unimplemented!|placeholder|Pending verification|deferred|planned|future|not supported|unsupported" README.md docs crates examples --glob '!target'`
- `rg -n "raw\\(|raw_exec|all_as|first_as|transaction\\(|save_changes|with_tenant|soft_delete|health_check|from_pool|MigrationModelSource|model_snapshot" crates/mssql-orm/src crates/mssql-orm/tests docs README.md`

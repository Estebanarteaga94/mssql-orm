# mssql-orm

`mssql-orm` is a Rust code-first ORM for Microsoft SQL Server. It uses Tiberius as the low-level driver and keeps the public application API centered on `DbContext`, `DbSet<T>`, derives, a typed query builder, migrations, typed raw SQL, and typed projections.

The verified API and implementation inventory lives in [docs/repository-audit.md](docs/repository-audit.md). The conceptual guide starts at [docs/core-concepts.md](docs/core-concepts.md).

## Current Shape

The workspace is split by responsibility:

- `mssql-orm-core`: contracts, metadata, shared types, and errors.
- `mssql-orm-macros`: derives and metadata generation.
- `mssql-orm-query`: query AST and builder types, without SQL generation.
- `mssql-orm-sqlserver`: SQL Server query and DDL compilation.
- `mssql-orm-tiberius`: execution, connections, rows, transactions, and Tiberius adaptation.
- `mssql-orm-migrate`: snapshots, diffs, operations, and migration filesystem helpers.
- `mssql-orm-cli`: migration and database-update commands.
- `mssql-orm`: public crate and normal user-facing API.

SQL Server is the only supported database target in this phase.

## Minimal Example

```rust
use mssql_orm::prelude::*;

#[derive(Entity, Debug, Clone)]
#[orm(table = "users", schema = "dbo")]
pub struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    pub id: i64,

    #[orm(length = 180)]
    #[orm(unique)]
    pub email: String,

    #[orm(length = 120)]
    pub name: String,
}

#[derive(Insertable)]
#[orm(entity = User)]
pub struct NewUser {
    pub email: String,
    pub name: String,
}

#[derive(DbContext)]
pub struct AppDb {
    pub users: DbSet<User>,
}
```

```rust
# use mssql_orm::prelude::*;
# #[derive(Entity, Debug, Clone)]
# #[orm(table = "users", schema = "dbo")]
# pub struct User {
#     #[orm(primary_key)]
#     #[orm(identity)]
#     pub id: i64,
#     #[orm(length = 180)]
#     pub email: String,
#     #[orm(length = 120)]
#     pub name: String,
# }
# #[derive(Insertable)]
# #[orm(entity = User)]
# pub struct NewUser {
#     pub email: String,
#     pub name: String,
# }
# #[derive(DbContext)]
# pub struct AppDb {
#     pub users: DbSet<User>,
# }
# async fn run(connection_string: &str) -> Result<(), OrmError> {
let db = AppDb::connect(connection_string).await?;

let saved = db
    .users
    .insert(NewUser {
        email: "ana@example.com".to_string(),
        name: "Ana".to_string(),
    })
    .await?;

let active_users = db
    .users
    .query()
    .filter(User::email.contains("@example.com"))
    .order_by(User::email.asc())
    .take(20)
    .all()
    .await?;
# let _ = (saved, active_users);
# Ok(())
# }
```

## Documentation Map

- [Core concepts](docs/core-concepts.md): mental model and end-to-end flow.
- [Quickstart](docs/quickstart.md): connection, CRUD, and query builder basics.
- [Code-first](docs/code-first.md): entities, derives, `DbContext`, `DbSet`, and model metadata.
- [Public API](docs/api.md): exported surface from the root crate and prelude.
- [Query builder](docs/query-builder.md): predicates, ordering, pagination, joins, and projections.
- [Typed projections](docs/projections.md): `select(...)`, `all_as::<T>()`, `first_as::<T>()`, aliases, and DTOs.
- [Typed raw SQL](docs/raw-sql.md): `raw<T>()`, `raw_exec()`, parameters, DTOs, and safety rules.
- [Relationships](docs/relationships.md): foreign keys and explicit joins.
- [Transactions](docs/transactions.md): runtime transaction behavior and pool limits.
- [Migrations](docs/migrations.md): snapshots, diff, `migration add`, and `database update`.
- [Entity Policies](docs/entity-policies.md): audit metadata, soft delete, tenant scoping, and deferred runtime audit provider design.
- [Use without manual download](docs/use-without-downloading.md): Git dependency usage from another project.

## Examples

- [examples/README.md](examples/README.md)
- [examples/todo-app/README.md](examples/todo-app/README.md)

Pending verification: historical validation of `todo-app` against real SQL Server is recorded in [docs/worklog.md](docs/worklog.md), but it should be rerun with a real connection string in the current environment before using it as fresh evidence.

## Current Limits

- SQL Server only.
- No navigation properties, lazy loading, or automatic eager loading.
- Public CRUD, Active Record, and tracking are still focused on simple primary keys.
- `AuditProvider` has a public runtime contract and audit-owned column metadata, but it is not wired into contexts or insert/update paths yet.
- `raw<T>()` and `raw_exec()` do not automatically apply ORM `tenant` or `soft_delete` filters.
- `migration.rs` is deferred; the migration MVP uses `up.sql`, `down.sql`, and `model_snapshot.json`.
- `db.transaction(...)` is blocked on contexts created from pools until pooled transactions pin one physical connection for the full closure.

## Local Validation

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features
```

Real SQL Server tests require `MSSQL_ORM_TEST_CONNECTION_STRING`.

## Project Documents

- [CONTRIBUTING.md](CONTRIBUTING.md)
- [SECURITY.md](SECURITY.md)
- [LICENSE](LICENSE)

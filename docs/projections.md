# Typed Projections

Typed projections are the Stage 18 public query-builder feature. They let a query select specific columns or expressions from SQL Server and materialize them into DTOs that implement `FromRow`, without breaking the existing full-entity materialization path through `all()` and `first()`.

See also [Core concepts](core-concepts.md).

## Public Surface

The projection surface is available on `DbSetQuery<E>`:

- `select(...)`
- `all_as::<T>()`
- `first_as::<T>()`

`all()` and `first()` still mean “materialize the full entity `E`”. Partial selections must use the `_as` methods.

## Basic Example

```rust
use mssql_orm::prelude::*;

#[derive(Debug, PartialEq)]
struct UserSummary {
    id: i64,
    email: String,
}

impl FromRow for UserSummary {
    fn from_row<R: Row>(row: &R) -> Result<Self, OrmError> {
        Ok(Self {
            id: row.get_required_typed("id")?,
            email: row.get_required_typed("email")?,
        })
    }
}

let users = db
    .users
    .query()
    .select((User::id, User::email))
    .all_as::<UserSummary>()
    .await?;
```

## AST Shape

The AST stores projections as `SelectProjection { expr, alias }`.

Rules:

- an empty projection keeps full-entity semantics and compiles as `SELECT *`;
- generated `EntityColumn<E>` projections receive a default alias equal to `column_name`;
- expression projections require an explicit alias;
- aliases must be stable, non-empty, and unique.

## SQL Compilation

`mssql-orm-sqlserver` renders projected values with explicit aliases:

```sql
SELECT [dbo].[users].[id] AS [id], [dbo].[users].[email] AS [email]
FROM [dbo].[users]
```

The alias is part of the contract with `FromRow`: the DTO reads `"id"` and `"email"` rather than relying on driver-specific expression names.

## Expressions

Expressions need explicit aliases:

```rust
use mssql_orm::query::SelectProjection;

let rows = db
    .users
    .query()
    .select((
        User::id,
        SelectProjection::expr_as(User::email.lower(), "email_lower"),
    ))
    .all_as::<UserEmailProjection>()
    .await?;
```

## Joins and Aliases

Initial projections can select columns from explicitly joined tables when the query does not require table aliases.

Current limits:

- no self-joins;
- no repeated table in one query;
- if two projected columns share the same `column_name`, assign an explicit alias to one of them.

This avoids ambiguous DTOs for common names such as `id`, `created_at`, or `name`.

## SQL Projections vs. In-Memory `map`

This is an in-memory transformation:

```rust
let summaries = db
    .users
    .query()
    .all()
    .await?
    .into_iter()
    .map(|user| UserSummary {
        id: user.id,
        email: user.email,
    })
    .collect::<Vec<_>>();
```

It is valid when the business flow needs full entities, but it is not a SQL projection. SQL Server still returns all selected entity columns.

Use SQL projections when you want to reduce row width, avoid materializing unused fields, or map directly into read DTOs.

## Runtime Filters

Projections reuse the effective `DbSetQuery` path. Mandatory tenant filters and soft-delete visibility for the root entity still apply before SQL compilation and execution.

Raw SQL remains different: `raw<T>()` does not apply ORM runtime filters automatically.

## Not in This Cut

- High-level typed aggregation DSL.
- Automatic table aliases.
- Self-join support.
- Navigation-property projection.
- Automatic DTO derivation.

## Validation

Coverage lives in:

- `crates/mssql-orm/tests/stage18_public_projections.rs`
- `crates/mssql-orm/tests/ui/query_projection_public_valid.rs`
- SQL compiler snapshot tests in `crates/mssql-orm-sqlserver`

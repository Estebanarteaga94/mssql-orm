# Query Builder

The public query builder does not build SQL directly from the root crate. It produces a `mssql-orm-query` AST. SQL Server parameterized SQL is compiled by `mssql-orm-sqlserver`, and execution happens in the Tiberius adapter.

See also [Core concepts](core-concepts.md).

## Entry Point

The normal entry point is `DbSet<T>::query()` from a derived `DbContext`.

```rust
let users = db
    .users
    .query()
    .filter(User::email.contains("@company.com"))
    .order_by(User::email.asc())
    .take(20)
    .all()
    .await?;
```

## Column Symbols

`#[derive(Entity)]` generates column symbols such as `User::email` and `User::active`. These are typed references to model columns, not reads of Rust field values.

## Predicates

Public column predicates include:

- `eq`, `ne`
- `gt`, `gte`, `lt`, `lte`
- `is_null`, `is_not_null`
- `contains`, `starts_with`, `ends_with` for strings

Predicates can be composed with `and`, `or`, and `not`.

```rust
let predicate = User::email
    .contains("@company.com")
    .and(User::active.eq(true));
```

Values compile to SQL Server parameters (`@P1`, `@P2`, ...), not string interpolation.

## Ordering

Use `asc()` or `desc()` on generated columns:

```rust
db.users
    .query()
    .order_by(User::created_at.desc())
    .all()
    .await?;
```

Ordering is preserved in the AST and then in compiled SQL.

## Pagination and Limits

Use `take(...)` or `limit(...)` for simple row limits:

```rust
db.users.query().take(10).all().await?;
```

Use `PageRequest::new(page, page_size)` for explicit pages. Pages are 1-based:

```rust
db.users
    .query()
    .order_by(User::id.asc())
    .paginate(PageRequest::new(1, 25))
    .all()
    .await?;
```

SQL Server pagination requires deterministic ordering.

## Joins

Joins are explicit. There are no navigation properties or automatic eager loading.

```rust
let rows = db
    .orders
    .query()
    .inner_join::<Customer>(Order::customer_id.eq(Customer::id))
    .filter(Customer::email.contains("@company.com"))
    .all()
    .await?;
```

The public API exposes `inner_join::<T>(...)` and `left_join::<T>(...)`.

The current AST does not support table aliases, so avoid self-joins or repeating the same table in one query.

## Count

`count()` preserves the base `from` and filters. In the current state it does not carry joins, ordering, or pagination into the internal `CountQuery`; use it for base-entity counts with filters that do not depend on joined tables.

## Projections

The public API supports two separate materialization paths:

- `all()` and `first()` materialize full entities.
- `select(...).all_as::<T>()` and `select(...).first_as::<T>()` materialize DTOs with `FromRow`.

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

Projected columns receive default aliases equal to their column names. Expressions require explicit aliases. Empty or duplicate aliases are rejected before execution.
Projection DTOs can use `#[derive(FromRow)]`; fields read aliases by field name unless overridden with `#[orm(column = "...")]`.

## Runtime Filters

`DbSetQuery<T>` does not publicly expose its internal `SelectQuery`. The effective query can add mandatory runtime filters before compilation or execution, such as soft-delete visibility and tenant security filters.

## Limits

- The public query builder does not accept arbitrary SQL fragments.
- Table aliases in joins are not supported.
- Navigation properties and automatic relationship loading are not supported.
- Initial public projections exist, but high-level typed aggregations are not available.

## Related

- Projections: [docs/projections.md](projections.md)
- Raw SQL escape hatch: [docs/raw-sql.md](raw-sql.md)
- Real example queries: [examples/todo-app/src/queries.rs](../examples/todo-app/src/queries.rs)

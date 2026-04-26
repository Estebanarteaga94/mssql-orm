# Relationships and Joins

In `mssql-orm`, a relationship declared with `foreign_key` produces relational metadata, migration snapshots, diffs, and SQL Server DDL. Queries remain explicit: declaring a foreign key does not add navigation properties or automatic joins.

See also [Core concepts](core-concepts.md).

## Declaring a Foreign Key

A one-to-many relationship is declared on the dependent entity field that stores the local column.

```rust
#[derive(Entity, Debug, Clone)]
#[orm(table = "users", schema = "todo")]
pub struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    pub id: i64,
}

#[derive(Entity, Debug, Clone)]
#[orm(table = "todo_lists", schema = "todo")]
pub struct TodoList {
    #[orm(primary_key)]
    #[orm(identity)]
    pub id: i64,

    #[orm(foreign_key(entity = User, column = id))]
    pub owner_id: i64,
}
```

The structured form is preferred because it points to a Rust entity type and a generated target column symbol. The macro validates at compile time that the referenced column exists.

## Legacy String Syntax

The string syntax remains supported for compatibility:

```rust
#[orm(foreign_key = "users.id")]
pub owner_id: i64,

#[orm(foreign_key = "todo.users.id")]
pub owner_id: i64,
```

With two segments, the referenced schema defaults to `dbo`. With three segments, the first segment is the schema.

## Constraint Names

If no name is declared, the derive generates a stable name from the local table, local column, and referenced table.

Generated names are intended for deterministic metadata and migration output, not as a public naming convention guarantee for all future releases.

## Delete Behavior

The current public surface supports:

- `#[orm(on_delete = "no action")]`
- `#[orm(on_delete = "cascade")]`
- `#[orm(on_delete = "set null")]`

`set null` requires the local column to be nullable. The derive rejects non-nullable `set null` at compile time.

## Metadata Helpers

`ForeignKeyMetadata` and `EntityMetadata` expose helpers for inspecting relationships by name, local column, or referenced table. These helpers are for inspection, migrations, and tooling; they do not execute queries.

## Migrations and DDL

Derived foreign keys enter the code-first pipeline as normal metadata:

```text
EntityMetadata -> ModelSnapshot -> MigrationOperation -> SQL Server DDL
```

Generated DDL uses:

```sql
ALTER TABLE ... ADD CONSTRAINT ... FOREIGN KEY ... REFERENCES ...
```

and preserves `ON DELETE` when applicable.

The public derive syntax declares foreign keys from individual fields. Snapshots, diffs, and DDL already have shapes for composite foreign keys, but automatically deriving them from public attributes is outside this phase.

## Explicit Joins

Foreign keys describe the model. Joins decide how a specific query uses related tables.

```rust
let rows = db
    .todo_lists
    .query()
    .inner_join::<User>(TodoList::owner_id.eq(User::id))
    .filter(User::id.eq(7_i64))
    .all()
    .await?;
```

Use `left_join::<T>(...)` when the relationship can be missing or when you need to preserve rows from the base entity.

## Materialization

The current public `DbSetQuery<T>` materializes entities from the base table (`T`). Joins are used to filter or order through related tables; they do not automatically construct object graphs.

## Limits

- No navigation properties.
- No lazy loading or automatic eager loading.
- No automatic join inference from `ForeignKeyMetadata`.
- No table aliases in the AST; SQL Server rejects repeated table references without aliases.
- No automatic projection of joined entity graphs.
- Tenant and soft-delete automatic filters apply to the root entity only; filters on joined entities must be explicit.

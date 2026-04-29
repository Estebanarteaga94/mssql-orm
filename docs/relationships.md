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

The default public `DbSetQuery<T>` materializes entities from the base table (`T`). Joins are used to filter or order through related tables. A first `include::<T>(...)` cut exists for single navigations and explicitly constructs one related `Navigation<T>`.

## 0.2 Navigation Surface

Navigation properties are being introduced incrementally for `0.2.0`. The implemented cut supports syntax, metadata, table aliases, explicit join inference from navigation metadata, eager loading for one `belongs_to` / `has_one` navigation, join-based `has_many` eager loading, and explicit `has_many` collection loading from materialized roots. Fields can declare navigation attributes, the derive excludes those fields from column metadata, and `EntityMetadata.navigations` exposes neutral relationship metadata. Lazy loading remains design-only and is not implemented.

The relationship kinds are:

- `belongs_to`: the dependent entity stores the foreign key and points to one principal entity.
- `has_one`: the principal entity points to at most one dependent entity.
- `has_many`: the principal entity points to a collection of dependent entities.
- `many_to_many`: initially modeled through an explicit join entity. Direct many-to-many navigation remains a later layer until update semantics are stable.

The supported field shapes are marker wrappers, not persisted columns:

```rust
#[derive(Entity, Debug, Clone)]
#[orm(table = "todo_lists", schema = "todo")]
pub struct TodoList {
    #[orm(primary_key)]
    #[orm(identity)]
    pub id: i64,

    #[orm(foreign_key(entity = User, column = id))]
    pub owner_id: i64,

    #[orm(belongs_to(User, foreign_key = owner_id))]
    pub owner: Navigation<User>,
}

#[derive(Entity, Debug, Clone)]
#[orm(table = "users", schema = "todo")]
pub struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    pub id: i64,

    #[orm(has_many(TodoList, foreign_key = owner_id))]
    pub lists: Collection<TodoList>,
}
```

`Navigation<T>` and `Collection<T>` are public marker/value wrappers. The derive does not turn those fields into `ColumnMetadata`; it only uses them to generate navigation metadata. Materializing an entity without an explicit include/load initializes these wrappers empty.

### Explicit Navigation Joins

`DbSetQuery` can build a SQL join predicate from a declared navigation:

```rust
let lists = db
    .users
    .query()
    .try_left_join_navigation_as::<TodoList>("lists", "lists")?
    .filter(TodoList::id.aliased("lists").gt(0_i64))
    .all()
    .await?;
```

The helper validates that the navigation exists on the root entity and that its
target table matches the joined entity type. It uses `local_columns` and
`target_columns` from `NavigationMetadata` to construct the `ON` predicate.
This is not eager loading: the query still materializes the root entity or an
explicit projection only.

### `include(...)` for Single Navigations

The current eager-loading API is explicit and supports `belongs_to` / `has_one`:

```rust
let lists = db
    .todo_lists
    .query()
    .include::<User>("owner")?
    .all()
    .await?;
```

The implementation uses a left join, projects root columns with their normal
aliases, projects included columns with an internal prefix, materializes the
related row through `FromRow`, and attaches it to the root `Navigation<T>`.
When the joined side is absent, the navigation stays empty.

If the included entity declares `tenant` or `soft_delete`, those filters are
applied inside the include join predicate. This preserves `LEFT JOIN`
semantics: a related row hidden by tenant or soft-delete policy leaves the
navigation empty instead of dropping the root entity. Tenant-scoped included
entities fail closed when the context has no compatible active tenant.

`include_as::<T>("owner", "owner_alias")` is available when the query needs a
specific SQL table alias.

The include query can still be refined before execution with root or aliased
related predicates:

```rust
let lists = db
    .todo_lists
    .query()
    .include_as::<User>("owner", "owner")?
    .filter(User::id.aliased("owner").gt(0_i64))
    .order_by(User::id.aliased("owner").desc())
    .take(20)
    .all()
    .await?;
```

Projection DTOs remain separate from includes; `include` materializes root
entities and attaches a `Navigation<T>`.

### `include_many(...)` for Collection Navigations

`has_many` collection eager loading is exposed separately:

```rust
let users = db
    .users
    .query()
    .include_many_as::<TodoList>("lists", "lists")?
    .max_joined_rows(2_000)
    .all()
    .await?;

let lists = users[0].lists.as_slice();
```

The first implementation uses a left join and groups joined rows by the root
entity primary key before assigning `Collection<T>`. This avoids returning
duplicate root entities to the caller. Pagination is rejected on this path
because limiting joined rows would produce unstable root-entity pages.

The default join strategy has a safety limit of 10,000 joined rows before
grouping. `max_joined_rows(...)` adjusts that limit for a query, and
`unbounded_join()` is available only as an explicit opt-out. `split_query()` is
available on the builder to make the intended large-collection strategy
visible, but it currently returns an error until the two-query loader is
implemented.

For large collections, the preferred implemented direction remains split
queries:

```text
1. Load root rows.
2. Load related rows with one filtered query.
3. Attach related rows to the matching root navigation collection.
```

Split queries keep row duplication predictable and avoid forcing every large
collection include through a wide join. The execution path for split queries
remains a separate backlog item.

### Explicit Loading

The current explicit loading cut supports `has_many` collections from a
materialized root entity:

```rust
let mut user = db.users.find(7_i64).await?.expect("user");

db.users
    .load_collection::<TodoList>(&mut user, "lists")
    .await?;

let lists = user.lists.as_slice();
```

For tracked entities, use the tracked variant:

```rust
let mut user = db.users.find_tracked(7_i64).await?.expect("user");

db.users
    .load_collection_tracked::<TodoList>(&mut user, "lists")
    .await?;
```

The tracked variant attaches the collection without marking the entity as
`Modified`. This first cut supports single-column root primary keys where the
`has_many` local column is that primary key. It is an explicit async call; no
field access performs I/O.

### Planned Lazy Loading

Lazy loading is not a default behavior. If it is added, it must be opt-in and visible in types, through wrappers that are distinct from `Navigation<T>` and `Collection<T>`. Normal entity field access must never perform I/O.

The planned shape is explicit at both the entity type and call site:

```rust
#[derive(Entity, Debug, Clone)]
#[orm(table = "todo_lists", schema = "todo")]
pub struct TodoList {
    #[orm(primary_key)]
    pub id: i64,

    #[orm(foreign_key(entity = User, column = id))]
    pub owner_id: i64,

    #[orm(belongs_to(User, foreign_key = owner_id))]
    pub owner: LazyNavigation<User>,
}

let mut list = db.todo_lists.find(7_i64).await?.expect("list");

let owner = list
    .owner
    .load(&db.todo_lists)
    .await?;
```

The exact type names are still pending implementation, but the contract is:

- `LazyNavigation<T>` / `LazyCollection<T>` are separate wrappers from eager-loaded `Navigation<T>` / `Collection<T>`.
- Loading requires an explicit async method call such as `load(...)`, `load_mut(...)` or `load_collection(...)`.
- The call receives an explicit context-bearing value, such as `&DbSet<E>` or a future entry API. Lazy wrappers do not store an open SQL Server connection by themselves.
- A loaded value is cached inside the wrapper for that entity instance until the caller clears or refreshes it through an explicit method.
- The wrapper exposes state inspection, for example `is_loaded()`, so code can avoid accidental repeated loads.
- Missing single navigations produce an empty loaded state, matching `Navigation<T>`.

This keeps the I/O boundary visible in Rust syntax: `await` appears where the query happens, and ordinary field reads remain memory-only.

#### Why It Is Not The First Executable Cut

Rust async and ownership make transparent lazy loading a poor default for this ORM:

- `async` work cannot run inside a normal `Deref` or field accessor, so implicit loading would require blocking, hidden runtimes, or surprising APIs.
- Storing context or connection handles inside every entity would blur the current architecture where execution stays in `mssql-orm-tiberius` and public entity values remain plain data.
- Entity clones would need clear rules for whether they share lazy state, cached values, and connection capability.
- Long-lived entities holding context references would introduce lifetime constraints that are hard to compose with web handlers, transactions and pools.
- Hidden per-row loads create N+1 query regressions that are hard to see in review and telemetry.

For those reasons, the stable path remains explicit eager loading with
`include(...)` / `include_many(...)`, explicit collection loading with
`load_collection(...)`, explicit joins for query shaping, and raw SQL for fully
manual shapes.

#### Required Guardrails Before Implementation

A lazy-loading implementation must provide all of these guardrails before it can
be considered executable:

- Opt-in field types only; existing `Navigation<T>` and `Collection<T>` must not become lazy by default.
- No query from `Deref`, `as_ref`, `as_slice`, `Debug`, `Clone`, serialization or equality operations.
- Explicit context parameter on every load call, so no entity silently owns a connection.
- Compatibility with `tenant` and default `soft_delete` filters equal to explicit loading.
- Clear behavior inside transactions: a load inside a transaction must use that transaction's context-bearing value, not a separate pooled connection.
- Repeated-load semantics, cache invalidation and refresh behavior documented and tested.
- Diagnostics or API friction for collection lazy loading, because `LazyCollection<T>` is the highest N+1 risk.
- Tests proving that constructing, cloning, reading and formatting lazy wrappers do not execute SQL.

### Policies and Projections

Navigation loading must preserve existing safety behavior:

- `tenant` filters apply to included tenant-scoped entities inside the include `JOIN ... ON` predicate and fail closed when the active tenant is missing or incompatible.
- default `soft_delete` visibility applies to included soft-deleted entities inside the include `JOIN ... ON` predicate; a future API may add an explicit include-time visibility override.
- Raw SQL remains explicit, does not infer navigation filters and does not attach `Navigation<T>` / `Collection<T>`.
- `select(...)`, `all_as::<T>()`, `first_as::<T>()` and DTO projections remain separate from entity graph materialization; include builders intentionally do not expose projection methods.

### Required Infrastructure

Navigation support depends on earlier internal work:

- navigation metadata in `mssql-orm-core`;
- macro validation for navigation fields that are not columns;
- table aliases in `mssql-orm-query`;
- SQL Server alias compilation in `mssql-orm-sqlserver`;
- explicit navigation join inference in `DbSetQuery`;
- materialization that can separate root columns from included-entity columns;
- grouping by root primary key for `has_many` collection includes;
- tests for repeated joins, self-joins, `tenant`, `soft_delete`, and row ordering.

## Limits

- `include::<T>(...)` currently supports one `belongs_to` or `has_one` navigation.
- `include_many::<T>(...)` currently supports one `has_many` navigation, defaults to join loading with a 10,000 joined-row safety limit, exposes explicit `split_query()`, and rejects pagination in the join-based loading path.
- `load_collection::<T>(...)` currently supports `has_many` collection loading for single-column root primary keys.
- No lazy loading.
- No automatic projection of joined entity graphs.
- Tenant and soft-delete automatic filters apply to the root entity and to explicitly included entities; filters on manually joined entities must be explicit.

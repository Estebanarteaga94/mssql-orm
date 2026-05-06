# Tracking Unit Of Work

This document defines the target unit-of-work design for Etapa 21. It is the
implementation guide for replacing the current wrapper-lifetime registry with a
context-owned tracker.

The current implementation still depends on live `Tracked<T>` wrappers. This
document does not claim that the runtime has already been stabilized.

## Implementation Status

As of 2026-05-06, the first registry slice is implemented:

- loaded tracked entities are registered with identity derived from entity
  type, schema, table and single-column primary key value,
- duplicate loaded identities in one context are rejected with `OrmError`,
- added entities use temporary local identities,
- successful tracked inserts update the registry identity to the persisted
  primary key returned by SQL Server,
- `DbSet::detach_tracked(...)` removes one wrapper from the current tracker,
- `DbContext::clear_tracker()` removes all current tracker entries,
- `save_changes()` skips `Modified` entries when their original/current
  snapshots produce the same `EntityPersist::update_changes()` payload,
- `save_changes()` plans tracked operations deterministically from context
  entity metadata: `Added` and `Modified` run parent tables before child tables
  for simple foreign keys present in the context, and `Deleted` runs the same
  order in reverse so child tables are deleted before parent tables,
- `save_changes()` opens an internal transaction when the shared connection is
  not already inside `db.transaction(...)`, and reuses the outer transaction
  when one is active.

The registry still stores pointers to live `Tracked<T>` wrappers for snapshots
and state. Removing the wrapper-lifetime dependency remains assigned to the
next ownership/state transition tasks.

## Current Detach And State Policy

The current experimental policy is explicit:

- `Unchanged`: `save_changes()` ignores the entry. `detach_tracked(...)`,
  `clear_tracker()` or dropping the wrapper removes it with no SQL.
- `Modified`: `save_changes()` persists through the normal update pipeline.
  `detach_tracked(...)`, `clear_tracker()` or dropping the wrapper discards the
  pending update from the tracker; the wrapper keeps its `Modified` state.
- `Added`: `save_changes()` persists through the normal insert pipeline and
  syncs the registry identity to the persisted key. `remove_tracked(...)`
  cancels the pending insert by marking the wrapper `Deleted` and detaching it.
  `detach_tracked(...)`, `clear_tracker()` or dropping the wrapper discards the
  pending insert without SQL.
- `Deleted`: `save_changes()` persists through the normal delete pipeline,
  using soft-delete when the entity declares that policy, and unregisters the
  entry after success. `detach_tracked(...)`, `clear_tracker()` or dropping the
  wrapper discards the pending delete from the tracker; the wrapper keeps its
  `Deleted` state.

Because the registry is still pointer-backed, dropping a wrapper remains
equivalent to detach in this slice. This behavior is documented for the current
experimental implementation only. The stable target remains registry-owned
snapshots where dropping a handle does not discard pending work.

## Goal

`save_changes()` must persist changes owned by the `DbContext`, not by the
lifetime of individual `Tracked<T>` values.

The stable unit of work must:

- store tracked entries inside the context-owned `TrackingRegistry`,
- identify persisted rows by deterministic entity identity,
- keep pending operations after a `Tracked<T>` wrapper is dropped,
- avoid duplicate tracked rows for the same persisted identity,
- preserve existing `DbSet` insert/update/delete policy pipelines,
- and leave SQL compilation in `mssql-orm-sqlserver` and execution in
  `mssql-orm-tiberius`.

## Current Baseline

Today, `TrackingRegistry` stores raw addresses of `TrackedInner<T>`.

That has two important limits:

- dropping `Tracked<T>` unregisters the entry and discards pending work,
- the registry cannot own original/current snapshots independently of the
  wrapper.

This is acceptable only while tracking is experimental. Stable tracking must
move ownership to the context registry.

## Ownership Model

The registry becomes the owner of tracked entries.

Target shape:

```rust
pub struct TrackingRegistry {
    state: Mutex<TrackingRegistryState>,
}

struct TrackingRegistryState {
    next_entry_id: u64,
    entries: Vec<TrackedEntry>,
}

struct TrackedEntry {
    entry_id: u64,
    identity: TrackedIdentity,
    original: Box<dyn TrackedSnapshot>,
    current: Box<dyn TrackedSnapshot>,
    state: EntityState,
}
```

`Tracked<T>` becomes a typed handle over one registry entry:

```rust
pub struct Tracked<T> {
    entry_id: u64,
    registry: TrackingRegistryHandle,
    detached_value: Option<T>,
}
```

The wrapper may cache or clone values for ergonomic access, but the registry is
the source of truth for `save_changes()`. Dropping the wrapper must not
unregister the entry.

## Identity Key

Each persisted tracked row uses a deterministic identity:

```rust
struct TrackedIdentity {
    entity_type: TypeId,
    rust_name: &'static str,
    schema: &'static str,
    table: &'static str,
    primary_key: PrimaryKeyIdentity,
}

enum PrimaryKeyIdentity {
    Simple(SqlValue),
}
```

The first stable cut keeps composite primary keys out of scope. Entities with a
composite primary key must fail with a stable error when used with
`find_tracked(...)`, `remove_tracked(...)` or `save_changes()`.

For `Added` entities without a database-generated key yet, the registry uses a
temporary local identity:

```rust
enum PrimaryKeyIdentity {
    Simple(SqlValue),
    Temporary(u64),
}
```

After insert, the entry identity is replaced with the materialized persisted
primary key returned by SQL Server.

## Duplicate Tracking

Stable behavior must reject duplicate persisted identities in one context.

Rules:

- `find_tracked(id)` returns an error if the identity is already tracked and a
  reusable typed handle API is not implemented in the same cut.
- `add_tracked(entity)` uses a temporary identity until insert when the entity
  has an identity/generated key.
- if `add_tracked(entity)` receives an explicit non-default primary key that is
  already tracked, it fails before registering.
- identity comparison uses entity type, schema, table and primary key value.

Returning an existing handle is deferred because it requires a typed borrow API
over heterogeneous registry entries. Rejecting duplicates is simpler and safe.

## State Ownership

State lives in the registry entry. Wrapper methods delegate to the registry.

Stable state transitions:

- `Unchanged -> Modified` through explicit `mark_modified()` or value mutation,
- `Unchanged -> Deleted` through `remove_tracked(...)`,
- `Modified -> Unchanged` through explicit accept/sync after persistence,
- `Modified -> Deleted` through `remove_tracked(...)`,
- `Added -> Unchanged` after successful insert,
- `Added -> Deleted` as local cancellation,
- `Deleted -> detached` after successful delete.

Dropping `Tracked<T>` does not change state. Explicit `detach(...)` removes the
registry entry and makes the handle detached.

## Snapshot Contract

The registry needs typed snapshots without runtime reflection.

The implementation should introduce a root-crate trait generated or implemented
for entities:

```rust
pub trait TrackedEntitySnapshot: Entity + Clone + Send + 'static {
    fn persisted_identity(&self) -> Result<Option<PrimaryKeyIdentity>, OrmError>;
    fn current_snapshot(&self) -> Self;
    fn has_persisted_changes(original: &Self, current: &Self) -> bool;
}
```

The first runtime slice may conservatively keep mutable access as `Modified`.
Before removing the experimental label, `has_persisted_changes(...)` must skip
updates when persisted columns did not change, ignoring navigation wrappers,
identity, computed, rowversion and non-updatable columns.

Generated comparison belongs in `mssql-orm-macros` and public traits in
`mssql-orm`. It must not be placed in `mssql-orm-query`,
`mssql-orm-sqlserver` or `mssql-orm-tiberius`.

The current implementation uses `EntityPersist::has_persisted_changes(...)`,
whose default compares `original.update_changes()` with
`current.update_changes()`. That gives structural change detection over the
same generated updatable-column payload used by updates. It therefore ignores
navigation wrappers, primary keys, identity columns, rowversion columns,
computed columns and non-updatable columns, because those values are not part
of `update_changes()`.

## Save Pipeline

`save_changes()` remains generated by `#[derive(DbContext)]`, but it should ask
the shared registry for entries by entity type instead of asking live wrappers.

Per entity type:

1. collect registry entries for the context field entity type,
2. persist `Added` through `DbSet::insert_entity(...)`,
3. persist `Modified` through `DbSet::update_entity_by_sql_value(...)`,
4. persist `Deleted` through `DbSet::delete_tracked_by_sql_value(...)`,
5. sync successful entries back into the registry,
6. detach entries deleted successfully.

The current implementation keeps the phase order `Added -> Modified ->
Deleted`, but no longer relies on raw context field order inside each phase.
`#[derive(DbContext)]` asks `mssql-orm` for a metadata-based operation plan.
For simple foreign keys between entities present in the same context, inserts
and updates run parent tables before child tables and deletes run child tables
before parent tables. Ties are resolved by the original context field order.
Foreign-key cycles are rejected with `OrmError`. Composite foreign keys and
self-references remain outside this ordering guarantee in the current slice.

## Transaction Boundary

The unit of work must be compatible with both direct connections and
transaction contexts.

The transaction slice of Etapa 21 is implemented for direct shared
connections:

- registry state is shared across context clones created by policy helpers and
  `db.transaction(...)`,
- save execution must keep using each `DbSet`'s existing `SharedConnection`,
- no SQL execution is introduced inside `TrackingRegistry`,
- `SharedConnection` tracks active transaction depth in runtime state shared by
  policy-derived connection handles,
- generated `save_changes()` starts `db.transaction(...)` internally when no
  transaction is active,
- generated `save_changes()` executes its persistence body directly when an
  outer `db.transaction(...)` is already active, avoiding nested `BEGIN
  TRANSACTION` calls.

This guarantees atomicity for the current pointer-backed `save_changes()`
execution on direct connections. Contexts backed by pools remain blocked for
transactions until Etapa 22 pins one physical pooled connection for the entire
transaction closure.

## Public API Surface

The implementation tasks following this design should add explicit APIs before
stabilization:

- `Tracked<T>::state()`,
- `Tracked<T>::mark_modified()`,
- `Tracked<T>::mark_unchanged()`,
- `DbSet::remove_tracked(&mut Tracked<T>)`,
- `DbSet::detach(&mut Tracked<T>)`,
- `DbContext::clear_tracker()`,
- `DbContext::tracked_entries()` or a read-only equivalent for diagnostics.

These APIs must be exposed from `mssql_orm::prelude` only after they have tests
and rustdoc. Until then, tracking remains experimental.

## Migration Steps

Implementation should be split in this order:

1. introduce owned registry entry identifiers and public diagnostics without
   changing persistence behavior,
2. move `Tracked<T>` state reads/writes through registry-owned entries,
3. stop unregistering on `Drop`,
4. add explicit detach/clear APIs,
5. add duplicate identity detection for simple primary keys,
6. update `save_changes()` helpers to iterate owned registry snapshots,
7. add no-op change detection,
8. add deterministic FK-aware operation ordering,
9. finalize transaction behavior and public docs.

Each step must keep `core`, `query`, `sqlserver`, `tiberius`, `migrate` and
`cli` responsibilities unchanged.

## Out Of Scope

This design deliberately excludes:

- composite primary key persistence in the first stable cut,
- automatic lazy loading,
- graph-wide cascade persistence,
- direct many-to-many mutation persistence,
- SQL generation inside tracking,
- and Tiberius-specific state in the registry.

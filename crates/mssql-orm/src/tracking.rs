//! Experimental change tracking surface.
//!
//! This module intentionally defines only the minimal public contracts for the
//! future tracking pipeline. In this stage it does not:
//! - replace the explicit `DbSet`/`ActiveRecord` APIs
//! - infer inserts, updates or deletes globally outside of `Tracked<T>`
//! - diff entity fields structurally before deciding whether an entity changed
//! - keep dropped wrappers in the unit of work
//! - support composite primary keys through `save_changes()`
//!
//! Current experimental entry points:
//! - `DbSet::find_tracked(id)` for existing entities with single-column PK
//! - `DbSet::add_tracked(entity)` for new entities pending insertion
//! - `DbSet::remove_tracked(&mut tracked)` for explicit tracked deletion
//! - `DbContext::save_changes()` for explicit persistence of live wrappers
//!
//! Observable limits in the current stage:
//! - only wrappers still alive participate in `save_changes()`
//! - mutable access marks `Unchanged` entities as `Modified` immediately
//! - removing a tracked `Added` entity cancels the pending insert locally
//! - successful tracked deletes unregister the wrapper from the internal registry
//! - rowversion conflicts are still surfaced as `OrmError::ConcurrencyConflict`

use core::ops::{Deref, DerefMut};
use mssql_orm_core::Entity;
use std::any::TypeId;
use std::marker::PhantomData;
use std::sync::{Arc, Mutex};

/// Lifecycle state for an experimentally tracked entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EntityState {
    Unchanged,
    Added,
    Modified,
    Deleted,
}

/// Snapshot-based wrapper for entities tracked experimentally.
///
/// `Tracked<T>` keeps the original snapshot together with the current value so
/// later stages can compare and persist changes without relying on runtime
/// proxies or reflection.
pub struct Tracked<T> {
    inner: Box<TrackedInner<T>>,
    registration_id: Option<usize>,
    tracking_registry: Option<TrackingRegistryHandle>,
}

#[doc(hidden)]
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrackedEntityRegistration {
    pub entity_rust_name: &'static str,
    pub state: EntityState,
}

#[doc(hidden)]
#[derive(Debug, Default)]
pub struct TrackingRegistry {
    state: Mutex<TrackingRegistryState>,
}

#[doc(hidden)]
pub type TrackingRegistryHandle = Arc<TrackingRegistry>;

struct TrackedInner<T> {
    original: T,
    current: T,
    state: EntityState,
}

#[derive(Debug, Default)]
struct TrackingRegistryState {
    next_registration_id: usize,
    entries: Vec<TrackingRegistration>,
}

#[derive(Debug)]
struct TrackingRegistration {
    registration_id: usize,
    entity_type_id: TypeId,
    entity_rust_name: &'static str,
    inner_address: usize,
    state_reader: unsafe fn(*const ()) -> EntityState,
}

#[derive(Clone, Copy)]
pub(crate) struct RegisteredTracked<E> {
    registration_id: usize,
    inner_address: usize,
    _entity: PhantomData<fn() -> E>,
}

impl<T: Clone> Tracked<T> {
    /// Creates a tracked value loaded from persistence.
    pub fn from_loaded(entity: T) -> Self {
        Self {
            inner: Box::new(TrackedInner {
                original: entity.clone(),
                current: entity,
                state: EntityState::Unchanged,
            }),
            registration_id: None,
            tracking_registry: None,
        }
    }

    /// Creates a tracked value that represents a new entity pending insertion.
    pub fn from_added(entity: T) -> Self {
        Self {
            inner: Box::new(TrackedInner {
                original: entity.clone(),
                current: entity,
                state: EntityState::Added,
            }),
            registration_id: None,
            tracking_registry: None,
        }
    }
}

impl<T> Tracked<T> {
    /// Returns the original snapshot captured when tracking started.
    pub fn original(&self) -> &T {
        &self.inner.original
    }

    /// Returns the current in-memory value.
    pub fn current(&self) -> &T {
        &self.inner.current
    }

    /// Returns the current tracking state.
    pub const fn state(&self) -> EntityState {
        self.inner.state
    }

    /// Returns mutable access to the current value and marks the entity as
    /// modified when it was previously loaded as unchanged.
    pub fn current_mut(&mut self) -> &mut T {
        self.mark_modified_if_unchanged();
        &mut self.inner.current
    }

    fn mark_modified_if_unchanged(&mut self) {
        if self.inner.state == EntityState::Unchanged {
            self.inner.state = EntityState::Modified;
        }
    }

    pub(crate) fn mark_deleted(&mut self) {
        self.inner.state = EntityState::Deleted;
    }

    pub(crate) fn detach_registry(&mut self) {
        if let (Some(registration_id), Some(registry)) =
            (self.registration_id.take(), self.tracking_registry.take())
        {
            registry.unregister(registration_id);
        }
    }
}

impl<T: Clone> Tracked<T> {
    /// Consumes the tracked wrapper and returns the current entity value.
    pub fn into_current(self) -> T {
        self.current().clone()
    }
}

impl<T: Entity> Tracked<T> {
    pub(crate) fn attach_registry(&mut self, registry: TrackingRegistryHandle) {
        let registration_id = registry.register(self);
        self.registration_id = Some(registration_id);
        self.tracking_registry = Some(registry);
    }
}

impl<T> Deref for Tracked<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.current()
    }
}

impl<T> DerefMut for Tracked<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.current_mut()
    }
}

impl TrackingRegistry {
    pub(crate) fn register<E: Entity>(&self, tracked: &Tracked<E>) -> usize {
        let mut state = self.state.lock().expect("tracking registry mutex poisoned");
        let registration_id = state.next_registration_id;
        state.next_registration_id += 1;
        state.entries.push(TrackingRegistration {
            registration_id,
            entity_type_id: TypeId::of::<E>(),
            entity_rust_name: E::metadata().rust_name,
            inner_address: tracked.inner.as_ref() as *const TrackedInner<E> as usize,
            state_reader: state_reader::<E>,
        });
        registration_id
    }

    pub(crate) fn unregister(&self, registration_id: usize) {
        let mut state = self.state.lock().expect("tracking registry mutex poisoned");
        state
            .entries
            .retain(|entry| entry.registration_id != registration_id);
    }

    pub(crate) fn tracked_for<E: Entity>(&self) -> Vec<RegisteredTracked<E>> {
        let state = self.state.lock().expect("tracking registry mutex poisoned");

        state
            .entries
            .iter()
            .filter(|entry| entry.entity_type_id == TypeId::of::<E>())
            .map(|entry| RegisteredTracked::<E> {
                registration_id: entry.registration_id,
                inner_address: entry.inner_address,
                _entity: PhantomData,
            })
            .collect()
    }

    pub fn entry_count(&self) -> usize {
        self.state
            .lock()
            .expect("tracking registry mutex poisoned")
            .entries
            .len()
    }

    pub fn registrations(&self) -> Vec<TrackedEntityRegistration> {
        self.state
            .lock()
            .expect("tracking registry mutex poisoned")
            .entries
            .iter()
            .map(|entry| TrackedEntityRegistration {
                entity_rust_name: entry.entity_rust_name,
                state: unsafe { (entry.state_reader)(entry.inner_address as *const ()) },
            })
            .collect()
    }
}

impl<E: Clone> RegisteredTracked<E> {
    pub(crate) fn registration_id(&self) -> usize {
        self.registration_id
    }

    pub(crate) fn state(&self) -> EntityState {
        unsafe { (&*(self.inner_address as *const TrackedInner<E>)).state }
    }

    pub(crate) fn current_clone(&self) -> E {
        unsafe {
            (&*(self.inner_address as *const TrackedInner<E>))
                .current
                .clone()
        }
    }

    pub(crate) fn sync_persisted(&self, persisted: E) {
        unsafe {
            let inner = self.inner_address as *mut TrackedInner<E>;
            (*inner).original = persisted.clone();
            (*inner).current = persisted;
            (*inner).state = EntityState::Unchanged;
        }
    }
}

unsafe fn state_reader<E>(ptr: *const ()) -> EntityState {
    unsafe { (&*(ptr.cast::<TrackedInner<E>>())).state }
}

impl<T: Clone> Clone for Tracked<T> {
    fn clone(&self) -> Self {
        Self {
            inner: Box::new(TrackedInner {
                original: self.original().clone(),
                current: self.current().clone(),
                state: self.state(),
            }),
            registration_id: None,
            tracking_registry: None,
        }
    }
}

impl<T: core::fmt::Debug> core::fmt::Debug for Tracked<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Tracked")
            .field("original", self.original())
            .field("current", self.current())
            .field("state", &self.state())
            .finish()
    }
}

impl<T: PartialEq> PartialEq for Tracked<T> {
    fn eq(&self, other: &Self) -> bool {
        self.original() == other.original()
            && self.current() == other.current()
            && self.state() == other.state()
    }
}

impl<T: Eq> Eq for Tracked<T> {}

impl<T> Drop for Tracked<T> {
    fn drop(&mut self) {
        if let (Some(registration_id), Some(registry)) =
            (self.registration_id.take(), self.tracking_registry.take())
        {
            registry.unregister(registration_id);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EntityState, Tracked, TrackedEntityRegistration, TrackingRegistry};
    use mssql_orm_core::{Entity, EntityMetadata, PrimaryKeyMetadata};
    use std::sync::Arc;

    #[derive(Clone)]
    struct DummyEntity;

    static DUMMY_ENTITY_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "DummyEntity",
        schema: "dbo",
        table: "dummy_entities",
        renamed_from: None,
        columns: &[],
        primary_key: PrimaryKeyMetadata {
            name: None,
            columns: &[],
        },
        indexes: &[],
        foreign_keys: &[],
    };

    impl Entity for DummyEntity {
        fn metadata() -> &'static EntityMetadata {
            &DUMMY_ENTITY_METADATA
        }
    }

    #[test]
    fn tracked_loaded_value_keeps_original_and_current_snapshots() {
        let tracked = Tracked::from_loaded(String::from("Ana"));

        assert_eq!(tracked.state(), EntityState::Unchanged);
        assert_eq!(tracked.original(), "Ana");
        assert_eq!(tracked.current(), "Ana");
    }

    #[test]
    fn tracked_added_value_starts_in_added_state() {
        let tracked = Tracked::from_added(String::from("Luis"));

        assert_eq!(tracked.state(), EntityState::Added);
        assert_eq!(tracked.original(), "Luis");
        assert_eq!(tracked.current(), "Luis");
    }

    #[test]
    fn tracked_can_release_current_value() {
        let tracked = Tracked::from_loaded(String::from("Maria"));

        assert_eq!(tracked.into_current(), "Maria");
    }

    #[test]
    fn mutable_access_transitions_loaded_entity_to_modified() {
        let mut tracked = Tracked::from_loaded(String::from("Ana"));

        tracked.push_str(" Maria");

        assert_eq!(tracked.state(), EntityState::Modified);
        assert_eq!(tracked.original(), "Ana");
        assert_eq!(tracked.current(), "Ana Maria");
    }

    #[test]
    fn current_mut_transitions_loaded_entity_to_modified() {
        let mut tracked = Tracked::from_loaded(String::from("Luis"));

        tracked.current_mut().push_str(" Alberto");

        assert_eq!(tracked.state(), EntityState::Modified);
        assert_eq!(tracked.original(), "Luis");
        assert_eq!(tracked.current(), "Luis Alberto");
    }

    #[test]
    fn mark_deleted_transitions_any_registered_entity_to_deleted() {
        let registry = Arc::new(TrackingRegistry::default());
        let mut tracked = Tracked::from_loaded(DummyEntity);
        tracked.attach_registry(Arc::clone(&registry));

        tracked.mark_deleted();

        assert_eq!(tracked.state(), EntityState::Deleted);
        assert_eq!(registry.registrations()[0].state, EntityState::Deleted);
    }

    #[test]
    fn mutable_access_keeps_added_state_for_new_entities() {
        let mut tracked = Tracked::from_added(String::from("Maria"));

        tracked.push_str(" Fernanda");

        assert_eq!(tracked.state(), EntityState::Added);
        assert_eq!(tracked.original(), "Maria");
        assert_eq!(tracked.current(), "Maria Fernanda");
    }

    #[test]
    fn tracking_registry_records_loaded_entities() {
        let registry = Arc::new(TrackingRegistry::default());
        let mut tracked = Tracked::from_loaded(DummyEntity);

        tracked.attach_registry(Arc::clone(&registry));

        assert_eq!(registry.entry_count(), 1);
        assert_eq!(
            registry.registrations(),
            vec![TrackedEntityRegistration {
                entity_rust_name: "DummyEntity",
                state: EntityState::Unchanged,
            }]
        );
    }

    #[test]
    fn tracking_registry_records_added_entities() {
        let registry = Arc::new(TrackingRegistry::default());
        let mut tracked = Tracked::from_added(DummyEntity);

        tracked.attach_registry(Arc::clone(&registry));

        assert_eq!(registry.entry_count(), 1);
        assert_eq!(
            registry.registrations(),
            vec![TrackedEntityRegistration {
                entity_rust_name: "DummyEntity",
                state: EntityState::Added,
            }]
        );
    }

    #[test]
    fn detach_registry_unregisters_without_dropping_wrapper() {
        let registry = Arc::new(TrackingRegistry::default());
        let mut tracked = Tracked::from_loaded(DummyEntity);
        tracked.attach_registry(Arc::clone(&registry));

        tracked.detach_registry();

        assert_eq!(registry.entry_count(), 0);
        assert_eq!(tracked.state(), EntityState::Unchanged);
    }

    #[test]
    fn dropping_tracked_entity_unregisters_it_from_registry() {
        let registry = Arc::new(TrackingRegistry::default());

        {
            let mut tracked = Tracked::from_loaded(DummyEntity);
            tracked.attach_registry(Arc::clone(&registry));
            assert_eq!(registry.entry_count(), 1);
        }

        assert_eq!(registry.entry_count(), 0);
    }
}

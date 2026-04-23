//! Experimental change tracking surface.
//!
//! This module intentionally defines only the minimal public contracts for the
//! future tracking pipeline. In this stage it does not:
//! - register tracked entities inside a `DbContext`
//! - persist changes through `save_changes()`
//! - detect dirty state automatically from mutable access
//! - replace the explicit `DbSet`/`ActiveRecord` APIs

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
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Tracked<T> {
    original: T,
    current: T,
    state: EntityState,
}

impl<T: Clone> Tracked<T> {
    /// Creates a tracked value loaded from persistence.
    pub fn from_loaded(entity: T) -> Self {
        Self {
            original: entity.clone(),
            current: entity,
            state: EntityState::Unchanged,
        }
    }

    /// Creates a tracked value that represents a new entity pending insertion.
    pub fn from_added(entity: T) -> Self {
        Self {
            original: entity.clone(),
            current: entity,
            state: EntityState::Added,
        }
    }
}

impl<T> Tracked<T> {
    /// Returns the original snapshot captured when tracking started.
    pub fn original(&self) -> &T {
        &self.original
    }

    /// Returns the current in-memory value.
    pub fn current(&self) -> &T {
        &self.current
    }

    /// Returns the current tracking state.
    pub const fn state(&self) -> EntityState {
        self.state
    }

    /// Consumes the tracked wrapper and returns the current entity value.
    pub fn into_current(self) -> T {
        self.current
    }
}

#[cfg(test)]
mod tests {
    use super::{EntityState, Tracked};

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
}

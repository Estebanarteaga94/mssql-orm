//! Migration support foundations.

use mssql_orm_core::CrateIdentity;

/// Placeholder migration engine marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MigrationEngine;

pub const CRATE_IDENTITY: CrateIdentity = CrateIdentity {
    name: "mssql-orm-migrate",
    responsibility: "code-first snapshots, diffs and migration operations",
};

#[cfg(test)]
mod tests {
    use super::{CRATE_IDENTITY, MigrationEngine};

    #[test]
    fn declares_migration_boundary() {
        let engine = MigrationEngine;
        assert_eq!(engine, MigrationEngine);
        assert!(CRATE_IDENTITY.responsibility.contains("migration"));
    }
}

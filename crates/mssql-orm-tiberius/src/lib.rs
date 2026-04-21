//! Tiberius adapter boundary for execution concerns.

use mssql_orm_core::CrateIdentity;

/// Placeholder execution adapter marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TiberiusAdapter;

pub const CRATE_IDENTITY: CrateIdentity = CrateIdentity {
    name: "mssql-orm-tiberius",
    responsibility: "connections, execution, rows and transactions over Tiberius",
};

#[cfg(test)]
mod tests {
    use super::{CRATE_IDENTITY, TiberiusAdapter};

    #[test]
    fn declares_execution_boundary() {
        let adapter = TiberiusAdapter;
        assert_eq!(adapter, TiberiusAdapter);
        assert!(CRATE_IDENTITY.responsibility.contains("transactions"));
    }
}

//! Tiberius adapter boundary for execution concerns.

mod config;
mod connection;
mod executor;
mod parameter;

use mssql_orm_core::CrateIdentity;

/// Placeholder execution adapter marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TiberiusAdapter;

pub use config::MssqlConnectionConfig;
pub use connection::{MssqlConnection, TokioConnectionStream};
pub use executor::{ExecuteResult, Executor};

pub const CRATE_IDENTITY: CrateIdentity = CrateIdentity {
    name: "mssql-orm-tiberius",
    responsibility: "connections, execution, rows and transactions over Tiberius",
};

#[cfg(test)]
mod tests {
    use super::{CRATE_IDENTITY, ExecuteResult, MssqlConnectionConfig, TiberiusAdapter};

    #[test]
    fn declares_execution_boundary() {
        let adapter = TiberiusAdapter;
        assert_eq!(adapter, TiberiusAdapter);
        assert!(CRATE_IDENTITY.responsibility.contains("transactions"));
    }

    #[test]
    fn reexports_connection_config() {
        let config = MssqlConnectionConfig::from_connection_string(
            "server=tcp:localhost,1433;database=master;user=sa;password=Password123;TrustServerCertificate=true",
        )
        .unwrap();

        assert_eq!(config.addr(), "localhost:1433");
    }

    #[test]
    fn reexports_execute_result() {
        let result = ExecuteResult::new(vec![1, 2]);

        assert_eq!(result.total(), 3);
    }
}

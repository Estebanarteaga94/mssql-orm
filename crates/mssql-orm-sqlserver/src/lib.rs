//! SQL Server compilation layer.

use mssql_orm_core::CrateIdentity;

/// Placeholder compiler marker for the SQL Server dialect.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SqlServerCompiler;

pub const CRATE_IDENTITY: CrateIdentity = CrateIdentity {
    name: "mssql-orm-sqlserver",
    responsibility: "AST compilation and SQL Server specific quoting and SQL emission",
};

#[cfg(test)]
mod tests {
    use super::{CRATE_IDENTITY, SqlServerCompiler};

    #[test]
    fn declares_sqlserver_compilation_boundary() {
        let compiler = SqlServerCompiler;
        assert_eq!(compiler, SqlServerCompiler);
        assert!(CRATE_IDENTITY.responsibility.contains("SQL emission"));
    }
}

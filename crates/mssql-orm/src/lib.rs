//! Public API surface for the workspace.

pub use mssql_orm_core as core;
pub use mssql_orm_migrate as migrate;
pub use mssql_orm_query as query;
pub use mssql_orm_sqlserver as sqlserver;
pub use mssql_orm_tiberius as tiberius;

pub mod prelude {
    pub use mssql_orm_core::OrmError;
}

#[cfg(test)]
mod tests {
    use super::prelude::OrmError;

    #[test]
    fn exposes_public_prelude() {
        let error = OrmError::new("public-api");
        assert_eq!(error.message(), "public-api");
    }
}

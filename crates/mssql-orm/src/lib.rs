//! Public API surface for the workspace.

pub use mssql_orm_core as core;
pub use mssql_orm_macros as macros;
pub use mssql_orm_migrate as migrate;
pub use mssql_orm_query as query;
pub use mssql_orm_sqlserver as sqlserver;
pub use mssql_orm_tiberius as tiberius;

pub mod prelude {
    pub use mssql_orm_core::{
        ColumnMetadata, Entity, EntityMetadata, ForeignKeyMetadata, IdentityMetadata,
        IndexColumnMetadata, IndexMetadata, OrmError, PrimaryKeyMetadata, ReferentialAction,
        SqlServerType,
    };
    pub use mssql_orm_macros::Entity;
}

#[cfg(test)]
mod tests {
    use super::prelude::{Entity, EntityMetadata, OrmError, PrimaryKeyMetadata};

    struct PublicEntity;

    static PUBLIC_ENTITY_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "PublicEntity",
        schema: "dbo",
        table: "public_entities",
        columns: &[],
        primary_key: PrimaryKeyMetadata {
            name: None,
            columns: &[],
        },
        indexes: &[],
        foreign_keys: &[],
    };

    impl Entity for PublicEntity {
        fn metadata() -> &'static EntityMetadata {
            &PUBLIC_ENTITY_METADATA
        }
    }

    #[test]
    fn exposes_public_prelude() {
        let error = OrmError::new("public-api");
        assert_eq!(error.message(), "public-api");
    }

    #[test]
    fn exposes_entity_contract_in_prelude() {
        assert_eq!(PublicEntity::metadata().table, "public_entities");
    }
}

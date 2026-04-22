//! Public API surface for the workspace.

extern crate self as mssql_orm;

pub use mssql_orm_core as core;
pub use mssql_orm_macros as macros;
pub use mssql_orm_migrate as migrate;
pub use mssql_orm_query as query;
pub use mssql_orm_sqlserver as sqlserver;
pub use mssql_orm_tiberius as tiberius;

pub mod prelude {
    pub use mssql_orm_core::{
        Changeset, ColumnMetadata, ColumnValue, Entity, EntityColumn, EntityMetadata,
        ForeignKeyMetadata, FromRow, IdentityMetadata, IndexColumnMetadata, IndexMetadata,
        Insertable, OrmError, PrimaryKeyMetadata, ReferentialAction, Row, SqlServerType,
        SqlTypeMapping, SqlValue,
    };
    pub use mssql_orm_macros::Entity;
}

#[cfg(test)]
mod tests {
    use super::prelude::{
        ColumnValue, Entity, EntityColumn, EntityMetadata, IdentityMetadata, OrmError,
        PrimaryKeyMetadata, SqlServerType, SqlTypeMapping, SqlValue,
    };

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
        assert_eq!(
            ColumnValue::new("email", SqlValue::String("ana@example.com".to_string())),
            ColumnValue {
                column_name: "email",
                value: SqlValue::String("ana@example.com".to_string()),
            }
        );
        assert_eq!(String::SQL_SERVER_TYPE, SqlServerType::NVarChar);
    }

    #[test]
    fn exposes_entity_contract_in_prelude() {
        assert_eq!(PublicEntity::metadata().table, "public_entities");
    }

    #[allow(dead_code)]
    #[derive(Entity, Debug, Clone)]
    #[orm(table = "users", schema = "auth")]
    struct DerivedUser {
        #[orm(primary_key)]
        #[orm(identity)]
        id: i64,

        #[orm(length = 180)]
        #[orm(unique)]
        email: String,

        #[orm(nullable)]
        #[orm(index(name = "ix_users_display_name"))]
        display_name: Option<String>,

        #[orm(default_sql = "'system'")]
        created_by: String,

        #[orm(rowversion)]
        version: Vec<u8>,
    }

    #[allow(dead_code)]
    #[derive(Entity, Debug, Clone)]
    struct AuditEntry {
        id: i64,
        payload: String,
    }

    #[test]
    fn derives_entity_metadata_from_struct_attributes() {
        let metadata = DerivedUser::metadata();

        assert_eq!(metadata.rust_name, "DerivedUser");
        assert_eq!(metadata.schema, "auth");
        assert_eq!(metadata.table, "users");
        assert_eq!(metadata.primary_key.columns, &["id"]);
        assert_eq!(metadata.indexes.len(), 2);

        let id = metadata.field("id").expect("id column metadata");
        assert_eq!(id.sql_type, SqlServerType::BigInt);
        assert_eq!(id.identity, Some(IdentityMetadata::new(1, 1)));
        assert!(!id.insertable);
        assert!(!id.updatable);

        let email = metadata.field("email").expect("email column metadata");
        assert_eq!(email.sql_type, SqlServerType::NVarChar);
        assert_eq!(email.max_length, Some(180));
        assert!(!email.nullable);

        let display_name = metadata
            .field("display_name")
            .expect("display_name column metadata");
        assert!(display_name.nullable);
        assert_eq!(display_name.max_length, Some(255));

        let created_by = metadata
            .field("created_by")
            .expect("created_by column metadata");
        assert_eq!(created_by.default_sql, Some("'system'"));

        let version = metadata.field("version").expect("version column metadata");
        assert_eq!(version.sql_type, SqlServerType::RowVersion);
        assert!(version.rowversion);
        assert!(!version.insertable);
        assert!(!version.updatable);

        assert_eq!(metadata.indexes[0].name, "ux_users_email");
        assert!(metadata.indexes[0].unique);
        assert_eq!(metadata.indexes[1].name, "ix_users_display_name");
        assert!(!metadata.indexes[1].unique);
    }

    #[test]
    fn derives_default_table_and_primary_key_convention() {
        let metadata = AuditEntry::metadata();

        assert_eq!(metadata.schema, "dbo");
        assert_eq!(metadata.table, "audit_entries");
        assert_eq!(metadata.primary_key.columns, &["id"]);

        let payload = metadata.field("payload").expect("payload column metadata");
        assert_eq!(payload.sql_type, SqlServerType::NVarChar);
        assert_eq!(payload.max_length, Some(255));
        assert!(payload.insertable);
        assert!(payload.updatable);
    }

    #[test]
    fn exposes_static_columns_for_future_query_builder() {
        let email: EntityColumn<DerivedUser> = DerivedUser::email;
        let version = DerivedUser::version;
        let payload = AuditEntry::payload;

        assert_eq!(email.rust_field(), "email");
        assert_eq!(email.column_name(), "email");
        assert_eq!(email.entity_metadata().table, "users");
        assert_eq!(email.metadata().max_length, Some(180));

        assert_eq!(version.column_name(), "version");
        assert_eq!(version.metadata().sql_type, SqlServerType::RowVersion);
        assert!(!version.metadata().insertable);

        assert_eq!(payload.entity_metadata().table, "audit_entries");
        assert_eq!(payload.metadata().column_name, "payload");
    }
}

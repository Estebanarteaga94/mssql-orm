//! Public API surface for the workspace.

extern crate self as mssql_orm;

mod active_record;
mod context;
mod dbset_query;
mod page_request;
mod predicate_composition;
mod query_order;
mod query_predicates;
mod tracking;

pub use mssql_orm_core as core;
pub use mssql_orm_macros as macros;
pub use mssql_orm_migrate as migrate;
pub use mssql_orm_query as query;
pub use mssql_orm_sqlserver as sqlserver;
pub use mssql_orm_tiberius as tiberius;
pub use tokio;

pub use active_record::{ActiveRecord, EntityPersist, EntityPersistMode, EntityPrimaryKey};
pub use context::{
    DbContext, DbContextEntitySet, DbSet, SharedConnection, connect_shared,
    connect_shared_with_config, connect_shared_with_options,
};
pub use dbset_query::DbSetQuery;
pub use mssql_orm_tiberius::{
    MssqlConnectionConfig, MssqlHealthCheckOptions, MssqlHealthCheckQuery, MssqlOperationalOptions,
    MssqlParameterLogMode, MssqlPoolBackend, MssqlPoolOptions, MssqlRetryOptions,
    MssqlSlowQueryOptions, MssqlTimeoutOptions, MssqlTracingOptions,
};
pub use page_request::PageRequest;
pub use predicate_composition::PredicateCompositionExt;
pub use query_order::EntityColumnOrderExt;
pub use query_predicates::EntityColumnPredicateExt;
pub use tracking::{EntityState, Tracked};
#[doc(hidden)]
pub use tracking::{TrackedEntityRegistration, TrackingRegistry, TrackingRegistryHandle};

pub mod prelude {
    pub use crate::{
        ActiveRecord, DbContext, DbContextEntitySet, DbSet, DbSetQuery, EntityColumnOrderExt,
        EntityColumnPredicateExt, EntityState, MssqlConnectionConfig, MssqlHealthCheckOptions,
        MssqlHealthCheckQuery, MssqlOperationalOptions, MssqlParameterLogMode, MssqlPoolBackend,
        MssqlPoolOptions, MssqlRetryOptions, MssqlSlowQueryOptions, MssqlTimeoutOptions,
        MssqlTracingOptions, PageRequest, PredicateCompositionExt, Tracked,
    };
    pub use mssql_orm_core::{
        Changeset, ColumnMetadata, ColumnValue, Entity, EntityColumn, EntityMetadata,
        ForeignKeyMetadata, FromRow, IdentityMetadata, IndexColumnMetadata, IndexMetadata,
        Insertable, OrmError, PrimaryKeyMetadata, ReferentialAction, Row, SqlServerType,
        SqlTypeMapping, SqlValue,
    };
    pub use mssql_orm_macros::{Changeset, DbContext, Entity, Insertable};
    pub use mssql_orm_query::{Join, JoinType};
}

#[cfg(test)]
mod tests {
    use super::prelude::{
        ActiveRecord, Changeset, ColumnValue, DbContext, DbContextEntitySet, DbSet, Entity,
        EntityColumn, EntityColumnOrderExt, EntityColumnPredicateExt, EntityMetadata, EntityState,
        FromRow, IdentityMetadata, Insertable, MssqlConnectionConfig, MssqlOperationalOptions,
        MssqlPoolBackend, MssqlPoolOptions, MssqlRetryOptions, MssqlTimeoutOptions, OrmError,
        PageRequest, PredicateCompositionExt, PrimaryKeyMetadata, SqlServerType, SqlTypeMapping,
        SqlValue, Tracked,
    };
    use mssql_orm_query::{Expr, OrderBy, Predicate, SortDirection, TableRef};
    use std::time::Duration;

    struct PublicEntity;

    static PUBLIC_ENTITY_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "PublicEntity",
        schema: "dbo",
        table: "public_entities",
        renamed_from: None,
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
        assert_eq!(PageRequest::new(2, 25).page, 2);
    }

    #[test]
    fn exposes_entity_contract_in_prelude() {
        assert_eq!(PublicEntity::metadata().table, "public_entities");
    }

    #[test]
    fn exposes_operational_configuration_surface() {
        let options = MssqlOperationalOptions::new()
            .with_timeouts(MssqlTimeoutOptions::new().with_query_timeout(Duration::from_secs(30)))
            .with_retry(MssqlRetryOptions::enabled(
                2,
                Duration::from_millis(50),
                Duration::from_secs(1),
            ))
            .with_pool(MssqlPoolOptions::bb8(12));
        let config = MssqlConnectionConfig::from_connection_string_with_options(
            "server=tcp:localhost,1433;database=master;user=sa;password=Password123;TrustServerCertificate=true",
            options,
        )
        .unwrap();

        assert_eq!(config.options().pool.backend, MssqlPoolBackend::Bb8);
        assert_eq!(config.options().pool.max_size, 12);
    }

    #[test]
    fn exposes_dbcontext_entity_set_contract_in_prelude() {
        fn require_trait<C, E>()
        where
            C: DbContextEntitySet<E>,
            E: Entity,
        {
        }

        require_trait::<DerivedDbContext, DerivedUser>();
    }

    #[test]
    fn exposes_dbcontext_health_check_contract_in_prelude() {
        let _health_check = DerivedDbContext::health_check;
        let _trait_health_check = <DerivedDbContext as DbContext>::health_check;
    }

    #[test]
    fn exposes_active_record_contract_in_prelude() {
        fn require_trait<E: ActiveRecord>() {}

        require_trait::<PublicEntity>();
    }

    #[test]
    fn exposes_tracking_surface_in_prelude() {
        let tracked = Tracked::from_loaded(String::from("tracked"));

        assert_eq!(tracked.state(), EntityState::Unchanged);
        assert_eq!(tracked.current(), "tracked");
    }

    #[allow(dead_code)]
    #[derive(Entity, Debug, Clone)]
    #[orm(table = "users", schema = "auth")]
    #[orm(index(name = "ix_users_email_created_by", columns(email, created_by)))]
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

    impl FromRow for DerivedUser {
        fn from_row<R: mssql_orm_core::Row>(_row: &R) -> Result<Self, OrmError> {
            Ok(Self {
                id: 1,
                email: "ana@example.com".to_string(),
                display_name: Some("Ana".to_string()),
                created_by: "system".to_string(),
                version: vec![1, 2, 3, 4],
            })
        }
    }

    impl FromRow for AuditEntry {
        fn from_row<R: mssql_orm_core::Row>(_row: &R) -> Result<Self, OrmError> {
            Ok(Self {
                id: 1,
                payload: "payload".to_string(),
            })
        }
    }

    #[derive(Insertable, Debug, Clone)]
    #[orm(entity = DerivedUser)]
    struct NewDerivedUser {
        email: String,
        display_name: Option<String>,
        #[orm(column = "created_by")]
        author: String,
    }

    #[derive(Changeset, Debug, Clone)]
    #[orm(entity = DerivedUser)]
    struct UpdateDerivedUser {
        email: Option<String>,
        display_name: Option<Option<String>>,
        #[orm(column = "created_by")]
        author: Option<String>,
    }

    #[allow(dead_code)]
    #[derive(DbContext, Debug, Clone)]
    struct DerivedDbContext {
        pub users: DbSet<DerivedUser>,
        pub audit_entries: DbSet<AuditEntry>,
    }

    #[test]
    fn derives_entity_metadata_from_struct_attributes() {
        let metadata = DerivedUser::metadata();

        assert_eq!(metadata.rust_name, "DerivedUser");
        assert_eq!(metadata.schema, "auth");
        assert_eq!(metadata.table, "users");
        assert_eq!(metadata.primary_key.columns, &["id"]);
        assert_eq!(metadata.indexes.len(), 3);

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
        assert_eq!(metadata.indexes[2].name, "ix_users_email_created_by");
        assert_eq!(metadata.indexes[2].columns.len(), 2);
        assert_eq!(metadata.indexes[2].columns[0].column_name, "email");
        assert_eq!(metadata.indexes[2].columns[1].column_name, "created_by");
        assert!(!metadata.indexes[2].columns[0].descending);
        assert!(!metadata.indexes[2].columns[1].descending);
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

    #[test]
    fn exposes_public_column_predicate_extensions() {
        assert_eq!(
            DerivedUser::email.eq("ana@example.com".to_string()),
            Predicate::eq(
                Expr::from(DerivedUser::email),
                Expr::value(SqlValue::String("ana@example.com".to_string()))
            )
        );
        assert_eq!(
            DerivedUser::display_name.is_null(),
            Predicate::is_null(Expr::from(DerivedUser::display_name))
        );
        assert_eq!(
            DerivedUser::email.contains("@example.com"),
            Predicate::like(
                Expr::from(DerivedUser::email),
                Expr::value(SqlValue::String("%@example.com%".to_string()))
            )
        );
        assert_eq!(
            DerivedUser::email.asc(),
            OrderBy::new(TableRef::new("auth", "users"), "email", SortDirection::Asc)
        );
        assert_eq!(
            DerivedUser::email
                .contains("@example.com")
                .and(DerivedUser::display_name.is_not_null()),
            Predicate::and(vec![
                Predicate::like(
                    Expr::from(DerivedUser::email),
                    Expr::value(SqlValue::String("%@example.com%".to_string()))
                ),
                Predicate::is_not_null(Expr::from(DerivedUser::display_name))
            ])
        );
    }

    #[test]
    fn derives_insertable_values_from_named_fields() {
        let insertable = NewDerivedUser {
            email: "ana@example.com".to_string(),
            display_name: None,
            author: "system".to_string(),
        };

        let values = <NewDerivedUser as Insertable<DerivedUser>>::values(&insertable);

        assert_eq!(
            values,
            vec![
                ColumnValue::new("email", SqlValue::String("ana@example.com".to_string())),
                ColumnValue::new("display_name", SqlValue::Null),
                ColumnValue::new("created_by", SqlValue::String("system".to_string())),
            ]
        );
    }

    #[test]
    fn derives_changeset_with_outer_option_semantics() {
        let changeset = UpdateDerivedUser {
            email: Some("ana.maria@example.com".to_string()),
            display_name: Some(None),
            author: None,
        };

        let changes = <UpdateDerivedUser as Changeset<DerivedUser>>::changes(&changeset);

        assert_eq!(
            changes,
            vec![
                ColumnValue::new(
                    "email",
                    SqlValue::String("ana.maria@example.com".to_string())
                ),
                ColumnValue::new("display_name", SqlValue::Null),
            ]
        );
    }
}

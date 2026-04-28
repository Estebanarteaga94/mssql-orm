//! Public API surface for the workspace.

extern crate self as mssql_orm;

mod active_record;
mod audit_runtime;
mod context;
mod dbset_query;
mod page_request;
mod predicate_composition;
mod query_order;
mod query_predicates;
mod query_projection;
mod raw_sql;
mod soft_delete_runtime;
mod tracking;

pub use mssql_orm_core as core;
pub use mssql_orm_macros as macros;
pub use mssql_orm_migrate as migrate;
pub use mssql_orm_query as query;
pub use mssql_orm_sqlserver as sqlserver;
pub use mssql_orm_tiberius as tiberius;
pub use tokio;

pub use active_record::{ActiveRecord, EntityPersist, EntityPersistMode, EntityPrimaryKey};
pub use audit_runtime::{
    AuditContext, AuditOperation, AuditProvider, AuditRequestValues, resolve_audit_values,
};
#[cfg(feature = "pool-bb8")]
pub use context::connect_shared_from_pool;
pub use context::{
    ActiveTenant, DbContext, DbContextEntitySet, DbSet, SharedConnection, connect_shared,
    connect_shared_with_config, connect_shared_with_options,
};
pub use dbset_query::DbSetQuery;
pub use mssql_orm_core::EntityMetadata;
pub use mssql_orm_tiberius::{
    MssqlConnectionConfig, MssqlHealthCheckOptions, MssqlHealthCheckQuery, MssqlOperationalOptions,
    MssqlParameterLogMode, MssqlPoolBackend, MssqlPoolOptions, MssqlRetryOptions,
    MssqlSlowQueryOptions, MssqlTimeoutOptions, MssqlTracingOptions,
};
#[cfg(feature = "pool-bb8")]
pub use mssql_orm_tiberius::{MssqlPool, MssqlPoolBuilder, MssqlPooledConnection};
pub use page_request::PageRequest;
pub use predicate_composition::PredicateCompositionExt;
pub use query_order::EntityColumnOrderExt;
pub use query_predicates::EntityColumnPredicateExt;
pub use query_projection::SelectProjections;
pub use raw_sql::{QueryHint, RawCommand, RawParam, RawParams, RawQuery};
pub use soft_delete_runtime::{
    SoftDeleteContext, SoftDeleteOperation, SoftDeleteProvider, SoftDeleteRequestValues,
};
pub use tracking::{EntityState, Tracked};
#[doc(hidden)]
pub use tracking::{TrackedEntityRegistration, TrackingRegistry, TrackingRegistryHandle};

pub trait MigrationModelSource {
    fn entity_metadata() -> &'static [&'static EntityMetadata];
}

pub trait AuditEntity: core::Entity {
    fn audit_policy() -> Option<core::EntityPolicyMetadata>;
}

pub trait SoftDeleteEntity: core::Entity {
    fn soft_delete_policy() -> Option<core::EntityPolicyMetadata>;
}

pub trait TenantContext: core::EntityPolicy {
    const COLUMN_NAME: &'static str;

    fn tenant_value(&self) -> core::SqlValue;
}

pub trait TenantScopedEntity: core::Entity {
    fn tenant_policy() -> Option<core::EntityPolicyMetadata>;
}

pub fn model_snapshot_from_source<S: MigrationModelSource>() -> migrate::ModelSnapshot {
    migrate::ModelSnapshot::from_entities(S::entity_metadata())
}

pub fn model_snapshot_json_from_source<S: MigrationModelSource>() -> Result<String, core::OrmError>
{
    model_snapshot_from_source::<S>().to_json_pretty()
}

pub mod prelude {
    #[cfg(feature = "pool-bb8")]
    pub use crate::connect_shared_from_pool;
    pub use crate::{
        ActiveRecord, ActiveTenant, AuditEntity, DbContext, DbContextEntitySet, DbSet, DbSetQuery,
        EntityColumnOrderExt, EntityColumnPredicateExt, EntityState, MigrationModelSource,
        MssqlConnectionConfig, MssqlHealthCheckOptions, MssqlHealthCheckQuery,
        MssqlOperationalOptions, MssqlParameterLogMode, MssqlPoolBackend, MssqlPoolOptions,
        MssqlRetryOptions, MssqlSlowQueryOptions, MssqlTimeoutOptions, MssqlTracingOptions,
        PageRequest, PredicateCompositionExt, QueryHint, RawCommand, RawParam, RawParams, RawQuery,
        SelectProjections, SharedConnection, SoftDeleteContext, SoftDeleteEntity,
        SoftDeleteOperation, SoftDeleteProvider, SoftDeleteRequestValues, TenantContext,
        TenantScopedEntity, Tracked, model_snapshot_from_source, model_snapshot_json_from_source,
    };
    pub use crate::{
        AuditContext, AuditOperation, AuditProvider, AuditRequestValues, resolve_audit_values,
    };
    #[cfg(feature = "pool-bb8")]
    pub use crate::{MssqlPool, MssqlPoolBuilder, MssqlPooledConnection};
    pub use mssql_orm_core::{
        Changeset, ColumnMetadata, ColumnValue, Entity, EntityColumn, EntityMetadata, EntityPolicy,
        EntityPolicyMetadata, ForeignKeyMetadata, FromRow, IdentityMetadata, IndexColumnMetadata,
        IndexMetadata, Insertable, OrmError, PrimaryKeyMetadata, ReferentialAction, Row,
        SqlServerType, SqlTypeMapping, SqlValue,
    };
    pub use mssql_orm_macros::{
        AuditFields, Changeset, DbContext, Entity, FromRow, Insertable, SoftDeleteFields,
        TenantContext,
    };
    pub use mssql_orm_query::{Join, JoinType, SelectProjection};
}

#[cfg(test)]
mod tests {
    use super::prelude::{
        ActiveRecord, ActiveTenant, AuditContext, AuditEntity, AuditFields, AuditOperation,
        AuditProvider, AuditRequestValues, Changeset, ColumnValue, DbContext, DbContextEntitySet,
        DbSet, Entity, EntityColumn, EntityColumnOrderExt, EntityColumnPredicateExt,
        EntityMetadata, EntityPolicy, EntityPolicyMetadata, EntityState, IdentityMetadata,
        Insertable, MssqlConnectionConfig, MssqlOperationalOptions, MssqlPoolBackend,
        MssqlPoolOptions, MssqlRetryOptions, MssqlTimeoutOptions, OrmError, PageRequest,
        PredicateCompositionExt, PrimaryKeyMetadata, QueryHint, RawCommand, RawParam, RawParams,
        RawQuery, SelectProjection, SelectProjections, SharedConnection, SoftDeleteEntity,
        SoftDeleteFields, SqlServerType, SqlTypeMapping, SqlValue, TenantContext,
        TenantScopedEntity, Tracked,
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

    struct PublicPolicy;

    impl EntityPolicy for PublicPolicy {
        const POLICY_NAME: &'static str = "public_policy";
        const COLUMN_NAMES: &'static [&'static str] = &[];

        fn columns() -> &'static [super::core::ColumnMetadata] {
            &[]
        }
    }

    #[allow(dead_code)]
    #[derive(SoftDeleteFields)]
    struct PublicSoftDelete {
        #[orm(sql_type = "datetime2")]
        deleted_at: Option<String>,

        #[orm(nullable)]
        #[orm(length = 120)]
        deleted_by: Option<String>,
    }

    #[allow(dead_code)]
    #[derive(AuditFields)]
    struct PublicAudit {
        #[orm(default_sql = "SYSUTCDATETIME()")]
        #[orm(sql_type = "datetime2")]
        #[orm(updatable = false)]
        created_at: String,

        #[orm(column = "created_by_user_id")]
        created_by: Option<i64>,

        #[orm(nullable)]
        #[orm(length = 120)]
        updated_by: Option<String>,
    }

    #[allow(dead_code)]
    #[derive(TenantContext)]
    struct PublicTenant {
        #[orm(column = "company_id")]
        tenant_id: i64,
    }

    #[test]
    fn exposes_public_prelude() {
        let error = OrmError::new("public-api");
        let raw_query_type = core::any::type_name::<RawQuery<PublicEntity>>();
        let raw_command_type = core::any::type_name::<RawCommand>();
        let projection_type = core::any::type_name::<SelectProjection>();
        let query_hint = QueryHint::Recompile;
        fn assert_raw_param<T: RawParam>() {}
        fn assert_raw_params<T: RawParams>() {}
        fn assert_select_projections<T: SelectProjections>() {}

        assert!(raw_query_type.contains("RawQuery"));
        assert!(raw_command_type.contains("RawCommand"));
        assert!(projection_type.contains("SelectProjection"));
        assert_raw_param::<i64>();
        assert_raw_param::<SqlValue>();
        assert_raw_params::<(bool, i64)>();
        assert_select_projections::<(EntityColumn<PublicEntity>,)>();
        assert_eq!(query_hint, QueryHint::Recompile);
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
    fn exposes_entity_policy_contract_in_prelude() {
        assert_eq!(
            PublicPolicy::metadata(),
            EntityPolicyMetadata::new("public_policy", &[])
        );
    }

    #[test]
    fn exposes_audit_entity_contract_in_prelude() {
        struct PublicAuditEntity;

        impl Entity for PublicAuditEntity {
            fn metadata() -> &'static EntityMetadata {
                &PUBLIC_ENTITY_METADATA
            }
        }

        impl AuditEntity for PublicAuditEntity {
            fn audit_policy() -> Option<EntityPolicyMetadata> {
                Some(EntityPolicyMetadata::new("audit", &[]))
            }
        }

        assert_eq!(
            PublicAuditEntity::audit_policy(),
            Some(EntityPolicyMetadata::new("audit", &[]))
        );
    }

    #[test]
    fn exposes_soft_delete_contract_in_prelude() {
        struct PublicSoftDeleteEntity;

        impl Entity for PublicSoftDeleteEntity {
            fn metadata() -> &'static EntityMetadata {
                &PUBLIC_ENTITY_METADATA
            }
        }

        impl SoftDeleteEntity for PublicSoftDeleteEntity {
            fn soft_delete_policy() -> Option<EntityPolicyMetadata> {
                Some(EntityPolicyMetadata::new("soft_delete", &[]))
            }
        }

        assert_eq!(
            PublicSoftDeleteEntity::soft_delete_policy(),
            Some(EntityPolicyMetadata::new("soft_delete", &[]))
        );
    }

    #[test]
    fn exposes_tenant_contract_in_prelude() {
        struct PublicTenantEntity;

        impl Entity for PublicTenantEntity {
            fn metadata() -> &'static EntityMetadata {
                &PUBLIC_ENTITY_METADATA
            }
        }

        impl TenantScopedEntity for PublicTenantEntity {
            fn tenant_policy() -> Option<EntityPolicyMetadata> {
                Some(EntityPolicyMetadata::new("tenant", &[]))
            }
        }

        assert_eq!(
            PublicTenantEntity::tenant_policy(),
            Some(EntityPolicyMetadata::new("tenant", &[]))
        );
    }

    #[test]
    fn exposes_audit_runtime_contract_in_prelude() {
        struct PublicAuditProvider;

        impl AuditProvider for PublicAuditProvider {
            fn values(&self, context: AuditContext<'_>) -> Result<Vec<ColumnValue>, OrmError> {
                assert_eq!(context.operation, AuditOperation::Update);
                assert!(context.request_values.is_some());

                Ok(vec![ColumnValue::new(
                    "updated_at",
                    SqlValue::String("provider-updated-at".to_string()),
                )])
            }
        }

        let request_values = AuditRequestValues::new(vec![ColumnValue::new(
            "updated_by",
            SqlValue::String("request-updated-by".to_string()),
        )]);
        let context = AuditContext {
            entity: PublicEntity::metadata(),
            operation: AuditOperation::Update,
            request_values: Some(&request_values),
        };

        let provider = PublicAuditProvider;
        let values = provider.values(context).unwrap();

        assert_eq!(request_values.values()[0].column_name, "updated_by");
        assert_eq!(values[0].column_name, "updated_at");
    }

    #[test]
    fn derives_audit_fields_policy_metadata_from_public_prelude() {
        let metadata = PublicAudit::metadata();

        assert_eq!(metadata.name, "audit");
        assert_eq!(metadata.columns.len(), 3);
        assert_eq!(metadata.columns[0].rust_field, "created_at");
        assert_eq!(metadata.columns[0].column_name, "created_at");
        assert_eq!(metadata.columns[0].sql_type, SqlServerType::DateTime2);
        assert_eq!(metadata.columns[0].default_sql, Some("SYSUTCDATETIME()"));
        assert!(metadata.columns[0].insertable);
        assert!(!metadata.columns[0].updatable);
        assert_eq!(metadata.columns[1].column_name, "created_by_user_id");
        assert!(metadata.columns[1].nullable);
        assert_eq!(metadata.columns[1].sql_type, SqlServerType::BigInt);
        assert_eq!(metadata.columns[2].max_length, Some(120));
        assert!(metadata.columns[2].updatable);
        assert_eq!(
            <PublicAudit as EntityPolicy>::COLUMN_NAMES,
            &["created_at", "created_by_user_id", "updated_by"]
        );
    }

    #[test]
    fn derives_tenant_context_policy_metadata_from_public_prelude() {
        let metadata = PublicTenant::metadata();
        let tenant = PublicTenant { tenant_id: 42 };
        let active_tenant = ActiveTenant::from_context(&tenant);

        assert_eq!(metadata.name, "tenant");
        assert_eq!(metadata.columns.len(), 1);
        assert_eq!(metadata.columns[0].rust_field, "tenant_id");
        assert_eq!(metadata.columns[0].column_name, "company_id");
        assert_eq!(metadata.columns[0].sql_type, SqlServerType::BigInt);
        assert!(metadata.columns[0].insertable);
        assert!(!metadata.columns[0].updatable);
        assert_eq!(
            <PublicTenant as EntityPolicy>::COLUMN_NAMES,
            &["company_id"]
        );
        assert_eq!(PublicTenant::COLUMN_NAME, "company_id");
        assert_eq!(tenant.tenant_value(), SqlValue::I64(42));
        assert_eq!(active_tenant.column_name, "company_id");
        assert_eq!(active_tenant.value, SqlValue::I64(42));
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

    #[cfg(feature = "pool-bb8")]
    #[test]
    fn exposes_pool_surface_when_feature_is_enabled() {
        let builder = super::MssqlPool::builder().max_size(8);

        assert_eq!(builder.options().max_size, 8);
    }

    #[cfg(feature = "pool-bb8")]
    #[test]
    fn exposes_dbcontext_pool_wiring_when_feature_is_enabled() {
        let _from_pool = DerivedDbContext::from_pool;
        let _shared_from_pool = super::connect_shared_from_pool;
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
    fn exposes_dbcontext_soft_delete_runtime_helpers() {
        let _with_soft_delete_provider = DerivedDbContext::with_soft_delete_provider;
        let _with_soft_delete_request_values = DerivedDbContext::with_soft_delete_request_values;
        let _clear_soft_delete_request_values = DerivedDbContext::clear_soft_delete_request_values;
    }

    #[test]
    fn exposes_dbcontext_audit_runtime_helpers() {
        let _with_audit_provider = DerivedDbContext::with_audit_provider;
        let _with_audit_request_values = DerivedDbContext::with_audit_request_values;
        let _clear_audit_request_values = DerivedDbContext::clear_audit_request_values;
        let _shared_with_audit_provider = SharedConnection::with_audit_provider;
        let _shared_with_audit_request_values = SharedConnection::with_audit_request_values;
        let _shared_clear_audit_request_values = SharedConnection::clear_audit_request_values;
    }

    #[test]
    fn exposes_dbcontext_tenant_runtime_helpers() {
        let _with_tenant = DerivedDbContext::with_tenant::<PublicTenant>;
        let _clear_tenant = DerivedDbContext::clear_tenant;
        let _shared_with_tenant = SharedConnection::with_tenant::<PublicTenant>;
        let _shared_clear_tenant = SharedConnection::clear_tenant;
    }

    #[test]
    fn exposes_migration_model_source_contract_in_prelude() {
        fn require_trait<C: super::MigrationModelSource>() {}

        require_trait::<DerivedDbContext>();
        assert_eq!(
            <DerivedDbContext as super::MigrationModelSource>::entity_metadata()
                .iter()
                .map(|metadata| metadata.table)
                .collect::<Vec<_>>(),
            vec!["users", "audit_entries"]
        );
    }

    #[test]
    fn exposes_model_snapshot_export_helpers() {
        let snapshot = super::model_snapshot_from_source::<DerivedDbContext>();
        let json = super::model_snapshot_json_from_source::<DerivedDbContext>().unwrap();

        assert_eq!(
            snapshot
                .schemas
                .iter()
                .flat_map(|schema| schema.tables.iter().map(|table| table.name.as_str()))
                .collect::<Vec<_>>(),
            vec!["users", "audit_entries"]
        );
        assert!(json.contains("\"name\": \"auth\""));
        assert!(json.contains("\"name\": \"users\""));
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
                ColumnValue::new("display_name", SqlValue::TypedNull(SqlServerType::NVarChar)),
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
                ColumnValue::new("display_name", SqlValue::TypedNull(SqlServerType::NVarChar)),
            ]
        );
    }
}

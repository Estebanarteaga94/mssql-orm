//! Migration support foundations.

use mssql_orm_core::CrateIdentity;

mod snapshot;

pub use snapshot::{
    ColumnSnapshot, IndexColumnSnapshot, IndexSnapshot, ModelSnapshot, SchemaSnapshot,
    TableSnapshot,
};

/// Placeholder migration engine marker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MigrationEngine;

pub const CRATE_IDENTITY: CrateIdentity = CrateIdentity {
    name: "mssql-orm-migrate",
    responsibility: "code-first snapshots, diffs and migration operations",
};

#[cfg(test)]
mod tests {
    use super::{
        CRATE_IDENTITY, ColumnSnapshot, IndexColumnSnapshot, IndexSnapshot, MigrationEngine,
        ModelSnapshot, SchemaSnapshot, TableSnapshot,
    };
    use mssql_orm_core::{
        ColumnMetadata, EntityMetadata, IdentityMetadata, IndexColumnMetadata, IndexMetadata,
        PrimaryKeyMetadata, SqlServerType,
    };

    const CUSTOMER_COLUMNS: [ColumnMetadata; 3] = [
        ColumnMetadata {
            rust_field: "id",
            column_name: "id",
            sql_type: SqlServerType::BigInt,
            nullable: false,
            primary_key: true,
            identity: Some(IdentityMetadata::new(1, 1)),
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: false,
            updatable: false,
            max_length: None,
            precision: None,
            scale: None,
        },
        ColumnMetadata {
            rust_field: "email",
            column_name: "email",
            sql_type: SqlServerType::NVarChar,
            nullable: false,
            primary_key: false,
            identity: None,
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: true,
            updatable: true,
            max_length: Some(160),
            precision: None,
            scale: None,
        },
        ColumnMetadata {
            rust_field: "version",
            column_name: "version",
            sql_type: SqlServerType::RowVersion,
            nullable: false,
            primary_key: false,
            identity: None,
            default_sql: None,
            computed_sql: None,
            rowversion: true,
            insertable: false,
            updatable: false,
            max_length: None,
            precision: None,
            scale: None,
        },
    ];

    const CUSTOMER_PK_COLUMNS: [&str; 1] = ["id"];
    const CUSTOMER_INDEX_COLUMNS: [IndexColumnMetadata; 1] = [IndexColumnMetadata::asc("email")];
    const CUSTOMER_INDEXES: [IndexMetadata; 1] = [IndexMetadata {
        name: "ix_customers_email",
        columns: &CUSTOMER_INDEX_COLUMNS,
        unique: true,
    }];
    const CUSTOMER_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "Customer",
        schema: "sales",
        table: "customers",
        columns: &CUSTOMER_COLUMNS,
        primary_key: PrimaryKeyMetadata::new(Some("pk_customers"), &CUSTOMER_PK_COLUMNS),
        indexes: &CUSTOMER_INDEXES,
        foreign_keys: &[],
    };

    const TENANT_COLUMNS: [ColumnMetadata; 2] = [
        ColumnMetadata {
            rust_field: "id",
            column_name: "id",
            sql_type: SqlServerType::BigInt,
            nullable: false,
            primary_key: true,
            identity: Some(IdentityMetadata::new(100, 5)),
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: false,
            updatable: false,
            max_length: None,
            precision: None,
            scale: None,
        },
        ColumnMetadata {
            rust_field: "display_name",
            column_name: "display_name",
            sql_type: SqlServerType::NVarChar,
            nullable: false,
            primary_key: false,
            identity: None,
            default_sql: Some("'tenant'"),
            computed_sql: None,
            rowversion: false,
            insertable: true,
            updatable: true,
            max_length: Some(120),
            precision: None,
            scale: None,
        },
    ];

    const TENANT_PK_COLUMNS: [&str; 1] = ["id"];
    const TENANT_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "Tenant",
        schema: "admin",
        table: "tenants",
        columns: &TENANT_COLUMNS,
        primary_key: PrimaryKeyMetadata::new(None, &TENANT_PK_COLUMNS),
        indexes: &[],
        foreign_keys: &[],
    };

    const ORDER_COLUMNS: [ColumnMetadata; 2] = [
        ColumnMetadata {
            rust_field: "id",
            column_name: "id",
            sql_type: SqlServerType::BigInt,
            nullable: false,
            primary_key: true,
            identity: Some(IdentityMetadata::new(1, 1)),
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: false,
            updatable: false,
            max_length: None,
            precision: None,
            scale: None,
        },
        ColumnMetadata {
            rust_field: "customer_id",
            column_name: "customer_id",
            sql_type: SqlServerType::BigInt,
            nullable: false,
            primary_key: false,
            identity: None,
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: true,
            updatable: true,
            max_length: None,
            precision: None,
            scale: None,
        },
    ];

    const ORDER_PK_COLUMNS: [&str; 1] = ["id"];
    const ORDER_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "Order",
        schema: "sales",
        table: "orders",
        columns: &ORDER_COLUMNS,
        primary_key: PrimaryKeyMetadata::new(Some("pk_orders"), &ORDER_PK_COLUMNS),
        indexes: &[],
        foreign_keys: &[],
    };

    #[test]
    fn declares_migration_boundary() {
        let engine = MigrationEngine;
        assert_eq!(engine, MigrationEngine);
        assert!(CRATE_IDENTITY.responsibility.contains("migration"));
    }

    #[test]
    fn model_snapshot_exposes_schema_table_column_and_index_lookups() {
        let snapshot = ModelSnapshot::new(vec![SchemaSnapshot::new(
            "sales",
            vec![TableSnapshot::new(
                "customers",
                vec![
                    ColumnSnapshot::new(
                        "id",
                        SqlServerType::BigInt,
                        false,
                        true,
                        Some(IdentityMetadata::new(1, 1)),
                        None,
                        None,
                        false,
                        false,
                        false,
                        None,
                        None,
                        None,
                    ),
                    ColumnSnapshot::new(
                        "email",
                        SqlServerType::NVarChar,
                        false,
                        false,
                        None,
                        None,
                        None,
                        false,
                        true,
                        true,
                        Some(160),
                        None,
                        None,
                    ),
                ],
                Some("pk_customers".to_string()),
                vec!["id".to_string()],
                vec![IndexSnapshot::new(
                    "ix_customers_email",
                    vec![IndexColumnSnapshot::asc("email")],
                    false,
                )],
            )],
        )]);

        let schema = snapshot.schema("sales").expect("schema must exist");
        let table = schema.table("customers").expect("table must exist");
        let id = table.column("id").expect("column must exist");
        let index = table.index("ix_customers_email").expect("index must exist");

        assert_eq!(table.primary_key_name.as_deref(), Some("pk_customers"));
        assert_eq!(table.primary_key_columns, vec!["id"]);
        assert_eq!(id.identity, Some(IdentityMetadata::new(1, 1)));
        assert_eq!(index.columns, vec![IndexColumnSnapshot::asc("email")]);
    }

    #[test]
    fn column_snapshot_preserves_sql_server_specific_shape() {
        let column = ColumnSnapshot::new(
            "version",
            SqlServerType::RowVersion,
            false,
            false,
            None,
            Some("CONVERT(binary(8), 0)".to_string()),
            Some("([major] + [minor])".to_string()),
            true,
            false,
            false,
            Some(8),
            Some(18),
            Some(4),
        );

        assert_eq!(column.name, "version");
        assert_eq!(column.sql_type, SqlServerType::RowVersion);
        assert_eq!(column.default_sql.as_deref(), Some("CONVERT(binary(8), 0)"));
        assert_eq!(column.computed_sql.as_deref(), Some("([major] + [minor])"));
        assert!(column.rowversion);
        assert!(!column.insertable);
        assert!(!column.updatable);
        assert_eq!(column.max_length, Some(8));
        assert_eq!(column.precision, Some(18));
        assert_eq!(column.scale, Some(4));
    }

    #[test]
    fn table_snapshot_can_be_built_from_entity_metadata() {
        let table = TableSnapshot::from(&CUSTOMER_METADATA);

        assert_eq!(table.name, "customers");
        assert_eq!(table.primary_key_name.as_deref(), Some("pk_customers"));
        assert_eq!(table.primary_key_columns, vec!["id"]);
        assert_eq!(table.columns.len(), 3);
        assert_eq!(table.columns[0].name, "id");
        assert_eq!(table.columns[1].name, "email");
        assert_eq!(table.indexes.len(), 1);
        assert_eq!(table.indexes[0].name, "ix_customers_email");
        assert!(table.indexes[0].unique);
    }

    #[test]
    fn model_snapshot_groups_entities_by_schema_and_sorts_tables() {
        let snapshot =
            ModelSnapshot::from_entities(&[&ORDER_METADATA, &TENANT_METADATA, &CUSTOMER_METADATA]);

        assert_eq!(snapshot.schemas.len(), 2);
        assert_eq!(snapshot.schemas[0].name, "admin");
        assert_eq!(snapshot.schemas[1].name, "sales");

        let admin = snapshot.schema("admin").expect("admin schema must exist");
        assert_eq!(admin.tables.len(), 1);
        assert_eq!(admin.tables[0].name, "tenants");

        let sales = snapshot.schema("sales").expect("sales schema must exist");
        assert_eq!(
            sales
                .tables
                .iter()
                .map(|table| table.name.as_str())
                .collect::<Vec<_>>(),
            vec!["customers", "orders"]
        );
        assert_eq!(
            sales
                .table("customers")
                .expect("customers table must exist")
                .column("email")
                .expect("email column must exist")
                .max_length,
            Some(160)
        );
    }
}

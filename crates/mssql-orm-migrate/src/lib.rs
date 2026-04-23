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
    use mssql_orm_core::{IdentityMetadata, SqlServerType};

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
}

use mssql_orm::prelude::*;

struct SoftDelete;

impl EntityPolicy for SoftDelete {
    const POLICY_NAME: &'static str = "soft_delete";
    const COLUMN_NAMES: &'static [&'static str] = &["deleted_at", "deleted_by"];

    fn columns() -> &'static [ColumnMetadata] {
        const COLUMNS: &[ColumnMetadata] = &[
            ColumnMetadata {
                rust_field: "deleted_at",
                column_name: "deleted_at",
                renamed_from: None,
                sql_type: SqlServerType::DateTime2,
                nullable: true,
                primary_key: false,
                identity: None,
                default_sql: None,
                computed_sql: None,
                rowversion: false,
                insertable: false,
                updatable: true,
                max_length: None,
                precision: None,
                scale: None,
            },
            ColumnMetadata {
                rust_field: "deleted_by",
                column_name: "deleted_by",
                renamed_from: None,
                sql_type: SqlServerType::NVarChar,
                nullable: true,
                primary_key: false,
                identity: None,
                default_sql: None,
                computed_sql: None,
                rowversion: false,
                insertable: false,
                updatable: true,
                max_length: Some(120),
                precision: None,
                scale: None,
            },
        ];

        COLUMNS
    }
}

#[derive(Entity, Debug, Clone)]
#[orm(table = "soft_deleted_entities", schema = "audit", soft_delete = SoftDelete)]
struct SoftDeletedEntity {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(length = 120)]
    name: String,
}

fn main() {
    let metadata = SoftDeletedEntity::metadata();
    assert_eq!(metadata.columns.len(), 4);
    assert_eq!(metadata.columns[2].column_name, "deleted_at");
    assert_eq!(metadata.columns[3].column_name, "deleted_by");

    let soft_delete = SoftDeletedEntity::soft_delete_policy().expect("soft delete policy");
    assert_eq!(soft_delete.name, "soft_delete");
    assert_eq!(soft_delete.columns.len(), 2);
}

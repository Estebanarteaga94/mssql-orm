use mssql_orm::prelude::*;
use std::collections::BTreeMap;

#[derive(AuditFields)]
#[allow(dead_code)]
struct Audit {
    #[orm(default_sql = "SYSUTCDATETIME()")]
    #[orm(sql_type = "datetime2")]
    #[orm(insertable = false)]
    #[orm(updatable = false)]
    created_at: String,

    #[orm(column = "created_by_user_id")]
    #[orm(nullable)]
    created_by: Option<i64>,

    #[orm(default_sql = "SYSUTCDATETIME()")]
    #[orm(sql_type = "datetime2")]
    #[orm(insertable = false)]
    updated_at: Option<String>,

    #[orm(nullable)]
    #[orm(length = 120)]
    updated_by: Option<String>,
}

#[derive(Entity, Debug, Clone, PartialEq)]
#[orm(table = "audited_entities", schema = "audit", audit = Audit)]
struct AuditedEntity {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(length = 120)]
    name: String,

    #[orm(length = 40)]
    #[orm(default_sql = "'new'")]
    status: Option<String>,
}

struct TestRow {
    values: BTreeMap<&'static str, SqlValue>,
}

impl Row for TestRow {
    fn try_get(&self, column: &str) -> Result<Option<SqlValue>, OrmError> {
        Ok(self.values.get(column).cloned())
    }
}

#[test]
fn audit_policy_columns_are_expanded_into_entity_metadata() {
    let metadata = AuditedEntity::metadata();

    assert_eq!(metadata.rust_name, "AuditedEntity");
    assert_eq!(metadata.schema, "audit");
    assert_eq!(metadata.table, "audited_entities");
    assert_eq!(metadata.renamed_from, None);
    assert_eq!(metadata.primary_key.columns, &["id"]);
    assert!(metadata.indexes.is_empty());
    assert!(metadata.foreign_keys.is_empty());

    assert_eq!(metadata.columns.len(), 7);
    assert_eq!(metadata.columns[0].column_name, "id");
    assert_eq!(metadata.columns[1].column_name, "name");
    assert_eq!(metadata.columns[2].column_name, "status");
    assert_eq!(metadata.columns[3].column_name, "created_at");
    assert_eq!(metadata.columns[4].column_name, "created_by_user_id");
    assert_eq!(metadata.columns[5].column_name, "updated_at");
    assert_eq!(metadata.columns[6].column_name, "updated_by");

    let id = metadata
        .column("id")
        .expect("entity column should be present");
    assert_eq!(id.rust_field, "id");
    assert_eq!(id.sql_type, SqlServerType::BigInt);
    assert!(!id.nullable);
    assert!(id.primary_key);
    assert_eq!(id.identity, Some(IdentityMetadata::new(1, 1)));
    assert_eq!(id.default_sql, None);
    assert!(!id.insertable);
    assert!(!id.updatable);

    let name = metadata
        .column("name")
        .expect("entity column should be present");
    assert_eq!(name.rust_field, "name");
    assert_eq!(name.sql_type, SqlServerType::NVarChar);
    assert_eq!(name.max_length, Some(120));
    assert!(!name.nullable);
    assert_eq!(name.default_sql, None);
    assert!(name.insertable);
    assert!(name.updatable);

    let status = metadata
        .column("status")
        .expect("entity column should be present");
    assert_eq!(status.rust_field, "status");
    assert_eq!(status.sql_type, SqlServerType::NVarChar);
    assert_eq!(status.max_length, Some(40));
    assert!(status.nullable);
    assert_eq!(status.default_sql, Some("'new'"));
    assert!(status.insertable);
    assert!(status.updatable);

    let created_at = metadata
        .column("created_at")
        .expect("audit column should be present");
    assert_eq!(created_at.rust_field, "created_at");
    assert_eq!(created_at.sql_type, SqlServerType::DateTime2);
    assert_eq!(created_at.default_sql, Some("SYSUTCDATETIME()"));
    assert!(!created_at.nullable);
    assert!(!created_at.insertable);
    assert!(!created_at.updatable);

    let created_by = metadata
        .column("created_by_user_id")
        .expect("audit column should be present");
    assert_eq!(created_by.rust_field, "created_by");
    assert_eq!(created_by.sql_type, SqlServerType::BigInt);
    assert!(created_by.nullable);
    assert_eq!(created_by.default_sql, None);
    assert!(created_by.insertable);
    assert!(created_by.updatable);

    let updated_at = metadata
        .column("updated_at")
        .expect("audit column should be present");
    assert_eq!(updated_at.rust_field, "updated_at");
    assert_eq!(updated_at.sql_type, SqlServerType::DateTime2);
    assert!(updated_at.nullable);
    assert_eq!(updated_at.default_sql, Some("SYSUTCDATETIME()"));
    assert!(!updated_at.insertable);
    assert!(updated_at.updatable);

    let updated_by = metadata
        .column("updated_by")
        .expect("audit column should be present");
    assert!(updated_by.nullable);
    assert_eq!(updated_by.max_length, Some(120));
}

#[test]
fn audited_entity_from_row_materializes_only_real_entity_fields() {
    let row = TestRow {
        values: BTreeMap::from([
            ("id", SqlValue::I64(7)),
            ("name", SqlValue::String("sample".to_string())),
        ]),
    };

    let entity = AuditedEntity::from_row(&row).expect("audited entity should materialize");

    assert_eq!(
        entity,
        AuditedEntity {
            id: 7,
            name: "sample".to_string(),
            status: None,
        }
    );
}

#[test]
fn audited_entity_from_row_ignores_audit_metadata_columns_when_present() {
    let row = TestRow {
        values: BTreeMap::from([
            ("id", SqlValue::I64(9)),
            ("name", SqlValue::String("with audit columns".to_string())),
            (
                "created_at",
                SqlValue::String("2026-04-25T00:00:00".to_string()),
            ),
            ("updated_by", SqlValue::String("system".to_string())),
        ]),
    };

    let entity = AuditedEntity::from_row(&row).expect("audited entity should materialize");

    assert_eq!(
        entity,
        AuditedEntity {
            id: 9,
            name: "with audit columns".to_string(),
            status: None,
        }
    );
}

use mssql_orm::prelude::*;

#[derive(AuditFields)]
#[allow(dead_code)]
struct Audit {
    #[orm(default_sql = "SYSUTCDATETIME()")]
    #[orm(sql_type = "datetime2")]
    #[orm(updatable = false)]
    created_at: String,

    #[orm(nullable)]
    #[orm(length = 120)]
    updated_by: Option<String>,
}

#[derive(Entity, Debug, Clone)]
#[orm(table = "audited_entities", schema = "audit", audit = Audit)]
struct AuditedEntity {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(length = 120)]
    name: String,
}

#[test]
fn audit_policy_columns_are_expanded_into_entity_metadata() {
    let metadata = AuditedEntity::metadata();

    assert_eq!(metadata.columns.len(), 4);
    assert_eq!(metadata.columns[0].column_name, "id");
    assert_eq!(metadata.columns[1].column_name, "name");
    assert_eq!(metadata.columns[2].column_name, "created_at");
    assert_eq!(metadata.columns[3].column_name, "updated_by");

    let created_at = metadata
        .column("created_at")
        .expect("audit column should be present");
    assert_eq!(created_at.rust_field, "created_at");
    assert_eq!(created_at.default_sql, Some("SYSUTCDATETIME()"));
    assert!(!created_at.nullable);
    assert!(created_at.insertable);
    assert!(!created_at.updatable);

    let updated_by = metadata
        .column("updated_by")
        .expect("audit column should be present");
    assert!(updated_by.nullable);
    assert_eq!(updated_by.max_length, Some(120));
}

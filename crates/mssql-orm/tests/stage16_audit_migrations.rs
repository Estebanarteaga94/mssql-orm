use mssql_orm::migrate::{ModelSnapshot, diff_schema_and_table_operations};
use mssql_orm::prelude::*;
use mssql_orm::sqlserver::SqlServerCompiler;

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

#[test]
fn new_audited_entity_migration_creates_table_with_audit_columns() {
    let previous = ModelSnapshot::default();
    let current = ModelSnapshot::from_entities(&[AuditedEntity::metadata()]);
    let operations = diff_schema_and_table_operations(&previous, &current);

    assert_eq!(operations.len(), 2);
    assert_eq!(operations[0].schema_name(), "audit");
    assert_eq!(operations[0].table_name(), None);
    assert_eq!(operations[1].schema_name(), "audit");
    assert_eq!(operations[1].table_name(), Some("audited_entities"));

    let sql = SqlServerCompiler::compile_migration_operations(&operations)
        .expect("audited migration should compile");

    assert_eq!(
        sql[0],
        "IF SCHEMA_ID(N'audit') IS NULL EXEC(N'CREATE SCHEMA [audit]')"
    );
    assert_eq!(
        sql[1],
        "CREATE TABLE [audit].[audited_entities] (\n    [id] bigint IDENTITY(1, 1) NOT NULL,\n    [name] nvarchar(120) NOT NULL,\n    [status] nvarchar(40) NULL DEFAULT 'new',\n    [created_at] datetime2 NOT NULL DEFAULT SYSUTCDATETIME(),\n    [created_by_user_id] bigint NULL,\n    [updated_at] datetime2 NULL DEFAULT SYSUTCDATETIME(),\n    [updated_by] nvarchar(120) NULL,\n    PRIMARY KEY ([id])\n)"
    );
}

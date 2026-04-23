use insta::assert_snapshot;
use mssql_orm_core::ReferentialAction;
use mssql_orm_migrate::{AddForeignKey, DropForeignKey, ForeignKeySnapshot, MigrationOperation};
use mssql_orm_sqlserver::SqlServerCompiler;

#[test]
fn snapshots_foreign_key_migration_sql() {
    let operations = vec![
        MigrationOperation::AddForeignKey(AddForeignKey::new(
            "sales",
            "orders",
            ForeignKeySnapshot::new(
                "fk_orders_customer_id_customers",
                vec!["customer_id".to_string()],
                "sales",
                "customers",
                vec!["id".to_string()],
                ReferentialAction::Cascade,
                ReferentialAction::NoAction,
            ),
        )),
        MigrationOperation::DropForeignKey(DropForeignKey::new(
            "sales",
            "orders",
            "fk_orders_customer_id_customers",
        )),
    ];

    let sql = SqlServerCompiler::compile_migration_operations(&operations).unwrap();

    assert_snapshot!("foreign_key_migration_sql", render_statements(&sql));
}

fn render_statements(statements: &[String]) -> String {
    statements.join("\nGO\n")
}

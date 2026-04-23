use insta::assert_snapshot;
use mssql_orm_core::ReferentialAction;
use mssql_orm_migrate::{
    AddColumn, AddForeignKey, ColumnSnapshot, DropColumn, DropForeignKey, ForeignKeySnapshot,
    MigrationOperation, RenameColumn,
};
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

#[test]
fn snapshots_advanced_foreign_key_migration_sql() {
    let operations = vec![
        MigrationOperation::AddForeignKey(AddForeignKey::new(
            "sales",
            "order_allocations",
            ForeignKeySnapshot::new(
                "fk_order_allocations_customer_branch_customers",
                vec!["customer_id".to_string(), "branch_id".to_string()],
                "sales",
                "customers",
                vec!["id".to_string(), "branch_id".to_string()],
                ReferentialAction::SetDefault,
                ReferentialAction::Cascade,
            ),
        )),
        MigrationOperation::DropForeignKey(DropForeignKey::new(
            "sales",
            "order_allocations",
            "fk_order_allocations_customer_branch_customers",
        )),
    ];

    let sql = SqlServerCompiler::compile_migration_operations(&operations).unwrap();

    assert_snapshot!(
        "advanced_foreign_key_migration_sql",
        render_statements(&sql)
    );
}

#[test]
fn snapshots_computed_column_migration_sql() {
    let operations = vec![
        MigrationOperation::AddColumn(AddColumn::new(
            "sales",
            "order_lines",
            ColumnSnapshot::new(
                "line_total",
                mssql_orm_core::SqlServerType::Decimal,
                false,
                false,
                None,
                None,
                Some("[unit_price] * [quantity]".to_string()),
                false,
                false,
                false,
                None,
                Some(18),
                Some(2),
            ),
        )),
        MigrationOperation::DropColumn(DropColumn::new("sales", "order_lines", "line_total")),
    ];

    let sql = SqlServerCompiler::compile_migration_operations(&operations).unwrap();

    assert_snapshot!("computed_column_migration_sql", render_statements(&sql));
}

#[test]
fn snapshots_rename_column_migration_sql() {
    let operations = vec![MigrationOperation::RenameColumn(RenameColumn::new(
        "sales",
        "customers",
        "email",
        "email_address",
    ))];

    let sql = SqlServerCompiler::compile_migration_operations(&operations).unwrap();

    assert_snapshot!("rename_column_migration_sql", render_statements(&sql));
}

fn render_statements(statements: &[String]) -> String {
    statements.join("\nGO\n")
}

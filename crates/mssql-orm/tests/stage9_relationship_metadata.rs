use mssql_orm::prelude::*;

#[allow(dead_code)]
#[derive(Entity, Debug, Clone)]
#[orm(table = "orders", schema = "sales")]
struct Order {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(foreign_key = "sales.customers.id")]
    customer_id: i64,

    #[orm(column = "approver_user_id")]
    #[orm(foreign_key = "users.id")]
    approved_by: i64,

    total_cents: i64,
}

#[test]
fn derives_relationship_metadata_for_multiple_foreign_keys() {
    let metadata = Order::metadata();

    assert_eq!(metadata.foreign_keys.len(), 2);

    let customer_fk = metadata
        .foreign_key("fk_orders_customer_id_customers")
        .expect("customer foreign key metadata");
    assert_eq!(customer_fk.columns, &["customer_id"]);
    assert_eq!(customer_fk.referenced_schema, "sales");
    assert_eq!(customer_fk.referenced_table, "customers");
    assert_eq!(customer_fk.referenced_columns, &["id"]);
    assert_eq!(customer_fk.on_delete, ReferentialAction::NoAction);
    assert_eq!(customer_fk.on_update, ReferentialAction::NoAction);

    let approver_fk = metadata
        .foreign_key("fk_orders_approver_user_id_users")
        .expect("approver foreign key metadata");
    assert_eq!(approver_fk.columns, &["approver_user_id"]);
    assert_eq!(approver_fk.referenced_schema, "dbo");
    assert_eq!(approver_fk.referenced_table, "users");
    assert_eq!(approver_fk.referenced_columns, &["id"]);

    assert_eq!(Order::approved_by.column_name(), "approver_user_id");
    assert_eq!(
        Order::approved_by.metadata().column_name,
        approver_fk.columns[0]
    );
}

#[test]
fn relationship_metadata_helpers_filter_generated_foreign_keys() {
    let metadata = Order::metadata();

    let customer_column_matches = metadata.foreign_keys_for_column("customer_id");
    assert_eq!(customer_column_matches.len(), 1);
    assert_eq!(
        customer_column_matches[0].name,
        "fk_orders_customer_id_customers"
    );

    let approver_column_matches = metadata.foreign_keys_for_column("approver_user_id");
    assert_eq!(approver_column_matches.len(), 1);
    assert_eq!(
        approver_column_matches[0].name,
        "fk_orders_approver_user_id_users"
    );

    let sales_customer_refs = metadata.foreign_keys_referencing("sales", "customers");
    assert_eq!(sales_customer_refs.len(), 1);
    assert_eq!(
        sales_customer_refs[0].name,
        "fk_orders_customer_id_customers"
    );

    let dbo_user_refs = metadata.foreign_keys_referencing("dbo", "users");
    assert_eq!(dbo_user_refs.len(), 1);
    assert_eq!(dbo_user_refs[0].name, "fk_orders_approver_user_id_users");

    assert!(metadata.foreign_keys_for_column("total_cents").is_empty());
    assert!(
        metadata
            .foreign_keys_referencing("dbo", "accounts")
            .is_empty()
    );
}

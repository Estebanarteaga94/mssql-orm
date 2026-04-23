use crate::{
    AddColumn, AddForeignKey, AlterColumn, ColumnSnapshot, CreateIndex, CreateSchema, CreateTable,
    DropColumn, DropForeignKey, DropIndex, DropSchema, DropTable, ForeignKeySnapshot,
    IndexSnapshot, MigrationOperation, ModelSnapshot, SchemaSnapshot, TableSnapshot,
};
use std::collections::BTreeMap;

/// Computes the minimum stage-7 diff for schema and table creation/deletion.
pub fn diff_schema_and_table_operations(
    previous: &ModelSnapshot,
    current: &ModelSnapshot,
) -> Vec<MigrationOperation> {
    let previous_schemas = schema_map(previous);
    let current_schemas = schema_map(current);
    let mut operations = Vec::new();

    for (schema_name, current_schema) in &current_schemas {
        if !previous_schemas.contains_key(schema_name) {
            operations.push(MigrationOperation::CreateSchema(CreateSchema::new(
                schema_name.clone(),
            )));

            for table in &current_schema.tables {
                operations.push(MigrationOperation::CreateTable(CreateTable::new(
                    schema_name.clone(),
                    table.clone(),
                )));
            }

            continue;
        }

        let previous_tables = table_map(previous_schemas[schema_name]);
        let current_tables = table_map(current_schema);

        for (table_name, table) in &current_tables {
            if !previous_tables.contains_key(table_name) {
                operations.push(MigrationOperation::CreateTable(CreateTable::new(
                    schema_name.clone(),
                    (*table).clone(),
                )));
            }
        }

        for table_name in previous_tables.keys() {
            if !current_tables.contains_key(table_name) {
                operations.push(MigrationOperation::DropTable(DropTable::new(
                    schema_name.clone(),
                    table_name.clone(),
                )));
            }
        }
    }

    for (schema_name, previous_schema) in &previous_schemas {
        if current_schemas.contains_key(schema_name) {
            continue;
        }

        let previous_tables = table_map(previous_schema);
        for table_name in previous_tables.keys() {
            operations.push(MigrationOperation::DropTable(DropTable::new(
                schema_name.clone(),
                table_name.clone(),
            )));
        }

        operations.push(MigrationOperation::DropSchema(DropSchema::new(
            schema_name.clone(),
        )));
    }

    operations
}

/// Computes additive/removal/basic-alteration column operations for tables present
/// in both snapshots. Table creation/deletion remains the responsibility of
/// `diff_schema_and_table_operations`.
pub fn diff_column_operations(
    previous: &ModelSnapshot,
    current: &ModelSnapshot,
) -> Vec<MigrationOperation> {
    let previous_schemas = schema_map(previous);
    let current_schemas = schema_map(current);
    let mut operations = Vec::new();

    for (schema_name, current_schema) in &current_schemas {
        let Some(previous_schema) = previous_schemas.get(schema_name) else {
            continue;
        };

        let previous_tables = table_map(previous_schema);
        let current_tables = table_map(current_schema);

        for (table_name, current_table) in &current_tables {
            let Some(previous_table) = previous_tables.get(table_name) else {
                continue;
            };

            let previous_columns = column_map(previous_table);
            let current_columns = column_map(current_table);

            for (column_name, current_column) in &current_columns {
                match previous_columns.get(column_name) {
                    None => operations.push(MigrationOperation::AddColumn(AddColumn::new(
                        schema_name.clone(),
                        table_name.clone(),
                        (*current_column).clone(),
                    ))),
                    Some(previous_column) if *previous_column != *current_column => {
                        operations.push(MigrationOperation::AlterColumn(AlterColumn::new(
                            schema_name.clone(),
                            table_name.clone(),
                            (*previous_column).clone(),
                            (*current_column).clone(),
                        )));
                    }
                    Some(_) => {}
                }
            }

            for column_name in previous_columns.keys() {
                if !current_columns.contains_key(column_name) {
                    operations.push(MigrationOperation::DropColumn(DropColumn::new(
                        schema_name.clone(),
                        table_name.clone(),
                        column_name.clone(),
                    )));
                }
            }
        }
    }

    operations
}

/// Computes additive/removal operations for indexes and foreign keys in tables
/// present in both snapshots. Table creation/deletion remains the responsibility
/// of `diff_schema_and_table_operations`.
pub fn diff_relational_operations(
    previous: &ModelSnapshot,
    current: &ModelSnapshot,
) -> Vec<MigrationOperation> {
    let previous_schemas = schema_map(previous);
    let current_schemas = schema_map(current);
    let mut operations = Vec::new();

    for (schema_name, current_schema) in &current_schemas {
        let Some(previous_schema) = previous_schemas.get(schema_name) else {
            continue;
        };

        let previous_tables = table_map(previous_schema);
        let current_tables = table_map(current_schema);

        for (table_name, current_table) in &current_tables {
            let Some(previous_table) = previous_tables.get(table_name) else {
                continue;
            };

            let previous_indexes = index_map(previous_table);
            let current_indexes = index_map(current_table);

            for (index_name, index) in &current_indexes {
                match previous_indexes.get(index_name) {
                    None => operations.push(MigrationOperation::CreateIndex(CreateIndex::new(
                        schema_name.clone(),
                        table_name.clone(),
                        (*index).clone(),
                    ))),
                    Some(previous_index) if *previous_index != *index => {
                        operations.push(MigrationOperation::DropIndex(DropIndex::new(
                            schema_name.clone(),
                            table_name.clone(),
                            index_name.clone(),
                        )));
                        operations.push(MigrationOperation::CreateIndex(CreateIndex::new(
                            schema_name.clone(),
                            table_name.clone(),
                            (*index).clone(),
                        )));
                    }
                    Some(_) => {}
                }
            }

            for index_name in previous_indexes.keys() {
                if !current_indexes.contains_key(index_name) {
                    operations.push(MigrationOperation::DropIndex(DropIndex::new(
                        schema_name.clone(),
                        table_name.clone(),
                        index_name.clone(),
                    )));
                }
            }

            let previous_foreign_keys = foreign_key_map(previous_table);
            let current_foreign_keys = foreign_key_map(current_table);

            for (foreign_key_name, foreign_key) in &current_foreign_keys {
                match previous_foreign_keys.get(foreign_key_name) {
                    None => operations.push(MigrationOperation::AddForeignKey(AddForeignKey::new(
                        schema_name.clone(),
                        table_name.clone(),
                        (*foreign_key).clone(),
                    ))),
                    Some(previous_foreign_key) if *previous_foreign_key != *foreign_key => {
                        operations.push(MigrationOperation::DropForeignKey(DropForeignKey::new(
                            schema_name.clone(),
                            table_name.clone(),
                            foreign_key_name.clone(),
                        )));
                        operations.push(MigrationOperation::AddForeignKey(AddForeignKey::new(
                            schema_name.clone(),
                            table_name.clone(),
                            (*foreign_key).clone(),
                        )));
                    }
                    Some(_) => {}
                }
            }

            for foreign_key_name in previous_foreign_keys.keys() {
                if !current_foreign_keys.contains_key(foreign_key_name) {
                    operations.push(MigrationOperation::DropForeignKey(DropForeignKey::new(
                        schema_name.clone(),
                        table_name.clone(),
                        foreign_key_name.clone(),
                    )));
                }
            }
        }
    }

    operations
}

fn schema_map(snapshot: &ModelSnapshot) -> BTreeMap<String, &SchemaSnapshot> {
    snapshot
        .schemas
        .iter()
        .map(|schema| (schema.name.clone(), schema))
        .collect()
}

fn table_map(schema: &SchemaSnapshot) -> BTreeMap<String, &TableSnapshot> {
    schema
        .tables
        .iter()
        .map(|table| (table.name.clone(), table))
        .collect()
}

fn column_map(table: &TableSnapshot) -> BTreeMap<String, &ColumnSnapshot> {
    table
        .columns
        .iter()
        .map(|column| (column.name.clone(), column))
        .collect()
}

fn index_map(table: &TableSnapshot) -> BTreeMap<String, &IndexSnapshot> {
    table
        .indexes
        .iter()
        .map(|index| (index.name.clone(), index))
        .collect()
}

fn foreign_key_map(table: &TableSnapshot) -> BTreeMap<String, &ForeignKeySnapshot> {
    table
        .foreign_keys
        .iter()
        .map(|foreign_key| (foreign_key.name.clone(), foreign_key))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{
        diff_column_operations, diff_relational_operations, diff_schema_and_table_operations,
    };
    use crate::{
        AddColumn, AddForeignKey, AlterColumn, ColumnSnapshot, CreateIndex, CreateSchema,
        CreateTable, DropColumn, DropForeignKey, DropIndex, DropSchema, DropTable,
        ForeignKeySnapshot, IndexColumnSnapshot, IndexSnapshot, MigrationOperation, ModelSnapshot,
        SchemaSnapshot, TableSnapshot,
    };
    use mssql_orm_core::{IdentityMetadata, ReferentialAction, SqlServerType};

    fn column(
        name: &str,
        sql_type: SqlServerType,
        nullable: bool,
        max_length: Option<u32>,
    ) -> ColumnSnapshot {
        ColumnSnapshot::new(
            name,
            sql_type,
            nullable,
            name == "id",
            (name == "id").then(|| IdentityMetadata::new(1, 1)),
            None,
            None,
            name == "version",
            name != "id" && name != "version",
            name != "id" && name != "version",
            max_length,
            None,
            None,
        )
    }

    fn table(
        name: &str,
        columns: Vec<ColumnSnapshot>,
        indexes: Vec<IndexSnapshot>,
        foreign_keys: Vec<ForeignKeySnapshot>,
    ) -> TableSnapshot {
        TableSnapshot::new(
            name,
            columns,
            Some(format!("pk_{name}")),
            vec!["id".to_string()],
            indexes,
            foreign_keys,
        )
    }

    fn schema(name: &str, tables: Vec<TableSnapshot>) -> SchemaSnapshot {
        SchemaSnapshot::new(name, tables)
    }

    fn foreign_key(name: &str, schema: &str, table: &str, column: &str) -> ForeignKeySnapshot {
        ForeignKeySnapshot::new(
            name,
            vec![column.to_string()],
            schema,
            table,
            vec!["id".to_string()],
            ReferentialAction::NoAction,
            ReferentialAction::NoAction,
        )
    }

    #[test]
    fn schema_and_table_diff_keeps_safe_operation_order() {
        let previous = ModelSnapshot::new(vec![
            schema("legacy", vec![table("old_orders", vec![], vec![], vec![])]),
            schema("sales", vec![table("orders", vec![], vec![], vec![])]),
        ]);
        let current = ModelSnapshot::new(vec![
            schema(
                "reporting",
                vec![table("daily_sales", vec![], vec![], vec![])],
            ),
            schema("sales", vec![table("orders", vec![], vec![], vec![])]),
        ]);

        let operations = diff_schema_and_table_operations(&previous, &current);

        assert_eq!(
            operations,
            vec![
                MigrationOperation::CreateSchema(CreateSchema::new("reporting")),
                MigrationOperation::CreateTable(CreateTable::new(
                    "reporting",
                    table("daily_sales", vec![], vec![], vec![]),
                )),
                MigrationOperation::DropTable(DropTable::new("legacy", "old_orders")),
                MigrationOperation::DropSchema(DropSchema::new("legacy")),
            ]
        );
    }

    #[test]
    fn schema_and_table_diff_detects_table_creation_and_deletion_in_existing_schema() {
        let previous = ModelSnapshot::new(vec![schema(
            "sales",
            vec![
                table("customers", vec![], vec![], vec![]),
                table("orders", vec![], vec![], vec![]),
            ],
        )]);
        let current = ModelSnapshot::new(vec![schema(
            "sales",
            vec![
                table("customers", vec![], vec![], vec![]),
                table("invoices", vec![], vec![], vec![]),
            ],
        )]);

        let operations = diff_schema_and_table_operations(&previous, &current);

        assert_eq!(
            operations,
            vec![
                MigrationOperation::CreateTable(CreateTable::new(
                    "sales",
                    table("invoices", vec![], vec![], vec![]),
                )),
                MigrationOperation::DropTable(DropTable::new("sales", "orders")),
            ]
        );
    }

    #[test]
    fn schema_and_table_diff_returns_empty_for_equal_snapshots() {
        let snapshot = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table(
                "customers",
                vec![column("id", SqlServerType::BigInt, false, None)],
                vec![],
                vec![],
            )],
        )]);

        let operations = diff_schema_and_table_operations(&snapshot, &snapshot);

        assert!(operations.is_empty());
    }

    #[test]
    fn column_diff_detects_add_and_drop_in_shared_table() {
        let previous = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table(
                "customers",
                vec![
                    column("id", SqlServerType::BigInt, false, None),
                    column("email", SqlServerType::NVarChar, false, Some(160)),
                ],
                vec![],
                vec![],
            )],
        )]);
        let current = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table(
                "customers",
                vec![
                    column("id", SqlServerType::BigInt, false, None),
                    column("version", SqlServerType::RowVersion, false, None),
                ],
                vec![],
                vec![],
            )],
        )]);

        let operations = diff_column_operations(&previous, &current);

        assert_eq!(
            operations,
            vec![
                MigrationOperation::AddColumn(AddColumn::new(
                    "sales",
                    "customers",
                    column("version", SqlServerType::RowVersion, false, None),
                )),
                MigrationOperation::DropColumn(DropColumn::new("sales", "customers", "email")),
            ]
        );
    }

    #[test]
    fn column_diff_detects_basic_alterations() {
        let previous = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table(
                "customers",
                vec![column("email", SqlServerType::NVarChar, false, Some(160))],
                vec![],
                vec![],
            )],
        )]);
        let current = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table(
                "customers",
                vec![column("email", SqlServerType::NVarChar, true, Some(255))],
                vec![],
                vec![],
            )],
        )]);

        let operations = diff_column_operations(&previous, &current);

        assert_eq!(
            operations,
            vec![MigrationOperation::AlterColumn(AlterColumn::new(
                "sales",
                "customers",
                column("email", SqlServerType::NVarChar, false, Some(160)),
                column("email", SqlServerType::NVarChar, true, Some(255)),
            ))]
        );
    }

    #[test]
    fn column_diff_ignores_tables_handled_by_table_diff() {
        let previous = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table(
                "customers",
                vec![column("email", SqlServerType::NVarChar, false, Some(160))],
                vec![],
                vec![],
            )],
        )]);
        let current = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table(
                "orders",
                vec![column("customer_id", SqlServerType::BigInt, false, None)],
                vec![],
                vec![],
            )],
        )]);

        let operations = diff_column_operations(&previous, &current);

        assert!(operations.is_empty());
    }

    #[test]
    fn relational_diff_detects_index_and_foreign_key_additions_and_removals() {
        let previous = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table(
                "orders",
                vec![column("customer_id", SqlServerType::BigInt, false, None)],
                vec![IndexSnapshot::new(
                    "ix_orders_customer_id",
                    vec![IndexColumnSnapshot::asc("customer_id")],
                    false,
                )],
                vec![],
            )],
        )]);
        let current = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table(
                "orders",
                vec![column("customer_id", SqlServerType::BigInt, false, None)],
                vec![],
                vec![foreign_key(
                    "fk_orders_customer_id_customers",
                    "sales",
                    "customers",
                    "customer_id",
                )],
            )],
        )]);

        let operations = diff_relational_operations(&previous, &current);

        assert_eq!(
            operations,
            vec![
                MigrationOperation::DropIndex(DropIndex::new(
                    "sales",
                    "orders",
                    "ix_orders_customer_id",
                )),
                MigrationOperation::AddForeignKey(AddForeignKey::new(
                    "sales",
                    "orders",
                    foreign_key(
                        "fk_orders_customer_id_customers",
                        "sales",
                        "customers",
                        "customer_id",
                    ),
                )),
            ]
        );
    }

    #[test]
    fn relational_diff_recreates_foreign_key_when_definition_changes() {
        let previous = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table(
                "orders",
                vec![column("customer_id", SqlServerType::BigInt, false, None)],
                vec![],
                vec![foreign_key(
                    "fk_orders_customer_id_customers",
                    "dbo",
                    "customers",
                    "customer_id",
                )],
            )],
        )]);
        let current = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table(
                "orders",
                vec![column("customer_id", SqlServerType::BigInt, false, None)],
                vec![IndexSnapshot::new(
                    "ix_orders_customer_id",
                    vec![IndexColumnSnapshot::asc("customer_id")],
                    false,
                )],
                vec![foreign_key(
                    "fk_orders_customer_id_customers",
                    "sales",
                    "customers",
                    "customer_id",
                )],
            )],
        )]);

        let operations = diff_relational_operations(&previous, &current);

        assert_eq!(
            operations,
            vec![
                MigrationOperation::CreateIndex(CreateIndex::new(
                    "sales",
                    "orders",
                    IndexSnapshot::new(
                        "ix_orders_customer_id",
                        vec![IndexColumnSnapshot::asc("customer_id")],
                        false,
                    ),
                )),
                MigrationOperation::DropForeignKey(DropForeignKey::new(
                    "sales",
                    "orders",
                    "fk_orders_customer_id_customers",
                )),
                MigrationOperation::AddForeignKey(AddForeignKey::new(
                    "sales",
                    "orders",
                    foreign_key(
                        "fk_orders_customer_id_customers",
                        "sales",
                        "customers",
                        "customer_id",
                    ),
                )),
            ]
        );
    }

    #[test]
    fn relational_diff_recreates_index_when_composite_definition_changes() {
        let previous = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table(
                "orders",
                vec![
                    column("customer_id", SqlServerType::BigInt, false, None),
                    column("total_cents", SqlServerType::BigInt, false, None),
                ],
                vec![IndexSnapshot::new(
                    "ix_orders_customer_total",
                    vec![IndexColumnSnapshot::asc("customer_id")],
                    false,
                )],
                vec![],
            )],
        )]);
        let current = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table(
                "orders",
                vec![
                    column("customer_id", SqlServerType::BigInt, false, None),
                    column("total_cents", SqlServerType::BigInt, false, None),
                ],
                vec![IndexSnapshot::new(
                    "ix_orders_customer_total",
                    vec![
                        IndexColumnSnapshot::asc("customer_id"),
                        IndexColumnSnapshot::desc("total_cents"),
                    ],
                    false,
                )],
                vec![],
            )],
        )]);

        let operations = diff_relational_operations(&previous, &current);

        assert_eq!(
            operations,
            vec![
                MigrationOperation::DropIndex(DropIndex::new(
                    "sales",
                    "orders",
                    "ix_orders_customer_total",
                )),
                MigrationOperation::CreateIndex(CreateIndex::new(
                    "sales",
                    "orders",
                    IndexSnapshot::new(
                        "ix_orders_customer_total",
                        vec![
                            IndexColumnSnapshot::asc("customer_id"),
                            IndexColumnSnapshot::desc("total_cents"),
                        ],
                        false,
                    ),
                )),
            ]
        );
    }

    #[test]
    fn full_diff_on_minimal_snapshots_is_stable_when_combined() {
        let previous = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table(
                "customers",
                vec![
                    column("id", SqlServerType::BigInt, false, None),
                    column("email", SqlServerType::NVarChar, false, Some(160)),
                ],
                vec![],
                vec![],
            )],
        )]);
        let current = ModelSnapshot::new(vec![
            schema(
                "reporting",
                vec![table(
                    "daily_sales",
                    vec![column("id", SqlServerType::BigInt, false, None)],
                    vec![],
                    vec![],
                )],
            ),
            schema(
                "sales",
                vec![
                    table(
                        "customers",
                        vec![
                            column("id", SqlServerType::BigInt, false, None),
                            column("email", SqlServerType::NVarChar, true, Some(255)),
                            column("version", SqlServerType::RowVersion, false, None),
                        ],
                        vec![IndexSnapshot::new(
                            "ix_customers_email",
                            vec![IndexColumnSnapshot::asc("email")],
                            true,
                        )],
                        vec![],
                    ),
                    table(
                        "orders",
                        vec![
                            column("id", SqlServerType::BigInt, false, None),
                            column("customer_id", SqlServerType::BigInt, false, None),
                        ],
                        vec![IndexSnapshot::new(
                            "ix_orders_customer_id",
                            vec![IndexColumnSnapshot::asc("customer_id")],
                            false,
                        )],
                        vec![foreign_key(
                            "fk_orders_customer_id_customers",
                            "sales",
                            "customers",
                            "customer_id",
                        )],
                    ),
                ],
            ),
        ]);

        let mut operations = diff_schema_and_table_operations(&previous, &current);
        operations.extend(diff_column_operations(&previous, &current));
        operations.extend(diff_relational_operations(&previous, &current));

        assert_eq!(
            operations,
            vec![
                MigrationOperation::CreateSchema(CreateSchema::new("reporting")),
                MigrationOperation::CreateTable(CreateTable::new(
                    "reporting",
                    table(
                        "daily_sales",
                        vec![column("id", SqlServerType::BigInt, false, None)],
                        vec![],
                        vec![],
                    ),
                )),
                MigrationOperation::CreateTable(CreateTable::new(
                    "sales",
                    table(
                        "orders",
                        vec![
                            column("id", SqlServerType::BigInt, false, None),
                            column("customer_id", SqlServerType::BigInt, false, None),
                        ],
                        vec![IndexSnapshot::new(
                            "ix_orders_customer_id",
                            vec![IndexColumnSnapshot::asc("customer_id")],
                            false,
                        )],
                        vec![foreign_key(
                            "fk_orders_customer_id_customers",
                            "sales",
                            "customers",
                            "customer_id",
                        )],
                    ),
                )),
                MigrationOperation::AlterColumn(AlterColumn::new(
                    "sales",
                    "customers",
                    column("email", SqlServerType::NVarChar, false, Some(160)),
                    column("email", SqlServerType::NVarChar, true, Some(255)),
                )),
                MigrationOperation::AddColumn(AddColumn::new(
                    "sales",
                    "customers",
                    column("version", SqlServerType::RowVersion, false, None),
                )),
                MigrationOperation::CreateIndex(CreateIndex::new(
                    "sales",
                    "customers",
                    IndexSnapshot::new(
                        "ix_customers_email",
                        vec![IndexColumnSnapshot::asc("email")],
                        true,
                    ),
                )),
            ]
        );
    }
}

use crate::{
    AddColumn, AlterColumn, ColumnSnapshot, CreateSchema, CreateTable, DropColumn, DropSchema,
    DropTable, MigrationOperation, ModelSnapshot, SchemaSnapshot, TableSnapshot,
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

#[cfg(test)]
mod tests {
    use super::{diff_column_operations, diff_schema_and_table_operations};
    use crate::{
        AddColumn, AlterColumn, ColumnSnapshot, CreateSchema, CreateTable, DropColumn, DropSchema,
        DropTable, MigrationOperation, ModelSnapshot, SchemaSnapshot, TableSnapshot,
    };
    use mssql_orm_core::{IdentityMetadata, SqlServerType};

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

    fn table(name: &str, columns: Vec<ColumnSnapshot>) -> TableSnapshot {
        TableSnapshot::new(
            name,
            columns,
            Some(format!("pk_{name}")),
            vec!["id".to_string()],
            vec![],
        )
    }

    fn schema(name: &str, tables: Vec<TableSnapshot>) -> SchemaSnapshot {
        SchemaSnapshot::new(name, tables)
    }

    #[test]
    fn schema_and_table_diff_keeps_safe_operation_order() {
        let previous = ModelSnapshot::new(vec![
            schema("legacy", vec![table("old_orders", vec![])]),
            schema("sales", vec![table("orders", vec![])]),
        ]);
        let current = ModelSnapshot::new(vec![
            schema("reporting", vec![table("daily_sales", vec![])]),
            schema("sales", vec![table("orders", vec![])]),
        ]);

        let operations = diff_schema_and_table_operations(&previous, &current);

        assert_eq!(
            operations,
            vec![
                MigrationOperation::CreateSchema(CreateSchema::new("reporting")),
                MigrationOperation::CreateTable(CreateTable::new(
                    "reporting",
                    table("daily_sales", vec![]),
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
            vec![table("customers", vec![]), table("orders", vec![])],
        )]);
        let current = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table("customers", vec![]), table("invoices", vec![])],
        )]);

        let operations = diff_schema_and_table_operations(&previous, &current);

        assert_eq!(
            operations,
            vec![
                MigrationOperation::CreateTable(CreateTable::new(
                    "sales",
                    table("invoices", vec![])
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
            )],
        )]);
        let current = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table(
                "customers",
                vec![column("email", SqlServerType::NVarChar, true, Some(255))],
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
            )],
        )]);
        let current = ModelSnapshot::new(vec![schema(
            "sales",
            vec![table(
                "orders",
                vec![column("customer_id", SqlServerType::BigInt, false, None)],
            )],
        )]);

        let operations = diff_column_operations(&previous, &current);

        assert!(operations.is_empty());
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
            )],
        )]);
        let current = ModelSnapshot::new(vec![
            schema(
                "reporting",
                vec![table(
                    "daily_sales",
                    vec![column("id", SqlServerType::BigInt, false, None)],
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
                    ),
                    table(
                        "orders",
                        vec![column("id", SqlServerType::BigInt, false, None)],
                    ),
                ],
            ),
        ]);

        let mut operations = diff_schema_and_table_operations(&previous, &current);
        operations.extend(diff_column_operations(&previous, &current));

        assert_eq!(
            operations,
            vec![
                MigrationOperation::CreateSchema(CreateSchema::new("reporting")),
                MigrationOperation::CreateTable(CreateTable::new(
                    "reporting",
                    table(
                        "daily_sales",
                        vec![column("id", SqlServerType::BigInt, false, None)],
                    ),
                )),
                MigrationOperation::CreateTable(CreateTable::new(
                    "sales",
                    table(
                        "orders",
                        vec![column("id", SqlServerType::BigInt, false, None)]
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
            ]
        );
    }
}

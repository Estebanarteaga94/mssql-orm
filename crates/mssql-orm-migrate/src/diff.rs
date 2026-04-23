use crate::{
    CreateSchema, CreateTable, DropSchema, DropTable, MigrationOperation, ModelSnapshot,
    SchemaSnapshot, TableSnapshot,
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

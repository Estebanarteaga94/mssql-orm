use crate::{ColumnSnapshot, TableSnapshot};

/// Ordered migration operations emitted by the diff engine.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MigrationOperation {
    CreateSchema(CreateSchema),
    DropSchema(DropSchema),
    CreateTable(CreateTable),
    DropTable(DropTable),
    AddColumn(AddColumn),
    DropColumn(DropColumn),
    AlterColumn(AlterColumn),
}

impl MigrationOperation {
    pub fn schema_name(&self) -> &str {
        match self {
            Self::CreateSchema(operation) => &operation.schema_name,
            Self::DropSchema(operation) => &operation.schema_name,
            Self::CreateTable(operation) => &operation.schema_name,
            Self::DropTable(operation) => &operation.schema_name,
            Self::AddColumn(operation) => &operation.schema_name,
            Self::DropColumn(operation) => &operation.schema_name,
            Self::AlterColumn(operation) => &operation.schema_name,
        }
    }

    pub fn table_name(&self) -> Option<&str> {
        match self {
            Self::CreateSchema(_) | Self::DropSchema(_) => None,
            Self::CreateTable(operation) => Some(&operation.table.name),
            Self::DropTable(operation) => Some(&operation.table_name),
            Self::AddColumn(operation) => Some(&operation.table_name),
            Self::DropColumn(operation) => Some(&operation.table_name),
            Self::AlterColumn(operation) => Some(&operation.table_name),
        }
    }
}

/// Create a missing SQL Server schema.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateSchema {
    pub schema_name: String,
}

impl CreateSchema {
    pub fn new(schema_name: impl Into<String>) -> Self {
        Self {
            schema_name: schema_name.into(),
        }
    }
}

/// Drop a SQL Server schema that no longer exists in the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropSchema {
    pub schema_name: String,
}

impl DropSchema {
    pub fn new(schema_name: impl Into<String>) -> Self {
        Self {
            schema_name: schema_name.into(),
        }
    }
}

/// Create a table from its full snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateTable {
    pub schema_name: String,
    pub table: TableSnapshot,
}

impl CreateTable {
    pub fn new(schema_name: impl Into<String>, table: TableSnapshot) -> Self {
        Self {
            schema_name: schema_name.into(),
            table,
        }
    }
}

/// Drop a table by schema and name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropTable {
    pub schema_name: String,
    pub table_name: String,
}

impl DropTable {
    pub fn new(schema_name: impl Into<String>, table_name: impl Into<String>) -> Self {
        Self {
            schema_name: schema_name.into(),
            table_name: table_name.into(),
        }
    }
}

/// Add a new column to an existing table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AddColumn {
    pub schema_name: String,
    pub table_name: String,
    pub column: ColumnSnapshot,
}

impl AddColumn {
    pub fn new(
        schema_name: impl Into<String>,
        table_name: impl Into<String>,
        column: ColumnSnapshot,
    ) -> Self {
        Self {
            schema_name: schema_name.into(),
            table_name: table_name.into(),
            column,
        }
    }
}

/// Drop an existing column from a table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DropColumn {
    pub schema_name: String,
    pub table_name: String,
    pub column_name: String,
}

impl DropColumn {
    pub fn new(
        schema_name: impl Into<String>,
        table_name: impl Into<String>,
        column_name: impl Into<String>,
    ) -> Self {
        Self {
            schema_name: schema_name.into(),
            table_name: table_name.into(),
            column_name: column_name.into(),
        }
    }
}

/// Alter an existing column by comparing the previous and desired snapshots.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlterColumn {
    pub schema_name: String,
    pub table_name: String,
    pub previous: ColumnSnapshot,
    pub next: ColumnSnapshot,
}

impl AlterColumn {
    pub fn new(
        schema_name: impl Into<String>,
        table_name: impl Into<String>,
        previous: ColumnSnapshot,
        next: ColumnSnapshot,
    ) -> Self {
        Self {
            schema_name: schema_name.into(),
            table_name: table_name.into(),
            previous,
            next,
        }
    }
}

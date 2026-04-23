use mssql_orm_core::{IdentityMetadata, SqlServerType};

/// Serializable model snapshot shape used by future migration history artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModelSnapshot {
    pub schemas: Vec<SchemaSnapshot>,
}

impl ModelSnapshot {
    pub fn new(schemas: Vec<SchemaSnapshot>) -> Self {
        Self { schemas }
    }

    pub fn schema(&self, name: &str) -> Option<&SchemaSnapshot> {
        self.schemas.iter().find(|schema| schema.name == name)
    }
}

/// Snapshot of a SQL Server schema and the tables currently modeled inside it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SchemaSnapshot {
    pub name: String,
    pub tables: Vec<TableSnapshot>,
}

impl SchemaSnapshot {
    pub fn new(name: impl Into<String>, tables: Vec<TableSnapshot>) -> Self {
        Self {
            name: name.into(),
            tables,
        }
    }

    pub fn table(&self, name: &str) -> Option<&TableSnapshot> {
        self.tables.iter().find(|table| table.name == name)
    }
}

/// Snapshot of a SQL Server table with the minimum structural information needed
/// for the first migration diff passes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TableSnapshot {
    pub name: String,
    pub columns: Vec<ColumnSnapshot>,
    pub primary_key_name: Option<String>,
    pub primary_key_columns: Vec<String>,
    pub indexes: Vec<IndexSnapshot>,
}

impl TableSnapshot {
    pub fn new(
        name: impl Into<String>,
        columns: Vec<ColumnSnapshot>,
        primary_key_name: Option<String>,
        primary_key_columns: Vec<String>,
        indexes: Vec<IndexSnapshot>,
    ) -> Self {
        Self {
            name: name.into(),
            columns,
            primary_key_name,
            primary_key_columns,
            indexes,
        }
    }

    pub fn column(&self, name: &str) -> Option<&ColumnSnapshot> {
        self.columns.iter().find(|column| column.name == name)
    }

    pub fn index(&self, name: &str) -> Option<&IndexSnapshot> {
        self.indexes.iter().find(|index| index.name == name)
    }
}

/// Snapshot of a table column, aligned with the code-first metadata already
/// defined in `mssql-orm-core`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnSnapshot {
    pub name: String,
    pub sql_type: SqlServerType,
    pub nullable: bool,
    pub primary_key: bool,
    pub identity: Option<IdentityMetadata>,
    pub default_sql: Option<String>,
    pub computed_sql: Option<String>,
    pub rowversion: bool,
    pub insertable: bool,
    pub updatable: bool,
    pub max_length: Option<u32>,
    pub precision: Option<u8>,
    pub scale: Option<u8>,
}

impl ColumnSnapshot {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        sql_type: SqlServerType,
        nullable: bool,
        primary_key: bool,
        identity: Option<IdentityMetadata>,
        default_sql: Option<String>,
        computed_sql: Option<String>,
        rowversion: bool,
        insertable: bool,
        updatable: bool,
        max_length: Option<u32>,
        precision: Option<u8>,
        scale: Option<u8>,
    ) -> Self {
        Self {
            name: name.into(),
            sql_type,
            nullable,
            primary_key,
            identity,
            default_sql,
            computed_sql,
            rowversion,
            insertable,
            updatable,
            max_length,
            precision,
            scale,
        }
    }
}

/// Snapshot of an index, including the participating columns and sort order.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IndexSnapshot {
    pub name: String,
    pub columns: Vec<IndexColumnSnapshot>,
    pub unique: bool,
}

impl IndexSnapshot {
    pub fn new(name: impl Into<String>, columns: Vec<IndexColumnSnapshot>, unique: bool) -> Self {
        Self {
            name: name.into(),
            columns,
            unique,
        }
    }
}

/// Snapshot of a column inside an index definition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexColumnSnapshot {
    pub column_name: String,
    pub descending: bool,
}

impl IndexColumnSnapshot {
    pub fn asc(column_name: impl Into<String>) -> Self {
        Self {
            column_name: column_name.into(),
            descending: false,
        }
    }

    pub fn desc(column_name: impl Into<String>) -> Self {
        Self {
            column_name: column_name.into(),
            descending: true,
        }
    }
}

use mssql_orm_core::{
    ColumnMetadata, EntityMetadata, ForeignKeyMetadata, IdentityMetadata, IndexColumnMetadata,
    IndexMetadata, ReferentialAction, SqlServerType,
};
use std::collections::BTreeMap;

/// Serializable model snapshot shape used by future migration history artifacts.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ModelSnapshot {
    pub schemas: Vec<SchemaSnapshot>,
}

impl ModelSnapshot {
    pub fn new(schemas: Vec<SchemaSnapshot>) -> Self {
        Self { schemas }
    }

    pub fn from_entities(entities: &[&'static EntityMetadata]) -> Self {
        let mut schemas = BTreeMap::<String, Vec<&'static EntityMetadata>>::new();

        for entity in entities {
            schemas
                .entry(entity.schema.to_string())
                .or_default()
                .push(*entity);
        }

        let schemas = schemas
            .into_iter()
            .map(|(schema_name, mut entities)| {
                entities.sort_by(|left, right| left.table.cmp(right.table));

                SchemaSnapshot::new(
                    schema_name,
                    entities.into_iter().map(TableSnapshot::from).collect(),
                )
            })
            .collect();

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
    pub foreign_keys: Vec<ForeignKeySnapshot>,
}

impl TableSnapshot {
    pub fn new(
        name: impl Into<String>,
        columns: Vec<ColumnSnapshot>,
        primary_key_name: Option<String>,
        primary_key_columns: Vec<String>,
        indexes: Vec<IndexSnapshot>,
        foreign_keys: Vec<ForeignKeySnapshot>,
    ) -> Self {
        Self {
            name: name.into(),
            columns,
            primary_key_name,
            primary_key_columns,
            indexes,
            foreign_keys,
        }
    }

    pub fn column(&self, name: &str) -> Option<&ColumnSnapshot> {
        self.columns.iter().find(|column| column.name == name)
    }

    pub fn index(&self, name: &str) -> Option<&IndexSnapshot> {
        self.indexes.iter().find(|index| index.name == name)
    }

    pub fn foreign_key(&self, name: &str) -> Option<&ForeignKeySnapshot> {
        self.foreign_keys
            .iter()
            .find(|foreign_key| foreign_key.name == name)
    }
}

/// Snapshot of a table column, aligned with the code-first metadata already
/// defined in `mssql-orm-core`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnSnapshot {
    pub name: String,
    pub renamed_from: Option<String>,
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
            renamed_from: None,
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

    pub fn with_renamed_from(mut self, renamed_from: impl Into<String>) -> Self {
        self.renamed_from = Some(renamed_from.into());
        self
    }
}

impl From<&ColumnMetadata> for ColumnSnapshot {
    fn from(column: &ColumnMetadata) -> Self {
        Self {
            name: column.column_name.to_string(),
            renamed_from: column.renamed_from.map(str::to_owned),
            sql_type: column.sql_type,
            nullable: column.nullable,
            primary_key: column.primary_key,
            identity: column.identity,
            default_sql: column.default_sql.map(str::to_owned),
            computed_sql: column.computed_sql.map(str::to_owned),
            rowversion: column.rowversion,
            insertable: column.insertable,
            updatable: column.updatable,
            max_length: column.max_length,
            precision: column.precision,
            scale: column.scale,
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

impl From<&IndexMetadata> for IndexSnapshot {
    fn from(index: &IndexMetadata) -> Self {
        Self {
            name: index.name.to_string(),
            columns: index
                .columns
                .iter()
                .map(IndexColumnSnapshot::from)
                .collect(),
            unique: index.unique,
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

impl From<&IndexColumnMetadata> for IndexColumnSnapshot {
    fn from(column: &IndexColumnMetadata) -> Self {
        Self {
            column_name: column.column_name.to_string(),
            descending: column.descending,
        }
    }
}

/// Snapshot of a foreign key, including referenced target and referential actions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForeignKeySnapshot {
    pub name: String,
    pub columns: Vec<String>,
    pub referenced_schema: String,
    pub referenced_table: String,
    pub referenced_columns: Vec<String>,
    pub on_delete: ReferentialAction,
    pub on_update: ReferentialAction,
}

impl ForeignKeySnapshot {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        name: impl Into<String>,
        columns: Vec<String>,
        referenced_schema: impl Into<String>,
        referenced_table: impl Into<String>,
        referenced_columns: Vec<String>,
        on_delete: ReferentialAction,
        on_update: ReferentialAction,
    ) -> Self {
        Self {
            name: name.into(),
            columns,
            referenced_schema: referenced_schema.into(),
            referenced_table: referenced_table.into(),
            referenced_columns,
            on_delete,
            on_update,
        }
    }
}

impl From<&ForeignKeyMetadata> for ForeignKeySnapshot {
    fn from(foreign_key: &ForeignKeyMetadata) -> Self {
        Self {
            name: foreign_key.name.to_string(),
            columns: foreign_key
                .columns
                .iter()
                .map(|column| (*column).to_string())
                .collect(),
            referenced_schema: foreign_key.referenced_schema.to_string(),
            referenced_table: foreign_key.referenced_table.to_string(),
            referenced_columns: foreign_key
                .referenced_columns
                .iter()
                .map(|column| (*column).to_string())
                .collect(),
            on_delete: foreign_key.on_delete,
            on_update: foreign_key.on_update,
        }
    }
}

impl From<&EntityMetadata> for TableSnapshot {
    fn from(entity: &EntityMetadata) -> Self {
        Self {
            name: entity.table.to_string(),
            columns: entity.columns.iter().map(ColumnSnapshot::from).collect(),
            primary_key_name: entity.primary_key.name.map(str::to_owned),
            primary_key_columns: entity
                .primary_key
                .columns
                .iter()
                .map(|column| (*column).to_string())
                .collect(),
            indexes: entity.indexes.iter().map(IndexSnapshot::from).collect(),
            foreign_keys: entity
                .foreign_keys
                .iter()
                .map(ForeignKeySnapshot::from)
                .collect(),
        }
    }
}

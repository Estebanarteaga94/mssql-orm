//! Core contracts and shared types for the ORM.

use core::fmt;

/// Common error type placeholder for the workspace foundations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrmError {
    message: &'static str,
}

impl OrmError {
    pub const fn new(message: &'static str) -> Self {
        Self { message }
    }

    pub const fn message(&self) -> &'static str {
        self.message
    }
}

impl fmt::Display for OrmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message)
    }
}

impl std::error::Error for OrmError {}

/// Minimal crate identity metadata used while the rest of the model is defined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrateIdentity {
    pub name: &'static str,
    pub responsibility: &'static str,
}

pub const CRATE_IDENTITY: CrateIdentity = CrateIdentity {
    name: "mssql-orm-core",
    responsibility: "contracts, metadata, shared types and errors",
};

/// Stable contract implemented by persisted entities.
pub trait Entity: Sized + Send + Sync + 'static {
    fn metadata() -> &'static EntityMetadata;
}

/// SQL Server types supported by the metadata layer.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlServerType {
    BigInt,
    Int,
    SmallInt,
    TinyInt,
    Bit,
    UniqueIdentifier,
    Date,
    DateTime2,
    Decimal,
    Float,
    Money,
    NVarChar,
    VarBinary,
    RowVersion,
    Custom(&'static str),
}

/// Metadata for SQL Server identity columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IdentityMetadata {
    pub seed: i64,
    pub increment: i64,
}

impl IdentityMetadata {
    pub const fn new(seed: i64, increment: i64) -> Self {
        Self { seed, increment }
    }
}

/// Primary key metadata for an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimaryKeyMetadata {
    pub name: Option<&'static str>,
    pub columns: &'static [&'static str],
}

impl PrimaryKeyMetadata {
    pub const fn new(name: Option<&'static str>, columns: &'static [&'static str]) -> Self {
        Self { name, columns }
    }
}

/// Per-column metadata generated from entity definitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ColumnMetadata {
    pub rust_field: &'static str,
    pub column_name: &'static str,
    pub sql_type: SqlServerType,
    pub nullable: bool,
    pub primary_key: bool,
    pub identity: Option<IdentityMetadata>,
    pub default_sql: Option<&'static str>,
    pub computed_sql: Option<&'static str>,
    pub rowversion: bool,
    pub insertable: bool,
    pub updatable: bool,
    pub max_length: Option<u32>,
    pub precision: Option<u8>,
    pub scale: Option<u8>,
}

impl ColumnMetadata {
    pub const fn is_computed(&self) -> bool {
        self.computed_sql.is_some()
    }

    pub const fn is_generated_on_insert(&self) -> bool {
        self.identity.is_some()
            || self.default_sql.is_some()
            || self.rowversion
            || self.is_computed()
    }
}

/// Columns participating in an index and their sort direction.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexColumnMetadata {
    pub column_name: &'static str,
    pub descending: bool,
}

impl IndexColumnMetadata {
    pub const fn asc(column_name: &'static str) -> Self {
        Self {
            column_name,
            descending: false,
        }
    }

    pub const fn desc(column_name: &'static str) -> Self {
        Self {
            column_name,
            descending: true,
        }
    }
}

/// Index metadata attached to an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IndexMetadata {
    pub name: &'static str,
    pub columns: &'static [IndexColumnMetadata],
    pub unique: bool,
}

/// Delete/update behavior for foreign keys.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReferentialAction {
    NoAction,
    Cascade,
    SetNull,
    SetDefault,
}

/// Foreign key metadata attached to an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ForeignKeyMetadata {
    pub name: &'static str,
    pub columns: &'static [&'static str],
    pub referenced_schema: &'static str,
    pub referenced_table: &'static str,
    pub referenced_columns: &'static [&'static str],
    pub on_delete: ReferentialAction,
    pub on_update: ReferentialAction,
}

/// Static metadata describing an entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntityMetadata {
    pub rust_name: &'static str,
    pub schema: &'static str,
    pub table: &'static str,
    pub columns: &'static [ColumnMetadata],
    pub primary_key: PrimaryKeyMetadata,
    pub indexes: &'static [IndexMetadata],
    pub foreign_keys: &'static [ForeignKeyMetadata],
}

impl EntityMetadata {
    pub fn column(&self, column_name: &str) -> Option<&'static ColumnMetadata> {
        self.columns
            .iter()
            .find(|column| column.column_name == column_name)
    }

    pub fn field(&self, rust_field: &str) -> Option<&'static ColumnMetadata> {
        self.columns
            .iter()
            .find(|column| column.rust_field == rust_field)
    }

    pub fn primary_key_columns(&self) -> Vec<&'static ColumnMetadata> {
        self.columns
            .iter()
            .filter(|column| self.primary_key.columns.contains(&column.column_name))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        CRATE_IDENTITY, ColumnMetadata, Entity, EntityMetadata, ForeignKeyMetadata,
        IdentityMetadata, IndexColumnMetadata, IndexMetadata, OrmError, PrimaryKeyMetadata,
        ReferentialAction, SqlServerType,
    };

    const USER_COLUMNS: [ColumnMetadata; 3] = [
        ColumnMetadata {
            rust_field: "id",
            column_name: "id",
            sql_type: SqlServerType::BigInt,
            nullable: false,
            primary_key: true,
            identity: Some(IdentityMetadata::new(1, 1)),
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: false,
            updatable: false,
            max_length: None,
            precision: None,
            scale: None,
        },
        ColumnMetadata {
            rust_field: "email",
            column_name: "email",
            sql_type: SqlServerType::NVarChar,
            nullable: false,
            primary_key: false,
            identity: None,
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: true,
            updatable: true,
            max_length: Some(180),
            precision: None,
            scale: None,
        },
        ColumnMetadata {
            rust_field: "version",
            column_name: "version",
            sql_type: SqlServerType::RowVersion,
            nullable: false,
            primary_key: false,
            identity: None,
            default_sql: None,
            computed_sql: None,
            rowversion: true,
            insertable: false,
            updatable: false,
            max_length: None,
            precision: None,
            scale: None,
        },
    ];

    const USER_PRIMARY_KEY_COLUMNS: [&str; 1] = ["id"];

    const USER_INDEXES: [IndexMetadata; 1] = [IndexMetadata {
        name: "ux_users_email",
        columns: &[IndexColumnMetadata::asc("email")],
        unique: true,
    }];

    const USER_FOREIGN_KEYS: [ForeignKeyMetadata; 1] = [ForeignKeyMetadata {
        name: "fk_users_tenants",
        columns: &["tenant_id"],
        referenced_schema: "dbo",
        referenced_table: "tenants",
        referenced_columns: &["id"],
        on_delete: ReferentialAction::NoAction,
        on_update: ReferentialAction::NoAction,
    }];

    const USER_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "User",
        schema: "dbo",
        table: "users",
        columns: &USER_COLUMNS,
        primary_key: PrimaryKeyMetadata::new(Some("pk_users"), &USER_PRIMARY_KEY_COLUMNS),
        indexes: &USER_INDEXES,
        foreign_keys: &USER_FOREIGN_KEYS,
    };

    struct User;

    impl Entity for User {
        fn metadata() -> &'static EntityMetadata {
            &USER_METADATA
        }
    }

    #[test]
    fn exposes_foundation_identity() {
        assert_eq!(CRATE_IDENTITY.name, "mssql-orm-core");
    }

    #[test]
    fn preserves_error_message() {
        let error = OrmError::new("foundation");
        assert_eq!(error.message(), "foundation");
        assert_eq!(error.to_string(), "foundation");
    }

    #[test]
    fn entity_trait_exposes_static_metadata() {
        let metadata = User::metadata();

        assert_eq!(metadata.rust_name, "User");
        assert_eq!(metadata.schema, "dbo");
        assert_eq!(metadata.table, "users");
        assert_eq!(metadata.primary_key.name, Some("pk_users"));
        assert_eq!(metadata.indexes.len(), 1);
        assert_eq!(metadata.foreign_keys.len(), 1);
    }

    #[test]
    fn metadata_can_lookup_columns_by_field_and_name() {
        let metadata = User::metadata();

        assert_eq!(metadata.column("email"), metadata.field("email"));
        assert_eq!(
            metadata.column("version").map(|column| column.sql_type),
            Some(SqlServerType::RowVersion)
        );
        assert!(metadata.column("missing").is_none());
    }

    #[test]
    fn metadata_returns_primary_key_columns() {
        let metadata = User::metadata();
        let columns = metadata.primary_key_columns();

        assert_eq!(columns.len(), 1);
        assert_eq!(columns[0].column_name, "id");
        assert!(columns[0].primary_key);
    }

    #[test]
    fn column_metadata_marks_generated_values() {
        assert!(USER_COLUMNS[0].is_generated_on_insert());
        assert!(USER_COLUMNS[2].is_generated_on_insert());
        assert!(!USER_COLUMNS[1].is_generated_on_insert());
    }

    #[test]
    fn index_columns_preserve_sort_direction() {
        let descending = IndexColumnMetadata::desc("created_at");

        assert_eq!(
            USER_INDEXES[0].columns[0],
            IndexColumnMetadata::asc("email")
        );
        assert!(descending.descending);
        assert_eq!(descending.column_name, "created_at");
    }
}

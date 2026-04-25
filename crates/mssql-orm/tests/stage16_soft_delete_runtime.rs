use std::sync::Arc;

use mssql_orm::prelude::*;
use mssql_orm::query::CompiledQuery;
use mssql_orm::tiberius::MssqlConnection;

const TEST_CONNECTION_ENV: &str = "MSSQL_ORM_TEST_CONNECTION_STRING";
const KEEP_TABLES_ENV: &str = "KEEP_TEST_TABLES";
const TEST_TABLE_NAME: &str = "dbo.mssql_orm_soft_delete_runtime";

struct DeletedAtPolicy;

impl EntityPolicy for DeletedAtPolicy {
    const POLICY_NAME: &'static str = "soft_delete";
    const COLUMN_NAMES: &'static [&'static str] = &["deleted_at"];

    fn columns() -> &'static [ColumnMetadata] {
        const COLUMNS: &[ColumnMetadata] = &[ColumnMetadata {
            rust_field: "deleted_at",
            column_name: "deleted_at",
            renamed_from: None,
            sql_type: SqlServerType::DateTime2,
            nullable: true,
            primary_key: false,
            identity: None,
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: false,
            updatable: true,
            max_length: None,
            precision: None,
            scale: None,
        }];

        COLUMNS
    }
}

#[derive(Entity, Debug, Clone, PartialEq)]
#[orm(table = "mssql_orm_soft_delete_runtime", schema = "dbo", soft_delete = DeletedAtPolicy)]
struct SoftDeleteUser {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,
    #[orm(length = 120)]
    name: String,
}

#[derive(Insertable, Debug, Clone)]
#[orm(entity = SoftDeleteUser)]
struct NewSoftDeleteUser {
    name: String,
}

#[derive(DbContext)]
struct SoftDeleteDb {
    pub users: DbSet<SoftDeleteUser>,
}

struct FixedDeletedAtProvider;

struct DeletedAtRow {
    deleted_at: SqlValue,
}

impl SoftDeleteProvider for FixedDeletedAtProvider {
    fn apply(
        &self,
        context: SoftDeleteContext<'_>,
        changes: &mut Vec<ColumnValue>,
    ) -> Result<(), OrmError> {
        assert_eq!(context.operation, SoftDeleteOperation::Delete);
        changes.push(ColumnValue::new(
            "deleted_at",
            SqlValue::String("2026-04-25T00:00:00".to_string()),
        ));
        Ok(())
    }
}

impl FromRow for DeletedAtRow {
    fn from_row<R: Row>(row: &R) -> Result<Self, OrmError> {
        Ok(Self {
            deleted_at: row.get_required("deleted_at")?,
        })
    }
}

#[tokio::test]
async fn public_dbcontext_soft_delete_provider_routes_delete_through_update() -> Result<(), OrmError>
{
    let Some(connection_string) = test_connection_string() else {
        eprintln!(
            "skipping soft_delete runtime integration test because {TEST_CONNECTION_ENV} is not set"
        );
        return Ok(());
    };

    let keep_tables = keep_test_tables();
    reset_test_table(&connection_string).await?;

    let result = async {
        let db = SoftDeleteDb::connect(&connection_string).await?;
        let db = db.with_soft_delete_provider(Arc::new(FixedDeletedAtProvider));

        let inserted = db
            .users
            .insert(NewSoftDeleteUser {
                name: "Soft Delete".to_string(),
            })
            .await?;

        let deleted = db.users.delete(inserted.id).await?;
        assert!(deleted);

        // Query visibility is still pending, so the row remains visible until
        // DbSetQuery gains the implicit soft_delete filter.
        let found = db.users.find(inserted.id).await?;
        assert_eq!(found, Some(inserted.clone()));
        assert_eq!(db.users.query().count().await?, 1);

        let mut connection = MssqlConnection::connect(&connection_string).await?;
        let row = connection
            .fetch_one::<DeletedAtRow>(CompiledQuery::new(
                format!("SELECT [deleted_at] FROM {TEST_TABLE_NAME} WHERE [id] = @P1"),
                vec![SqlValue::I64(inserted.id)],
            ))
            .await?
            .map(|row| row.deleted_at)
            .expect("row should still exist after soft delete");

        assert_ne!(row, SqlValue::Null);

        Ok(())
    }
    .await;

    cleanup_test_table(&connection_string, keep_tables).await?;
    result
}

fn test_connection_string() -> Option<String> {
    std::env::var(TEST_CONNECTION_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn keep_test_tables() -> bool {
    matches!(
        std::env::var(KEEP_TABLES_ENV)
            .ok()
            .map(|value| value.trim().to_ascii_lowercase())
            .as_deref(),
        Some("1" | "true" | "yes" | "on")
    )
}

async fn reset_test_table(connection_string: &str) -> Result<(), OrmError> {
    let mut connection = MssqlConnection::connect(connection_string).await?;

    connection
        .execute(CompiledQuery::new(
            format!(
                "IF OBJECT_ID('{TEST_TABLE_NAME}', 'U') IS NOT NULL DROP TABLE {TEST_TABLE_NAME}"
            ),
            vec![],
        ))
        .await?;

    connection
        .execute(CompiledQuery::new(
            format!(
                "CREATE TABLE {TEST_TABLE_NAME} (\
                    id BIGINT IDENTITY(1,1) PRIMARY KEY,\
                    name NVARCHAR(120) NOT NULL,\
                    deleted_at DATETIME2 NULL\
                )"
            ),
            vec![],
        ))
        .await?;

    Ok(())
}

async fn cleanup_test_table(connection_string: &str, keep_tables: bool) -> Result<(), OrmError> {
    if keep_tables {
        return Ok(());
    }

    let mut connection = MssqlConnection::connect(connection_string).await?;
    connection
        .execute(CompiledQuery::new(
            format!(
                "IF OBJECT_ID('{TEST_TABLE_NAME}', 'U') IS NOT NULL DROP TABLE {TEST_TABLE_NAME}"
            ),
            vec![],
        ))
        .await?;
    Ok(())
}

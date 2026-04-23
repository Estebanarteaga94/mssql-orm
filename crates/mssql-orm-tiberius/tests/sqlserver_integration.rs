use chrono::{NaiveDate, NaiveDateTime};
use core::sync::atomic::{AtomicU64, Ordering};
use mssql_orm_core::{FromRow, OrmError, SqlValue};
use mssql_orm_query::CompiledQuery;
use mssql_orm_tiberius::MssqlConnection;

const TEST_CONNECTION_ENV: &str = "MSSQL_ORM_TEST_CONNECTION_STRING";

static NEXT_TABLE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, PartialEq, Eq)]
struct IntegrationUser {
    id: i32,
    email: String,
    active: bool,
    created_at: NaiveDateTime,
}

impl FromRow for IntegrationUser {
    fn from_row<R: mssql_orm_core::Row>(row: &R) -> Result<Self, OrmError> {
        Ok(Self {
            id: row.get_required_typed::<i32>("id")?,
            email: row.get_required_typed::<String>("email")?,
            active: row.get_required_typed::<bool>("active")?,
            created_at: row.get_required_typed::<NaiveDateTime>("created_at")?,
        })
    }
}

#[tokio::test]
async fn sqlserver_adapter_executes_and_maps_rows_against_real_database() -> Result<(), OrmError> {
    let Some(connection_string) = test_connection_string() else {
        eprintln!("skipping SQL Server integration test because {TEST_CONNECTION_ENV} is not set");
        return Ok(());
    };

    let mut connection = MssqlConnection::connect(&connection_string).await?;
    let table_name = unique_table_name();
    let first_created_at = fixed_datetime(2026, 4, 23, 10, 20, 30);
    let second_created_at = fixed_datetime(2026, 4, 23, 11, 21, 31);

    create_test_table(&mut connection, &table_name).await?;

    let insert_first = connection
        .execute(CompiledQuery::new(
            format!("INSERT INTO {table_name} (email, active, created_at) VALUES (@P1, @P2, @P3)"),
            vec![
                SqlValue::String("ana@example.com".to_string()),
                SqlValue::Bool(true),
                SqlValue::DateTime(first_created_at),
            ],
        ))
        .await?;

    let insert_second = connection
        .execute(CompiledQuery::new(
            format!("INSERT INTO {table_name} (email, active, created_at) VALUES (@P1, @P2, @P3)"),
            vec![
                SqlValue::String("bruno@example.com".to_string()),
                SqlValue::Bool(false),
                SqlValue::DateTime(second_created_at),
            ],
        ))
        .await?;

    assert_eq!(insert_first.total(), 1);
    assert_eq!(insert_second.total(), 1);

    let fetched_one = connection
        .fetch_one::<IntegrationUser>(CompiledQuery::new(
            format!(
                "SELECT TOP (1) id, email, active, created_at \
                 FROM {table_name} WHERE email = @P1 ORDER BY id ASC"
            ),
            vec![SqlValue::String("ana@example.com".to_string())],
        ))
        .await?;

    assert_eq!(
        fetched_one,
        Some(IntegrationUser {
            id: 1,
            email: "ana@example.com".to_string(),
            active: true,
            created_at: first_created_at,
        })
    );

    let fetched_all = connection
        .fetch_all::<IntegrationUser>(CompiledQuery::new(
            format!("SELECT id, email, active, created_at FROM {table_name} ORDER BY id ASC"),
            vec![],
        ))
        .await?;

    assert_eq!(
        fetched_all,
        vec![
            IntegrationUser {
                id: 1,
                email: "ana@example.com".to_string(),
                active: true,
                created_at: first_created_at,
            },
            IntegrationUser {
                id: 2,
                email: "bruno@example.com".to_string(),
                active: false,
                created_at: second_created_at,
            },
        ]
    );

    drop_test_table(&mut connection, &table_name).await?;

    Ok(())
}

#[tokio::test]
async fn sqlserver_adapter_surfaces_missing_rows_as_none() -> Result<(), OrmError> {
    let Some(connection_string) = test_connection_string() else {
        eprintln!("skipping SQL Server integration test because {TEST_CONNECTION_ENV} is not set");
        return Ok(());
    };

    let mut connection = MssqlConnection::connect(&connection_string).await?;
    let table_name = unique_table_name();

    create_test_table(&mut connection, &table_name).await?;

    let fetched = connection
        .fetch_one::<IntegrationUser>(CompiledQuery::new(
            format!(
                "SELECT TOP (1) id, email, active, created_at \
                 FROM {table_name} WHERE email = @P1 ORDER BY id ASC"
            ),
            vec![SqlValue::String("missing@example.com".to_string())],
        ))
        .await?;

    assert_eq!(fetched, None);

    drop_test_table(&mut connection, &table_name).await?;

    Ok(())
}

fn test_connection_string() -> Option<String> {
    std::env::var(TEST_CONNECTION_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn fixed_datetime(
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(year, month, day)
        .unwrap()
        .and_hms_opt(hour, minute, second)
        .unwrap()
}

fn unique_table_name() -> String {
    let table_id = NEXT_TABLE_ID.fetch_add(1, Ordering::Relaxed);
    let process_id = std::process::id();

    format!("tempdb.dbo.mssql_orm_integration_{process_id}_{table_id}")
}

async fn create_test_table(
    connection: &mut MssqlConnection,
    table_name: &str,
) -> Result<(), OrmError> {
    connection
        .execute(CompiledQuery::new(
            format!(
                "CREATE TABLE {table_name} (\
                    id INT IDENTITY(1,1) PRIMARY KEY,\
                    email NVARCHAR(180) NOT NULL,\
                    active BIT NOT NULL,\
                    created_at DATETIME2 NOT NULL\
                )"
            ),
            vec![],
        ))
        .await?;

    Ok(())
}

async fn drop_test_table(
    connection: &mut MssqlConnection,
    table_name: &str,
) -> Result<(), OrmError> {
    connection
        .execute(CompiledQuery::new(
            format!("IF OBJECT_ID('{table_name}', 'U') IS NOT NULL DROP TABLE {table_name}"),
            vec![],
        ))
        .await?;

    Ok(())
}

use mssql_orm::prelude::*;

const TEST_CONNECTION_ENV: &str = "MSSQL_ORM_TEST_CONNECTION_STRING";
const KEEP_TABLES_ENV: &str = "KEEP_TEST_TABLES";
const TEST_TABLE_NAME: &str = "dbo.mssql_orm_public_raw_sql";

#[derive(Entity, Debug, Clone, PartialEq)]
#[orm(table = "mssql_orm_public_raw_sql", schema = "dbo")]
struct RawSqlAnchor {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,
    #[orm(length = 120)]
    name: String,
    active: bool,
}

#[derive(DbContext)]
struct RawSqlDb {
    pub anchors: DbSet<RawSqlAnchor>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RawSqlUser {
    id: i64,
    name: String,
    active: bool,
}

impl FromRow for RawSqlUser {
    fn from_row<R: Row>(row: &R) -> Result<Self, OrmError> {
        Ok(Self {
            id: row.get_required_typed::<i64>("id")?,
            name: row.get_required_typed::<String>("name")?,
            active: row.get_required_typed::<bool>("active")?,
        })
    }
}

#[tokio::test]
async fn public_raw_sql_api_roundtrips_against_real_sql_server() -> Result<(), OrmError> {
    let Some(connection_string) = test_connection_string() else {
        eprintln!(
            "skipping public raw SQL integration test because {TEST_CONNECTION_ENV} is not set"
        );
        return Ok(());
    };

    let keep_tables = keep_test_tables();
    let db = RawSqlDb::connect(&connection_string).await?;

    reset_test_table(&db).await?;
    announce_test_table(keep_tables);

    let result = async {
        let insert_first = db
            .raw_exec(format!(
                "INSERT INTO {TEST_TABLE_NAME} (name, active) VALUES (@P1, @P2)"
            ))
            .params(("Ana", true))
            .execute()
            .await?;
        assert_eq!(insert_first.total(), 1);

        let insert_second = db
            .raw_exec(format!(
                "INSERT INTO {TEST_TABLE_NAME} (name, active) VALUES (@P1, @P2)"
            ))
            .params(("Bruno", false))
            .execute()
            .await?;
        assert_eq!(insert_second.total(), 1);

        let active_users = db
            .raw::<RawSqlUser>(format!(
                "SELECT id, name, active FROM {TEST_TABLE_NAME} \
                 WHERE active = @P1 ORDER BY id ASC"
            ))
            .param(true)
            .query_hint(QueryHint::Recompile)
            .all()
            .await?;

        assert_eq!(active_users.len(), 1);
        assert_eq!(active_users[0].name, "Ana");
        assert!(active_users[0].active);

        let updated = db
            .raw_exec(format!(
                "UPDATE {TEST_TABLE_NAME} SET active = @P1 WHERE name = @P2"
            ))
            .params((true, "Bruno"))
            .execute()
            .await?;
        assert_eq!(updated.total(), 1);

        let first = db
            .raw::<RawSqlUser>(format!(
                "SELECT TOP (1) id, name, active FROM {TEST_TABLE_NAME} \
                 WHERE name = @P1 ORDER BY id ASC"
            ))
            .param("Bruno")
            .first()
            .await?;

        assert_eq!(
            first,
            Some(RawSqlUser {
                id: 2,
                name: "Bruno".to_string(),
                active: true,
            })
        );

        let all_users = db
            .raw::<RawSqlUser>(format!(
                "SELECT id, name, active FROM {TEST_TABLE_NAME} ORDER BY id ASC"
            ))
            .all()
            .await?;

        assert_eq!(
            all_users,
            vec![
                RawSqlUser {
                    id: 1,
                    name: "Ana".to_string(),
                    active: true,
                },
                RawSqlUser {
                    id: 2,
                    name: "Bruno".to_string(),
                    active: true,
                },
            ]
        );

        Ok(())
    }
    .await;

    cleanup_test_table(&db, keep_tables).await?;

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

fn announce_test_table(keep_tables: bool) {
    if keep_tables {
        eprintln!(
            "keeping public raw SQL integration table `{TEST_TABLE_NAME}` because {KEEP_TABLES_ENV}=1"
        );
    } else {
        eprintln!("created public raw SQL integration table `{TEST_TABLE_NAME}`");
    }
}

async fn reset_test_table(db: &RawSqlDb) -> Result<(), OrmError> {
    db.raw_exec(format!(
        "IF OBJECT_ID('{TEST_TABLE_NAME}', 'U') IS NOT NULL DROP TABLE {TEST_TABLE_NAME}"
    ))
    .execute()
    .await?;

    db.raw_exec(format!(
        "CREATE TABLE {TEST_TABLE_NAME} (\
            id BIGINT IDENTITY(1,1) PRIMARY KEY,\
            name NVARCHAR(120) NOT NULL,\
            active BIT NOT NULL\
        )"
    ))
    .execute()
    .await?;

    Ok(())
}

async fn cleanup_test_table(db: &RawSqlDb, keep_tables: bool) -> Result<(), OrmError> {
    if keep_tables {
        return Ok(());
    }

    db.raw_exec(format!(
        "IF OBJECT_ID('{TEST_TABLE_NAME}', 'U') IS NOT NULL DROP TABLE {TEST_TABLE_NAME}"
    ))
    .execute()
    .await?;

    Ok(())
}

use mssql_orm::prelude::*;
use mssql_orm::query::{CompiledQuery, Expr, Predicate, SelectQuery};
use mssql_orm::tiberius::MssqlConnection;

const TEST_CONNECTION_ENV: &str = "MSSQL_ORM_TEST_CONNECTION_STRING";
const TEST_TABLE_NAME: &str = "dbo.mssql_orm_public_crud";

#[derive(Entity, Debug, Clone, PartialEq)]
#[orm(table = "mssql_orm_public_crud", schema = "dbo")]
struct PublicCrudUser {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,
    #[orm(length = 120)]
    name: String,
    active: bool,
}

impl FromRow for PublicCrudUser {
    fn from_row<R: Row>(row: &R) -> Result<Self, OrmError> {
        Ok(Self {
            id: row.get_required_typed::<i64>("id")?,
            name: row.get_required_typed::<String>("name")?,
            active: row.get_required_typed::<bool>("active")?,
        })
    }
}

#[derive(Insertable, Debug, Clone)]
#[orm(entity = PublicCrudUser)]
struct NewPublicCrudUser {
    name: String,
    active: bool,
}

#[derive(Changeset, Debug, Clone)]
#[orm(entity = PublicCrudUser)]
struct UpdatePublicCrudUser {
    name: Option<String>,
    active: Option<bool>,
}

#[derive(DbContext)]
struct PublicCrudDb {
    pub users: DbSet<PublicCrudUser>,
}

#[tokio::test]
async fn public_dbset_crud_api_roundtrips_against_real_sql_server() -> Result<(), OrmError> {
    let Some(connection_string) = test_connection_string() else {
        eprintln!("skipping public CRUD integration test because {TEST_CONNECTION_ENV} is not set");
        return Ok(());
    };

    reset_test_table(&connection_string).await?;

    let result = async {
        let db = PublicCrudDb::connect(&connection_string).await?;

        let inserted = db
            .users
            .insert(NewPublicCrudUser {
                name: "Ana".to_string(),
                active: true,
            })
            .await?;

        assert!(inserted.id > 0);
        assert_eq!(inserted.name, "Ana");
        assert!(inserted.active);

        let found = db.users.find(inserted.id).await?;
        assert_eq!(found, Some(inserted.clone()));

        let count = db.users.query().count().await?;
        assert_eq!(count, 1);

        let all = db.users.query().all().await?;
        assert_eq!(all, vec![inserted.clone()]);

        let filtered = db
            .users
            .query_with(
                SelectQuery::from_entity::<PublicCrudUser>().filter(Predicate::eq(
                    Expr::from(PublicCrudUser::id),
                    Expr::value(SqlValue::I64(inserted.id)),
                )),
            )
            .first()
            .await?;
        assert_eq!(filtered, Some(inserted.clone()));

        let updated = db
            .users
            .update(
                inserted.id,
                UpdatePublicCrudUser {
                    name: Some("Ana Maria".to_string()),
                    active: Some(false),
                },
            )
            .await?;
        assert_eq!(
            updated,
            Some(PublicCrudUser {
                id: inserted.id,
                name: "Ana Maria".to_string(),
                active: false,
            })
        );

        let updated_found = db.users.find(inserted.id).await?;
        assert_eq!(updated_found, updated);

        let deleted = db.users.delete(inserted.id).await?;
        assert!(deleted);

        let after_delete = db.users.find(inserted.id).await?;
        assert_eq!(after_delete, None);
        assert_eq!(db.users.query().count().await?, 0);

        let deleted_again = db.users.delete(inserted.id).await?;
        assert!(!deleted_again);

        Ok(())
    }
    .await;

    cleanup_test_table(&connection_string).await?;

    result
}

fn test_connection_string() -> Option<String> {
    std::env::var(TEST_CONNECTION_ENV)
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
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
                    active BIT NOT NULL\
                )"
            ),
            vec![],
        ))
        .await?;

    Ok(())
}

async fn cleanup_test_table(connection_string: &str) -> Result<(), OrmError> {
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

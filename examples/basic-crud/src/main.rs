use mssql_orm::prelude::*;
use mssql_orm::query::{CompiledQuery, Expr, Predicate, SelectQuery};
use mssql_orm::tiberius::MssqlConnection;

const TABLE_NAME: &str = "dbo.basic_crud_users";

#[derive(Entity, Debug, Clone, PartialEq)]
#[orm(table = "basic_crud_users", schema = "dbo")]
struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,
    #[orm(length = 120)]
    name: String,
    active: bool,
}

impl FromRow for User {
    fn from_row<R: Row>(row: &R) -> Result<Self, OrmError> {
        Ok(Self {
            id: row.get_required_typed::<i64>("id")?,
            name: row.get_required_typed::<String>("name")?,
            active: row.get_required_typed::<bool>("active")?,
        })
    }
}

#[derive(Insertable, Debug, Clone)]
#[orm(entity = User)]
struct NewUser {
    name: String,
    active: bool,
}

#[derive(Changeset, Debug, Clone)]
#[orm(entity = User)]
struct UpdateUser {
    name: Option<String>,
    active: Option<bool>,
}

#[derive(DbContext)]
struct AppDb {
    pub users: DbSet<User>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let connection_string = std::env::var("DATABASE_URL")
        .map_err(|_| "DATABASE_URL no está configurada para el ejemplo basic-crud")?;

    reset_table(&connection_string).await?;

    let result = async {
        let db = AppDb::connect(&connection_string).await?;

        let inserted = db
            .users
            .insert(NewUser {
                name: "Ana".to_string(),
                active: true,
            })
            .await?;
        println!("inserted: {inserted:?}");

        let found = db.users.find(inserted.id).await?;
        println!("found: {found:?}");

        let count = db.users.query().count().await?;
        println!("count after insert: {count}");

        let all = db.users.query().all().await?;
        println!("all: {all:?}");

        let first = db
            .users
            .query_with(
                SelectQuery::from_entity::<User>().filter(Predicate::eq(
                    Expr::from(User::id),
                    Expr::value(SqlValue::I64(inserted.id)),
                )),
            )
            .first()
            .await?;
        println!("filtered first: {first:?}");

        let updated = db
            .users
            .update(
                inserted.id,
                UpdateUser {
                    name: Some("Ana Maria".to_string()),
                    active: Some(false),
                },
            )
            .await?;
        println!("updated: {updated:?}");

        let deleted = db.users.delete(inserted.id).await?;
        println!("deleted: {deleted}");

        let after_delete = db.users.find(inserted.id).await?;
        println!("after delete: {after_delete:?}");

        Ok::<(), OrmError>(())
    }
    .await;

    cleanup_table(&connection_string).await?;

    result.map_err(|error| -> Box<dyn std::error::Error> { Box::new(error) })
}

async fn reset_table(connection_string: &str) -> Result<(), OrmError> {
    let mut connection = MssqlConnection::connect(connection_string).await?;

    connection
        .execute(CompiledQuery::new(
            format!("IF OBJECT_ID('{TABLE_NAME}', 'U') IS NOT NULL DROP TABLE {TABLE_NAME}"),
            vec![],
        ))
        .await?;

    connection
        .execute(CompiledQuery::new(
            format!(
                "CREATE TABLE {TABLE_NAME} (\
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

async fn cleanup_table(connection_string: &str) -> Result<(), OrmError> {
    let mut connection = MssqlConnection::connect(connection_string).await?;

    connection
        .execute(CompiledQuery::new(
            format!("IF OBJECT_ID('{TABLE_NAME}', 'U') IS NOT NULL DROP TABLE {TABLE_NAME}"),
            vec![],
        ))
        .await?;

    Ok(())
}

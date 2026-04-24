use mssql_orm::prelude::*;

#[derive(Entity, Debug, Clone)]
#[orm(table = "users", schema = "dbo")]
struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,
}

impl FromRow for User {
    fn from_row<R: Row>(row: &R) -> Result<Self, OrmError> {
        Ok(Self {
            id: row.get_required_typed::<i64>("id")?,
        })
    }
}

#[derive(Entity, Debug, Clone)]
#[orm(table = "orders", schema = "dbo")]
struct Order {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,
}

impl FromRow for Order {
    fn from_row<R: Row>(row: &R) -> Result<Self, OrmError> {
        Ok(Self {
            id: row.get_required_typed::<i64>("id")?,
        })
    }
}

#[derive(DbContext, Debug, Clone)]
struct AppDbContext {
    pub orders: DbSet<Order>,
}

fn main() {
    let _query = |db: &AppDbContext| {
        let _ = User::query(db);
    };
}

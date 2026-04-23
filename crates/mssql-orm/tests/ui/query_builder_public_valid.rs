use mssql_orm::prelude::*;
use mssql_orm::query::{Expr, Predicate};

#[derive(Entity, Debug, Clone)]
#[orm(table = "users", schema = "dbo")]
struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(length = 180)]
    email: String,

    active: bool,
}

impl FromRow for User {
    fn from_row<R: Row>(row: &R) -> Result<Self, OrmError> {
        Ok(Self {
            id: row.get_required_typed::<i64>("id")?,
            email: row.get_required_typed::<String>("email")?,
            active: row.get_required_typed::<bool>("active")?,
        })
    }
}

#[derive(Entity, Debug, Clone)]
#[orm(table = "orders", schema = "dbo")]
struct Order {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    user_id: i64,
    total_cents: i64,
}

impl FromRow for Order {
    fn from_row<R: Row>(row: &R) -> Result<Self, OrmError> {
        Ok(Self {
            id: row.get_required_typed::<i64>("id")?,
            user_id: row.get_required_typed::<i64>("user_id")?,
            total_cents: row.get_required_typed::<i64>("total_cents")?,
        })
    }
}

#[derive(DbContext, Debug, Clone)]
struct AppDbContext {
    pub users: DbSet<User>,
    pub orders: DbSet<Order>,
}

fn accept_query(_query: DbSetQuery<User>) {}

fn main() {
    let _build_query = |db: &AppDbContext| {
        accept_query(
            db.users
                .query()
                .inner_join::<Order>(Predicate::eq(
                    Expr::from(User::id),
                    Expr::from(Order::user_id),
                ))
                .left_join::<Order>(Order::total_cents.gt(0_i64))
                .filter(User::active.eq(true).and(User::email.contains("@example.com")))
                .order_by(User::email.asc())
                .limit(10)
                .paginate(PageRequest::new(2, 10)),
        );
    };
}

use mssql_orm::prelude::*;

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

#[derive(DbContext, Debug, Clone)]
struct AppDbContext {
    pub users: DbSet<User>,
}

fn accept_query(_query: DbSetQuery<User>) {}

fn main() {
    let _build_query = |db: &AppDbContext| {
        accept_query(
            db.users
                .query()
                .filter(User::active.eq(true).and(User::email.contains("@example.com")))
                .order_by(User::email.asc())
                .limit(10)
                .paginate(PageRequest::new(2, 10)),
        );
    };
}

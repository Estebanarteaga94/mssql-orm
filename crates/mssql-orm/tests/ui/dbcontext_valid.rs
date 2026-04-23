use mssql_orm::prelude::*;

#[derive(Entity, Debug, Clone)]
#[orm(table = "users")]
struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(length = 180)]
    email: String,
}

#[derive(DbContext, Debug, Clone)]
struct AppDbContext {
    pub users: DbSet<User>,
}

fn main() {
    let _connect = AppDbContext::connect;
    let _from_shared = AppDbContext::from_shared_connection;
    let _from_connection = AppDbContext::from_connection;
    let _transaction = AppDbContext::transaction::<
        fn(AppDbContext) -> std::future::Ready<Result<(), OrmError>>,
        std::future::Ready<Result<(), OrmError>>,
        (),
    >;
}

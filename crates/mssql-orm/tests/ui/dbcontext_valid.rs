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

#[derive(TenantContext)]
struct CurrentTenant {
    tenant_id: i64,
}

fn main() {
    let _connect = AppDbContext::connect;
    let _connect_with_options = AppDbContext::connect_with_options;
    let _connect_with_config = AppDbContext::connect_with_config;
    let _from_shared = AppDbContext::from_shared_connection;
    let _from_connection = AppDbContext::from_connection;
    let _with_soft_delete_provider = AppDbContext::with_soft_delete_provider;
    let _with_soft_delete_request_values = AppDbContext::with_soft_delete_request_values;
    let _clear_soft_delete_request_values = AppDbContext::clear_soft_delete_request_values;
    let _with_tenant = AppDbContext::with_tenant::<CurrentTenant>;
    let _clear_tenant = AppDbContext::clear_tenant;
    let _shared_with_tenant = SharedConnection::with_tenant::<CurrentTenant>;
    let _shared_clear_tenant = SharedConnection::clear_tenant;
    let _db_set: fn(&AppDbContext) -> &DbSet<User> =
        <AppDbContext as mssql_orm::DbContextEntitySet<User>>::db_set;
    let _entity_metadata: fn() -> &'static [&'static mssql_orm::EntityMetadata] =
        <AppDbContext as mssql_orm::MigrationModelSource>::entity_metadata;
    let _options = MssqlOperationalOptions::new()
        .with_timeouts(MssqlTimeoutOptions::new())
        .with_pool(MssqlPoolOptions::bb8(8));
    let _config = MssqlConnectionConfig::from_connection_string_with_options(
        "server=tcp:localhost,1433;database=master;user=sa;password=Password123;TrustServerCertificate=true",
        MssqlOperationalOptions::new(),
    )
    .unwrap();
    let _transaction = AppDbContext::transaction::<
        fn(AppDbContext) -> std::future::Ready<Result<(), OrmError>>,
        std::future::Ready<Result<(), OrmError>>,
        (),
    >;
}

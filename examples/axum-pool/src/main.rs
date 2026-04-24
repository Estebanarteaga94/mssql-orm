use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::get,
};
use mssql_orm::prelude::*;
use mssql_orm::query::CompiledQuery;
use serde::{Deserialize, Serialize};
use std::{net::SocketAddr, time::Duration};
use tracing::{error, info};

const TABLE_NAME: &str = "dbo.axum_pool_users";

#[derive(Entity, Debug, Clone, Serialize)]
#[orm(table = "axum_pool_users", schema = "dbo")]
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

#[derive(Insertable, Debug, Clone, Deserialize)]
#[orm(entity = User)]
struct NewUser {
    name: String,
    active: bool,
}

#[derive(DbContext, Clone)]
struct AppDb {
    pub users: DbSet<User>,
}

#[derive(Clone)]
struct AppState {
    db: AppDb,
}

#[derive(Debug, Serialize)]
struct ApiError {
    error: String,
}

struct HttpError {
    status: StatusCode,
    message: String,
}

impl HttpError {
    fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }
}

impl From<OrmError> for HttpError {
    fn from(error: OrmError) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, error.message())
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (
            self.status,
            Json(ApiError {
                error: self.message,
            }),
        )
            .into_response()
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_tracing();

    let settings = AppSettings::from_env()?;
    let pool = build_pool(&settings).await?;
    ensure_demo_table(&pool).await?;

    let db = AppDb::from_pool(pool);
    db.health_check().await?;

    let app = Router::new()
        .route("/healthz", get(healthz))
        .route("/users", get(list_users).post(create_user))
        .with_state(AppState { db });

    let listener = tokio::net::TcpListener::bind(settings.bind_addr).await?;
    info!("axum example listening on {}", listener.local_addr()?);

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn healthz(State(state): State<AppState>) -> Result<StatusCode, HttpError> {
    state
        .db
        .health_check()
        .await
        .map_err(|error| HttpError::new(StatusCode::SERVICE_UNAVAILABLE, error.message()))?;

    Ok(StatusCode::NO_CONTENT)
}

async fn list_users(State(state): State<AppState>) -> Result<Json<Vec<User>>, HttpError> {
    let users = state
        .db
        .users
        .query()
        .order_by(User::id.asc())
        .all()
        .await?;
    Ok(Json(users))
}

async fn create_user(
    State(state): State<AppState>,
    Json(payload): Json<NewUser>,
) -> Result<(StatusCode, Json<User>), HttpError> {
    let saved = state.db.users.insert(payload).await?;
    Ok((StatusCode::CREATED, Json(saved)))
}

fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "info,axum_pool=debug,mssql_orm=debug".into());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();
}

#[derive(Clone)]
struct AppSettings {
    bind_addr: SocketAddr,
    database_url: String,
    operational_options: MssqlOperationalOptions,
}

impl AppSettings {
    fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = std::env::var("DATABASE_URL")
            .map_err(|_| "DATABASE_URL no está configurada para el ejemplo axum-pool")?;
        let bind_addr = std::env::var("APP_ADDR")
            .unwrap_or_else(|_| "127.0.0.1:3000".to_string())
            .parse()?;

        let operational_options = MssqlOperationalOptions::new()
            .with_timeouts(
                MssqlTimeoutOptions::new()
                    .with_connect_timeout(Duration::from_secs(5))
                    .with_query_timeout(Duration::from_secs(10))
                    .with_acquire_timeout(Duration::from_secs(2)),
            )
            .with_retry(MssqlRetryOptions::enabled(
                2,
                Duration::from_millis(100),
                Duration::from_millis(500),
            ))
            .with_tracing(
                MssqlTracingOptions::enabled()
                    .with_parameter_logging(MssqlParameterLogMode::Redacted),
            )
            .with_slow_query(
                MssqlSlowQueryOptions::enabled(Duration::from_millis(250))
                    .with_parameter_logging(MssqlParameterLogMode::Redacted),
            )
            .with_health(
                MssqlHealthCheckOptions::enabled(MssqlHealthCheckQuery::SelectOne)
                    .with_timeout(Duration::from_secs(2)),
            )
            .with_pool(
                MssqlPoolOptions::bb8(16)
                    .with_min_idle(4)
                    .with_acquire_timeout(Duration::from_secs(2))
                    .with_idle_timeout(Duration::from_secs(300))
                    .with_max_lifetime(Duration::from_secs(1800)),
            );

        Ok(Self {
            bind_addr,
            database_url,
            operational_options,
        })
    }
}

async fn build_pool(settings: &AppSettings) -> Result<MssqlPool, OrmError> {
    let config = MssqlConnectionConfig::from_connection_string_with_options(
        &settings.database_url,
        settings.operational_options.clone(),
    )?;

    MssqlPool::builder()
        .with_pool_options(settings.operational_options.pool)
        .connect_with_config(config)
        .await
}

async fn ensure_demo_table(pool: &MssqlPool) -> Result<(), OrmError> {
    let mut connection = pool.acquire().await?;

    connection
        .execute(CompiledQuery::new(
            format!(
                "IF OBJECT_ID('{TABLE_NAME}', 'U') IS NULL \
                 BEGIN \
                    CREATE TABLE {TABLE_NAME} (\
                        id BIGINT IDENTITY(1,1) PRIMARY KEY,\
                        name NVARCHAR(120) NOT NULL,\
                        active BIT NOT NULL CONSTRAINT DF_axum_pool_users_active DEFAULT 1\
                    )\
                 END"
            ),
            vec![],
        ))
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    if let Err(error) = tokio::signal::ctrl_c().await {
        error!(message = "failed to install CTRL+C handler", %error);
    }
}

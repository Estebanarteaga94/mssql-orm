use axum::Router;
use mssql_orm::prelude::{
    MssqlConnectionConfig, MssqlHealthCheckOptions, MssqlHealthCheckQuery, MssqlOperationalOptions,
    MssqlParameterLogMode, MssqlPoolOptions, MssqlRetryOptions, MssqlSlowQueryOptions,
    MssqlTimeoutOptions, MssqlTracingOptions, OrmError,
};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

pub mod domain;
pub mod queries;

pub use domain::{TodoItem, TodoList, User as TodoUser};
pub use queries::{
    list_items_page_query, open_items_count_query, open_items_preview_query, user_lists_page_query,
};

const DEFAULT_APP_ADDR: &str = "127.0.0.1:3000";
const DEFAULT_RUST_LOG: &str = "info,todo_app=debug,mssql_orm=debug";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TodoAppSettings {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    pub rust_log: String,
    pub operational_options: MssqlOperationalOptions,
}

impl TodoAppSettings {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let env = std::env::vars().collect::<HashMap<_, _>>();
        Self::from_map(&env)
    }

    pub fn from_map(env: &HashMap<String, String>) -> Result<Self, Box<dyn std::error::Error>> {
        let database_url = env
            .get("DATABASE_URL")
            .cloned()
            .ok_or("DATABASE_URL no está configurada para el ejemplo todo-app")?;
        let bind_addr = env
            .get("APP_ADDR")
            .map(String::as_str)
            .unwrap_or(DEFAULT_APP_ADDR)
            .parse()?;
        let rust_log = env
            .get("RUST_LOG")
            .cloned()
            .unwrap_or_else(|| DEFAULT_RUST_LOG.to_string());

        Ok(Self {
            bind_addr,
            database_url,
            rust_log,
            operational_options: default_operational_options(),
        })
    }

    pub fn connection_config(&self) -> Result<MssqlConnectionConfig, OrmError> {
        MssqlConnectionConfig::from_connection_string_with_options(
            &self.database_url,
            self.operational_options.clone(),
        )
    }
}

pub fn default_operational_options() -> MssqlOperationalOptions {
    MssqlOperationalOptions::new()
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
            MssqlTracingOptions::enabled().with_parameter_logging(MssqlParameterLogMode::Redacted),
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
        )
}

#[derive(Debug, Clone)]
pub struct TodoAppState<Db> {
    pub db: Db,
    pub settings: TodoAppSettings,
}

impl<Db> TodoAppState<Db> {
    pub fn new(db: Db, settings: TodoAppSettings) -> Self {
        Self { db, settings }
    }
}

pub fn build_app<Db>(state: TodoAppState<Db>) -> Router
where
    Db: Clone + Send + Sync + 'static,
{
    Router::new().with_state(state)
}

pub fn init_tracing(rust_log: &str) {
    let filter = tracing_subscriber::EnvFilter::try_new(rust_log)
        .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new(DEFAULT_RUST_LOG));

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();
}

#[cfg(test)]
mod tests {
    use super::{
        DEFAULT_APP_ADDR, DEFAULT_RUST_LOG, TodoAppSettings, TodoAppState, build_app,
        default_operational_options,
    };
    use std::collections::HashMap;
    use std::net::SocketAddr;
    use std::time::Duration;

    fn env_with_database_url() -> HashMap<String, String> {
        HashMap::from([(
            "DATABASE_URL".to_string(),
            "server=tcp:localhost,1433;database=tempdb;user=sa;password=Password123;TrustServerCertificate=true".to_string(),
        )])
    }

    #[test]
    fn settings_require_database_url() {
        let env = HashMap::<String, String>::new();
        let error = TodoAppSettings::from_map(&env).unwrap_err();

        assert_eq!(
            error.to_string(),
            "DATABASE_URL no está configurada para el ejemplo todo-app"
        );
    }

    #[test]
    fn settings_use_defaults_when_optional_values_are_missing() {
        let env = env_with_database_url();
        let settings = TodoAppSettings::from_map(&env).unwrap();

        assert_eq!(
            settings.bind_addr,
            DEFAULT_APP_ADDR.parse::<SocketAddr>().unwrap()
        );
        assert_eq!(settings.rust_log, DEFAULT_RUST_LOG);
        assert!(settings.operational_options.pool.enabled);
        assert_eq!(settings.operational_options.pool.max_size, 16);
        assert_eq!(settings.operational_options.pool.min_idle, Some(4));
    }

    #[test]
    fn settings_accept_explicit_bind_addr_and_rust_log() {
        let mut env = env_with_database_url();
        env.insert("APP_ADDR".to_string(), "0.0.0.0:4040".to_string());
        env.insert("RUST_LOG".to_string(), "debug,todo_app=trace".to_string());

        let settings = TodoAppSettings::from_map(&env).unwrap();

        assert_eq!(
            settings.bind_addr,
            "0.0.0.0:4040".parse::<SocketAddr>().unwrap()
        );
        assert_eq!(settings.rust_log, "debug,todo_app=trace");
    }

    #[test]
    fn default_operational_options_fix_expected_runtime_profile() {
        let options = default_operational_options();

        assert_eq!(
            options.timeouts.connect_timeout,
            Some(Duration::from_secs(5))
        );
        assert_eq!(
            options.timeouts.query_timeout,
            Some(Duration::from_secs(10))
        );
        assert_eq!(
            options.timeouts.acquire_timeout,
            Some(Duration::from_secs(2))
        );
        assert!(options.retry.enabled);
        assert_eq!(options.retry.max_retries, 2);
        assert!(options.tracing.enabled);
        assert!(options.slow_query.enabled);
        assert!(options.health.enabled);
        assert!(options.pool.enabled);
    }

    #[test]
    fn settings_build_connection_config_with_operational_options() {
        let env = env_with_database_url();
        let settings = TodoAppSettings::from_map(&env).unwrap();
        let config = settings.connection_config().unwrap();

        assert_eq!(config.connection_string(), settings.database_url);
        assert_eq!(config.options(), &settings.operational_options);
    }

    #[test]
    fn app_state_and_router_can_be_built_without_http_server() {
        let settings = TodoAppSettings::from_map(&env_with_database_url()).unwrap();
        let state = TodoAppState::new((), settings.clone());
        let _app = build_app(state.clone());

        assert_eq!(state.settings, settings);
    }
}

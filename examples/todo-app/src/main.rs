use todo_app::{TodoAppSettings, build_app, init_tracing};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = TodoAppSettings::from_env()?;
    init_tracing(&settings.rust_log);

    #[cfg(feature = "pool-bb8")]
    let app = {
        let pool = todo_app::connect_pool(&settings).await?;
        build_app(todo_app::state_from_pool(pool, settings.clone()))
    };

    #[cfg(not(feature = "pool-bb8"))]
    let app = build_app(todo_app::TodoAppState::new(
        todo_app::PendingTodoAppDbContext,
        settings.clone(),
    ));

    let listener = tokio::net::TcpListener::bind(settings.bind_addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

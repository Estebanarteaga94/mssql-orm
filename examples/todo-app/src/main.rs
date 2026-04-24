use todo_app::{PendingTodoAppDbContext, TodoAppSettings, TodoAppState, build_app, init_tracing};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = TodoAppSettings::from_env()?;
    init_tracing(&settings.rust_log);

    let app = build_app(TodoAppState::new(PendingTodoAppDbContext, settings.clone()));
    let listener = tokio::net::TcpListener::bind(settings.bind_addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}

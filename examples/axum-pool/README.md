# axum-pool

Ejemplo de integración web async usando `axum` sobre la API pública de `mssql-orm`, con:

- `MssqlPool` detrás del feature `pool-bb8`
- `DbContext::from_pool(...)`
- `DbContext::health_check()`
- configuración operativa real (`timeouts`, `retry`, `tracing`, `slow_query`, `health`, `pool`)

## Requisitos

- SQL Server accesible
- una connection string válida en `DATABASE_URL`

Ejemplo:

```bash
export DATABASE_URL='Server=localhost;Database=tempdb;User Id=SA;Password=Ea.930318;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;'
export APP_ADDR='127.0.0.1:3000'
export RUST_LOG='info,mssql_orm=debug'
```

## Ejecutar

```bash
cargo run --manifest-path examples/axum-pool/Cargo.toml
```

## Endpoints

- `GET /healthz`
- `GET /users`
- `POST /users`

`POST /users` espera un JSON como:

```json
{
  "name": "Ana",
  "active": true
}
```

## Qué hace al iniciar

1. Construye `MssqlOperationalOptions` con timeouts, retry, tracing, slow query, health check y pool.
2. Crea `MssqlConnectionConfig` y `MssqlPool`.
3. Construye `AppDbContext` desde `from_pool(...)`.
4. Ejecuta `health_check()` antes de levantar HTTP.
5. Asegura una tabla demo `dbo.axum_pool_users`.

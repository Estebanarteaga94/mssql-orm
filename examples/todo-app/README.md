# todo-app

Base del ejemplo web async realista de Etapa 14.

Estado actual:

1. Define configuración de arranque desde variables de entorno.
2. Fija la configuración operativa de SQL Server/Tiberius (`timeouts`, `retry`, `tracing`, `slow_query`, `health`, `pool`).
3. Expone la shape de `AppState` y del `Router` sin endpoints todavía.

Las siguientes subtareas extenderán este ejemplo con:

1. dominio `todo_app`,
2. consultas públicas,
3. health check HTTP,
4. endpoints mínimos,
5. wiring real con `MssqlPool` y `DbContext::from_pool(...)`,
6. validación contra SQL Server real.

## Variables de entorno

- `DATABASE_URL` obligatoria
- `APP_ADDR` opcional, default `127.0.0.1:3000`
- `RUST_LOG` opcional, default `info,todo_app=debug,mssql_orm=debug`

## Ejecutar

```bash
cargo run --manifest-path examples/todo-app/Cargo.toml
```


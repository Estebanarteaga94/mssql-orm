# todo-app

Base del ejemplo web async realista de Etapa 14.

Estado actual:

1. Define configuración de arranque desde variables de entorno.
2. Fija la configuración operativa de SQL Server/Tiberius (`timeouts`, `retry`, `tracing`, `slow_query`, `health`, `pool`).
3. Define el dominio base `todo_app` con entidades `users`, `todo_lists` y `todo_items`, incluyendo relaciones y metadata estática.
4. Expone la shape de `AppState` y del `Router` sin endpoints todavía.

Las siguientes subtareas extenderán este ejemplo con:

1. consultas públicas,
2. health check HTTP,
3. endpoints mínimos,
4. wiring real con `MssqlPool` y `DbContext::from_pool(...)`,
5. validación contra SQL Server real.

## Variables de entorno

- `DATABASE_URL` obligatoria
- `APP_ADDR` opcional, default `127.0.0.1:3000`
- `RUST_LOG` opcional, default `info,todo_app=debug,mssql_orm=debug`

## Ejecutar

```bash
cargo run --manifest-path examples/todo-app/Cargo.toml
```

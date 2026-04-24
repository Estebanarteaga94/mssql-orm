# todo-app

Base del ejemplo web async realista de Etapa 14.

Estado actual:

1. Define configuración de arranque desde variables de entorno.
2. Fija la configuración operativa de SQL Server/Tiberius (`timeouts`, `retry`, `tracing`, `slow_query`, `health`, `pool`).
3. Define el dominio base `todo_app` con entidades `users`, `todo_lists` y `todo_items`, incluyendo relaciones y metadata estática.
4. Expone queries reutilizables del ejemplo usando la surface real del consumidor (`db.todo_lists.query()...`, `db.todo_items.query()...`).
5. Expone `TodoAppDbContext` derivado, un endpoint HTTP real `GET /health` y endpoints mínimos de lectura para listas e ítems.
6. Mantiene `main.rs` compilable con `PendingTodoAppDbContext` hasta integrar el wiring real con pool en la siguiente subtarea.

Las siguientes subtareas extenderán este ejemplo con:

1. wiring real con `MssqlPool` y `DbContext::from_pool(...)`,
2. validación contra SQL Server real.

## Variables de entorno

- `DATABASE_URL` obligatoria
- `APP_ADDR` opcional, default `127.0.0.1:3000`
- `RUST_LOG` opcional, default `info,todo_app=debug,mssql_orm=debug`

## Ejecutar

```bash
cargo run --manifest-path examples/todo-app/Cargo.toml
```

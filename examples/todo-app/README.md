# todo-app

Base del ejemplo web async realista de Etapa 14.

Estado actual:

1. Define configuración de arranque desde variables de entorno.
2. Fija la configuración operativa de SQL Server/Tiberius (`timeouts`, `retry`, `tracing`, `slow_query`, `health`, `pool`).
3. Define el dominio base `todo_app` con entidades `users`, `todo_lists` y `todo_items`, incluyendo relaciones y metadata estática.
4. Incluye `audit_events` como entidad code-first dedicada para mostrar `#[orm(audit = TodoAudit)]` y columnas auditables generadas como metadata/schema.
5. Expone queries reutilizables del ejemplo usando la surface real del consumidor (`db.todo_lists.query()...`, `db.todo_items.query()...`).
6. Expone `TodoAppDbContext` derivado, un endpoint HTTP real `GET /health` y endpoints mínimos de lectura para listas e ítems.
7. Integra `MssqlPool` y `TodoAppDbContext::from_pool(...)` en `main.rs`, reutilizando la configuración operativa del ejemplo.
8. Ya fue validado contra SQL Server real con fixture reproducible y smoke HTTP.

La siguiente etapa del backlog ya no es del ejemplo en sí, sino del release/documentación pública del proyecto.

Índice de ejemplos disponibles en el repo: [../README.md](../README.md).

## Variables de entorno

- `DATABASE_URL` obligatoria
- `APP_ADDR` opcional, default `127.0.0.1:3000`
- `RUST_LOG` opcional, default `info,todo_app=debug,mssql_orm=debug`

## Ejecutar

```bash
cargo run --manifest-path examples/todo-app/Cargo.toml
```

## Smoke reproducible en SQL Server local

Preparar fixture:

```bash
sqlcmd -S localhost -U '<usuario>' -P '<password>' -d tempdb -C -b -i examples/todo-app/scripts/smoke_setup.sql
```

Levantar el ejemplo:

```bash
DATABASE_URL='Server=localhost;Database=tempdb;User Id=<usuario>;Password=<password>;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;' \
APP_ADDR='127.0.0.1:4011' \
RUST_LOG='warn,todo_app=info,mssql_orm=warn' \
cargo run --manifest-path examples/todo-app/Cargo.toml
```

Verificar endpoints:

```bash
curl -i 'http://127.0.0.1:4011/health'
curl -i 'http://127.0.0.1:4011/todo-lists/10'
curl -i 'http://127.0.0.1:4011/users/7/todo-lists?page=1&page_size=20'
curl -i 'http://127.0.0.1:4011/todo-lists/10/items/preview?limit=2'
curl -i 'http://127.0.0.1:4011/todo-lists/10/open-items/count'
```

También queda una prueba ignorada, útil para repetir la lectura real sin pasar por HTTP:

```bash
DATABASE_URL='Server=localhost;Database=tempdb;User Id=<usuario>;Password=<password>;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;' \
cargo test --manifest-path examples/todo-app/Cargo.toml smoke_preview_query_runs_against_sql_server_fixture -- --ignored --nocapture
```

## Migraciones automáticas del ejemplo

El ejemplo incluye un binario exportador de snapshot para validar el flujo code-first real:

```bash
cargo run --manifest-path examples/todo-app/Cargo.toml --bin model_snapshot
```

También hay un script reproducible que genera una migración inicial desde el modelo actual, una segunda migración incremental no-op y el script acumulado de `database update`:

```bash
examples/todo-app/scripts/migration_e2e.sh
```

El script valida además que `model_snapshot.json` capture `audit_events` y las columnas auditables generadas por `TodoAudit`. También imprime el directorio temporal usado y el `database_update.sql` generado. Si configuras `MSSQL_ORM_SQLCMD_SERVER`, `MSSQL_ORM_SQLCMD_USER`, `MSSQL_ORM_SQLCMD_PASSWORD` y opcionalmente `MSSQL_ORM_SQLCMD_DATABASE`, también intenta aplicar el script con `sqlcmd`.

Nota operativa:

- El dominio del ejemplo usa `NO ACTION` para `completed_by_user_id` porque SQL Server rechaza `SET NULL` en esa relación dentro de este esquema mínimo por `multiple cascade paths`.

# Ejemplos

Estado actual del árbol `examples/`:

- `todo-app/`
  ejemplo web async realista con `DbContext`, pool opcional, health check, dominio relacional, queries públicas y smoke reproducible contra SQL Server real

## Ejemplo disponible hoy

### `todo-app`

Variables de entorno principales:

- `DATABASE_URL`
- `APP_ADDR`
- `RUST_LOG`

Ejecutar:

```bash
cargo run --manifest-path examples/todo-app/Cargo.toml
```

Smoke local:

```bash
sqlcmd -S localhost -U SA -P 'Ea.930318' -d tempdb -C -b -i examples/todo-app/scripts/smoke_setup.sql

DATABASE_URL='Server=localhost;Database=tempdb;User Id=SA;Password=Ea.930318;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;' \
APP_ADDR='127.0.0.1:4011' \
RUST_LOG='warn,todo_app=info,mssql_orm=warn' \
cargo run --manifest-path examples/todo-app/Cargo.toml
```

Más detalle en [todo-app/README.md](todo-app/README.md).

## Nota

Históricamente el backlog y el worklog registran trabajo sobre un ejemplo `basic-crud`, pero ese ejemplo no está presente en el árbol actual del repositorio. La documentación operativa vigente de Etapa 15 ya toma `todo-app` como ejemplo ejecutable real disponible.

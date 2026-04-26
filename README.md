# mssql-orm

ORM `code-first` para Rust y Microsoft SQL Server, con metadata generada por `proc_macros`, query builder tipado, migraciones y ejecución sobre Tiberius.

El modelo operativo central es:

```text
Entity -> Metadata -> Query AST -> SQL Server SQL -> Tiberius -> Row -> Entity
```

Para entender ese flujo, los límites entre crates y el modelo mental del ORM, empieza por [docs/core-concepts.md](docs/core-concepts.md).

## Estado

El repositorio ya contiene las crates principales del diseño:

- `mssql-orm`: crate pública y `prelude`
- `mssql-orm-core`: contratos, metadata, tipos y errores
- `mssql-orm-macros`: derives
- `mssql-orm-query`: AST y query builder, sin SQL directo
- `mssql-orm-sqlserver`: compilación SQL Server
- `mssql-orm-tiberius`: conexión y ejecución
- `mssql-orm-migrate`: snapshots, diff y migraciones
- `mssql-orm-cli`: comandos operativos

El inventario verificado de APIs reales, features implementadas, límites y features diferidas está en [docs/repository-audit.md](docs/repository-audit.md).

## Ejemplo mínimo

```rust
use mssql_orm::prelude::*;

#[derive(Entity, Debug, Clone)]
#[orm(table = "users", schema = "dbo")]
struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(length = 180)]
    email: String,

    active: bool,
}

#[derive(DbContext, Debug, Clone)]
struct AppDb {
    pub users: DbSet<User>,
}

#[tokio::main]
async fn main() -> Result<(), OrmError> {
    let db = AppDb::connect(
        "Server=localhost;Database=tempdb;User Id=<usuario>;Password=<password>;TrustServerCertificate=True;Encrypt=False;"
    )
    .await?;

    let users = db
        .users
        .query()
        .filter(User::active.eq(true))
        .order_by(User::email.asc())
        .take(10)
        .all()
        .await?;

    println!("loaded {} users", users.len());
    Ok(())
}
```

## Documentación principal

- Conceptos centrales: [docs/core-concepts.md](docs/core-concepts.md)
- Quickstart reproducible: [docs/quickstart.md](docs/quickstart.md)
- Code-first y derives: [docs/code-first.md](docs/code-first.md)
- API pública: [docs/api.md](docs/api.md)
- Query builder: [docs/query-builder.md](docs/query-builder.md)
- Proyecciones tipadas: [docs/projections.md](docs/projections.md)
- Raw SQL tipado: [docs/raw-sql.md](docs/raw-sql.md)
- Relaciones y joins: [docs/relationships.md](docs/relationships.md)
- Transacciones: [docs/transactions.md](docs/transactions.md)
- Migraciones: [docs/migrations.md](docs/migrations.md)
- Entity Policies: [docs/entity-policies.md](docs/entity-policies.md)
- Uso desde otro proyecto sin clonar manualmente: [docs/use-without-downloading.md](docs/use-without-downloading.md)

## Ejemplos

- Índice de ejemplos: [examples/README.md](examples/README.md)
- Ejemplo web `todo-app`: [examples/todo-app/README.md](examples/todo-app/README.md)

Pending verification: la validación histórica de `todo-app` contra SQL Server real está registrada en [docs/worklog.md](docs/worklog.md), pero debe reejecutarse con un connection string real en el entorno actual antes de usarse como evidencia fresca.

## Límites actuales

- SQL Server es el único backend objetivo.
- `Tracked<T>` y `save_changes()` son experimentales.
- Las rutas públicas de CRUD, Active Record y tracking siguen centradas en primary keys simples.
- `AuditProvider` runtime no está implementado.
- No hay navigation properties, joins inferidos, aliases automáticos para self-joins ni agregaciones tipadas de alto nivel.
- `raw<T>()` y `raw_exec()` no aplican automáticamente filtros ORM de `tenant` ni `soft_delete`.
- `migration.rs` está diferido; el MVP de migraciones usa `up.sql`, `down.sql` y `model_snapshot.json`.

## Validación local

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features
```

## Proyecto

- Plan maestro: [docs/plan_orm_sqlserver_tiberius_code_first.md](docs/plan_orm_sqlserver_tiberius_code_first.md)
- Arquitectura: [docs/architecture/overview.md](docs/architecture/overview.md)
- Backlog: [docs/tasks.md](docs/tasks.md)
- Contexto operativo: [docs/context.md](docs/context.md)
- Worklog: [docs/worklog.md](docs/worklog.md)
- Contribución: [CONTRIBUTING.md](CONTRIBUTING.md)
- Seguridad: [SECURITY.md](SECURITY.md)
- Licencia: [LICENSE](LICENSE)

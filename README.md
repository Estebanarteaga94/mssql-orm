# mssql-orm

ORM `code-first` para Rust y SQL Server, con `proc_macros`, query builder tipado, migraciones y ejecución sobre Tiberius.

Si te gusta la ergonomía de `Entity Framework Core` o `Eloquent`, pero quieres mantener control explícito sobre tipos, ownership y límites arquitectónicos en Rust, este proyecto apunta exactamente a eso.

## Por Qué Existe

Trabajar con SQL Server desde Rust suele caer en uno de dos extremos:

- acceso de muy bajo nivel, con mucho SQL manual y poco modelo de dominio
- abstracciones demasiado genéricas, poco alineadas con SQL Server real

`mssql-orm` toma una posición distinta:

- SQL Server es el objetivo principal, no un backend “más”
- la metadata vive en entidades Rust
- el query builder construye AST, no strings SQL
- la compilación SQL está separada de la ejecución
- Tiberius queda encapsulado como adaptador de infraestructura

## Qué Ya Puedes Hacer

Hoy el repositorio ya tiene soporte funcional para:

- `#[derive(Entity)]` con metadata estática, columnas, foreign keys, índices compuestos, `rowversion`, columnas computed y hints de rename
- materialización automática de filas vía `FromRow` derivado desde `Entity`
- `DbContext` y `DbSet<T>` tipados
- CRUD base: `find`, `insert`, `update`, `delete`
- query builder público con `filter`, `order_by`, `limit`, `take`, `paginate`, `count`, `inner_join`, `left_join`
- Active Record base: `Entity::query(&db)`, `Entity::find(&db, id)`, `entity.save(&db)`, `entity.delete(&db)`
- concurrencia optimista con `rowversion`
- change tracking experimental con `Tracked<T>` y `save_changes()`
- compilación de AST a SQL Server parametrizado
- adaptador Tiberius para conexión, ejecución, transacciones, health checks, retry, tracing, slow query logs y pool opcional
- migraciones `code-first` con snapshots, diff, DDL SQL Server y CLI mínima
- validación real contra SQL Server en pruebas de integración y en el ejemplo `todo_app`

## Ejemplo Rápido

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
        "Server=localhost;Database=tempdb;User Id=SA;Password=secret;TrustServerCertificate=True;Encrypt=False;"
    )
    .await?;

    let active_users = db
        .users
        .query()
        .filter(User::active.eq(true))
        .order_by(User::email.asc())
        .take(10)
        .all()
        .await?;

    println!("loaded {} users", active_users.len());
    Ok(())
}
```

Si quieres el recorrido paso a paso, con tabla de prueba, `Cargo.toml`, CRUD base y query builder público, revisa [docs/quickstart.md](docs/quickstart.md).

## Arquitectura

El workspace está dividido por responsabilidad:

- `mssql-orm-core`
  contratos, metadata, tipos, errores y mapping neutral
- `mssql-orm-macros`
  derives y generación en compile-time
- `mssql-orm-query`
  AST del query builder, sin generar SQL
- `mssql-orm-sqlserver`
  compilación del AST a SQL Server parametrizado
- `mssql-orm-tiberius`
  conexión, ejecución, filas, transacciones, retry, tracing y pool
- `mssql-orm-migrate`
  snapshots, diff y operaciones de migración
- `mssql-orm-cli`
  comandos de migración y soporte operativo
- `mssql-orm`
  crate pública que concentra la API

La separación es deliberada:

- `core` no depende de Tiberius
- `query` no emite SQL
- `sqlserver` no ejecuta
- `tiberius` no define metadata del dominio

## Quickstart y Ejemplos

### Quickstart público

Guía reproducible para:

- conectar `DbContext`
- modelar una entidad
- usar `Insertable` y `Changeset`
- ejecutar `find`, `insert`, `update`, `delete`
- usar `filter`, `order_by` y `take`

Documento: [docs/quickstart.md](docs/quickstart.md)

### `todo-app`

Ejemplo web async más realista, con:

- `DbContext` derivado
- dominio relacional (`users`, `todo_lists`, `todo_items`)
- query builder público real
- health check HTTP
- endpoints mínimos de lectura
- wiring con `MssqlPool`
- smoke reproducible contra SQL Server real

```bash
cargo run --manifest-path examples/todo-app/Cargo.toml
```

Más detalle en [examples/todo-app/README.md](examples/todo-app/README.md).

## Estado Real del Proyecto

El proyecto ya no está en fase “placeholder”. Las etapas fundacionales y gran parte de la superficie pública principal están implementadas y validadas.

Estado resumido:

- Etapa 12 cerrada: change tracking experimental
- Etapa 13 cerrada: migraciones avanzadas
- Etapa 14 cerrada: surface operativa de producción y ejemplo web `todo_app`
- Etapa 15 en curso: release, documentación pública, quickstart y changelog

La fuente de verdad del roadmap técnico sigue siendo [docs/plan_orm_sqlserver_tiberius_code_first.md](docs/plan_orm_sqlserver_tiberius_code_first.md).

## Lo Que Aún No Promete

Este repo todavía no pretende vender humo. Hay límites explícitos en esta fase:

- SQL Server es el único backend objetivo
- la API de change tracking sigue siendo experimental
- no se está intentando soportar múltiples motores de base de datos
- el release público todavía está en consolidación
- algunas decisiones de UX todavía se están afinando alrededor de ejemplos, quickstart y documentación de entrada

## Validación

Validación base del workspace:

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

El ejemplo `todo_app` también quedó validado contra SQL Server real con fixture reproducible.

## Documentación del Repositorio

- plan maestro: [docs/plan_orm_sqlserver_tiberius_code_first.md](docs/plan_orm_sqlserver_tiberius_code_first.md)
- arquitectura: [docs/architecture/overview.md](docs/architecture/overview.md)
- backlog: [docs/tasks.md](docs/tasks.md)
- contexto operativo: [docs/context.md](docs/context.md)
- historial de sesiones: [docs/worklog.md](docs/worklog.md)

## Dirección del Proyecto

La meta no es solo “tener un ORM”.

La meta es ofrecer una librería que haga que trabajar con SQL Server desde Rust se sienta:

- tipado
- explícito
- productivo
- portable dentro del propio ecosistema del proyecto
- útil en aplicaciones reales, no solo en demos de metadata

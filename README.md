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
- raw SQL tipado con `raw<T>()`, `raw_exec()`, parametros `@P1..@Pn` y materializacion mediante `FromRow`
- proyecciones tipadas con `select(...)`, `all_as::<T>()` y `first_as::<T>()` hacia DTOs `FromRow`
- Active Record base: `Entity::query(&db)`, `Entity::find(&db, id)`, `entity.save(&db)`, `entity.delete(&db)`
- concurrencia optimista con `rowversion`
- change tracking experimental con `Tracked<T>` y `save_changes()`
- compilación de AST a SQL Server parametrizado
- adaptador Tiberius para conexión, ejecución, transacciones, health checks, retry, tracing, slow query logs y pool opcional
- migraciones `code-first` con snapshots, diff, DDL SQL Server y CLI mínima
- Entity Policies iniciales con `#[derive(AuditFields)]` y `#[orm(audit = Audit)]` como columnas reutilizables de metadata/schema
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
        "Server=localhost;Database=tempdb;User Id=<usuario>;Password=<password>;TrustServerCertificate=True;Encrypt=False;"
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
Si quieres la explicación del modelo `code-first` actual, sus derives y límites explícitos, revisa [docs/code-first.md](docs/code-first.md).
Si quieres reutilizar columnas transversales con `Entity Policies`, revisa [docs/entity-policies.md](docs/entity-policies.md).
Si quieres ver el inventario de API publicada por la crate raíz, revisa [docs/api.md](docs/api.md).
Si quieres usar el ORM desde otro proyecto sin clonar manualmente este repositorio, revisa [docs/use-without-downloading.md](docs/use-without-downloading.md).
Si quieres profundizar en filtros, ordenamiento, joins, paginación y conteos, revisa [docs/query-builder.md](docs/query-builder.md).
Si necesitas SQL Server escrito a mano con DTOs tipados y comandos parametrizados, revisa [docs/raw-sql.md](docs/raw-sql.md).
Si quieres seleccionar columnas o expresiones concretas y materializarlas en DTOs, revisa [docs/projections.md](docs/projections.md).
Si quieres modelar foreign keys y usarlas en joins explícitos, revisa [docs/relationships.md](docs/relationships.md).
Si quieres usar operaciones atómicas con commit/rollback, revisa [docs/transactions.md](docs/transactions.md).
Si quieres contribuir o revisar la postura de seguridad, revisa [CONTRIBUTING.md](CONTRIBUTING.md) y [SECURITY.md](SECURITY.md).

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

### Guía `code-first`

Guía conceptual y práctica para:

- modelar entidades con `#[derive(Entity)]`
- definir `Insertable`, `Changeset` y `DbContext`
- entender el rol de `DbSet<T>` en CRUD y consultas
- fijar límites reales de la estrategia `code-first` actual

Documento: [docs/code-first.md](docs/code-first.md)

### API pública

Inventario de la surface expuesta por la crate raíz:

- `prelude`
- derives públicos
- `DbContext`, `DbSet` y `DbSetQuery`
- query builder, Active Record y tracking experimental
- migraciones code-first
- configuración SQL Server/Tiberius
- módulos avanzados reexportados y exclusiones explícitas

Documento: [docs/api.md](docs/api.md)

### Uso sin descargar el repositorio

Guía para consumir `mssql-orm` desde otro proyecto mediante dependencia Git,
sin clonar manualmente este repositorio dentro del workspace de la aplicación.

Documento: [docs/use-without-downloading.md](docs/use-without-downloading.md)

### Entity Policies

Evolución conservadora del modelo `code-first` para reutilizar columnas transversales sin crear un segundo pipeline de esquema.

Lo implementado hoy:

- `#[derive(AuditFields)]` para definir un struct reusable de auditoría
- `#[orm(audit = Audit)]` sobre entidades derivadas con `Entity`
- expansión de columnas auditables dentro de `EntityMetadata.columns`
- participación normal de esas columnas en snapshots, diff, DDL SQL Server y migraciones
- cobertura `trybuild`, pruebas de metadata, snapshot/diff, DDL y ejemplo `todo-app`

Lo que queda deliberadamente diferido:

- autollenado runtime de `created_at`, `created_by`, `updated_at` o `updated_by`
- campos Rust visibles y símbolos asociados como `Todo::created_at` cuando la columna viene de una policy
- `AuditProvider` runtime
- filtros automaticos sobre entidades unidas manualmente por `soft_delete` o `tenant`

Documento: [docs/entity-policies.md](docs/entity-policies.md)

### Guía del query builder

Guía práctica para:

- construir filtros con predicados tipados
- ordenar por columnas generadas
- usar `take`, `limit` y `paginate`
- declarar joins explícitos
- ejecutar `all`, `first` y `count`
- proyectar columnas a DTOs con `select(...)`, `all_as::<T>()` y `first_as::<T>()`
- entender límites actuales como aliases de tabla, agregaciones y conteos con joins

Documento: [docs/query-builder.md](docs/query-builder.md)

### Raw SQL tipado

Guia practica para:

- ejecutar consultas SQL Server escritas a mano con `db.raw::<T>(...)`
- materializar DTOs mediante `FromRow`
- ejecutar comandos con `db.raw_exec(...).execute()`
- usar parametros `@P1`, `@P2`, ..., `@Pn`
- entender que raw SQL no aplica automaticamente `tenant` ni `soft_delete`

Documento: [docs/raw-sql.md](docs/raw-sql.md)

### Guía de relaciones y joins

Guía práctica para:

- declarar `foreign_key` desde entidades dependientes
- usar la sintaxis estructurada `foreign_key(entity = User, column = id)`
- configurar `on_delete`
- inspeccionar `ForeignKeyMetadata`
- escribir `inner_join` y `left_join` explícitos
- entender límites actuales como aliases, navigation properties y proyecciones

Documento: [docs/relationships.md](docs/relationships.md)

### Guía de transacciones

Guía operativa para:

- usar `db.transaction(|tx| async move { ... })`
- entender commit en `Ok` y rollback en `Err`
- conocer la interacción con timeouts, tracing y retry
- respetar los límites actuales de conexión compartida, pool y savepoints

Documento: [docs/transactions.md](docs/transactions.md)

### Guía de migraciones

Guía operativa para:

- usar `migration add`, `migration list` y `database update`
- entender que `database update` imprime SQL por defecto y puede ejecutar con `--execute`
- trabajar con checksums, historial e idempotencia sin reescribir migraciones aplicadas

Documento: [docs/migrations.md](docs/migrations.md)

### `todo-app`

Ejemplo web async más realista, con:

- `DbContext` derivado
- dominio relacional (`users`, `todo_lists`, `todo_items`)
- query builder público real
- health check HTTP
- endpoints mínimos de lectura
- wiring con `MssqlPool`
- smoke reproducible contra SQL Server real

Pending verification: antes de presentar el smoke como validacion actual de tu entorno, vuelve a ejecutarlo con un connection string real; la evidencia historica esta registrada en `docs/worklog.md`.

```bash
cargo run --manifest-path examples/todo-app/Cargo.toml
```

Más detalle en [examples/todo-app/README.md](examples/todo-app/README.md).

Índice de ejemplos disponibles: [examples/README.md](examples/README.md).

## Estado Real del Proyecto

El proyecto ya no está en fase “placeholder”. Las etapas fundacionales y gran parte de la superficie pública principal están implementadas y validadas.

Estado resumido:

- Etapa 12 cerrada: change tracking experimental
- Etapa 13 cerrada: migraciones avanzadas
- Etapa 14 cerrada: surface operativa de producción y ejemplo web `todo_app`
- Etapa 15 cerrada: release, documentación pública, quickstart, guías y changelog
- Etapa 16+ avanzada: `Entity Policies` ya cubre auditoría como metadata/schema, `soft_delete` runtime y filtros obligatorios `tenant`
- Etapa 17 cerrada: raw SQL tipado
- Etapa 18 avanzada: proyecciones tipadas a DTOs con cobertura publica

La fuente de verdad del roadmap técnico sigue siendo [docs/plan_orm_sqlserver_tiberius_code_first.md](docs/plan_orm_sqlserver_tiberius_code_first.md).

## Lo Que Aún No Promete

Este repo todavía no pretende vender humo. Hay límites explícitos en esta fase:

- SQL Server es el único backend objetivo
- la API de change tracking sigue siendo experimental
- `audit = Audit` no autollena valores en runtime ni agrega campos Rust visibles a las entidades
- `AuditProvider` runtime sigue como diseño futuro
- no hay aliases de tabla para joins ni self-joins
- no hay agregaciones tipadas de alto nivel en el query builder
- no se está intentando soportar múltiples motores de base de datos
- algunas decisiones de UX futura siguen abiertas alrededor de policies con comportamiento runtime

## Validación

Validación base del workspace:

```bash
cargo fmt --all --check
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Pending verification: el ejemplo `todo_app` tiene validacion historica contra SQL Server real registrada en `docs/worklog.md`, pero debe reejecutarse en el entorno actual antes de tratarla como evidencia fresca.

## Documentación del Repositorio

- plan maestro: [docs/plan_orm_sqlserver_tiberius_code_first.md](docs/plan_orm_sqlserver_tiberius_code_first.md)
- licencia: [LICENSE](LICENSE)
- contribución: [CONTRIBUTING.md](CONTRIBUTING.md)
- seguridad: [SECURITY.md](SECURITY.md)
- changelog: [CHANGELOG.md](CHANGELOG.md)
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

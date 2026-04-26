# API Publica

Inventario minimo de la surface publica actual expuesta por la crate raiz `mssql-orm`.

Para codigo consumidor, el punto de entrada recomendado es:

```rust
use mssql_orm::prelude::*;
```

La crate raiz concentra la API de usuario y reexporta internals seleccionados para pruebas, tooling y casos avanzados. Las responsabilidades siguen separadas por crate: `query` construye AST, `sqlserver` compila SQL, `tiberius` ejecuta, `migrate` gestiona snapshots/diff/migraciones y `core` define contratos compartidos.

## Derives publicos

Estos derives se usan desde la crate publica:

- `#[derive(Entity)]`
- `#[derive(Insertable)]`
- `#[derive(Changeset)]`
- `#[derive(DbContext)]`
- `#[derive(AuditFields)]`

Ejemplo base:

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
}

#[derive(Insertable)]
#[orm(entity = User)]
struct NewUser {
    email: String,
}

#[derive(Changeset)]
#[orm(entity = User)]
struct UpdateUser {
    email: Option<String>,
}

#[derive(DbContext)]
struct AppDb {
    pub users: DbSet<User>,
}
```

## Contratos de modelo

La `prelude` expone los contratos principales de metadata y mapping:

- `Entity`
- `EntityMetadata`
- `EntityColumn`
- `ColumnMetadata`
- `PrimaryKeyMetadata`
- `IdentityMetadata`
- `IndexMetadata`
- `IndexColumnMetadata`
- `ForeignKeyMetadata`
- `ReferentialAction`
- `EntityPolicy`
- `EntityPolicyMetadata`
- `SqlServerType`
- `SqlTypeMapping`
- `SqlValue`
- `ColumnValue`
- `Row`
- `FromRow`
- `Insertable`
- `Changeset`
- `OrmError`

Uso tipico:

```rust
let metadata = User::metadata();
let email_column = User::email;

assert_eq!(metadata.table, "users");
assert_eq!(email_column.column_name(), "email");
```

## DbContext y DbSet

La API principal de acceso a datos esta en:

- `DbContext`
- `DbSet<T>`
- `DbSetQuery<T>`
- `DbContextEntitySet<T>`
- `SharedConnection`
- `connect_shared(...)`
- `connect_shared_with_options(...)`
- `connect_shared_with_config(...)`

`#[derive(DbContext)]` genera metodos inherentes sobre tu contexto:

- `connect(...)`
- `connect_with_options(...)`
- `connect_with_config(...)`
- `from_connection(...)`
- `from_shared_connection(...)`
- `health_check().await`
- `transaction(|tx| async move { ... }).await`
- `save_changes().await`
- `from_pool(...)` cuando el feature `pool-bb8` esta activo

`DbSet<T>` expone la ruta CRUD y de consulta:

- `find(key).await`
- `insert(model).await`
- `update(key, changeset).await`
- `delete(key).await`
- `query()`
- `query_with(select_query)`
- `entity_metadata()`
- `find_tracked(key).await`
- `add_tracked(entity)`
- `remove_tracked(&mut tracked)`

Limites relevantes:

- `find`, `update`, `delete`, Active Record y tracking publico siguen orientados a primary key simple.
- `save_changes()` y `Tracked<T>` son surface experimental.
- `db.transaction(...)` sobre `from_pool(...)` no debe considerarse soportado hasta pinnear una conexion fisica durante todo el closure.

## Query builder

La `prelude` expone extensiones ergonomicas para columnas y predicados:

- `EntityColumnPredicateExt`
- `EntityColumnOrderExt`
- `PredicateCompositionExt`
- `PageRequest`
- `Join`
- `JoinType`

Uso recomendado:

```rust
let users = db
    .users
    .query()
    .filter(User::email.contains("@example.com"))
    .order_by(User::email.asc())
    .take(20)
    .all()
    .await?;
```

Para construir predicados de joins o AST avanzado, usa el modulo reexportado:

```rust
use mssql_orm::query::{Expr, Predicate};
```

Surface avanzada reexportada desde `mssql_orm::query`:

- `SelectQuery`
- `CountQuery`
- `InsertQuery`
- `UpdateQuery`
- `DeleteQuery`
- `Expr`
- `Predicate`
- `OrderBy`
- `Pagination`
- `TableRef`
- `ColumnRef`
- `CompiledQuery`

`mssql_orm::query` no genera SQL.

### Etapa 17: raw SQL tipado

Raw SQL tipado es el escape hatch explicito para consultas o comandos que todavia no encajan en el query builder. La guia publica esta en `docs/raw-sql.md` y la API disponible es:

```rust
let rows = db
    .raw::<UserListItem>("SELECT id, email FROM dbo.users WHERE active = @P1 AND tenant_id = @P2")
    .params((true, tenant_id))
    .all()
    .await?;

db.raw_exec("UPDATE dbo.users SET active = @P1 WHERE id = @P2")
    .params((false, 7_i64))
    .execute()
    .await?;
```

Esta surface existe en el `DbContext` trait y reexporta `RawQuery`, `RawCommand`, `RawParam` y `RawParams` desde `mssql_orm::prelude`. La implementacion reutiliza `SharedConnection`, `FromRow`, `SqlTypeMapping`, `SqlValue`, `CompiledQuery` y `ExecuteResult`. Raw SQL no aplica implicitamente filtros ORM de `tenant` ni `soft_delete`; el usuario debe escribir esos predicados de forma explicita en el SQL.

Reglas aprobadas de parametros: `.params((p1, p2))` es la forma recomendada para varios valores, repetir `@P1` es valido y reutiliza el mismo valor, los placeholders deben ser continuos desde `@P1` hasta `@Pn`, y la cantidad de parametros debe coincidir con el mayor placeholder usado. La implementacion debe validar por indices de placeholders, no por cantidad de ocurrencias.

### Roadmap cercano: proyecciones tipadas

Las proyecciones tipadas tambien quedan planificadas. La diferencia frente a usar `map` despues de `all().await` es que una proyeccion real cambia el `SELECT` emitido y reduce las columnas leidas desde SQL Server.

Direccion de API:

```rust
let users = db
    .users
    .query()
    .select((User::id, User::email))
    .all_as::<UserListItem>()
    .await?;
```

El diseno debe preservar la ruta actual `all()` / `first()` para entidades completas, y agregar materializacion a DTOs mediante `FromRow`.

## Active Record

La crate raiz expone `ActiveRecord` como capa de conveniencia sobre `DbSet`.

```rust
let user = User::find(&db, 1_i64).await?;

let users = User::query(&db)
    .filter(User::email.ends_with("@example.com"))
    .all()
    .await?;
```

Metodos disponibles:

- `Entity::query(&db)`
- `Entity::find(&db, key).await`
- `entity.save(&db).await`
- `entity.delete(&db).await`

El derive `Entity` genera los contratos internos necesarios para `save` y `delete`. La capa Active Record no reemplaza la ruta base `DbSet`; reutiliza su implementacion.

## Tracking experimental

La surface experimental actual incluye:

- `Tracked<T>`
- `EntityState`
- `DbSet::find_tracked(...)`
- `DbSet::add_tracked(...)`
- `DbSet::remove_tracked(...)`
- `DbContext::save_changes().await`

Estados actuales:

- `Added`
- `Unchanged`
- `Modified`
- `Deleted`

Limites principales:

- no hay proxies
- no hay tracking automatico global
- descartar el wrapper `Tracked<T>` elimina su participacion en la unidad de trabajo experimental
- las rutas de persistencia reutilizan `DbSet` y mantienen los limites de primary key simple

## Migraciones code-first

La crate raiz expone la fuente de metadata para migraciones:

- `MigrationModelSource`
- `model_snapshot_from_source::<C>()`
- `model_snapshot_json_from_source::<C>()`

`#[derive(DbContext)]` implementa `MigrationModelSource` para el contexto derivado.

```rust
let json = model_snapshot_json_from_source::<AppDb>()?;
println!("{json}");
```

Para tooling avanzado, la crate raiz reexporta:

```rust
use mssql_orm::migrate;
```

Surface relevante en `mssql_orm::migrate`:

- `ModelSnapshot`
- `SchemaSnapshot`
- `TableSnapshot`
- `ColumnSnapshot`
- `IndexSnapshot`
- `ForeignKeySnapshot`
- `MigrationOperation`
- `diff_schema_and_table_operations`
- `diff_column_operations`
- `diff_relational_operations`
- helpers de filesystem de migraciones

La CLI `mssql-orm-cli` es la entrada operativa para `migration add`, `migration list` y `database update`.

## SQL Server y Tiberius

Para configuracion operacional desde la crate publica:

- `MssqlConnectionConfig`
- `MssqlOperationalOptions`
- `MssqlTimeoutOptions`
- `MssqlRetryOptions`
- `MssqlTracingOptions`
- `MssqlSlowQueryOptions`
- `MssqlHealthCheckOptions`
- `MssqlHealthCheckQuery`
- `MssqlParameterLogMode`
- `MssqlPoolOptions`
- `MssqlPoolBackend`

Con `pool-bb8` activo:

- `MssqlPool`
- `MssqlPoolBuilder`
- `MssqlPooledConnection`
- `connect_shared_from_pool(...)`

Para casos avanzados:

```rust
use mssql_orm::sqlserver::SqlServerCompiler;
use mssql_orm::tiberius::MssqlConnection;
```

`mssql_orm::sqlserver` compila AST y DDL SQL Server. `mssql_orm::tiberius` ejecuta contra SQL Server. La API normal de aplicacion deberia pasar por `DbContext` y `DbSet`.

## Entity Policies

La surface inicial de policies incluye:

- `EntityPolicy`
- `EntityPolicyMetadata`
- `#[derive(AuditFields)]`
- `#[orm(audit = Audit)]` sobre `#[derive(Entity)]`

En el MVP, `audit = Audit` genera columnas de metadata/schema. No autollena valores en runtime y no agrega campos Rust visibles ni simbolos de columna sobre la entidad.

## Reexports de crates internas

La crate raiz expone estos modulos:

- `mssql_orm::core`
- `mssql_orm::macros`
- `mssql_orm::query`
- `mssql_orm::sqlserver`
- `mssql_orm::tiberius`
- `mssql_orm::migrate`
- `mssql_orm::tokio`

La regla practica:

- usa `mssql_orm::prelude::*` para aplicacion normal
- usa `mssql_orm::query` para AST o predicados avanzados
- usa `mssql_orm::migrate` para tooling de migraciones
- usa `mssql_orm::sqlserver` solo para compilar/validar SQL en tests o tooling
- usa `mssql_orm::tiberius` solo para configuracion/ejecucion avanzada

## Exclusiones explicitas

Esta surface no promete todavia:

- soporte multi-base de datos
- navigation properties
- lazy loading o eager loading automatico
- aliases de tabla
- proyecciones parciales publicas desde `DbSetQuery<T>`
- primary keys compuestas en CRUD publico
- `save_changes()` estable
- savepoints
- transacciones publicas correctas sobre pool hasta resolver el pinning de conexion
- autollenado runtime de auditoria

## Guias relacionadas

- Quickstart: [docs/quickstart.md](quickstart.md)
- Code-first: [docs/code-first.md](code-first.md)
- Query builder: [docs/query-builder.md](query-builder.md)
- Relaciones y joins: [docs/relationships.md](relationships.md)
- Transacciones: [docs/transactions.md](transactions.md)
- Migraciones: [docs/migrations.md](migrations.md)
- Entity Policies: [docs/entity-policies.md](entity-policies.md)
- Raw SQL tipado: [docs/raw-sql.md](raw-sql.md)

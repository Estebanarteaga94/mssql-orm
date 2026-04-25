# Query Builder Publico

Guia de uso de la API publica actual para consultas con `DbSetQuery<T>`.

El query builder de `mssql-orm` no construye SQL directo desde la crate publica. La API publica produce un AST de `mssql-orm-query`; la compilacion a SQL Server parametrizado ocurre en `mssql-orm-sqlserver` y la ejecucion ocurre en el adaptador Tiberius.

## Punto de entrada

La entrada normal es `DbSet<T>::query()` desde un `DbContext` derivado.

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

async fn load_active_users(db: &AppDb) -> Result<Vec<User>, OrmError> {
    db.users
        .query()
        .filter(User::active.eq(true))
        .order_by(User::email.asc())
        .take(20)
        .all()
        .await
}
```

`#[derive(Entity)]` genera simbolos de columna como `User::email` y `User::active`. Esos simbolos no son lectura de campos Rust; son referencias tipadas a columnas del modelo.

## Filtros

`filter(...)` recibe un `Predicate`. Llamar `filter` mas de una vez combina los predicados con `AND`.

```rust
let users = db
    .users
    .query()
    .filter(User::active.eq(true))
    .filter(User::email.ends_with("@example.com"))
    .all()
    .await?;
```

Los predicados publicos sobre columnas incluyen:

- comparacion: `eq`, `ne`, `gt`, `gte`, `lt`, `lte`
- nulabilidad: `is_null`, `is_not_null`
- strings: `contains`, `starts_with`, `ends_with`
- composicion: `and`, `or`, `not`

Ejemplo con composicion explicita:

```rust
let predicate = User::active
    .eq(true)
    .and(User::email.contains("@company.com").not());

let users = db.users.query().filter(predicate).all().await?;
```

Los valores se compilan como parametros SQL Server (`@P1`, `@P2`, ...), no como interpolacion de strings.

## Ordenamiento

`order_by(...)` recibe un `OrderBy`. La forma publica recomendada es usar `asc()` o `desc()` sobre una columna generada.

```rust
let users = db
    .users
    .query()
    .filter(User::active.eq(true))
    .order_by(User::email.asc())
    .order_by(User::id.desc())
    .all()
    .await?;
```

El orden se conserva en el AST y despues en el SQL compilado.

## Limite y paginacion

`limit(n)` y `take(n)` son equivalentes: ambos piden las primeras `n` filas mediante la paginacion interna con offset `0`.

```rust
let latest = db
    .users
    .query()
    .order_by(User::id.desc())
    .take(10)
    .all()
    .await?;
```

Para paginas explicitas usa `PageRequest::new(page, page_size)`. La pagina es 1-based: `PageRequest::new(1, 25)` representa las primeras 25 filas.

```rust
let page = db
    .users
    .query()
    .filter(User::active.eq(true))
    .order_by(User::email.asc())
    .paginate(PageRequest::new(2, 25))
    .all()
    .await?;
```

SQL Server requiere `ORDER BY` para `OFFSET/FETCH`. En consultas paginadas, agrega siempre un orden estable.

## Joins explicitos

Los joins actuales son explicitos. No existen todavia navigation properties ni eager loading automatico.

```rust
use mssql_orm::query::{Expr, Predicate};
use mssql_orm::prelude::*;

#[derive(Entity, Debug, Clone)]
#[orm(table = "orders", schema = "dbo")]
struct Order {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    user_id: i64,
    total_cents: i64,
}

let users = db
    .users
    .query()
    .inner_join::<Order>(Predicate::eq(
        Expr::from(User::id),
        Expr::from(Order::user_id),
    ))
    .filter(Order::total_cents.gte(10_000_i64))
    .order_by(User::email.asc())
    .all()
    .await?;
```

La API publica expone:

- `inner_join::<T>(on)`
- `left_join::<T>(on)`
- `join(Join)` para casos donde quieras construir el join manualmente desde el AST publico

Mientras no exista aliasing en el AST, evita self-joins o repetir la misma tabla en la misma consulta.

## Ejecucion

`DbSetQuery<T>` expone tres formas principales de ejecutar:

- `all().await`: materializa `Vec<T>`
- `first().await`: materializa `Option<T>`
- `count().await`: devuelve `i64`

```rust
let users = db.users.query().take(20).all().await?;
let first = db.users.query().order_by(User::id.asc()).first().await?;
let total_active = db.users.query().filter(User::active.eq(true)).count().await?;
```

`count()` conserva el `from` y los filtros de la consulta base. En el estado actual no traslada joins, ordenamiento ni paginacion al `CountQuery` interno; usalo para conteos de la entidad base con filtros que no dependan de tablas joinadas.

## Inspeccion del AST

Para pruebas o integracion avanzada, `DbSetQuery<T>` permite inspeccionar o extraer el `SelectQuery`.

```rust
let ast = db
    .users
    .query()
    .filter(User::active.eq(true))
    .order_by(User::id.asc())
    .into_select_query();
```

El AST sigue sin ser SQL. Si necesitas validar el SQL generado en tests de bajo nivel, compila ese AST desde la capa SQL Server.

## Limites actuales

- SQL Server es el unico backend objetivo.
- El query builder publico no ejecuta SQL manual ni acepta fragments SQL arbitrarios.
- No hay aliases de tabla en joins.
- No hay navigation properties ni carga automatica de relaciones.
- `count()` no preserva joins en esta etapa.
- La proyeccion publica de `DbSetQuery<T>` materializa entidades completas; proyecciones parciales quedan fuera del alcance actual.

## Referencias relacionadas

- API publica: [docs/api.md](api.md)
- Quickstart: [docs/quickstart.md](quickstart.md)
- Guia code-first: [docs/code-first.md](code-first.md)
- Relaciones y joins: [docs/relationships.md](relationships.md)
- Ejemplo real con queries: [examples/todo-app/src/queries.rs](../examples/todo-app/src/queries.rs)
- Plan maestro: [docs/plan_orm_sqlserver_tiberius_code_first.md](plan_orm_sqlserver_tiberius_code_first.md)

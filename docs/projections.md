# Proyecciones Tipadas

Las proyecciones tipadas son la Etapa 18 del query builder publico. El objetivo es seleccionar columnas o expresiones concretas desde SQL Server y materializarlas en DTOs con `FromRow`, sin romper la ruta actual que materializa entidades completas con `all()` y `first()`.

## Estado Actual

La base publica de proyecciones ya esta implementada:

- `mssql-orm-query::SelectQuery` contiene `projection: Vec<SelectProjection>`.
- `SelectQuery::select(...)` ya acepta elementos convertibles a `SelectProjection`.
- `mssql-orm-sqlserver` ya compila `projection` cuando no esta vacia, usando aliases estables.
- Si `projection` esta vacia, el compilador emite `SELECT *`.
- `DbSetQuery<E>::all()` y `first()` siempre materializan `E`.
- `DbSetQuery<E>::select(...)`, `all_as::<T>()` y `first_as::<T>()` ya exponen la API publica inicial de proyecciones.
- La cobertura publica vive en `crates/mssql-orm/tests/stage18_public_projections.rs` y `crates/mssql-orm/tests/ui/query_projection_public_valid.rs`.

## Ejemplo Base

```rust
#[derive(Debug, Clone, PartialEq)]
struct UserListItem {
    id: i64,
    email: String,
}

impl FromRow for UserListItem {
    fn from_row<R: Row>(row: &R) -> Result<Self, OrmError> {
        Ok(Self {
            id: row.get_required_typed::<i64>("id")?,
            email: row.get_required_typed::<String>("email")?,
        })
    }
}

let users = db
    .users
    .query()
    .select((User::id, User::email))
    .order_by(User::email.asc())
    .all_as::<UserListItem>()
    .await?;
```

`all()` y `first()` deben seguir significando "materializar entidad completa". Las proyecciones deben usar nombres distintos:

- `all_as::<T>() -> Result<Vec<T>, OrmError>`
- `first_as::<T>() -> Result<Option<T>, OrmError>`

Esta separacion evita que una consulta parcial intente materializar accidentalmente `E` con columnas faltantes.

## Contrato de AST

El AST representa la proyeccion publica con un tipo dedicado:

```rust
pub struct SelectProjection {
    pub expr: Expr,
    pub alias: Option<&'static str>,
}
```

```rust
pub projection: Vec<SelectProjection>
```

Reglas:

- proyeccion vacia conserva la semantica actual de entidad completa y compila a `SELECT *`;
- columnas generadas por `EntityColumn<E>` deben recibir alias por defecto igual a `column_name`;
- expresiones no-columna requieren alias explicito;
- aliases vacios deben fallar antes de compilar;
- aliases duplicados en la misma proyeccion deben fallar antes de compilar o al construir la proyeccion publica;
- el AST sigue sin generar SQL y no depende de Tiberius.

Ejemplo de AST esperado:

```rust
SelectQuery::from_entity::<User>().select(vec![
    SelectProjection::column(User::id),
    SelectProjection::column(User::email),
    SelectProjection::expr_as(Expr::function("LOWER", vec![Expr::from(User::email)]), "email_lower"),
])
```

## Compilacion SQL Server

`mssql-orm-sqlserver` debe renderizar aliases con `AS [alias]`:

```sql
SELECT
    [dbo].[users].[id] AS [id],
    [dbo].[users].[email] AS [email]
FROM [dbo].[users]
```

Aunque SQL Server permite leer columnas directas sin alias, el alias explicito es parte del contrato de `FromRow`: el DTO lee por `"id"` y `"email"`, no por un nombre dependiente del driver o de una expresion.

Para expresiones:

```sql
SELECT LOWER([dbo].[users].[email]) AS [email_lower]
```

El quoting de aliases pertenece a `mssql-orm-sqlserver` usando la infraestructura de quoting existente. `mssql-orm-query` solo transporta el alias como dato.

## API Publica de Proyeccion

La API publica vive en `mssql-orm`, encima de `DbSetQuery<E>`, y convierte formas ergonomicas a `SelectProjection`.

Forma recomendada para columnas:

```rust
db.users
    .query()
    .select((User::id, User::email))
    .all_as::<UserListItem>()
    .await?;
```

Para expresiones con alias explicito:

```rust
use mssql_orm::query::Expr;

db.users
    .query()
    .select(SelectProjection::expr_as(
        Expr::function("LOWER", vec![Expr::from(User::email)]),
        "email_lower",
    ))
    .all_as::<UserEmailSearchItem>()
    .await?;
```

`SelectProjection` esta reexportado desde `mssql_orm::prelude`. `Expr` sigue disponible desde `mssql_orm::query`.

## Integracion con Filtros Obligatorios

Las proyecciones deben reutilizar `DbSetQuery::effective_select_query()` o una ruta equivalente. Esto es obligatorio para conservar:

- filtros `tenant` opt-in;
- visibilidad por defecto de `soft_delete`;
- `with_deleted()` y `only_deleted()`;
- joins, filtros, ordenamiento y paginacion ya capturados por `DbSetQuery`.

La proyeccion debe cambiar solo el `SELECT`, no saltarse las reglas runtime de seguridad.

## Joins

El MVP puede permitir columnas de tablas joinadas siempre que no requiera aliases de tabla:

```rust
db.users
    .query()
    .inner_join::<Order>(Predicate::eq(
        Expr::from(User::id),
        Expr::from(Order::user_id),
    ))
    .select((User::email, Order::total_cents))
    .all_as::<UserOrderRow>()
    .await?;
```

Limitaciones del MVP:

- no self-joins;
- no repetir la misma tabla en la misma consulta;
- no aliases de tabla publicos;
- si dos columnas proyectadas tienen el mismo `column_name`, el usuario debe asignar alias explicito a una de ellas.

Esta regla evita DTOs ambiguos cuando dos tablas tienen columnas como `id`, `created_at` o `name`.

## Relacion con `map` en Memoria

`map` en memoria transforma datos despues de leerlos:

```rust
let rows = db.users.query().all().await?;
let dtos = rows
    .into_iter()
    .map(|user| UserListItem {
        id: user.id,
        email: user.email,
    })
    .collect::<Vec<_>>();
```

Esa forma sigue siendo valida cuando ya necesitas la entidad completa, pero no es una proyeccion SQL. Primero ejecuta `all()`, materializa `User` y lee todas las columnas que devuelve `SELECT *`.

Una proyeccion SQL cambia el `SELECT` que llega a SQL Server:

```rust
let dtos = db
    .users
    .query()
    .select((User::id, User::email))
    .all_as::<UserListItem>()
    .await?;
```

```sql
SELECT [dbo].[users].[id] AS [id], [dbo].[users].[email] AS [email]
FROM [dbo].[users]
```

Usa proyecciones SQL cuando quieres reducir ancho de fila, evitar materializar campos no usados o mapear directamente a un DTO de lectura. Usa `map` en memoria cuando el flujo de negocio realmente necesita cargar entidades completas y despues derivar otro shape local.

`all()` / `first()` y `all_as::<T>()` / `first_as::<T>()` son rutas distintas a proposito:

- `all()` y `first()` materializan entidades completas `E`.
- `all_as::<T>()` y `first_as::<T>()` materializan DTOs `T: FromRow` desde la proyeccion actual.
- llamar `select(...)` no cambia el tipo de `all()`; para DTOs usa siempre la ruta `*_as`.

## Aliases y DTOs

Los aliases son parte del contrato entre SQL compilado y `FromRow`.

- columnas proyectadas con `User::email` reciben alias por defecto `"email"`;
- expresiones como `LOWER(email)` requieren `SelectProjection::expr_as(..., "alias")`;
- aliases vacios o duplicados fallan antes de ejecutar la consulta;
- el DTO debe leer los mismos nombres en `FromRow`, por ejemplo `row.get_required_typed::<String>("email_lower")`.

Si una consulta con join proyecta dos columnas con el mismo `column_name`, como `User::id` y `Order::id`, una de ellas debe usar alias explicito:

```rust
db.users
    .query()
    .inner_join::<Order>(Predicate::eq(
        Expr::from(User::id),
        Expr::from(Order::user_id),
    ))
    .select((
        SelectProjection::expr_as(Expr::from(User::id), "user_id"),
        SelectProjection::expr_as(Expr::from(Order::id), "order_id"),
    ))
    .all_as::<UserOrderRow>()
    .await?;
```

## Limites Iniciales

Joins:

- se soportan joins explicitos ya disponibles en `DbSetQuery`;
- no hay aliases de tabla publicos;
- no hay self-joins ni repeticion de la misma tabla en una consulta;
- el filtro automatico de `tenant` y `soft_delete` aplica a la entidad raiz segun las reglas ya existentes; filtros adicionales sobre tablas joinadas deben escribirse explicitamente.

Aliases:

- no hay inferencia automatica de aliases para resolver colisiones entre tablas;
- no hay validacion compile-time de que el DTO lea exactamente todos los aliases proyectados;
- la validacion real ocurre al compilar la consulta y al materializar `FromRow`.

Agregaciones:

- no existe todavia API tipada de alto nivel para `COUNT`, `SUM`, `AVG`, `GROUP BY` o `HAVING` dentro de proyecciones;
- expresiones simples se pueden proyectar con `SelectProjection::expr_as(...)` cuando el AST actual pueda representarlas;
- para agregaciones complejas, grouping, ventanas o SQL especifico de SQL Server, usa raw SQL tipado con `db.raw::<T>(...)`.

## Fuera del Corte Actual

No entran en el corte actual:

- aliases de tabla;
- navigation properties;
- inferencia automatica de DTOs;
- derive especial para DTOs proyectados;
- agregaciones tipadas de alto nivel;
- `GROUP BY` / `HAVING`;
- selects anidados;
- multiple result sets;
- validacion compile-time de que el DTO contiene exactamente los campos proyectados.

Raw SQL tipado sigue siendo el escape hatch para consultas mas complejas mientras el AST crece.

## Referencias relacionadas

- Conceptos centrales: [docs/core-concepts.md](core-concepts.md)
- API publica: [docs/api.md](api.md)
- Query builder publico: [docs/query-builder.md](query-builder.md)
- Relaciones y joins: [docs/relationships.md](relationships.md)
- Raw SQL tipado: [docs/raw-sql.md](raw-sql.md)
- Plan maestro: [docs/plan_orm_sqlserver_tiberius_code_first.md](plan_orm_sqlserver_tiberius_code_first.md)

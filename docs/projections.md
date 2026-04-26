# Proyecciones Tipadas

Las proyecciones tipadas son la Etapa 18 del query builder publico. El objetivo es seleccionar columnas o expresiones concretas desde SQL Server y materializarlas en DTOs con `FromRow`, sin romper la ruta actual que materializa entidades completas con `all()` y `first()`.

## Estado Actual

El AST ya tiene una base parcial:

- `mssql-orm-query::SelectQuery` contiene `projection: Vec<SelectProjection>`.
- `SelectQuery::select(...)` ya acepta elementos convertibles a `SelectProjection`.
- `mssql-orm-sqlserver` ya compila `projection` cuando no esta vacia, usando aliases estables.
- Si `projection` esta vacia, el compilador emite `SELECT *`.
- `DbSetQuery<E>::all()` y `first()` siempre materializan `E` y no exponen proyeccion publica.

La pieza faltante ya no esta en el AST sino en la API publica de `DbSetQuery`: falta exponer una forma ergonomica de construir proyecciones y ejecutar `all_as::<T>()` / `first_as::<T>()`.

## Objetivo

La direccion de API publica es:

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

La API publica debe vivir en `mssql-orm`, encima de `DbSetQuery<E>`, y convertir formas ergonomicas a `SelectProjection`.

Forma inicial suficiente:

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
    .select((
        User::id,
        projection_as(
            Expr::function("LOWER", vec![Expr::from(User::email)]),
            "email_lower",
        ),
    ))
    .all_as::<UserEmailSearchItem>()
    .await?;
```

El helper exacto puede llamarse `projection_as`, `expr_as` o exponerse como constructor de un tipo publico. La decision de nombre debe tomarse en la tarea de implementacion de API, no en `core`.

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

Esto:

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

materializa entidades completas y lee todas las columnas que devuelve `SELECT *`.

Una proyeccion real debe cambiar el SQL:

```sql
SELECT [dbo].[users].[id] AS [id], [dbo].[users].[email] AS [email]
FROM [dbo].[users]
```

La documentacion publica posterior debe explicar esta diferencia, porque `map` en memoria sigue siendo valido pero no reduce el ancho de fila ni el costo de lectura desde SQL Server.

## Fuera del MVP

No entran en el primer corte:

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

## Secuencia Recomendada

1. Agregar una API publica minima en `mssql-orm` para `.select(...)`, `all_as::<T>()` y `first_as::<T>()`.
2. Cubrir snapshots SQL, orden de parametros, `trybuild` publico y materializacion de DTOs.
3. Actualizar `docs/query-builder.md` y `docs/api.md` diferenciando proyecciones reales de `map` en memoria.

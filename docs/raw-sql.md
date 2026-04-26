# Raw SQL Tipado

Raw SQL tipado es el escape hatch publico para consultas y comandos SQL Server que no encajan todavia en el query builder. Vive en la crate publica `mssql-orm`, se usa desde cualquier `DbContext` y esta reexportado por `mssql_orm::prelude::*`.

Esta API no cambia los limites de arquitectura:

- `core` define contratos como `FromRow`, `SqlValue`, `SqlTypeMapping` y `OrmError`.
- `query` solo transporta `CompiledQuery`; no genera SQL para raw SQL.
- `sqlserver` sigue compilando AST del query builder, no strings raw.
- `tiberius` ejecuta el SQL parametrizado.
- `mssql-orm` concentra la API publica `raw<T>()` y `raw_exec()`.

## Cuando Usarlo

Usa raw SQL para casos acotados donde necesitas SQL Server explicito y el query builder actual no cubre la forma requerida:

- consultas de lectura hacia DTOs especificos;
- comandos administrativos o DML puntual;
- SQL escrito a mano con hints, funciones, CTEs o formas aun no modeladas por el AST;
- migraciones o scripts operativos ejecutados desde codigo de aplicacion.

Prefiere `DbSetQuery` cuando puedas expresar la consulta con `filter`, `order_by`, joins, paginacion y `count`. Raw SQL es deliberadamente mas manual.

## API Publica

La API canonica esta disponible sobre cualquier `DbContext` derivado:

```rust
let rows = db
    .raw::<UserListItem>(
        "SELECT id, email FROM [dbo].[users] WHERE active = @P1 ORDER BY email",
    )
    .param(true)
    .all()
    .await?;

let first = db
    .raw::<UserListItem>("SELECT TOP (1) id, email FROM [dbo].[users] WHERE id = @P1")
    .param(user_id)
    .first()
    .await?;

let result = db
    .raw_exec("UPDATE [dbo].[users] SET active = @P1 WHERE id = @P2")
    .params((false, user_id))
    .execute()
    .await?;
```

Shape disponible:

```rust
pub trait DbContext {
    fn raw<T>(&self, sql: impl Into<String>) -> RawQuery<T>
    where
        T: FromRow + Send;

    fn raw_exec(&self, sql: impl Into<String>) -> RawCommand;
}

impl<T> RawQuery<T>
where
    T: FromRow + Send,
{
    pub fn param<P>(self, value: P) -> Self
    where
        P: RawParam;

    pub fn params<P>(self, values: P) -> Self
    where
        P: RawParams;

    pub async fn all(self) -> Result<Vec<T>, OrmError>;
    pub async fn first(self) -> Result<Option<T>, OrmError>;
}

impl RawCommand {
    pub fn param<P>(self, value: P) -> Self
    where
        P: RawParam;

    pub fn params<P>(self, values: P) -> Self
    where
        P: RawParams;

    pub async fn execute(self) -> Result<ExecuteResult, OrmError>;
}
```

`RawQuery<T>` materializa filas mediante `FromRow`. `RawCommand` ejecuta comandos y retorna `ExecuteResult`, con conteo de filas afectadas.

## DTOs de Lectura

Raw SQL puede materializar entidades completas, pero es especialmente util para DTOs de lectura.

```rust
use mssql_orm::prelude::*;

#[derive(Debug, Clone, PartialEq)]
struct UserListItem {
    id: i64,
    email: String,
    active: bool,
}

impl FromRow for UserListItem {
    fn from_row<R: Row>(row: &R) -> Result<Self, OrmError> {
        Ok(Self {
            id: row.get_required_typed::<i64>("id")?,
            email: row.get_required_typed::<String>("email")?,
            active: row.get_required_typed::<bool>("active")?,
        })
    }
}

let users = db
    .raw::<UserListItem>(
        "SELECT id, email, active FROM [dbo].[users] WHERE active = @P1 ORDER BY email",
    )
    .param(true)
    .all()
    .await?;
```

Los nombres usados en `get_required_typed` deben coincidir con las columnas o aliases devueltos por el `SELECT`. Si proyectas expresiones, asigna alias estables:

```rust
let summaries = db
    .raw::<UserSummary>(
        "SELECT id, email, CAST(CASE WHEN active = 1 THEN 1 ELSE 0 END AS bit) AS is_enabled \
         FROM [dbo].[users]",
    )
    .all()
    .await?;
```

## Comandos

Usa `raw_exec()` para comandos que no devuelven filas materializadas:

```rust
let result = db
    .raw_exec("UPDATE [dbo].[users] SET active = @P1 WHERE id = @P2")
    .params((false, user_id))
    .execute()
    .await?;

assert_eq!(result.total(), 1);
```

Tambien sirve para DDL o scripts operativos acotados:

```rust
db.raw_exec(
    "IF OBJECT_ID('[dbo].[archived_users]', 'U') IS NULL \
     CREATE TABLE [dbo].[archived_users] (id BIGINT NOT NULL PRIMARY KEY)",
)
.execute()
.await?;
```

Si el comando necesita devolver filas, usa `raw::<T>()` con un `SELECT` explicito.

## Parametros

Raw SQL usa placeholders SQL Server `@P1`, `@P2`, ..., `@Pn`. El string SQL contiene los placeholders y los valores se agregan con `.param(...)` o `.params(...)`.

```rust
let rows = db
    .raw::<UserListItem>(
        "SELECT id, email, active FROM [dbo].[users] \
         WHERE active = @P1 AND email LIKE @P2",
    )
    .params((true, "%@example.com"))
    .all()
    .await?;
```

Reglas obligatorias:

- los placeholders deben ser continuos desde `@P1` hasta `@Pn`;
- la cantidad de parametros debe coincidir con el mayor placeholder usado;
- `@P1` repetido es valido y reutiliza el primer valor;
- `@P0`, saltos como `@P1` + `@P3`, parametros faltantes y parametros extra fallan antes de ejecutar;
- `SqlValue::Null` y `Option::<T>::None` representan `NULL`;
- los tipos soportados son los que implementan `RawParam`, incluyendo los tipos base mapeados por `SqlTypeMapping`, `&str`, `SqlValue` y `Option<T>`.

Ejemplo valido con placeholder repetido:

```rust
let rows = db
    .raw::<TaskDto>(
        "SELECT id, title FROM [todo].[todo_items] \
         WHERE owner_id = @P1 OR reviewer_id = @P1",
    )
    .param(user_id)
    .all()
    .await?;
```

Para varios valores, la forma recomendada es una tupla:

```rust
.params((true, tenant_id, "open"))
```

`RawParams` soporta tuplas hasta 12 valores y `Vec<T>` cuando `T: RawParam`.

## Seguridad

No interpolar valores de usuario dentro del string SQL. Esta forma es insegura:

```rust
let sql = format!("SELECT id, email FROM [dbo].[users] WHERE email = '{email}'");
let rows = db.raw::<UserListItem>(sql).all().await?;
```

Usa parametros:

```rust
let rows = db
    .raw::<UserListItem>("SELECT id, email, active FROM [dbo].[users] WHERE email = @P1")
    .param(email)
    .all()
    .await?;
```

Raw SQL no hace quoting automatico de identificadores. Si necesitas construir nombres de tabla, schema o columna dinamicamente, usa una lista permitida por tu aplicacion antes de formar el string SQL. No aceptes identificadores directos desde input de usuario.

## Tenant y Soft Delete

Raw SQL no aplica automaticamente filtros ORM de `tenant` ni `soft_delete`.

Si una entidad tiene `#[orm(tenant = CurrentTenant)]`, `DbSetQuery` y las rutas CRUD publicas aplican filtros obligatorios. Raw SQL es un bypass explicito: debes escribir el predicate manualmente.

```rust
let rows = db
    .raw::<TodoItemDto>(
        "SELECT id, title, tenant_id \
         FROM [todo].[todo_items] \
         WHERE tenant_id = @P1 AND deleted_at IS NULL \
         ORDER BY id",
    )
    .param(tenant_id)
    .all()
    .await?;
```

Esto tambien aplica a `soft_delete`: si el modelo usa `#[orm(soft_delete = SoftDelete)]`, raw SQL no agrega `deleted_at IS NULL`, `is_deleted = 0` ni ningun equivalente. El consumidor debe escribirlo.

## Ejecucion y Transacciones

`raw<T>()` y `raw_exec()` usan el mismo `SharedConnection` del contexto, asi que participan en el mismo modelo de conexion que `DbSet`.

Dentro de una transaccion publica, llama raw SQL sobre el contexto transaccional recibido:

```rust
db.transaction(|tx| async move {
    tx.raw_exec("UPDATE [dbo].[users] SET active = @P1 WHERE id = @P2")
        .params((false, user_id))
        .execute()
        .await?;

    let row = tx
        .raw::<UserListItem>("SELECT id, email, active FROM [dbo].[users] WHERE id = @P1")
        .param(user_id)
        .first()
        .await?;

    Ok(row)
})
.await?;
```

Las mismas restricciones de transacciones documentadas en [docs/transactions.md](transactions.md) siguen aplicando.

## Limites

- No hay builder de identificadores ni quoting automatico para raw SQL.
- No hay interpolacion segura tipo format string.
- No hay validacion semantica de columnas, tablas, aliases o DTOs antes de ejecutar.
- No hay soporte especial para multiples result sets.
- No hay streaming publico; `all()` materializa `Vec<T>`.
- No hay aplicacion automatica de `tenant`, `soft_delete` ni otras policies.
- No hay integracion automatica con migraciones, `DbSetQuery` o proyecciones del query builder.

## Cobertura

La API esta cubierta por:

- pruebas unitarias de parametros en `crates/mssql-orm/src/raw_sql.rs`;
- validacion de `@P1` repetido, continuidad de placeholders, `NULL`, tuplas, `Vec<T>` y orden de parametros;
- prueba publica real en `crates/mssql-orm/tests/stage17_raw_sql.rs`, ejecutada contra SQL Server cuando `MSSQL_ORM_TEST_CONNECTION_STRING` esta configurado.

## Referencias relacionadas

- Conceptos centrales: [docs/core-concepts.md](core-concepts.md)
- API publica: [docs/api.md](api.md)
- Query builder publico: [docs/query-builder.md](query-builder.md)
- Proyecciones tipadas: [docs/projections.md](projections.md)
- Transacciones runtime: [docs/transactions.md](transactions.md)
- Plan maestro: [docs/plan_orm_sqlserver_tiberius_code_first.md](plan_orm_sqlserver_tiberius_code_first.md)

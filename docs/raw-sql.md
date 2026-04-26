# Raw SQL Tipado

Raw SQL tipado es el escape hatch publico para consultas y comandos SQL Server que todavia no encajan en el query builder. Vive en la crate publica `mssql-orm`, se reexporta desde `mssql_orm::prelude::*`, reutiliza la ejecucion existente sobre `SharedConnection` y no introduce SQL directo dentro de `core` ni `query`.

## Surface Publica Objetivo

La API canonica queda definida sobre cualquier `DbContext` derivado:

```rust
let rows = db
    .raw::<UserListItem>(
        "SELECT id, email FROM [dbo].[users] WHERE active = @P1 AND tenant_id = @P2",
    )
    .params((true, tenant_id))
    .all()
    .await?;

let first = db
    .raw::<UserListItem>("SELECT id, email FROM [dbo].[users] WHERE id = @P1")
    .param(user_id)
    .first()
    .await?;

let result = db
    .raw_exec("UPDATE [dbo].[users] SET active = @P1 WHERE id = @P2")
    .params((false, user_id))
    .execute()
    .await?;
```

El shape publico disponible es:

```rust
impl AppDbContext {
    pub fn raw<T>(&self, sql: impl Into<String>) -> RawQuery<T>
    where
        T: FromRow + Send;

    pub fn raw_exec(&self, sql: impl Into<String>) -> RawCommand;
}

pub struct RawQuery<T> { /* private fields */ }

impl<T> RawQuery<T>
where
    T: FromRow + Send,
{
    pub fn param<P>(self, value: P) -> Self
    where
        P: SqlTypeMapping;

    pub fn params<P>(self, values: P) -> Self
    where
        P: RawParams;

    pub async fn all(self) -> Result<Vec<T>, OrmError>;
    pub async fn first(self) -> Result<Option<T>, OrmError>;
}

pub struct RawCommand { /* private fields */ }

impl RawCommand {
    pub fn param<P>(self, value: P) -> Self
    where
        P: SqlTypeMapping;

    pub fn params<P>(self, values: P) -> Self
    where
        P: RawParams;

    pub async fn execute(self) -> Result<ExecuteResult, OrmError>;
}
```

`RawQuery<T>` materializa filas mediante `FromRow`; por eso sirve para entidades completas, DTOs o structs de lectura manuales. `RawCommand` ejecuta comandos y retorna `ExecuteResult` desde `mssql-orm-tiberius`.

## Parametros

Raw SQL usa exclusivamente placeholders SQL Server `@P1`, `@P2`, ..., `@Pn`. El usuario escribe esos placeholders en el SQL y agrega valores en el mismo orden logico.

La forma recomendada para dos o mas parametros es `.params((p1, p2, ...))`, porque mantiene visible la correspondencia con `@P1`, `@P2`, ..., `@Pn` en un solo punto de la llamada. `.param(value)` queda como forma incremental conveniente para un unico parametro o para construir consultas paso a paso.

Reglas obligatorias:

- Los placeholders deben ser continuos desde `@P1` hasta `@Pn`.
- La cantidad de parametros debe coincidir con el mayor placeholder usado.
- `@P1` repetido es valido y reutiliza el mismo primer parametro.
- `@P0`, indices vacios, saltos como `@P1` + `@P3` y parametros extra deben fallar antes de ejecutar.
- `SqlValue::Null` debe poder pasarse explicitamente con `.param(SqlValue::Null)`.
- Los tipos soportados son los que implementan `SqlTypeMapping`, mas `SqlValue` como escape explicito para valores ya normalizados.

La implementacion no debe reutilizar sin cambios la validacion actual de `PreparedQuery::validate_parameter_count()`, porque esa funcion cuenta ocurrencias. Para raw SQL se necesita escanear indices, calcular el maximo y verificar continuidad. Ejemplo valido:

```sql
WHERE owner_id = @P1 OR reviewer_id = @P1
```

con un solo parametro.

`RawParams` es el trait publico de soporte reexportado desde `mssql_orm::prelude::*` para habilitar tuplas ergonomicas. Cubre tuplas hasta 12 valores y `Vec<T>` cuando `T: RawParam`:

```rust
.params((true, tenant_id))
.params((SqlValue::Null, "draft".to_string(), 25_i32))
```

No se debe aceptar interpolacion automatica de valores dentro del string SQL.

## Ejecucion

`DbContext::raw<T>(...)` y `DbContext::raw_exec(...)` capturan `SharedConnection` desde `self.shared_connection()`.

Al ejecutar:

1. Construyen un `CompiledQuery { sql, params }` directamente desde el SQL crudo y los parametros normalizados.
2. Validan placeholders raw antes de llamar al adaptador Tiberius.
3. Bloquean la conexion compartida con `SharedConnection::lock().await`.
4. Llaman a `fetch_all`, `fetch_one` o `execute` ya existentes.

Esto preserva la separacion actual:

- `core`: contratos (`FromRow`, `SqlTypeMapping`, `SqlValue`, `OrmError`).
- `query`: solo transporta `CompiledQuery`; no parsea ni genera SQL.
- `sqlserver`: sigue siendo el compilador de AST, pero raw SQL no pasa por AST.
- `tiberius`: ejecuta `CompiledQuery` y bindea parametros.
- `mssql-orm`: concentra la surface publica y la validacion especifica de raw SQL.

## Seguridad y Policies

Raw SQL no aplica automaticamente filtros ORM de `tenant` ni `soft_delete`. Si una entidad tiene `#[orm(tenant = CurrentTenant)]` o `#[orm(soft_delete = SoftDelete)]`, el usuario debe escribir manualmente los predicados necesarios en el SQL crudo. Esto es intencional: raw SQL es un bypass explicito del query builder y de sus filtros implicitos.

Ejemplo:

```rust
let rows = db
    .raw::<TodoItemDto>(
        "SELECT id, title FROM [todo].[todo_items] WHERE tenant_id = @P1 AND deleted_at IS NULL",
    )
    .param(tenant_id)
    .all()
    .await?;
```

Esta decision debe quedar visible en docs y tests porque raw SQL es un bypass deliberado del query builder. La API no debe exponer un modo implicito tipo `apply_tenant()` en esta etapa.

## Limites del Primer Corte

- No hay builder de identificadores ni quoting automatico para raw SQL.
- No hay interpolacion segura tipo format string.
- No hay validacion semantica de columnas, tablas, aliases o DTOs antes de ejecutar.
- No hay soporte especial para multiples result sets.
- No hay streaming publico en el primer corte; `all()` materializa `Vec<T>`.
- No hay integracion automatica con migraciones, `DbSetQuery`, projections ni policies.

## Cobertura Esperada

Las siguientes tareas deben agregar:

- pruebas unitarias de scanner de placeholders con repetidos, saltos, `@P0`, parametros extra, SQL sin parametros y tipos soportados;
- pruebas de construccion de `CompiledQuery` preservando orden de parametros;
- pruebas publicas de `raw<T>().first()`, `raw<T>().all()` y `raw_exec().execute()` contra SQL Server real cuando `MSSQL_ORM_TEST_CONNECTION_STRING` este configurado;
- cobertura documental que advierta que raw SQL no aplica `tenant` ni `soft_delete`.

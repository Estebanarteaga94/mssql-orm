# Plan completo para crear un ORM code-first en Rust para SQL Server basado en Tiberius

**Versión:** 1.0  
**Fecha:** 2026-04-21  
**Objetivo:** diseñar y construir una crate/librería reutilizable en cualquier proyecto Rust, con enfoque **code-first**, funcional sobre **Microsoft SQL Server**, usando **Tiberius** como driver de bajo nivel.

---

## 1. Visión general

El objetivo ya no es crear una capa encima de Diesel. El nuevo objetivo es construir un ORM propio para SQL Server, inspirado conceptualmente en:

- **Entity Framework Core**, por su enfoque de `DbContext`, `DbSet`, modelos code-first, migraciones y evolución del esquema desde el código.
- **Eloquent ORM**, por su experiencia de uso simple, modelo expresivo, CRUD directo y API amigable.

Sin embargo, el diseño debe ser idiomático en Rust. Eso significa que no se debe intentar copiar literalmente EF Core o Eloquent, porque Rust no tiene reflexión runtime como C# ni un modelo dinámico como PHP. La ruta correcta es usar:

- `proc_macros` para generar metadata de entidades.
- Traits para definir contratos estables.
- Query builder tipado.
- Metadata code-first generada en compilación.
- Migraciones versionadas.
- Adaptador Tiberius para ejecutar SQL Server.
- API pública sencilla para que cualquier proyecto pueda usar la librería.

---

## 2. Principios arquitectónicos

### 2.1. Code-first real

El código Rust debe ser la fuente principal de verdad del modelo.

Ejemplo deseado:

```rust
use mssql_orm::prelude::*;

#[derive(Entity, Debug, Clone)]
#[orm(table = "users", schema = "dbo")]
pub struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    pub id: i64,

    #[orm(length = 180)]
    #[orm(unique)]
    pub email: String,

    #[orm(length = 120)]
    pub name: String,

    #[orm(default_sql = "SYSUTCDATETIME()")]
    pub created_at: chrono::NaiveDateTime,

    #[orm(rowversion)]
    pub version: Vec<u8>,
}
```

Desde ese modelo, el ORM debe poder:

1. Generar metadata interna.
2. Crear consultas SQL Server.
3. Mapear filas hacia structs.
4. Generar migraciones.
5. Aplicar migraciones.
6. Validar diferencias entre código y base de datos.

---

### 2.2. SQL Server primero

La librería debe diseñarse inicialmente solo para SQL Server. No conviene intentar soportar PostgreSQL, MySQL o SQLite desde el inicio.

Motivo: SQL Server tiene particularidades importantes:

- Identificadores con corchetes: `[dbo].[users]`.
- Parámetros estilo `@P1`, `@P2`, etc.
- `IDENTITY`.
- `ROWVERSION`.
- `OUTPUT INSERTED.*`.
- Constraints default con nombres propios o generados.
- `OFFSET ... FETCH` para paginación.
- Tipos como `uniqueidentifier`, `datetime2`, `nvarchar`, `varbinary`, `money`, `decimal`, `bit`.

Diseñar bien para SQL Server desde el principio es más importante que hacer una abstracción genérica prematura.

---

### 2.3. Tiberius como driver, no como ORM

Tiberius debe quedar encapsulado en una capa de infraestructura.

Responsabilidad de Tiberius:

- Abrir conexión.
- Ejecutar queries.
- Enviar parámetros.
- Leer resultados.

Responsabilidad del ORM:

- Definir entidades.
- Generar SQL.
- Mapear filas.
- Manejar migraciones.
- Manejar transacciones.
- Exponer API de repositorio, `DbSet`, query builder o Active Record.

---

### 2.4. API pública simple, internals estrictos

La API de usuario debe ser limpia:

```rust
let user = db.users.find(1).await?;

let users = db.users
    .query()
    .filter(User::email.contains("@company.com"))
    .order_by(User::created_at.desc())
    .paginate(1, 20)
    .all()
    .await?;
```

Pero internamente debe haber separación fuerte entre:

- Metadata.
- Expresiones.
- Compilación SQL.
- Parámetros.
- Ejecución.
- Mapeo.
- Migraciones.

---

## 3. Estilo de ORM recomendado

Para lograr una experiencia parecida a EF Core y Eloquent, se recomienda un diseño híbrido:

1. **API principal tipo EF Core**:
   - `DbContext`
   - `DbSet<T>`
   - migraciones code-first
   - transacciones
   - `save_changes` en una etapa avanzada

2. **API opcional tipo Active Record/Eloquent**:
   - `User::find(&db, id)`
   - `user.save(&db)`
   - `user.delete(&db)`
   - `User::query(&db)`

La API principal debe ser estilo EF Core porque encaja mejor con Rust: es explícita, testable y evita conexiones globales ocultas.

La API Active Record debe existir como capa de conveniencia, no como núcleo interno.

---

## 4. API objetivo

### 4.1. Instalación desde un proyecto consumidor

```toml
[dependencies]
mssql_orm = { version = "0.1", features = ["tokio", "chrono", "uuid", "rust_decimal"] }
```

Opcionalmente:

```toml
[dependencies]
mssql_orm = { version = "0.1", features = ["tokio", "chrono", "uuid", "rust_decimal", "pool-bb8"] }
```

---

### 4.2. Definición de entidades

```rust
use mssql_orm::prelude::*;

#[derive(Entity, Debug, Clone)]
#[orm(table = "customers", schema = "sales")]
pub struct Customer {
    #[orm(primary_key)]
    #[orm(identity)]
    pub id: i64,

    #[orm(length = 160)]
    #[orm(index(name = "ix_customers_email"))]
    pub email: String,

    #[orm(length = 120)]
    pub full_name: String,

    #[orm(nullable)]
    #[orm(length = 30)]
    pub phone: Option<String>,

    #[orm(default_sql = "1")]
    pub active: bool,

    #[orm(default_sql = "SYSUTCDATETIME()")]
    pub created_at: chrono::NaiveDateTime,
}
```

---

### 4.3. Definición de contexto

```rust
use mssql_orm::prelude::*;

#[derive(DbContext)]
pub struct AppDbContext {
    pub customers: DbSet<Customer>,
    pub orders: DbSet<Order>,
    pub order_items: DbSet<OrderItem>,
}
```

Uso:

```rust
let db = AppDbContext::connect("server=tcp:localhost,1433;database=AppDb;user=sa;password=Password123;trustServerCertificate=true").await?;

let customer = db.customers.find(1).await?;
```

---

### 4.4. Inserción

```rust
let customer = NewCustomer {
    email: "ana@example.com".to_string(),
    full_name: "Ana Pérez".to_string(),
    phone: None,
};

let saved = db.customers.insert(customer).await?;
```

Para evitar conflictos entre campos generados por base de datos y campos requeridos por el struct principal, el ORM debe permitir modelos de inserción:

```rust
#[derive(Insertable)]
#[orm(entity = Customer)]
pub struct NewCustomer {
    pub email: String,
    pub full_name: String,
    pub phone: Option<String>,
}
```

---

### 4.5. Actualización

```rust
#[derive(Changeset)]
#[orm(entity = Customer)]
pub struct UpdateCustomer {
    pub full_name: Option<String>,
    pub phone: Option<Option<String>>,
    pub active: Option<bool>,
}
```

Uso:

```rust
db.customers
    .update(1, UpdateCustomer {
        full_name: Some("Ana María Pérez".to_string()),
        phone: Some(Some("3001234567".to_string())),
        active: None,
    })
    .await?;
```

Nota: `Option<Option<T>>` permite distinguir entre:

- `None`: no actualizar campo.
- `Some(None)`: actualizar campo a `NULL`.
- `Some(Some(value))`: actualizar campo al valor indicado.

---

### 4.6. Query builder

```rust
let customers = db.customers
    .query()
    .filter(Customer::active.eq(true))
    .filter(Customer::email.contains("@example.com"))
    .order_by(Customer::created_at.desc())
    .limit(50)
    .all()
    .await?;
```

Ejemplo con paginación:

```rust
let page = db.customers
    .query()
    .filter(Customer::active.eq(true))
    .order_by(Customer::id.asc())
    .paginate(PageRequest {
        page: 1,
        page_size: 25,
    })
    .await?;
```

---

### 4.7. API Active Record opcional

```rust
let customer = Customer::find(&db, 1).await?;

let customers = Customer::query(&db)
    .filter(Customer::active.eq(true))
    .all()
    .await?;
```

Guardar:

```rust
let mut customer = Customer::find(&db, 1).await?.unwrap();
customer.full_name = "Nuevo nombre".to_string();
customer.save(&db).await?;
```

Eliminar:

```rust
customer.delete(&db).await?;
```

Esta API debe implementarse sobre `DbSet<T>`, no al revés.

---

## 5. Estructura del workspace

La librería debe desarrollarse como un workspace Rust con varias crates internas.

```text
mssql-orm/
├── Cargo.toml
├── README.md
├── LICENSE
├── crates/
│   ├── mssql-orm/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── prelude.rs
│   │
│   ├── mssql-orm-core/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── entity.rs
│   │       ├── metadata.rs
│   │       ├── model.rs
│   │       ├── value.rs
│   │       ├── error.rs
│   │       ├── types.rs
│   │       └── result.rs
│   │
│   ├── mssql-orm-macros/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── entity_derive.rs
│   │       ├── context_derive.rs
│   │       ├── insertable_derive.rs
│   │       ├── changeset_derive.rs
│   │       ├── migration_derive.rs
│   │       └── parser/
│   │
│   ├── mssql-orm-query/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── expr.rs
│   │       ├── predicate.rs
│   │       ├── select.rs
│   │       ├── insert.rs
│   │       ├── update.rs
│   │       ├── delete.rs
│   │       ├── order.rs
│   │       ├── join.rs
│   │       └── pagination.rs
│   │
│   ├── mssql-orm-sqlserver/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── compiler.rs
│   │       ├── dialect.rs
│   │       ├── types.rs
│   │       ├── quoting.rs
│   │       ├── ddl.rs
│   │       ├── dml.rs
│   │       └── parameter.rs
│   │
│   ├── mssql-orm-tiberius/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── connection.rs
│   │       ├── config.rs
│   │       ├── executor.rs
│   │       ├── row.rs
│   │       ├── transaction.rs
│   │       ├── pool.rs
│   │       └── error.rs
│   │
│   ├── mssql-orm-migrate/
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── migration.rs
│   │       ├── migration_builder.rs
│   │       ├── snapshot.rs
│   │       ├── diff.rs
│   │       ├── operations.rs
│   │       ├── sql_generator.rs
│   │       ├── history.rs
│   │       └── runner.rs
│   │
│   └── mssql-orm-cli/
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           ├── commands/
│           │   ├── init.rs
│           │   ├── migration_add.rs
│           │   ├── database_update.rs
│           │   ├── migration_script.rs
│           │   ├── migration_list.rs
│           │   ├── doctor.rs
│           │   └── entity_new.rs
│           └── config.rs
│
├── examples/
│   ├── basic-crud/
│   ├── migrations/
│   ├── relationships/
│   └── active-record-style/
│
├── tests/
│   ├── integration/
│   ├── sqlserver/
│   └── migration_roundtrip/
│
├── docs/
│   ├── architecture.md
│   ├── code-first.md
│   ├── migrations.md
│   ├── query-builder.md
│   ├── tiberius-adapter.md
│   ├── ai-collaboration.md
│   └── adr/
│       ├── 0001-code-first-design.md
│       ├── 0002-sqlserver-first.md
│       ├── 0003-dbcontext-over-active-record.md
│       └── 0004-migration-snapshots.md
│
└── .github/
    └── workflows/
        ├── ci.yml
        ├── sqlserver-integration.yml
        └── release.yml
```

---

## 6. Responsabilidad de cada crate

### 6.1. `mssql-orm`

Crate pública principal.

Debe reexportar lo necesario:

```rust
pub mod prelude {
    pub use mssql_orm_core::*;
    pub use mssql_orm_macros::*;
    pub use mssql_orm_query::*;
    pub use mssql_orm_tiberius::{MssqlConnection, MssqlPool};
}
```

El usuario final normalmente solo debe importar:

```rust
use mssql_orm::prelude::*;
```

---

### 6.2. `mssql-orm-core`

Contiene contratos fundamentales.

Responsabilidades:

- Traits base.
- Errores.
- Tipos de metadata.
- Representación neutral de entidades.
- Representación de valores SQL.
- Contratos de mapping.

Ejemplo de traits:

```rust
pub trait Entity: Sized + Send + Sync + 'static {
    fn metadata() -> &'static EntityMetadata;
}

pub trait FromRow: Sized {
    fn from_row(row: &MssqlRow) -> Result<Self>;
}

pub trait Insertable<E: Entity> {
    fn values(&self) -> Vec<ColumnValue>;
}

pub trait Changeset<E: Entity> {
    fn changes(&self) -> Vec<ColumnValue>;
}
```

---

### 6.3. `mssql-orm-macros`

Contiene los `derive` macros.

Macros principales:

```rust
#[derive(Entity)]
#[derive(DbContext)]
#[derive(Insertable)]
#[derive(Changeset)]
#[derive(Migration)]
```

Responsabilidades:

- Leer atributos `#[orm(...)]`.
- Generar metadata de entidades.
- Generar columnas estáticas para query builder.
- Generar `FromRow`.
- Generar `Insertable` y `Changeset`.
- Generar metadata del `DbContext`.

---

### 6.4. `mssql-orm-query`

Contiene el AST de queries.

Responsabilidades:

- Expresiones.
- Predicados.
- Select.
- Insert.
- Update.
- Delete.
- Join.
- Ordenamiento.
- Paginación.
- Agregaciones básicas.

No debe saber nada de Tiberius.

---

### 6.5. `mssql-orm-sqlserver`

Compila el AST del query builder a SQL Server.

Responsabilidades:

- Renderizar SQL.
- Crear parámetros `@P1`, `@P2`, `@P3`.
- Quote seguro de identificadores.
- Generar DDL.
- Generar DML.
- Mapear tipos Rust a tipos SQL Server.

Ejemplo de salida:

```sql
SELECT [u].[id], [u].[email], [u].[full_name]
FROM [sales].[customers] AS [u]
WHERE [u].[active] = @P1
ORDER BY [u].[id] ASC
OFFSET @P2 ROWS FETCH NEXT @P3 ROWS ONLY
```

---

### 6.6. `mssql-orm-tiberius`

Adaptador de ejecución.

Responsabilidades:

- Conectar usando Tiberius.
- Ejecutar `CompiledQuery`.
- Convertir parámetros internos a parámetros Tiberius.
- Convertir filas Tiberius a `MssqlRow`.
- Manejar transacciones.
- Opcionalmente manejar pooling.

---

### 6.7. `mssql-orm-migrate`

Motor de migraciones code-first.

Responsabilidades:

- Generar snapshots.
- Comparar modelo actual vs snapshot anterior.
- Detectar cambios.
- Generar operaciones de migración.
- Generar SQL Server DDL.
- Aplicar migraciones.
- Registrar historial en base de datos.

---

### 6.8. `mssql-orm-cli`

Herramienta de línea de comandos.

Comandos esperados:

```bash
mssql-orm init
mssql-orm entity new Customer
mssql-orm migration add CreateCustomers
mssql-orm database update
mssql-orm migration list
mssql-orm migration script
mssql-orm doctor
```

---

## 7. Diseño code-first detallado

### 7.1. Metadata de entidad

Cada entidad debe generar metadata estática.

```rust
pub struct EntityMetadata {
    pub rust_name: &'static str,
    pub schema: &'static str,
    pub table: &'static str,
    pub columns: &'static [ColumnMetadata],
    pub primary_key: PrimaryKeyMetadata,
    pub indexes: &'static [IndexMetadata],
    pub foreign_keys: &'static [ForeignKeyMetadata],
}
```

---

### 7.2. Metadata de columna

```rust
pub struct ColumnMetadata {
    pub rust_field: &'static str,
    pub column_name: &'static str,
    pub sql_type: SqlServerType,
    pub nullable: bool,
    pub primary_key: bool,
    pub identity: Option<IdentityMetadata>,
    pub default_sql: Option<&'static str>,
    pub computed_sql: Option<&'static str>,
    pub rowversion: bool,
    pub insertable: bool,
    pub updatable: bool,
    pub max_length: Option<u32>,
    pub precision: Option<u8>,
    pub scale: Option<u8>,
}
```

---

### 7.3. Atributos soportados

| Atributo | Uso | Ejemplo |
|---|---|---|
| `table` | Nombre de tabla | `#[orm(table = "users")]` |
| `schema` | Schema SQL Server | `#[orm(schema = "dbo")]` |
| `column` | Nombre de columna | `#[orm(column = "email_address")]` |
| `primary_key` | Llave primaria | `#[orm(primary_key)]` |
| `identity` | IDENTITY SQL Server | `#[orm(identity)]` |
| `length` | Longitud para strings/binarios | `#[orm(length = 180)]` |
| `nullable` | Campo nullable | `#[orm(nullable)]` |
| `sql_type` | Tipo SQL explícito | `#[orm(sql_type = "nvarchar(180)")]` |
| `precision` | Precisión decimal | `#[orm(precision = 18, scale = 2)]` |
| `default_sql` | Valor default SQL | `#[orm(default_sql = "SYSUTCDATETIME()")]` |
| `computed_sql` | Columna computada | `#[orm(computed_sql = "[price] * [quantity]")]` |
| `rowversion` | Concurrencia optimista | `#[orm(rowversion)]` |
| `index` | Índice | `#[orm(index(name = "ix_users_email"))]` |
| `unique` | Índice único | `#[orm(unique)]` |
| `foreign_key` | Relación | `#[orm(foreign_key = "users.id")]` |
| `renamed_from` | Ayuda para migraciones | `#[orm(renamed_from = "old_email")]` |
| `ignore` | No persistir | `#[orm(ignore)]` |

---

### 7.4. Convenciones por defecto

Se deben definir convenciones claras:

| Elemento Rust | Convención SQL Server |
|---|---|
| `UserProfile` | tabla `[dbo].[user_profiles]` o `[dbo].[UserProfiles]`, elegir una sola convención |
| `id` | primary key por convención si existe |
| `Option<T>` | columna nullable |
| `String` | `nvarchar(255)` por defecto, salvo `length` explícito |
| `bool` | `bit` |
| `i32` | `int` |
| `i64` | `bigint` |
| `Uuid` | `uniqueidentifier` |
| `NaiveDateTime` | `datetime2` |
| `Decimal` | `decimal(18,2)` por defecto |
| `Vec<u8>` | `varbinary(max)` |

Recomendación: aunque se defina `String -> nvarchar(255)` por defecto, el CLI debe emitir warning cuando un `String` no tenga `length` explícito.

---

### 7.5. Fluent configuration

Los atributos sirven para el 80% de casos, pero se necesita una configuración fluida para escenarios complejos.

Ejemplo:

```rust
impl EntityConfiguration<Customer> for CustomerConfig {
    fn configure(builder: &mut EntityBuilder<Customer>) {
        builder
            .table("customers")
            .schema("sales");

        builder
            .property(Customer::email)
            .has_column_name("email")
            .has_max_length(180)
            .is_required()
            .has_unique_index("ux_customers_email");

        builder
            .property(Customer::created_at)
            .has_default_sql("SYSUTCDATETIME()");
    }
}
```

Como Rust no tiene reflexión runtime tipo C#, esto debe implementarse con símbolos de columna generados por macro:

```rust
Customer::email
Customer::created_at
Customer::id
```

Esos símbolos no son los campos reales, sino constantes generadas que representan columnas.

---

## 8. Diseño del query builder

### 8.1. Objetivo

El query builder debe permitir consultas expresivas y seguras.

```rust
let result = db.customers
    .query()
    .filter(Customer::active.eq(true))
    .filter(Customer::email.ends_with("@example.com"))
    .order_by(Customer::created_at.desc())
    .take(20)
    .all()
    .await?;
```

---

### 8.2. AST interno

El query builder no debe construir strings directamente. Debe construir un AST.

```rust
pub enum Expr {
    Column(ColumnRef),
    Value(SqlValue),
    Binary {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
    },
    Function {
        name: String,
        args: Vec<Expr>,
    },
}
```

---

### 8.3. Predicados

```rust
pub enum Predicate {
    Eq(Expr, Expr),
    Ne(Expr, Expr),
    Gt(Expr, Expr),
    Gte(Expr, Expr),
    Lt(Expr, Expr),
    Lte(Expr, Expr),
    Like(Expr, Expr),
    IsNull(Expr),
    IsNotNull(Expr),
    And(Vec<Predicate>),
    Or(Vec<Predicate>),
    Not(Box<Predicate>),
}
```

---

### 8.4. Parámetros

El compilador SQL Server debe producir:

```rust
pub struct CompiledQuery {
    pub sql: String,
    pub params: Vec<SqlValue>,
}
```

Ejemplo:

```rust
CompiledQuery {
    sql: "SELECT [id], [email] FROM [dbo].[users] WHERE [email] = @P1".to_string(),
    params: vec![SqlValue::String("ana@example.com".to_string())],
}
```

---

### 8.5. Compilación a SQL Server

El dialecto SQL Server debe encargarse de:

- Escapar identificadores.
- Renderizar parámetros.
- Renderizar `TOP`.
- Renderizar `OFFSET FETCH`.
- Renderizar `OUTPUT INSERTED.*`.
- Renderizar `MERGE` si más adelante se soporta upsert.
- Renderizar `IDENTITY_INSERT` si se necesita en migraciones o seeds.

---

## 9. Diseño del adaptador Tiberius

### 9.1. Abstracción de ejecución

El core no debe depender directamente de Tiberius.

```rust
#[async_trait::async_trait]
pub trait Executor {
    async fn execute(&mut self, query: CompiledQuery) -> Result<ExecuteResult>;
    async fn fetch_one<T: FromRow>(&mut self, query: CompiledQuery) -> Result<Option<T>>;
    async fn fetch_all<T: FromRow>(&mut self, query: CompiledQuery) -> Result<Vec<T>>;
}
```

---

### 9.2. Connection

```rust
pub struct MssqlConnection<S> {
    client: tiberius::Client<S>,
}
```

Métodos:

```rust
impl<S> MssqlConnection<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    pub async fn connect(connection_string: &str) -> Result<Self>;
    pub async fn execute(&mut self, query: CompiledQuery) -> Result<ExecuteResult>;
    pub async fn fetch_all<T: FromRow>(&mut self, query: CompiledQuery) -> Result<Vec<T>>;
}
```

---

### 9.3. Binding de parámetros

Debe existir un enum propio:

```rust
pub enum SqlValue {
    Null,
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
    Uuid(uuid::Uuid),
    Decimal(rust_decimal::Decimal),
    DateTime(chrono::NaiveDateTime),
}
```

El adaptador Tiberius debe convertir `SqlValue` a valores aceptados por Tiberius.

Punto técnico importante: si se usa `tiberius::Query::bind`, hay que respetar la cantidad exacta de parámetros esperados por el SQL generado. El compilador SQL debe ser la fuente de verdad para que `@P1..@Pn` coincida con `params.len()`.

---

### 9.4. Lectura de filas

Crear wrapper propio:

```rust
pub struct MssqlRow<'a> {
    inner: &'a tiberius::Row,
}
```

Métodos:

```rust
impl<'a> MssqlRow<'a> {
    pub fn try_get<T>(&self, column: &str) -> Result<Option<T>>;
    pub fn get_required<T>(&self, column: &str) -> Result<T>;
}
```

Los derives de entidad deben generar implementaciones `FromRow`.

---

## 10. Migraciones code-first

### 10.1. Objetivo

El desarrollador debe poder modificar structs Rust y generar migraciones automáticamente.

Flujo esperado:

```bash
mssql-orm migration add CreateCustomers
mssql-orm database update
```

Luego cambia el modelo:

```rust
#[orm(length = 200)]
pub full_name: String,
```

Y ejecuta:

```bash
mssql-orm migration add ExpandCustomerName
mssql-orm database update
```

---

### 10.2. Snapshot del modelo

Cada migración debe guardar un snapshot del modelo después de aplicar esa migración.

Estructura recomendada:

```text
migrations/
├── 20260421120000_create_customers/
│   ├── migration.rs
│   ├── up.sql
│   ├── down.sql
│   └── model_snapshot.json
├── 20260421124500_expand_customer_name/
│   ├── migration.rs
│   ├── up.sql
│   ├── down.sql
│   └── model_snapshot.json
└── mod.rs
```

---

### 10.3. Algoritmo de diff

Entrada:

- Snapshot anterior.
- Metadata actual generada desde código.

Salida:

- Lista ordenada de operaciones.

Operaciones:

```rust
pub enum MigrationOperation {
    CreateSchema(CreateSchema),
    CreateTable(CreateTable),
    DropTable(DropTable),
    RenameTable(RenameTable),
    AddColumn(AddColumn),
    DropColumn(DropColumn),
    RenameColumn(RenameColumn),
    AlterColumn(AlterColumn),
    AddPrimaryKey(AddPrimaryKey),
    DropPrimaryKey(DropPrimaryKey),
    AddForeignKey(AddForeignKey),
    DropForeignKey(DropForeignKey),
    CreateIndex(CreateIndex),
    DropIndex(DropIndex),
    AddDefaultConstraint(AddDefaultConstraint),
    DropDefaultConstraint(DropDefaultConstraint),
}
```

---

### 10.4. Detección de renombres

No se debe asumir automáticamente que una columna eliminada y otra agregada equivalen a un rename.

Debe usarse un hint explícito:

```rust
#[orm(column = "email_address")]
#[orm(renamed_from = "email")]
pub email_address: String,
```

Esto permite generar:

```sql
EXEC sp_rename '[dbo].[customers].[email]', 'email_address', 'COLUMN';
```

---

### 10.5. Cambios destructivos

El CLI debe detectar cambios peligrosos:

- Drop table.
- Drop column.
- Reducir longitud de columna.
- Cambiar tipo incompatible.
- Convertir nullable a non-nullable sin default.

Por defecto, debe fallar con un mensaje claro.

Ejemplo:

```bash
Error: destructive migration detected.
Operation: DropColumn sales.customers.phone
Use --allow-destructive or edit migration manually.
```

---

### 10.6. Tabla de historial

El ORM debe crear una tabla interna:

```sql
CREATE TABLE [dbo].[__mssql_orm_migrations] (
    [id] nvarchar(150) NOT NULL PRIMARY KEY,
    [name] nvarchar(255) NOT NULL,
    [applied_at] datetime2 NOT NULL DEFAULT SYSUTCDATETIME(),
    [checksum] nvarchar(128) NOT NULL,
    [orm_version] nvarchar(50) NOT NULL
);
```

---

### 10.7. Migraciones editables

El archivo generado debe ser editable.

```rust
use mssql_orm::migration::*;

pub struct CreateCustomers;

impl Migration for CreateCustomers {
    fn name(&self) -> &'static str {
        "20260421120000_create_customers"
    }

    fn up(&self, builder: &mut MigrationBuilder) {
        builder.create_table("sales", "customers", |table| {
            table.bigint("id").identity().primary_key();
            table.nvarchar("email", 180).not_null();
            table.nvarchar("full_name", 120).not_null();
            table.nvarchar("phone", 30).nullable();
            table.bit("active").not_null().default_sql("1");
            table.datetime2("created_at").not_null().default_sql("SYSUTCDATETIME()");
        });
    }

    fn down(&self, builder: &mut MigrationBuilder) {
        builder.drop_table("sales", "customers");
    }
}
```

El CLI también puede generar `up.sql` y `down.sql` para revisión.

---

## 11. Relaciones

### 11.1. Relación uno a muchos

```rust
#[derive(Entity, Debug, Clone)]
#[orm(table = "orders", schema = "sales")]
pub struct Order {
    #[orm(primary_key)]
    #[orm(identity)]
    pub id: i64,

    #[orm(foreign_key = "sales.customers.id")]
    pub customer_id: i64,

    pub total: rust_decimal::Decimal,
}
```

Consulta:

```rust
let orders = db.orders
    .query()
    .filter(Order::customer_id.eq(customer_id))
    .all()
    .await?;
```

---

### 11.2. Includes / eager loading

Etapa avanzada:

```rust
let customers = db.customers
    .query()
    .include(Customer::orders)
    .all()
    .await?;
```

Recomendación: no implementar eager loading en el MVP. Primero implementar metadata de relaciones y joins explícitos.

---

### 11.3. Lazy loading

No recomendado para la primera versión.

Motivo:

- En Rust async, lazy loading implícito puede generar APIs incómodas.
- Oculta consultas a base de datos.
- Puede producir N+1 queries.

Si se implementa, debe ser opt-in.

---

### 11.4. Muchos a muchos

Recomendación: modelar primero con entidad intermedia explícita.

```rust
#[derive(Entity)]
#[orm(table = "user_roles", schema = "auth")]
pub struct UserRole {
    #[orm(primary_key)]
    pub user_id: i64,

    #[orm(primary_key)]
    pub role_id: i64,
}
```

El many-to-many automático debe dejarse para una etapa posterior.

---

## 12. Change tracking y `save_changes`

### 12.1. Problema

Entity Framework tiene change tracking avanzado. Rust no ofrece proxies dinámicos de la misma forma.

Por eso, no se recomienda iniciar con un change tracker completo.

---

### 12.2. MVP sin change tracker

Primera versión:

```rust
db.customers.insert(new_customer).await?;
db.customers.update(id, changeset).await?;
db.customers.delete(id).await?;
```

Esto es más explícito, seguro e idiomático.

---

### 12.3. Change tracker etapa avanzada

Diseño futuro:

```rust
let mut customer = db.customers.find_tracked(1).await?;
customer.full_name = "Nuevo nombre".to_string();

db.save_changes().await?;
```

Internamente:

```rust
pub enum EntityState {
    Unchanged,
    Added,
    Modified,
    Deleted,
}

pub struct Tracked<T> {
    original: T,
    current: T,
    state: EntityState,
}
```

Limitaciones:

- Requiere `Clone` o snapshot serializable.
- Requiere comparar campos.
- Debe manejar concurrencia.
- Debe tener cuidado con lifetimes.

---

## 13. Concurrencia optimista

Soporte recomendado con `rowversion`.

Entidad:

```rust
#[orm(rowversion)]
pub version: Vec<u8>,
```

Update generado:

```sql
UPDATE [sales].[customers]
SET [full_name] = @P1
OUTPUT INSERTED.*
WHERE [id] = @P2 AND [version] = @P3
```

Si no se actualiza ninguna fila, retornar:

```rust
OrmError::ConcurrencyConflict
```

---

## 14. Transacciones

### 14.1. API recomendada

```rust
db.transaction(|tx| async move {
    let customer = tx.customers.insert(new_customer).await?;
    tx.orders.insert(new_order_for(customer.id)).await?;
    Ok(())
}).await?;
```

---

### 14.2. Implementación

Implementar con comandos SQL explícitos:

```sql
BEGIN TRANSACTION;
COMMIT TRANSACTION;
ROLLBACK TRANSACTION;
```

La API debe garantizar:

- Commit si el closure termina en `Ok`.
- Rollback si el closure termina en `Err`.
- Rollback si ocurre error antes del commit.

Evitar depender de rollback en `Drop`, porque en Rust async no se puede ejecutar `await` dentro de `Drop`.

---

### 14.3. Savepoints

Etapa posterior:

```sql
SAVE TRANSACTION savepoint_name;
ROLLBACK TRANSACTION savepoint_name;
```

API:

```rust
tx.savepoint("before_order_items").await?;
```

---

## 15. Pooling

### 15.1. MVP

Primera versión puede funcionar con conexión directa.

```rust
let db = AppDbContext::connect(connection_string).await?;
```

---

### 15.2. Pool opcional

Segunda etapa:

```rust
let pool = MssqlPool::builder()
    .max_size(10)
    .connect(connection_string)
    .await?;

let db = AppDbContext::from_pool(pool);
```

La integración debe estar detrás de feature flag:

```toml
features = ["pool-bb8"]
```

---

## 16. Raw SQL

Debe existir, pero claramente separado.

```rust
let rows = db.raw_query("SELECT * FROM [dbo].[users] WHERE [email] = @P1")
    .bind("ana@example.com")
    .fetch_all::<User>()
    .await?;
```

También:

```rust
db.raw_execute("UPDATE [dbo].[users] SET [active] = 0 WHERE [id] = @P1")
    .bind(user_id)
    .execute()
    .await?;
```

Reglas:

- Raw SQL debe usar parámetros.
- No concatenar valores automáticamente.
- Documentar claramente los riesgos.

---

## 17. Seguridad

### 17.1. Protección contra SQL injection

Toda query generada debe usar parámetros.

Incorrecto:

```sql
WHERE [email] = 'valor concatenado'
```

Correcto:

```sql
WHERE [email] = @P1
```

---

### 17.2. Identificadores seguros

Los nombres de tablas, columnas, schemas e índices deben venir de metadata validada, no de input del usuario.

Implementar función:

```rust
fn quote_identifier(identifier: &str) -> Result<String>;
```

Debe rechazar caracteres inválidos o escapar correctamente `]`.

---

### 17.3. Logs sin datos sensibles

El ORM debe permitir loggear SQL sin exponer parámetros por defecto.

Ejemplo:

```text
SQL: SELECT [id], [email] FROM [dbo].[users] WHERE [email] = @P1
Params: [REDACTED]
Duration: 12ms
```

---

## 18. Observabilidad

Usar `tracing`.

Eventos recomendados:

- `orm.query.start`
- `orm.query.finish`
- `orm.query.error`
- `orm.migration.start`
- `orm.migration.finish`
- `orm.transaction.begin`
- `orm.transaction.commit`
- `orm.transaction.rollback`

Span ejemplo:

```rust
tracing::info_span!(
    "mssql_orm.query",
    table = "sales.customers",
    operation = "select"
);
```

Métricas futuras:

- Query duration.
- Query count.
- Slow queries.
- Pool acquisition time.
- Migration duration.
- Transaction rollback count.

---

## 19. Errores

Crear error propio estable.

```rust
#[derive(thiserror::Error, Debug)]
pub enum OrmError {
    #[error("connection error: {0}")]
    Connection(String),

    #[error("query error: {0}")]
    Query(String),

    #[error("mapping error: {0}")]
    Mapping(String),

    #[error("migration error: {0}")]
    Migration(String),

    #[error("entity not found")]
    NotFound,

    #[error("concurrency conflict")]
    ConcurrencyConflict,

    #[error("invalid model: {0}")]
    InvalidModel(String),

    #[error("unsupported operation: {0}")]
    Unsupported(String),
}
```

Nunca exponer directamente `tiberius::error::Error` en la API pública principal.

---

## 20. Trabajo con IA

Como el proyecto se va a trabajar en conjunto con IA, hay que diseñar el repositorio para que la IA pueda colaborar sin romper arquitectura.

### 20.1. Documentación especial para IA

Crear:

```text
docs/ai/
├── context.md
├── architecture-rules.md
├── coding-standards.md
├── sqlserver-dialect-rules.md
├── migration-engine-rules.md
├── macro-rules.md
├── testing-rules.md
└── prompts/
    ├── implement-feature.md
    ├── review-code.md
    ├── generate-tests.md
    ├── refactor-module.md
    └── write-docs.md
```

---

### 20.2. Reglas para IA

Archivo: `docs/ai/architecture-rules.md`

Contenido mínimo:

```md
# Architecture Rules for AI Contributions

1. Do not make mssql-orm-core depend on Tiberius.
2. Do not generate SQL directly in the query builder.
3. SQL generation belongs only to mssql-orm-sqlserver.
4. Tiberius execution belongs only to mssql-orm-tiberius.
5. Public API must remain in mssql-orm.
6. All SQL must use parameters, never value concatenation.
7. Every migration diff feature requires tests.
8. Every generated macro feature requires compile-fail and compile-pass tests.
9. Avoid hidden global state.
10. Prefer explicit async APIs.
```

---

### 20.3. ADRs obligatorios

Cada decisión grande debe registrarse como ADR.

Ejemplo:

```text
docs/adr/0001-code-first-design.md
docs/adr/0002-sqlserver-first.md
docs/adr/0003-dbcontext-over-active-record.md
docs/adr/0004-migration-snapshots.md
docs/adr/0005-no-lazy-loading-in-mvp.md
```

Formato:

```md
# ADR 0001: Code-first design

## Status
Accepted

## Context
...

## Decision
...

## Consequences
...
```

---

### 20.4. Contratos por módulo

Cada crate debe tener un `README.md` con:

- Propósito.
- Qué puede depender de qué.
- Qué no debe hacer.
- API principal.
- Ejemplos.
- Casos de prueba obligatorios.

Esto ayuda a que una IA pueda trabajar por módulos sin mezclar responsabilidades.

---

### 20.5. Prompts internos recomendados

`docs/ai/prompts/implement-feature.md`:

```md
You are modifying the mssql-orm repository.
Respect the dependency boundaries:
- core cannot depend on tiberius
- query cannot generate SQL strings
- sqlserver compiler cannot execute queries
- tiberius adapter cannot know about migrations

Before coding:
1. Identify the target crate.
2. List affected traits.
3. Add or update tests.
4. Keep public API stable unless explicitly requested.
```

---

## 21. Testing

### 21.1. Unit tests

Por módulo:

- Metadata parsing.
- Attribute parsing.
- Query AST.
- SQL compilation.
- Parameter ordering.
- Type mapping.
- Migration diff.
- DDL generation.

---

### 21.2. Macro tests

Usar `trybuild`.

Casos:

- Entidad válida.
- Entidad sin primary key.
- `String` sin length: warning o error según política.
- `identity` en campo no numérico.
- `rowversion` con tipo inválido.
- `foreign_key` mal formado.

---

### 21.3. Integration tests contra SQL Server

Casos:

- Conexión.
- Insert.
- Select.
- Update.
- Delete.
- Transacción commit.
- Transacción rollback.
- Migración inicial.
- Migración incremental.
- Rowversion conflict.
- Nullable fields.
- Decimal precision.
- Unique constraint violation.

---

### 21.4. Snapshot tests de SQL

Ejemplo:

```rust
#[test]
fn select_active_customers_sql() {
    let query = Customer::query()
        .filter(Customer::active.eq(true))
        .order_by(Customer::id.asc())
        .compile();

    assert_snapshot!(query.sql);
}
```

---

### 21.5. Migration roundtrip tests

Flujo:

1. Crear modelo inicial.
2. Generar migración.
3. Aplicar migración.
4. Consultar schema real.
5. Comparar schema real vs metadata esperada.
6. Aplicar down migration.
7. Verificar reversión.

---

## 22. Roadmap por etapas

## Etapa 0: Fundamentos del proyecto

### Objetivo

Crear workspace, reglas de arquitectura, CI, documentación base y estrategia de colaboración con IA.

### Entregables

- Workspace Rust.
- Crates vacías con dependencias correctas.
- README principal.
- ADR iniciales.
- Documentos `docs/ai`.
- CI básico.
- Formato con `rustfmt`.
- Lint con `clippy`.

### Definition of Done

- `cargo check --workspace` exitoso.
- `cargo test --workspace` exitoso.
- Documentación base creada.
- Estructura de dependencias validada.

---

## Etapa 1: Core metadata y macros de entidad

### Objetivo

Permitir declarar entidades code-first y generar metadata.

### Entregables

- Trait `Entity`.
- Structs `EntityMetadata`, `ColumnMetadata`, `IndexMetadata`, `ForeignKeyMetadata`.
- Macro `#[derive(Entity)]`.
- Atributos básicos:
  - `table`
  - `schema`
  - `primary_key`
  - `identity`
  - `length`
  - `nullable`
  - `default_sql`
  - `index`
  - `unique`
- Tests `trybuild`.

### Definition of Done

- Se puede escribir una entidad y obtener metadata.
- La macro genera columnas estáticas para query builder.
- La macro falla con errores claros ante modelos inválidos.

---

## Etapa 2: Mapping de filas y valores

### Objetivo

Convertir filas SQL Server a structs Rust y structs Rust a valores persistibles.

### Entregables

- Trait `FromRow`.
- Trait `Insertable`.
- Trait `Changeset`.
- Enum `SqlValue`.
- Mapeo Rust -> SQL Server.
- Derives:
  - `#[derive(Insertable)]`
  - `#[derive(Changeset)]`

### Definition of Done

- Se puede mapear una fila simulada a entidad.
- Se pueden extraer valores para insert/update.
- Hay tests de tipos básicos.

---

## Etapa 3: SQL Server compiler

### Objetivo

Compilar AST de operaciones a SQL Server parametrizado.

### Entregables

- `CompiledQuery`.
- Quote de identificadores.
- Render de parámetros `@P1..@Pn`.
- Compiler para:
  - select simple
  - insert
  - update
  - delete
  - count
- Snapshot tests de SQL.

### Definition of Done

- Toda query generada usa parámetros.
- El orden de parámetros coincide con el SQL.
- Los identificadores están protegidos.

---

## Etapa 4: Adaptador Tiberius

### Objetivo

Ejecutar queries reales en SQL Server.

### Entregables

- `MssqlConnection`.
- Configuración desde connection string.
- `Executor` implementado.
- Binding de parámetros.
- Lectura de resultados.
- Conversión de errores.
- Tests de integración.

### Definition of Done

- Se puede conectar a SQL Server.
- Se puede ejecutar `SELECT 1`.
- Se pueden hacer insert/select/update/delete con entidades reales.

---

## Etapa 5: DbContext y DbSet

### Objetivo

Exponer API tipo EF Core.

### Entregables

- `DbContext` trait.
- `DbSet<T>`.
- Macro `#[derive(DbContext)]`.
- API:
  - `find`
  - `insert`
  - `update`
  - `delete`
  - `query`
- Ejemplo `basic-crud`.

### Definition of Done

- Un proyecto consumidor puede definir `AppDbContext`.
- Se puede hacer CRUD con `db.users`.
- API documentada.

---

## Etapa 6: Query builder público

### Objetivo

Permitir consultas expresivas.

### Entregables

- Filtros:
  - `eq`
  - `ne`
  - `gt`
  - `gte`
  - `lt`
  - `lte`
  - `contains`
  - `starts_with`
  - `ends_with`
  - `is_null`
  - `is_not_null`
- Composición:
  - `and`
  - `or`
  - `not`
- Ordenamiento.
- Limit/take.
- Paginación.

### Definition of Done

- Se pueden construir consultas comunes sin SQL manual.
- SQL generado tiene tests snapshot.
- Parámetros seguros.

---

## Etapa 7: Migraciones code-first MVP

### Objetivo

Generar y aplicar migraciones desde modelos Rust.

### Entregables

- `ModelSnapshot`.
- Diff engine básico.
- Operaciones:
  - create table
  - drop table
  - add column
  - drop column
  - alter column
  - create index
  - drop index
  - add primary key
- CLI:
  - `migration add`
  - `database update`
  - `migration list`
- Tabla `__mssql_orm_migrations`.

### Definition of Done

- Se puede crear DB desde cero con modelos Rust.
- Se puede agregar una columna y generar migración.
- Se puede aplicar migración en SQL Server real.

---

## Etapa 8: Transacciones

### Objetivo

Agregar soporte seguro de transacciones.

### Entregables

- `db.transaction(...)`.
- Commit automático en `Ok`.
- Rollback automático en `Err`.
- Tests commit/rollback.

### Definition of Done

- Una operación compuesta puede confirmarse o revertirse.
- No se depende de `Drop` async.

---

## Etapa 9: Relaciones básicas

### Objetivo

Soportar metadata y consultas para relaciones.

### Entregables

- Foreign keys en metadata.
- DDL de foreign keys.
- Joins explícitos.
- Índices para FKs.
- Delete behavior:
  - no action
  - cascade
  - set null

### Definition of Done

- Se pueden definir relaciones uno a muchos.
- Las migraciones generan FKs.
- Se pueden hacer joins básicos.

---

## Etapa 10: Active Record opcional

### Objetivo

Agregar una experiencia similar a Eloquent sin comprometer la arquitectura.

### Entregables

- Trait `ActiveRecord`.
- `Entity::find(&db, id)`.
- `entity.save(&db)`.
- `entity.delete(&db)`.
- `Entity::query(&db)`.

### Definition of Done

- API Active Record funciona encima de `DbSet`.
- No introduce conexión global.
- Está detrás de feature flag si es necesario.

---

## Etapa 11: Concurrencia optimista

### Objetivo

Soportar `rowversion`.

### Entregables

- Atributo `#[orm(rowversion)]`.
- Update/delete con check de rowversion.
- Error `ConcurrencyConflict`.
- Tests de conflicto.

### Definition of Done

- Dos actualizaciones concurrentes no pisan datos silenciosamente.

---

## Etapa 12: Change tracking experimental

### Objetivo

Acercarse a EF Core con `save_changes`.

### Entregables

- `Tracked<T>`.
- `EntityState`.
- `find_tracked`.
- `add`.
- `remove`.
- `save_changes`.

### Definition of Done

- Se puede modificar una entidad trackeada y persistir cambios.
- Está marcado como experimental.
- No reemplaza la API explícita.

---

## Etapa 13: Migraciones avanzadas

### Objetivo

Hacer el sistema de migraciones más robusto.

### Entregables

- Rename table.
- Rename column.
- Default constraints nombrados.
- Computed columns.
- Foreign keys completas.
- Índices compuestos.
- Unique constraints.
- Scripts idempotentes.
- `migration script --from --to`.

### Definition of Done

- Flujo de migraciones usable en ambientes reales.
- Scripts revisables para producción.

---

## Etapa 14: Pooling y producción

### Objetivo

Preparar el ORM para aplicaciones web y servicios.

### Entregables

- Pool opcional.
- Timeouts.
- Retry policy opcional.
- Logging con `tracing`.
- Slow query logs.
- Health checks.

### Definition of Done

- Puede usarse en una API web async.
- Hay ejemplo con Axum o Actix Web.

---

## Etapa 15: Documentación pública y release

### Objetivo

Preparar publicación.

### Entregables

- README completo.
- Guía quickstart.
- Guía code-first.
- Guía migraciones.
- Guía query builder.
- Guía transacciones.
- Guía relaciones.
- API docs.
- Ejemplos completos.
- Changelog.

### Definition of Done

- Un usuario externo puede instalar y hacer CRUD siguiendo docs.
- Crates listas para publicar.

---

## 23. Riesgos principales

### 23.1. Proc macros demasiado complejas

Riesgo: intentar generar demasiada lógica desde macros.

Mitigación:

- Las macros solo deben generar metadata y código repetitivo.
- La lógica debe vivir en crates normales.

---

### 23.2. Migraciones destructivas

Riesgo: pérdida de datos por diffs automáticos.

Mitigación:

- Warnings fuertes.
- Bloqueo por defecto.
- `--allow-destructive` explícito.
- Migraciones editables.

---

### 23.3. Change tracking complejo

Riesgo: gastar mucho tiempo intentando copiar EF Core.

Mitigación:

- MVP explícito sin change tracking.
- `save_changes` en fase experimental.

---

### 23.4. Relaciones y eager loading

Riesgo: N+1 queries o API difícil.

Mitigación:

- Joins explícitos primero.
- Includes después.
- Lazy loading fuera del MVP.

---

### 23.5. Acoplamiento a Tiberius

Riesgo: que todo el ORM dependa de Tiberius.

Mitigación:

- Tiberius solo en `mssql-orm-tiberius`.
- Core no depende de Tiberius.
- Query builder no depende de Tiberius.
- SQL compiler no depende de Tiberius.

---

## 24. Decisiones técnicas recomendadas

| Tema | Decisión recomendada |
|---|---|
| Estilo principal | `DbContext` + `DbSet` |
| Estilo secundario | Active Record opcional |
| Base de datos inicial | SQL Server solamente |
| Driver | Tiberius |
| Async runtime inicial | Tokio |
| Migraciones | Code-first con snapshots |
| SQL generation | AST + compiler SQL Server |
| Macros | Para metadata, mapping y boilerplate |
| Change tracking | No en MVP |
| Lazy loading | No en MVP |
| Pooling | Opcional en fase posterior |
| Raw SQL | Permitido, parametrizado |
| IA | Documentación y reglas por módulo |

---

## 25. MVP recomendado

El MVP no debe intentar ser EF Core completo.

Debe incluir:

1. Definición de entidades con `#[derive(Entity)]`.
2. Metadata code-first.
3. Mapeo de filas.
4. CRUD básico.
5. Query builder simple.
6. SQL Server compiler.
7. Ejecución con Tiberius.
8. `DbContext` y `DbSet`.
9. Migraciones iniciales básicas.
10. CLI mínima.

No incluir en MVP:

- Change tracking completo.
- Lazy loading.
- Eager loading automático.
- Many-to-many automático.
- Soporte multi-database.
- LINQ-like avanzado.
- Upsert complejo.
- Bulk operations.

---

## 26. Ejemplo completo esperado del MVP

### Entidad

```rust
#[derive(Entity, Debug, Clone)]
#[orm(table = "users", schema = "dbo")]
pub struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    pub id: i64,

    #[orm(length = 180)]
    #[orm(unique)]
    pub email: String,

    #[orm(length = 100)]
    pub name: String,

    #[orm(default_sql = "1")]
    pub active: bool,
}
```

### Insert model

```rust
#[derive(Insertable)]
#[orm(entity = User)]
pub struct NewUser {
    pub email: String,
    pub name: String,
}
```

### Update model

```rust
#[derive(Changeset)]
#[orm(entity = User)]
pub struct UpdateUser {
    pub name: Option<String>,
    pub active: Option<bool>,
}
```

### Contexto

```rust
#[derive(DbContext)]
pub struct AppDb {
    pub users: DbSet<User>,
}
```

### Uso

```rust
#[tokio::main]
async fn main() -> mssql_orm::Result<()> {
    let db = AppDb::connect(std::env::var("DATABASE_URL")?).await?;

    let user = db.users.insert(NewUser {
        email: "esteban@example.com".to_string(),
        name: "Esteban".to_string(),
    }).await?;

    let found = db.users.find(user.id).await?;

    let active_users = db.users
        .query()
        .filter(User::active.eq(true))
        .order_by(User::id.desc())
        .take(10)
        .all()
        .await?;

    db.users.update(user.id, UpdateUser {
        name: Some("Esteban Arteaga".to_string()),
        active: None,
    }).await?;

    Ok(())
}
```

---

## 27. Checklist de implementación

### Base

- [ ] Crear workspace.
- [ ] Crear crates internas.
- [ ] Configurar CI.
- [ ] Crear ADRs.
- [ ] Crear docs para IA.

### Core

- [ ] `Entity` trait.
- [ ] `EntityMetadata`.
- [ ] `ColumnMetadata`.
- [ ] `SqlServerType`.
- [ ] `SqlValue`.
- [ ] `OrmError`.

### Macros

- [ ] `derive(Entity)`.
- [ ] Parser de atributos.
- [ ] Generación de metadata.
- [ ] Generación de columnas estáticas.
- [ ] `derive(Insertable)`.
- [ ] `derive(Changeset)`.
- [ ] `derive(DbContext)`.

### Query

- [ ] AST de expresiones.
- [ ] Predicados.
- [ ] Select builder.
- [ ] Insert builder.
- [ ] Update builder.
- [ ] Delete builder.
- [ ] Ordenamiento.
- [ ] Paginación.

### SQL Server

- [ ] Quote identifiers.
- [ ] Parámetros `@P1..@Pn`.
- [ ] Compile select.
- [ ] Compile insert.
- [ ] Compile update.
- [ ] Compile delete.
- [ ] Compile count.
- [ ] DDL create table.
- [ ] DDL alter table.

### Tiberius

- [ ] Connection.
- [ ] Config.
- [ ] Executor.
- [ ] Param binding.
- [ ] Row mapping.
- [ ] Error mapping.
- [ ] Integration tests.

### DbContext

- [ ] `DbContext` trait.
- [ ] `DbSet<T>`.
- [ ] CRUD.
- [ ] Query API.
- [ ] Examples.

### Migraciones

- [ ] Snapshot model.
- [ ] Diff engine.
- [ ] Migration operations.
- [ ] SQL generator.
- [ ] Migration history table.
- [ ] CLI migration add.
- [ ] CLI database update.

### Producción

- [ ] Transactions.
- [ ] Pooling opcional.
- [ ] Tracing.
- [ ] Slow query logs.
- [ ] Docs.
- [ ] Release.

---

## 28. Resultado esperado

Al final, la crate debe permitir esto:

```rust
#[derive(Entity)]
#[orm(table = "products")]
pub struct Product {
    #[orm(primary_key, identity)]
    pub id: i64,

    #[orm(length = 120)]
    pub name: String,

    #[orm(precision = 18, scale = 2)]
    pub price: rust_decimal::Decimal,
}

#[derive(DbContext)]
pub struct AppDb {
    pub products: DbSet<Product>,
}
```

Y luego:

```bash
mssql-orm migration add CreateProducts
mssql-orm database update
```

Y en código:

```rust
let products = db.products
    .query()
    .filter(Product::price.gt(dec!(100)))
    .order_by(Product::name.asc())
    .all()
    .await?;
```

Esa sería una experiencia code-first real, parecida en filosofía a EF Core y Eloquent, pero diseñada correctamente para Rust, SQL Server y Tiberius.

---

## 29. Referencias técnicas sugeridas

- Entity Framework Core migrations: https://learn.microsoft.com/en-us/ef/core/managing-schemas/migrations/
- Entity Framework Core overview: https://learn.microsoft.com/en-us/ef/core/
- Laravel Eloquent ORM: https://laravel.com/docs/13.x/eloquent
- Laravel migrations: https://laravel.com/docs/13.x/migrations
- Tiberius crate: https://docs.rs/tiberius
- Tiberius Query: https://docs.rs/tiberius/latest/tiberius/struct.Query.html
- Tiberius Client: https://docs.rs/tiberius/latest/tiberius/struct.Client.html

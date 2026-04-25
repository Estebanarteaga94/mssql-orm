# Quickstart

Guía mínima para conectar `mssql-orm`, crear un modelo, usar `DbContext`, hacer CRUD base y ejecutar una consulta con el query builder público.

Este quickstart está pensado para el estado real actual del repositorio:

- SQL Server como único backend
- crate pública `mssql-orm`
- `#[derive(Entity)]`, `#[derive(DbContext)]`, `#[derive(Insertable)]`, `#[derive(Changeset)]`
- `DbSet::find/insert/update/delete`
- `DbSet::query().filter().order_by().take().all()`

## 1. Preparar una tabla de prueba

Usa una base como `tempdb` y crea una tabla mínima:

```sql
IF OBJECT_ID('dbo.quickstart_users', 'U') IS NOT NULL
BEGIN
    DROP TABLE dbo.quickstart_users;
END;
GO

CREATE TABLE dbo.quickstart_users (
    id BIGINT IDENTITY(1,1) NOT NULL PRIMARY KEY,
    name NVARCHAR(120) NOT NULL,
    active BIT NOT NULL
);
GO
```

Si estás en tu entorno local:

```bash
sqlcmd -S localhost -U '<usuario>' -P '<password>' -d tempdb -C -Q "IF OBJECT_ID('dbo.quickstart_users', 'U') IS NOT NULL DROP TABLE dbo.quickstart_users; CREATE TABLE dbo.quickstart_users (id BIGINT IDENTITY(1,1) NOT NULL PRIMARY KEY, name NVARCHAR(120) NOT NULL, active BIT NOT NULL);"
```

## 2. Crear un proyecto Rust

```bash
cargo new quickstart-app
cd quickstart-app
```

Mientras el crate no esté publicado, dentro de un checkout local del repo puedes apuntar por `path`:

```toml
[dependencies]
mssql-orm = { path = "../mssql-orm/crates/mssql-orm" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## 3. Definir entidad, modelos de persistencia y contexto

```rust
use mssql_orm::prelude::*;

#[derive(Entity, Debug, Clone, PartialEq)]
#[orm(table = "quickstart_users", schema = "dbo")]
struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(length = 120)]
    name: String,

    active: bool,
}

#[derive(Insertable, Debug, Clone)]
#[orm(entity = User)]
struct NewUser {
    name: String,
    active: bool,
}

#[derive(Changeset, Debug, Clone)]
#[orm(entity = User)]
struct UpdateUser {
    name: Option<String>,
    active: Option<bool>,
}

#[derive(DbContext, Debug, Clone)]
struct AppDb {
    pub users: DbSet<User>,
}
```

Configura la conexión mediante una cadena propia de tu entorno, por ejemplo:

```text
Server=localhost;Database=tempdb;User Id=<usuario>;Password=<password>;TrustServerCertificate=True;Encrypt=False;
```

## 4. Conectar y ejecutar CRUD + query builder

```rust
use mssql_orm::prelude::*;

#[tokio::main]
async fn main() -> Result<(), OrmError> {
    let db = AppDb::connect(
        "Server=localhost;Database=tempdb;User Id=<usuario>;Password=<password>;TrustServerCertificate=True;Encrypt=False;"
    )
    .await?;

    let inserted = db
        .users
        .insert(NewUser {
            name: "Ana".to_string(),
            active: true,
        })
        .await?;

    let found = db.users.find(inserted.id).await?;

    let active_users = db
        .users
        .query()
        .filter(User::active.eq(true))
        .order_by(User::name.asc())
        .take(10)
        .all()
        .await?;

    let updated = db
        .users
        .update(
            inserted.id,
            UpdateUser {
                name: Some("Ana Maria".to_string()),
                active: Some(false),
            },
        )
        .await?;

    let deleted = db.users.delete(inserted.id).await?;

    println!("found: {found:?}");
    println!("active users: {}", active_users.len());
    println!("updated: {updated:?}");
    println!("deleted: {deleted}");

    Ok(())
}
```

## 5. Ejecutar

```bash
cargo run
```

## Qué demuestra este quickstart

- metadata y `FromRow` generados desde `#[derive(Entity)]`
- conexión pública vía `DbContext::connect(...)`
- insert materializado con retorno tipado
- `find` por primary key simple
- query builder público con `filter`, `order_by` y `take`
- `update` con `Changeset`
- `delete` por primary key simple

## Siguiente paso

Si quieres ver una integración más realista con HTTP, health checks, pool y relaciones entre tablas, revisa [examples/todo-app/README.md](../examples/todo-app/README.md).

Para revisar el inventario de la API publica disponible en la crate raiz, consulta [docs/api.md](api.md).

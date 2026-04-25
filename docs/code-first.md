# Code-First

Guia del enfoque `code-first` actual de `mssql-orm`, alineada con la API publica real del workspace.

Esta guia no describe features futuras como si ya existieran. Describe lo que hoy ya puedes modelar y ejecutar desde la crate raiz `mssql-orm`.

## Que significa `code-first` aqui

En este proyecto, `code-first` significa:

- la forma del modelo vive primero en structs Rust
- `#[derive(Entity)]` genera metadata estatica y `FromRow`
- `#[derive(Insertable)]` y `#[derive(Changeset)]` describen payloads de escritura
- `#[derive(DbContext)]` conecta esas entidades con `DbSet<T>`
- el query builder construye AST, no SQL directo
- la generacion SQL queda en `mssql-orm-sqlserver`
- la ejecucion queda en `mssql-orm-tiberius`

El punto de entrada para consumidores sigue siendo la crate publica `mssql-orm`.

## 1. Modelar entidades

La entidad define tabla, schema, primary key y metadata de columnas.

```rust
use mssql_orm::prelude::*;

#[derive(Entity, Debug, Clone)]
#[orm(table = "users", schema = "todo")]
struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(length = 180)]
    #[orm(unique)]
    email: String,

    #[orm(nullable)]
    display_name: Option<String>,

    #[orm(rowversion)]
    version: Vec<u8>,
}
```

Hoy la surface publica soporta al menos estos atributos relevantes del modelo:

- `table`
- `schema`
- `column`
- `primary_key`
- `identity`
- `length`
- `nullable`
- `sql_type`
- `precision` y `scale`
- `default_sql`
- `computed_sql`
- `rowversion`
- `index` y `unique`
- `foreign_key`
- `on_delete`
- `renamed_from`

`#[derive(Entity)]` tambien genera simbolos de columna como `User::email`, que despues se reutilizan desde el query builder publico.

## 2. Modelar payloads de escritura

La entidad no se usa directamente para todas las escrituras. La forma publica actual separa inserciones y updates parciales.

```rust
use mssql_orm::prelude::*;

#[derive(Insertable, Debug, Clone)]
#[orm(entity = User)]
struct NewUser {
    email: String,
    display_name: Option<String>,
}

#[derive(Changeset, Debug, Clone)]
#[orm(entity = User)]
struct UpdateUser {
    email: Option<String>,
    display_name: Option<Option<String>>,
}
```

Reglas practicas importantes:

- `Insertable` materializa columnas insertables para una entidad concreta.
- `Changeset` requiere `Option<T>` en el nivel externo de cada campo para distinguir "no tocar" de "actualizar".
- `Option<Option<T>>` sirve para expresar "actualizar a NULL".
- columnas `rowversion` participan como token de concurrencia, pero no entran al `SET`.

## 3. Declarar el `DbContext`

El contexto publico actual se define con `#[derive(DbContext)]` sobre una struct con campos `DbSet<T>`.

```rust
use mssql_orm::prelude::*;

#[derive(DbContext, Debug, Clone)]
struct TodoDb {
    pub users: DbSet<User>,
}
```

Ese derive genera la surface operativa minima para consumidores:

- `TodoDb::connect(...)`
- `TodoDb::connect_with_options(...)`
- `TodoDb::connect_with_config(...)`
- `TodoDb::from_connection(...)`
- `TodoDb::from_shared_connection(...)`
- `db.health_check().await`
- `db.transaction(|tx| async move { ... }).await`

Cada `DbSet<T>` mantiene la entrada tipada hacia CRUD, query builder y tracking experimental.

## 4. Usar `DbSet<T>` como frontera tipada

`DbSet<T>` es la pieza publica que conecta el modelo con las operaciones de lectura y escritura.

```rust
use mssql_orm::prelude::*;

async fn load_and_update(db: &TodoDb) -> Result<(), OrmError> {
    let inserted = db
        .users
        .insert(NewUser {
            email: "ana@example.com".to_string(),
            display_name: Some("Ana".to_string()),
        })
        .await?;

    let _found = db.users.find(inserted.id).await?;

    let _slice = db
        .users
        .query()
        .filter(User::email.contains("@example.com"))
        .order_by(User::email.asc())
        .take(20)
        .all()
        .await?;

    let _updated = db
        .users
        .update(
            inserted.id,
            UpdateUser {
                email: None,
                display_name: Some(Some("Ana Maria".to_string())),
            },
        )
        .await?;

    let _deleted = db.users.delete(inserted.id).await?;

    Ok(())
}
```

La surface publica actual ya incluye:

- `find`
- `insert`
- `update`
- `delete`
- `query`
- `find_tracked`
- `add_tracked`
- `remove_tracked`
- `save_changes` desde `DbContext`

## 5. Modelar relaciones en metadata

Las relaciones uno-a-muchos actuales se declaran sobre la entidad dependiente con `foreign_key`.

```rust
use mssql_orm::prelude::*;

#[derive(Entity, Debug, Clone)]
#[orm(table = "todo_lists", schema = "todo")]
struct TodoList {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(foreign_key(entity = User, column = id))]
    #[orm(on_delete = "cascade")]
    owner_user_id: i64,

    #[orm(length = 160)]
    title: String,
}
```

Hoy esa metadata sirve para:

- derivar `ForeignKeyMetadata`
- usar joins explicitos en el query builder
- alimentar snapshots, diff y DDL de migraciones

No existe todavia una API publica de navigation properties ni eager loading estilo ORM clasico.

## 6. Flujo real de trabajo `code-first`

El flujo recomendado hoy queda asi:

1. Declarar entidades con `#[derive(Entity)]`.
2. Declarar `Insertable` y `Changeset` para cada forma de escritura publica que necesite tu app.
3. Declarar un `DbContext` con `DbSet<T>` para cada tabla expuesta.
4. Conectar con `DbContext::connect(...)` o con opciones/config explicitas.
5. Usar `DbSet<T>` y `DbSetQuery<T>` para CRUD y consultas.
6. Cuando el modelo cambie, usar la pipeline de migraciones `code-first` de la CLI y del crate `migrate`.

El quickstart cubre el caso mas corto. Esta guia fija el modelo conceptual y las piezas publicas que componen ese flujo.

## Limites explicitos actuales

Para esta etapa del release, conviene asumir estos limites:

- SQL Server es el unico backend objetivo.
- La API `code-first` publica vive en atributos y derives; no existe todavia una capa de fluent configuration estilo `EntityConfiguration`.
- La sintaxis publica de relaciones sigue centrada en foreign keys declaradas por atributo.
- `DbSet::find`, `update` y `delete` estan pensados para primary key simple.
- `entity.save(&db)` y `entity.delete(&db)` existen, pero esta guia se centra en la ruta explicita `DbSet` porque es la surface base mas estable.
- El change tracking sigue siendo experimental aunque ya este disponible.
- No existe soporte multibase de datos en esta fase.

## Referencias relacionadas

- API publica: [docs/api.md](api.md)
- Quickstart reproducible: [docs/quickstart.md](quickstart.md)
- Query builder publico: [docs/query-builder.md](query-builder.md)
- Relaciones y joins: [docs/relationships.md](relationships.md)
- Transacciones runtime: [docs/transactions.md](transactions.md)
- Ejemplo real con relaciones y HTTP: [examples/todo-app/README.md](../examples/todo-app/README.md)
- Plan maestro: [docs/plan_orm_sqlserver_tiberius_code_first.md](plan_orm_sqlserver_tiberius_code_first.md)

# Relaciones y Joins

Guia de uso de relaciones code-first y joins explicitos en la API publica actual.

En `mssql-orm`, una relacion declarada con `foreign_key` produce metadata relacional, snapshots, diff y DDL de SQL Server. Las consultas siguen siendo explicitas: declarar una foreign key no agrega navigation properties ni hace joins automaticos.

## Modelo actual

La relacion uno-a-muchos se declara sobre la entidad dependiente, en el campo que contiene la columna local.

```rust
use mssql_orm::prelude::*;

#[derive(Entity, Debug, Clone)]
#[orm(table = "users", schema = "todo")]
struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(length = 180)]
    email: String,
}

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

La forma estructurada recomendada es:

```rust
#[orm(foreign_key(entity = User, column = id))]
```

El `entity` apunta al tipo Rust referenciado y `column` apunta al simbolo de columna generado por `#[derive(Entity)]` en esa entidad. Esta forma valida en compile-time que la columna exista.

## Forma legacy con strings

La sintaxis string sigue soportada por compatibilidad:

```rust
#[orm(foreign_key = "users.id")]
```

```rust
#[orm(foreign_key = "todo.users.id")]
```

Con dos segmentos, el schema referenciado por defecto es `dbo`. Con tres segmentos, el primer segmento es el schema. La forma estructurada es preferible porque usa tipos Rust y detecta columnas inexistentes antes.

## Nombre de constraint

Si no declaras nombre, el derive genera uno estable usando tabla local, columna local y tabla referenciada.

```rust
#[orm(foreign_key(entity = User, column = id))]
owner_user_id: i64,
```

Ejemplo de nombre generado:

```text
fk_todo_lists_owner_user_id_users
```

También puedes declarar el nombre de forma explicita:

```rust
#[orm(foreign_key(entity = User, column = id, name = "fk_todo_lists_owner"))]
owner_user_id: i64,
```

## Acciones referenciales

El atributo `on_delete` soporta estas acciones en la surface publica actual:

- `"no action"`
- `"cascade"`
- `"set null"`

```rust
#[derive(Entity, Debug, Clone)]
#[orm(table = "todo_items", schema = "todo")]
struct TodoItem {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(foreign_key(entity = TodoList, column = id))]
    #[orm(on_delete = "cascade")]
    list_id: i64,

    #[orm(nullable)]
    #[orm(foreign_key(entity = User, column = id))]
    #[orm(on_delete = "set null")]
    completed_by_user_id: Option<i64>,
}
```

`on_delete = "set null"` requiere que la columna local sea nullable. El derive rechaza en compile-time el caso no nullable.

En esta etapa, `on_update` se mantiene como `NoAction` desde el derive publico.

## Metadata derivada

`#[derive(Entity)]` convierte cada `foreign_key` en `ForeignKeyMetadata`.

```rust
let metadata = TodoList::metadata();
let owner_fk = metadata
    .foreign_key("fk_todo_lists_owner_user_id_users")
    .expect("owner relationship");

assert_eq!(owner_fk.columns, &["owner_user_id"]);
assert_eq!(owner_fk.referenced_schema, "todo");
assert_eq!(owner_fk.referenced_table, "users");
assert_eq!(owner_fk.referenced_columns, &["id"]);
```

Helpers disponibles sobre `EntityMetadata`:

- `foreign_key(name)`
- `foreign_keys_for_column(column_name)`
- `foreign_keys_referencing(schema, table)`

Estos helpers son para inspeccion, migraciones y tooling. No ejecutan consultas.

## Migraciones

Las foreign keys derivadas entran al pipeline code-first como metadata normal:

1. `EntityMetadata`
2. `ModelSnapshot`
3. diff relacional
4. `MigrationOperation::AddForeignKey` o `DropForeignKey`
5. DDL SQL Server en `mssql-orm-sqlserver`

El DDL generado usa `ALTER TABLE ... ADD CONSTRAINT ... FOREIGN KEY ... REFERENCES ...` y conserva `ON DELETE` cuando aplica.

La sintaxis publica del derive declara foreign keys desde campos individuales. Aunque snapshots, diff y DDL ya tienen shape para foreign keys compuestas, derivarlas automaticamente desde atributos publicos todavia queda fuera de esta fase.

## Joins explicitos

Para consultar usando una relacion, escribe el join explicitamente desde el query builder.

```rust
use mssql_orm::prelude::*;
use mssql_orm::query::{Expr, Predicate};

async fn list_items_for_owner(
    db: &TodoDb,
    owner_user_id: i64,
    list_id: i64,
) -> Result<Vec<TodoItem>, OrmError> {
    db.todo_items
        .query()
        .inner_join::<TodoList>(Predicate::eq(
            Expr::from(TodoItem::list_id),
            Expr::from(TodoList::id),
        ))
        .filter(
            TodoList::owner_user_id
                .eq(owner_user_id)
                .and(TodoItem::list_id.eq(list_id)),
        )
        .order_by(TodoItem::id.asc())
        .all()
        .await
}
```

La foreign key declara la relacion del modelo; el join decide como usarla en una consulta concreta.

## Left joins

Usa `left_join::<T>(...)` cuando la relacion puede no tener fila referenciada o cuando quieres preservar filas de la entidad base.

```rust
let items = db
    .todo_items
    .query()
    .left_join::<User>(Predicate::eq(
        Expr::from(TodoItem::completed_by_user_id),
        Expr::from(User::id),
    ))
    .filter(TodoItem::list_id.eq(list_id))
    .order_by(TodoItem::id.asc())
    .all()
    .await?;
```

La materializacion publica actual de `DbSetQuery<T>` sigue devolviendo entidades de la tabla base (`T`). El join sirve para filtrar u ordenar usando tablas relacionadas, no para construir automaticamente un grafo de objetos.

## Limites actuales

- No hay navigation properties.
- No hay lazy loading ni eager loading automatico.
- No hay inferencia automatica de joins desde `ForeignKeyMetadata`.
- No hay aliases de tabla en el AST; SQL Server rechaza self-joins o repetir la misma tabla en una consulta hasta que exista soporte de aliases.
- `DbSetQuery<T>` materializa entidades completas de `T`; no hay proyecciones parciales publicas ni DTOs derivados por join.
- La sintaxis publica del derive no genera foreign keys compuestas automaticamente, aunque el pipeline interno de snapshots/diff/DDL ya tenga soporte para representarlas.
- `count()` sobre `DbSetQuery<T>` conserva filtros de la entidad base, pero no traslada joins al `CountQuery` interno en esta etapa.

## Ejemplo real

El ejemplo `todo-app` usa exactamente este modelo:

- `TodoList.owner_user_id -> User.id`
- `TodoItem.list_id -> TodoList.id`
- `TodoItem.created_by_user_id -> User.id`
- `TodoItem.completed_by_user_id -> User.id`

Las consultas reutilizables estan en [examples/todo-app/src/queries.rs](../examples/todo-app/src/queries.rs), y el dominio esta en [examples/todo-app/src/domain.rs](../examples/todo-app/src/domain.rs).

## Referencias relacionadas

- Query builder publico: [docs/query-builder.md](query-builder.md)
- Guia code-first: [docs/code-first.md](code-first.md)
- Migraciones: [docs/migrations.md](migrations.md)
- Plan maestro: [docs/plan_orm_sqlserver_tiberius_code_first.md](plan_orm_sqlserver_tiberius_code_first.md)

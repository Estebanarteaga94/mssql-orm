# Transacciones

Guia de uso de la API publica actual de transacciones en `mssql-orm`.

La surface principal es `db.transaction(...)`, generada por `#[derive(DbContext)]` y disponible sobre cualquier contexto que implemente `DbContext`.

## Uso basico

```rust
use mssql_orm::prelude::*;

async fn create_user_and_profile(db: &AppDb, new_user: NewUser) -> Result<User, OrmError> {
    db.transaction(|tx| async move {
        let user = tx.users.insert(new_user).await?;

        tx.profiles
            .insert(NewProfile {
                user_id: user.id,
                display_name: "New user".to_string(),
            })
            .await?;

        Ok(user)
    })
    .await
}
```

El closure recibe un nuevo contexto del mismo tipo que el contexto original. Usa ese `tx` para todas las operaciones que deban participar en la transaccion.

## Contrato actual

La implementacion publica actual hace:

1. `BEGIN TRANSACTION`
2. ejecuta el closure con un contexto transaccional
3. `COMMIT TRANSACTION` si el closure devuelve `Ok`
4. `ROLLBACK TRANSACTION` si el closure devuelve `Err`

El valor de `Ok(T)` se devuelve al caller despues del commit.

```rust
let inserted = db
    .transaction(|tx| async move {
        tx.users
            .insert(NewUser {
                name: "Committed".to_string(),
                active: true,
            })
            .await
    })
    .await?;
```

Para forzar rollback, devuelve `Err(OrmError)`.

```rust
let result = db
    .transaction(|tx| async move {
        tx.users
            .insert(NewUser {
                name: "Rolled Back".to_string(),
                active: false,
            })
            .await?;

        Err::<(), OrmError>(OrmError::new("cancel transaction"))
    })
    .await;

assert!(result.is_err());
```

## Operaciones soportadas dentro del closure

Dentro del closure puedes usar la surface publica normal del contexto transaccional:

- `DbSet::find`
- `DbSet::insert`
- `DbSet::update`
- `DbSet::delete`
- `DbSet::query().all()`
- `DbSet::query().first()`
- `DbSet::query().count()`
- Active Record, siempre que lo invoques contra el contexto `tx`
- `save_changes()`, con los limites experimentales ya documentados del change tracking

Ejemplo con query builder:

```rust
let active_count = db
    .transaction(|tx| async move {
        let user = tx.users.insert(new_user).await?;

        let count = tx
            .users
            .query()
            .filter(User::active.eq(true))
            .count()
            .await?;

        Ok((user, count))
    })
    .await?;
```

## Usa el contexto `tx`

No mezcles operaciones del contexto original con operaciones del contexto `tx` dentro del closure.

```rust
db.transaction(|tx| async move {
    tx.users.insert(new_user).await?;
    tx.orders.insert(new_order).await?;
    Ok(())
})
.await?;
```

La regla practica es simple: todo lo que deba confirmar o revertir junto debe ejecutarse usando `tx`.

## Errores y rollback

Si una operacion dentro del closure falla y propagas el error con `?`, la transaccion hace rollback.

```rust
db.transaction(|tx| async move {
    let user = tx.users.insert(new_user).await?;
    tx.orders.insert(new_order_for(user.id)).await?;
    Ok(())
})
.await?;
```

Si `insert(new_order_for(...))` devuelve `Err`, el closure termina en `Err` y `db.transaction(...)` intenta `ROLLBACK TRANSACTION`.

La API no depende de rollback en `Drop`. Esto es intencional: en Rust async no se puede ejecutar `await` dentro de `Drop`, por lo que el cierre transaccional debe ser explicito.

## Timeouts, tracing y retry

Los comandos transaccionales (`BEGIN`, `COMMIT`, `ROLLBACK`) usan el `query_timeout` configurado en `MssqlOperationalOptions`.

La instrumentacion de Tiberius emite eventos estructurados para:

- `orm.transaction.begin`
- `orm.transaction.commit`
- `orm.transaction.rollback`
- `orm.transaction.error`

Las lecturas ejecutadas dentro de `MssqlTransaction` no aplican retry automatico. Esto evita repetir operaciones dentro de una transaccion activa sin un contrato de idempotencia mas fuerte.

## Limites operativos actuales

- No hay savepoints publicos (`SAVE TRANSACTION`) en esta etapa.
- No hay transacciones anidadas modeladas por la API publica.
- No hay rollback automatico ante panic; la ruta soportada es devolver `Err(OrmError)`.
- Si `COMMIT TRANSACTION` falla, el error se propaga. La API actual no ejecuta un rollback compensatorio despues de un fallo de commit.
- La transaccion es de alcance SQL Server sobre una conexion. No es una transaccion distribuida ni coordina recursos externos.
- Durante el closure, trata la conexion compartida como de uso logico exclusivo. No ejecutes operaciones concurrentes sobre clones del mismo contexto esperando aislamiento adicional.
- Con `pool-bb8`, `db.transaction(...)` aun no pinnea una conexion fisica del pool durante todo el closure. Hasta corregir ese punto, usa transacciones publicas sobre contextos creados desde conexion directa (`connect`, `connect_with_config`, `from_connection`) y no sobre `from_pool(...)`.

## Relacion con migraciones

`db.transaction(...)` es para operaciones runtime de la aplicacion.

Las migraciones tienen su propio mecanismo: `database update` genera scripts idempotentes con `BEGIN TRY`, `BEGIN TRANSACTION`, `COMMIT` y `ROLLBACK` por migracion. No mezcles esa garantia con la API runtime del `DbContext`.

## Validacion existente

La API publica ya tiene cobertura real contra SQL Server en la crate principal:

- commit: una insercion dentro de `db.transaction(...)` queda persistida cuando el closure retorna `Ok`
- rollback: una insercion dentro de `db.transaction(...)` desaparece cuando el closure retorna `Err`

Esas pruebas se ejecutan cuando `MSSQL_ORM_TEST_CONNECTION_STRING` apunta a una instancia valida de SQL Server.

## Referencias relacionadas

- API publica: [docs/api.md](api.md)
- Guia code-first: [docs/code-first.md](code-first.md)
- Query builder publico: [docs/query-builder.md](query-builder.md)
- Guia de migraciones: [docs/migrations.md](migrations.md)
- Plan maestro: [docs/plan_orm_sqlserver_tiberius_code_first.md](plan_orm_sqlserver_tiberius_code_first.md)

# Worklog

## 2026-04-23

### Sesión: `limit` y `take` en `DbSetQuery`

- Se movió en `docs/tasks.md` la subtarea `Etapa 6: Exponer limit y take en DbSetQuery` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm/src/dbset_query.rs` para exponer `DbSetQuery::limit(u64)` y `DbSetQuery::take(u64)`.
- La implementación reutiliza `Pagination::new(0, limit)`, dejando `take` como alias directo de `limit` y evitando crear una semántica paralela para recorte de resultados.
- Se añadieron pruebas unitarias para validar que `limit` genera paginación con `offset = 0` y que `take` produce exactamente el mismo `SelectQuery` interno.
- No fue necesario modificar `mssql-orm-query` ni `mssql-orm-sqlserver`, porque esta subtarea solo hizo accesible desde la API pública una capacidad ya soportada por `Pagination` y por la compilación SQL existente.

### Resultado

- `DbSetQuery` ya soporta recorte básico de resultados con `limit` y `take`, alineado con la API objetivo del plan maestro y sin duplicar contratos internos.

### Validación

- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Próximo paso recomendado

- Implementar `Etapa 6: Exponer paginación pública en DbSetQuery con request explícito y contrato estable`.

### Sesión: Métodos fluentes `filter` y `order_by` en `DbSetQuery`

- Se movió en `docs/tasks.md` la subtarea `Etapa 6: Exponer métodos fluentes en DbSetQuery para filter y order_by` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm/src/dbset_query.rs` para exponer `DbSetQuery::filter(Predicate)` y `DbSetQuery::order_by(OrderBy)`.
- Ambos métodos reutilizan directamente `SelectQuery::filter` y `SelectQuery::order_by`, manteniendo una única representación del AST y evitando introducir un builder paralelo en la crate pública.
- Se añadieron pruebas unitarias para validar `filter`, `order_by` y el encadenamiento de ambos sobre el `SelectQuery` interno.
- No fue necesario modificar el compilador SQL Server ni el AST base, porque la semántica ya existía y esta subtarea solo la hizo accesible desde la API pública del runner.

### Resultado

- `DbSetQuery` ya soporta la composición fluida básica del query builder público sobre filtros y ordenamiento, alineada con la API objetivo del plan maestro.

### Validación

- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Próximo paso recomendado

- Implementar `Etapa 6: Exponer limit y take en DbSetQuery`, reutilizando `Pagination` sin duplicar semántica.

### Sesión: Ordenamiento público por columna

- Se movió en `docs/tasks.md` la subtarea `Etapa 6: Exponer ordenamiento público por columna (asc, desc)` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió `crates/mssql-orm/src/query_order.rs` como capa pública de extensiones de ordenamiento sobre `EntityColumn<E>`.
- La implementación expone el trait `EntityColumnOrderExt` con `asc()` y `desc()`, delegando internamente a `OrderBy::asc` y `OrderBy::desc` del AST existente.
- Se reexportó `EntityColumnOrderExt` desde `mssql-orm` y desde la `prelude` pública, alineando la API con el shape definido en el plan maestro (`Customer::id.asc()`, `Customer::created_at.desc()`).
- Se añadieron pruebas unitarias específicas para fijar la forma exacta de `OrderBy` generado y se amplió la prueba de superficie pública en `crates/mssql-orm/src/lib.rs`.
- No fue necesario modificar `mssql-orm-query` ni `mssql-orm-sqlserver`, porque la representación y compilación de ordenamiento ya existían; esta subtarea solo expone la API pública encima de esa base.

### Resultado

- La tercera subtarea de Etapa 6 quedó completada y validada; la crate pública ya soporta ordenamiento por columna alineado con el AST y con la API objetivo del plan.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Próximo paso recomendado

- Implementar `Etapa 6: Exponer métodos fluentes en DbSetQuery para filter y order_by`, reutilizando `SelectQuery` y las nuevas extensiones públicas ya disponibles.

### Sesión: Predicados string públicos sobre `EntityColumn`

- Se movió en `docs/tasks.md` la subtarea `Etapa 6: Exponer predicados string públicos sobre EntityColumn` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm/src/query_predicates.rs` para exponer `contains`, `starts_with` y `ends_with` como parte del trait público `EntityColumnPredicateExt`.
- La implementación reutiliza `Predicate::like` del AST existente y construye patrones parametrizados (`%valor%`, `valor%`, `%valor`) dentro de la crate pública, sin introducir operadores nuevos ni mover lógica al core.
- Se añadió cobertura unitaria específica para fijar la forma exacta de los predicados `LIKE` generados y se amplió la prueba de superficie pública en `crates/mssql-orm/src/lib.rs`.
- No fue necesario modificar `mssql-orm-query` ni `mssql-orm-sqlserver`, porque la compilación de `LIKE` ya existía y esta subtarea solo expone una API pública encima del AST.

### Resultado

- La segunda subtarea de Etapa 6 quedó completada y validada; la crate pública ya expone la base de filtros string sobre columnas para el query builder fluido.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Próximo paso recomendado

- Implementar `Etapa 6: Exponer ordenamiento público por columna (asc, desc)`, reutilizando `OrderBy` sin crear una representación paralela.

### Sesión: Predicados de comparación públicos sobre `EntityColumn`

- Se movió en `docs/tasks.md` la subtarea `Etapa 6: Exponer predicados de comparación públicos sobre EntityColumn` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió `crates/mssql-orm/src/query_predicates.rs` como capa pública de extensiones sobre `EntityColumn<E>`.
- La implementación se resolvió en la crate pública `mssql-orm` mediante el trait `EntityColumnPredicateExt`, evitando introducir una dependencia desde `mssql-orm-core` hacia `mssql-orm-query`.
- La nueva API pública expone `eq`, `ne`, `gt`, `gte`, `lt`, `lte`, `is_null` e `is_not_null`, devolviendo `Predicate` del AST existente.
- La `prelude` pública y los reexports de `mssql-orm` ahora incluyen `EntityColumnPredicateExt`, habilitando llamadas estilo `Customer::active.eq(true)` desde código consumidor.
- Se añadieron pruebas unitarias específicas para fijar la forma exacta de los `Predicate` generados y una prueba adicional en `crates/mssql-orm/src/lib.rs` para verificar que la extensión está disponible desde la superficie pública.
- Fue necesario añadir una excepción puntual de `clippy::wrong_self_convention` porque el plan maestro exige explícitamente los nombres `is_null` e `is_not_null` como API pública.

### Resultado

- La primera subtarea de Etapa 6 quedó implementada y validada, dejando lista la base pública para continuar con predicados string y ordenamiento sin romper los límites arquitectónicos del workspace.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Próximo paso recomendado

- Implementar `Etapa 6: Exponer predicados string públicos sobre EntityColumn (contains, starts_with, ends_with)`, reutilizando la misma estrategia de trait público en `mssql-orm`.

### Sesión: Desglose detallado de la Etapa 6

- Se revisó la ruta real del plan maestro y se mantuvo como fuente de verdad `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se detectó que la tarea abierta de Etapa 6 seguía siendo demasiado amplia para ejecutarla sin mezclar varias responsabilidades públicas en una sola sesión.
- Se reestructuró `docs/tasks.md` para dividir Etapa 6 en subtareas cerrables y secuenciales: predicados de comparación, predicados string, ordenamiento, `filter`/`order_by` en `DbSetQuery`, `limit`/`take`, paginación explícita, composición lógica de predicados, pruebas unitarias de API y snapshots de seguridad de parámetros.
- Se retiró la tarea amplia de `En Progreso` y se dejó la sección sin trabajo activo, evitando que el backlog quede con una tarea ambigua o parcialmente definida.
- Se actualizó `docs/context.md` para que el foco operativo ya no sea “Etapa 6” en general, sino la primera subtarea concreta a ejecutar en la siguiente sesión.

### Resultado

- El backlog quedó más granular, ordenado y listo para atacar Etapa 6 sin dejar subtareas implícitas ni mezclas de alcance.

### Validación

- No se ejecutaron validaciones de Cargo porque esta sesión solo reestructuró documentación operativa y no modificó código fuente.
- Se verificó manualmente la consistencia del backlog revisando `docs/tasks.md` tras el desglose.

### Próximo paso recomendado

- Mover a `En Progreso` la subtarea `Etapa 6: Exponer predicados de comparación públicos sobre EntityColumn` e implementarla primero.

### Sesión: Registrar connection string operativa de test

- Se registró en `docs/context.md` la connection string local actualmente usada para validaciones reales e integraciones sobre SQL Server.
- La referencia quedó indicada para `MSSQL_ORM_TEST_CONNECTION_STRING` y `DATABASE_URL`, de modo que futuras sesiones autónomas reutilicen la misma configuración cuando el entorno local no haya cambiado.
- Se dejó nota explícita de que esta cadena es específica del entorno actual y debe actualizarse si cambian host, base o credenciales.

### Resultado

- La documentación operativa ahora contiene la configuración local concreta que se viene usando en validaciones reales, evitando ambigüedad entre sesiones.

### Sesión: Ejemplo funcional `basic-crud`

- Se movió en `docs/tasks.md` la subtarea `Etapa 5: Crear ejemplo funcional basic-crud` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se creó `examples/basic-crud/` como crate ejecutable mínima y autocontenida.
- Se añadieron `examples/basic-crud/src/main.rs`, `examples/basic-crud/Cargo.toml` y `examples/basic-crud/README.md`.
- El ejemplo reutiliza exactamente la superficie pública ya validada: `Entity`, `Insertable`, `Changeset`, `DbContext`, `DbSet::insert`, `DbSet::find`, `DbSet::query`, `DbSet::update` y `DbSet::delete`.
- El ejemplo prepara y limpia `dbo.basic_crud_users` con `MssqlConnection` solo como soporte de setup/cleanup, manteniendo el flujo CRUD en la crate pública.
- Fue necesario añadir un `[workspace]` vacío al `Cargo.toml` del ejemplo para aislarlo del workspace raíz sin incorporarlo a `workspace.members`.
- Se validó el ejemplo con `cargo check --manifest-path examples/basic-crud/Cargo.toml`, `cargo run --manifest-path examples/basic-crud/Cargo.toml` usando `DATABASE_URL` contra `tempdb`, y `cargo clippy --manifest-path examples/basic-crud/Cargo.toml -- -D warnings`.
- También se mantuvo validado el workspace principal con `cargo test --workspace` durante la misma sesión.

### Resultado

- La Etapa 5 quedó cerrada con un ejemplo ejecutable real que refleja la API pública actual y el flujo CRUD básico sobre SQL Server.

### Bloqueos

- No hubo bloqueos permanentes. Solo fue necesario aislar el ejemplo del workspace raíz para que Cargo aceptara `--manifest-path` sin añadirlo a `workspace.members`.

### Próximo paso recomendado

- Empezar `Etapa 6: Implementar query builder público con filtros, composición lógica, ordenamiento, limit y paginación`, reutilizando `DbSetQuery<T>` como base y evitando duplicar el AST ya existente.

### Sesión: Modo `KEEP_TEST_ROWS` para CRUD público

- Se ajustó `crates/mssql-orm/tests/stage5_public_crud.rs` para aceptar la variable de entorno `KEEP_TEST_ROWS=1`.
- Cuando esa variable está activa, la prueba pública conserva la tabla y también deja una fila final persistida tras el flujo CRUD para inspección manual.
- Con `KEEP_TEST_ROWS=1`, la prueba omite el borrado final del registro y evita el cleanup de la tabla, escribiendo en la salida que dejó la fila en `dbo.mssql_orm_public_crud`.
- Se validó el ajuste con `cargo fmt --all --check`, `cargo test -p mssql-orm --test stage5_public_crud` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- Ahora existe un flujo opt-in para inspeccionar manualmente no solo la tabla sino también una fila real generada por la API pública de CRUD.

### Próximo paso recomendado

- Ejecutar `KEEP_TEST_ROWS=1` junto con `MSSQL_ORM_TEST_CONNECTION_STRING=... cargo test -p mssql-orm --test stage5_public_crud -- --nocapture` cuando se quiera inspección manual con datos persistidos, y borrar luego la tabla explícitamente.

### Sesión: Modo `KEEP_TEST_TABLES` para CRUD público

- Se ajustó `crates/mssql-orm/tests/stage5_public_crud.rs` para aceptar la variable de entorno `KEEP_TEST_TABLES=1`.
- Cuando esa variable está activa, la prueba pública conserva la tabla `dbo.mssql_orm_public_crud` y escribe en la salida el nombre exacto de la tabla preservada.
- El comportamiento por defecto no cambió: si `KEEP_TEST_TABLES` no está activa, la prueba sigue eliminando la tabla al finalizar.
- Se validó el ajuste con `cargo fmt --all --check`, `cargo test -p mssql-orm --test stage5_public_crud` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- Ahora existe un flujo opt-in para inspeccionar manualmente en SQL Server la tabla usada por la integración pública de CRUD sin editar el archivo de tests.

### Próximo paso recomendado

- Ejecutar `KEEP_TEST_TABLES=1` junto con `MSSQL_ORM_TEST_CONNECTION_STRING=... cargo test -p mssql-orm --test stage5_public_crud -- --nocapture` cuando se quiera inspección manual, y borrar luego la tabla explícitamente.

### Sesión: Pruebas de integración públicas para CRUD base

- Se movió en `docs/tasks.md` la subtarea `Etapa 5: Agregar pruebas de integración de la API CRUD base en la crate pública` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió `crates/mssql-orm/tests/stage5_public_crud.rs` como prueba de integración real sobre la superficie pública de `mssql-orm`.
- La prueba nueva define una entidad pública con `#[derive(Entity)]`, modelos `Insertable`/`Changeset`, un `DbContext` derivado y un `FromRow` manual para recorrer la API tal como la usará un consumidor real.
- El flujo validado cubre `insert`, `find`, `query().all`, `query().count`, `query_with(...).first`, `update` y `delete` usando `DbSet<T>`.
- El setup y cleanup de la tabla de prueba se hace con `MssqlConnection` solo como soporte de infraestructura de test; la lógica CRUD validada ocurre a través de la crate pública.
- La tabla de prueba se crea en `dbo.mssql_orm_public_crud` dentro de la base activa del connection string, porque la metadata actual no soporta prefijar base de datos distinta en esta etapa.
- La prueba sigue usando `MSSQL_ORM_TEST_CONNECTION_STRING` y hace skip limpio cuando la variable no está presente.
- La ruta operativa del plan maestro siguió siendo `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se validó el workspace con `cargo check --workspace`, `cargo test --workspace`, `cargo fmt --all --check` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 5 ya no solo tiene la base CRUD implementada, sino también validación real de la superficie pública `mssql-orm` contra SQL Server.

### Bloqueos

- No hubo bloqueos permanentes. Solo apareció un warning local por un import no usado en el test nuevo y se corrigió antes de cerrar `clippy`.

### Próximo paso recomendado

- Implementar `Etapa 5: Crear ejemplo funcional basic-crud`, reutilizando exactamente la superficie pública y el patrón de setup ya validados por la prueba de integración.

### Sesión: `DbSet::delete` por primary key simple

- Se movió en `docs/tasks.md` la subtarea `Etapa 5: Implementar DbSet::delete por primary key simple` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm/src/context.rs` para exponer `DbSet::delete<K>() -> Result<bool, OrmError>`.
- `delete` reutiliza `DeleteQuery`, `SqlServerCompiler::compile_delete` y `MssqlConnection::execute`, devolviendo `true` cuando SQL Server reporta al menos una fila afectada.
- Se añadió el helper interno `delete_query` para mantener la forma del `DeleteQuery` testeable sin depender de una conexión real.
- En esta etapa, `delete` sigue soportando solo primary key simple; para PK compuesta retorna un `OrmError` explícito.
- Se eligió `Result<bool, OrmError>` como retorno para distinguir entre eliminación efectiva y ausencia de fila, sin adelantar todavía `OrmError::ConcurrencyConflict` de la Etapa 11.
- Se añadieron pruebas unitarias para verificar la forma exacta del `DeleteQuery` generado y para rechazar PK compuesta.
- La ruta operativa del plan maestro siguió siendo `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se validó el workspace con `cargo check --workspace`, `cargo test --workspace`, `cargo fmt --all --check` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La base CRUD de `DbSet<T>` para Etapa 5 quedó completa a nivel de operaciones fundamentales: `query`, `find`, `insert`, `update` y `delete`.

### Bloqueos

- No hubo bloqueos permanentes.

### Próximo paso recomendado

- Implementar `Etapa 5: Agregar pruebas de integración de la API CRUD base en la crate pública`, cubriendo el recorrido real de `find`, `insert`, `update`, `delete` y `query` sobre SQL Server.

### Sesión: `DbSet::update` por primary key simple

- Se movió en `docs/tasks.md` la subtarea `Etapa 5: Implementar DbSet::update por primary key simple sobre Changeset` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm/src/context.rs` para exponer `DbSet::update<K, C>() -> Result<Option<E>, OrmError>`.
- `update` reutiliza `UpdateQuery`, `SqlServerCompiler::compile_update` y `MssqlConnection::fetch_one`, apoyándose en `OUTPUT INSERTED.*` ya emitido por la compilación SQL Server.
- Se factoró un helper interno `primary_key_predicate` para compartir la construcción del filtro por PK simple entre `find` y `update`.
- Se añadió el helper interno `update_query(&C)` para mantener la forma del `UpdateQuery` testeable sin depender de una conexión real.
- En esta etapa, `update` sigue soportando solo primary key simple; para PK compuesta retorna un `OrmError` explícito.
- Se eligió `Result<Option<E>, OrmError>` como retorno para conservar la posibilidad de “fila no encontrada” sin inventar aún semántica de conflicto de concurrencia previa a la Etapa 11.
- Se añadieron pruebas unitarias para verificar la forma exacta del `UpdateQuery` generado y para rechazar PK compuesta.
- La ruta operativa del plan maestro siguió siendo `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se validó el workspace con `cargo check --workspace`, `cargo test --workspace`, `cargo fmt --all --check` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- `DbSet<T>` ya expone actualización base por primary key simple y deja lista la última operación CRUD fundamental de Etapa 5: `delete`.

### Bloqueos

- No hubo bloqueos permanentes. Solo apareció un ajuste menor de imports en el módulo de tests durante la validación.

### Próximo paso recomendado

- Implementar `Etapa 5: Implementar DbSet::delete por primary key simple`, reutilizando metadata de PK simple, `DeleteQuery`, `SqlServerCompiler::compile_delete` y `ExecuteResult`.

### Sesión: `DbSet::insert` con retorno materializado

- Se movió en `docs/tasks.md` la subtarea `Etapa 5: Implementar DbSet::insert sobre modelos Insertable con retorno materializado` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm/src/context.rs` para exponer `DbSet::insert<I>() -> Result<E, OrmError>`.
- `insert` reutiliza `InsertQuery`, `SqlServerCompiler::compile_insert` y `MssqlConnection::fetch_one`, apoyándose en `OUTPUT INSERTED.*` ya emitido por la crate SQL Server.
- Se añadió el helper interno `insert_query(&I) -> InsertQuery` para mantener la construcción del query testeable sin depender de una conexión real.
- Si la inserción no devuelve una fila materializable, la API pública ahora falla explícitamente con `OrmError("insert query did not return a row")`.
- Se añadieron pruebas unitarias para verificar la forma exacta del `InsertQuery` generado desde un modelo `Insertable`.
- La ruta operativa del plan maestro siguió siendo `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se validó el workspace con `cargo check --workspace`, `cargo test --workspace`, `cargo fmt --all --check` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- `DbSet<T>` ya expone inserción base con retorno materializado de la entidad, cerrando otra pieza fundamental de la Etapa 5 sin mover compilación SQL ni ejecución fuera de sus crates correspondientes.

### Bloqueos

- No hubo bloqueos permanentes.

### Próximo paso recomendado

- Implementar `Etapa 5: Implementar DbSet::update por primary key simple sobre Changeset`, reutilizando metadata de PK simple, `UpdateQuery`, `SqlServerCompiler::compile_update` y `fetch_one`.

### Sesión: `DbSet::find` por primary key simple

- Se movió en `docs/tasks.md` la subtarea `Etapa 5: Implementar DbSet::find por primary key simple` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm/src/context.rs` para exponer `DbSet::find<K>() -> Result<Option<E>, OrmError>`.
- `find` reutiliza `DbSet::query_with(...)` y genera internamente un `SelectQuery` filtrado por la metadata de primary key de la entidad.
- En esta etapa, `find` soporta solo primary key simple; si la entidad tiene PK compuesta, retorna un `OrmError` explícito.
- La construcción del predicado usa `TableRef`, `ColumnRef`, `Expr` y `Predicate` del AST existente, sin mover generación SQL a la crate pública.
- Se añadieron pruebas unitarias para verificar la forma exacta del `SelectQuery` generado por `find` y para rechazar PK compuesta.
- La ruta operativa del plan maestro siguió siendo `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se validó el workspace con `cargo check --workspace`, `cargo test --workspace`, `cargo fmt --all --check` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- `DbSet<T>` ya expone `find` sobre primary key simple y queda alineado con la progresión prevista de la Etapa 5, apoyándose en el runner base introducido en la sesión anterior.

### Bloqueos

- No hubo bloqueos permanentes. Solo apareció un ajuste menor de formato antes de cerrar la validación final.

### Próximo paso recomendado

- Implementar `Etapa 5: Implementar DbSet::insert sobre modelos Insertable con retorno materializado`, reutilizando `InsertQuery`, `SqlServerCompiler::compile_insert` y `fetch_one`.

### Sesión: `DbSet::query()` y query runner base

- Se movió en `docs/tasks.md` la subtarea `Etapa 5: Exponer DbSet::query() y query runner base (all, first, count) sobre SelectQuery` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió `crates/mssql-orm/src/dbset_query.rs` como nueva capa pública para ejecutar queries de entidad sobre la conexión compartida del `DbSet`.
- `DbSetQuery<E>` ahora encapsula un `SelectQuery` y expone `with_select_query`, `select_query`, `into_select_query`, `all`, `first` y `count`.
- Se actualizó `crates/mssql-orm/src/context.rs` para que `DbSet<T>` exponga `query()` y `query_with(SelectQuery)`, reutilizando la misma conexión compartida y sin mover generación SQL fuera de `mssql-orm-sqlserver`.
- Se reexportó `DbSetQuery` desde `crates/mssql-orm/src/lib.rs` y desde la `prelude` pública para dejar estable la superficie base de la Etapa 5.
- Para soportar materialización consistente del conteo, `mssql-orm-sqlserver` ahora compila `CountQuery` como `SELECT COUNT(*) AS [count] ...`.
- Se actualizaron las pruebas unitarias de la crate pública y el snapshot de `count` en `mssql-orm-sqlserver` para fijar el alias observable y cubrir `CountRow` con resultados `i32` e `i64`.
- La ruta operativa del plan maestro siguió siendo `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se validó el workspace con `cargo check --workspace`, `cargo test --workspace`, `cargo fmt --all --check` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La crate pública ya tiene la primera pieza ejecutable del CRUD de Etapa 5: queries de entidad con ejecución base para `all`, `first` y `count`, apoyadas en `SelectQuery` y sin adelantar todavía el query builder fluido de la Etapa 6.

### Bloqueos

- No hubo bloqueos permanentes. Solo aparecieron ajustes locales de compilación y tests por imports en módulos `#[cfg(test)]` y por el lifetime del lock sobre la conexión compartida.

### Próximo paso recomendado

- Implementar `Etapa 5: Implementar DbSet::find por primary key simple`, reutilizando `query_with`, metadata de primary key y el runner recién incorporado.

### Sesión: División de la tarea amplia de Etapa 5

- Se releyó el plan maestro en la ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md`; no existe una copia operativa en la raíz del repositorio.
- Se dividió la tarea amplia `Etapa 5: Exponer API CRUD base find, insert, update, delete, query` en subtareas verificables dentro de `docs/tasks.md`.
- La nueva descomposición separa `query()/all/first/count`, `find`, `insert`, `update`, `delete` y pruebas de integración de la API CRUD pública.
- No se modificó código en esta sesión; el cambio fue únicamente de planificación operativa para mejorar trazabilidad y evitar trabajo parcial ambiguo.

### Resultado

- El backlog de Etapa 5 quedó más granular y listo para ejecutar una subtarea concreta por sesión sin mezclar responsabilidades.

### Próximo paso recomendado

- Mover a `En Progreso` la subtarea `Etapa 5: Exponer DbSet::query() y query runner base (all, first, count) sobre SelectQuery` e implementarla primero, porque destraba `find` y reduce duplicación para el resto del CRUD.

### Sesión: `DbContext`, `DbSet<T>` y `#[derive(DbContext)]`

- Se movió en `docs/tasks.md` la tarea `Etapa 5: Implementar DbContext trait, DbSet<T> y #[derive(DbContext)]` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió en `crates/mssql-orm/src/context.rs` la nueva capa pública de contexto con `SharedConnection`, el trait `DbContext`, el tipo `DbSet<T>` y el helper `connect_shared`.
- `DbSet<T>` ahora encapsula una conexión compartida sobre `Arc<tokio::sync::Mutex<MssqlConnection<_>>>`, expone metadata de entidad y deja preparado el punto de apoyo para la próxima tarea de CRUD.
- Se añadió `tokio` como dependencia de la crate pública y se reexportó desde `mssql-orm` para que el derive pueda generar código estable sin exigir imports extra al proyecto consumidor.
- Se actualizó `crates/mssql-orm/src/lib.rs` para reexportar `DbContext`, `DbSet`, `SharedConnection` y `connect_shared`, y para incluir el derive `DbContext` dentro de la `prelude`.
- Se implementó en `crates/mssql-orm-macros` el derive real `#[derive(DbContext)]` para structs con campos `DbSet<Entidad>`.
- El derive genera `impl DbContext`, el método `from_shared_connection`, el helper `from_connection` y el método async `connect(&str) -> Result<Self, OrmError>`.
- El derive valida en compilación que cada campo del contexto tenga tipo `DbSet<Entidad>`; si no se cumple, produce un error explícito.
- Se añadieron casos `trybuild` nuevos en `crates/mssql-orm/tests/ui/` para un contexto válido y para un caso inválido con un campo que no es `DbSet<Entidad>`.
- También se añadieron pruebas unitarias en la crate pública para `DbSet<T>` sobre metadata y `Debug`, sin simular una conexión falsa no válida.
- `Cargo.lock` se actualizó para registrar la incorporación de `tokio` en la crate pública y el ajuste de dependencias asociado.
- Se validó el workspace con `cargo check --workspace`, `cargo fmt --all --check`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 5 ya tiene la base pública de contexto y sets de entidad alineada con el plan maestro, dejando listo el soporte para introducir la API CRUD sobre `DbSet<T>`.

### Bloqueos

- No hubo bloqueos permanentes. Solo aparecieron ajustes locales de validación: una prueba `trybuild` válida que estaba ejecutando código en runtime y varios fixtures de test que inicialmente intentaban fabricar conexiones falsas no inicializables.

### Próximo paso recomendado

- Implementar `Etapa 5: Exponer API CRUD base find, insert, update, delete, query`, reutilizando el `SharedConnection` ya introducido en `DbSet<T>`.

### Sesión: Modo `KEEP_TEST_TABLES` para inspección manual

- Se ajustó `crates/mssql-orm-tiberius/tests/sqlserver_integration.rs` para aceptar la variable de entorno `KEEP_TEST_TABLES=1`.
- Cuando esa variable está activa, las pruebas de integración conservan la tabla creada en `tempdb.dbo` y escriben en la salida el nombre exacto de la tabla para inspección manual posterior.
- El comportamiento por defecto no cambió: si `KEEP_TEST_TABLES` no está activa, la prueba sigue limpiando la tabla al finalizar.

### Resultado

- Ahora existe un flujo opt-in para inspeccionar manualmente en SQL Server los datos creados por la prueba real sin editar el archivo de tests.

### Próximo paso recomendado

- Ejecutar la prueba con `KEEP_TEST_TABLES=1` cuando se quiera inspección manual, y luego borrar la tabla explícitamente tras revisar el contenido.

### Sesión: Pruebas de integración reales contra SQL Server

- Se movió en `docs/tasks.md` la tarea `Etapa 4: Agregar pruebas de integración contra SQL Server real` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió la prueba de integración `crates/mssql-orm-tiberius/tests/sqlserver_integration.rs` para cubrir conexión real, `execute`, `fetch_one` y `fetch_all` contra SQL Server.
- Las pruebas nuevas usan `MSSQL_ORM_TEST_CONNECTION_STRING` como fuente de configuración para no hardcodear secretos en el repositorio y permitir ejecución opt-in en otros entornos.
- Se añadió un fixture `IntegrationUser` con implementación manual de `FromRow`, verificando mapping real desde `MssqlRow` hacia tipos del core.
- La prueba principal crea una tabla efímera real, inserta filas usando `CompiledQuery` y `SqlValue`, valida `rows_affected()`, lee un registro con `fetch_one` y luego materializa la colección completa con `fetch_all`.
- Se añadió una segunda prueba para confirmar que `fetch_one` retorna `None` cuando la consulta no produce filas.
- Durante la primera validación real apareció una particularidad importante de SQL Server/Tiberius: las `#temp tables` creadas en una llamada RPC no persistieron entre ejecuciones separadas, por lo que las pruebas se rediseñaron para usar tablas únicas en `tempdb.dbo`.
- La connection string proporcionada originalmente (`Database=test`) no fue usable porque la base `test` no estaba accesible para el login `sa`; se comprobó esto con `sqlcmd` y la validación real se ejecutó con la misma credencial sobre `master`.
- Se verificó conectividad TCP a `localhost:1433` y autenticación real con `sqlcmd` antes de cerrar la implementación, para separar problemas de infraestructura de problemas del adaptador.
- Se validó de forma explícita la prueba real con `MSSQL_ORM_TEST_CONNECTION_STRING='Server=localhost;Database=master;User Id=SA;Password=...;' cargo test -p mssql-orm-tiberius --test sqlserver_integration -- --nocapture`.
- También se validó el workspace con `cargo check --workspace`, `cargo fmt --all --check`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 4 quedó cerrada con cobertura de integración real sobre SQL Server, confirmando el recorrido de conexión, ejecución y materialización de filas del adaptador Tiberius.

### Bloqueos

- No hubo bloqueos permanentes. Solo aparecieron dos hallazgos operativos durante la sesión: la base `test` del connection string inicial no estaba disponible, y las `#temp tables` no servían para este patrón de ejecución RPC entre llamadas separadas.

### Próximo paso recomendado

- Empezar `Etapa 5: Implementar DbContext trait, DbSet<T> y #[derive(DbContext)]`, reutilizando la infraestructura del adaptador ya validada en real.

### Sesión: `MssqlRow`, `fetch_one`/`fetch_all` y conversión de errores

- Se confirmó otra vez que el plan maestro no está en la raíz; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se movió en `docs/tasks.md` la tarea `Etapa 4: Implementar wrapper MssqlRow y conversión de errores a OrmError` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadieron en `crates/mssql-orm-tiberius` los módulos nuevos `row` y `error` para encapsular lectura de filas y traducción de errores de Tiberius sin exponer el driver fuera del adaptador.
- Se implementó `MssqlRow<'a>` como wrapper sobre `tiberius::Row`, con implementación del trait neutral `mssql_orm_core::Row`.
- `MssqlRow` ahora convierte a `SqlValue` los tipos hoy soportados por el core: `bit`, `tinyint`, `smallint`, `int`, `bigint`, `float`, strings, binarios, `uniqueidentifier`, `decimal`, `date` y `datetime`.
- Los tipos de SQL Server todavía no soportados por el core o sin mapping estable en esta etapa, como `money`, `time`, `datetimeoffset`, `xml`, `sql_variant` y `udt`, ahora fallan de forma explícita con `OrmError`.
- Se añadió una capa interna `map_tiberius_error` para traducir errores del driver a `OrmError` con contexto de conexión, inicialización de cliente, ejecución y lectura de filas; los deadlocks se distinguen con un mensaje específico.
- Se extendió `Executor` y `MssqlConnection<S>` con `fetch_one<T: FromRow>` y `fetch_all<T: FromRow>`, reutilizando `query_raw` y mapeando cada fila mediante `MssqlRow`.
- Se actualizó el código existente de conexión y ejecución para usar la misma capa interna de conversión de errores, centralizando el comportamiento del adaptador.
- Se añadieron pruebas unitarias para el mapeo contextual de errores, la clasificación de tipos no soportados y la reexportación pública de `MssqlRow`.
- No se añadieron todavía pruebas contra SQL Server real; esa tarea sigue pendiente como siguiente paso explícito de la Etapa 4.
- Se validó el workspace con `cargo check --workspace`, `cargo fmt --all --check`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 4 ya cuenta con lectura de filas y materialización hacia el contrato `FromRow`, además de encapsulación consistente de errores del driver dentro de `OrmError`.

### Bloqueos

- No hubo bloqueos permanentes. Durante la implementación solo fue necesario ajustar dos detalles locales: mapear errores devueltos por `QueryStream::into_row`/`into_first_result`, y adaptar strings/binarios porque Tiberius los expone por referencia en lectura.

### Próximo paso recomendado

- Implementar `Etapa 4: Agregar pruebas de integración contra SQL Server real` para validar el recorrido completo del adaptador sobre una base real.

### Sesión: `Executor` sobre Tiberius con binding de parámetros

- Se movió en `docs/tasks.md` la tarea `Etapa 4: Implementar Executor sobre Tiberius con binding de parámetros` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió en `crates/mssql-orm-tiberius` la capa nueva `executor` con el trait `Executor`, el tipo `ExecuteResult` y métodos reales `execute` y `query_raw` sobre `MssqlConnection<S>`.
- Se añadió el módulo `parameter` para preparar `CompiledQuery` antes de pasarla a Tiberius, preservando orden de parámetros y validando que la cantidad de placeholders `@P1..@Pn` coincida con `params.len()`.
- El binder ahora convierte `SqlValue` a parámetros aceptados por `tiberius::Query::bind`, cubriendo `bool`, `i32`, `i64`, `f64`, `String`, `Vec<u8>`, `Uuid`, `NaiveDate`, `NaiveDateTime` y `Decimal`.
- Para `Decimal` fue necesario convertir explícitamente a `tiberius::numeric::Numeric`, porque `rust_decimal::Decimal` no implementa `IntoSql` por valor en el camino usado por `Query::bind`.
- Se habilitaron las features `chrono` y `rust_decimal` en la dependencia `tiberius`, y se añadieron `async-trait`, `chrono`, `rust_decimal` y `uuid` como dependencias explícitas del adaptador.
- Se añadieron pruebas unitarias para `ExecuteResult`, preparación ordenada de parámetros, validación de conteo de placeholders y soporte de fechas en el pipeline de parámetros.
- `query_raw` quedó expuesto como base inmediata para la futura lectura de filas sin adelantar todavía el wrapper público `MssqlRow`.
- El binding de `SqlValue::Null` quedó implementado temporalmente como `Option::<String>::None`, porque el valor `Null` del core aún no transporta tipo SQL asociado; esta limitación quedó registrada para revisarla cuando exista metadata/tipo suficiente o wrapper de filas más completo.
- `Cargo.lock` se actualizó para registrar `async-trait` y las dependencias adicionales requeridas por el executor y el binder.
- Se validó el workspace con `cargo fmt --all --check`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 4 ya tiene ejecución base sobre Tiberius y binding real de `CompiledQuery`, dejando preparada la crate para agregar `MssqlRow`, `fetch_one`, `fetch_all` y mejor conversión de errores.

### Bloqueos

- No hubo bloqueos permanentes. Solo aparecieron tres ajustes locales durante la implementación: bounds/lifetimes al prestar parámetros a `tiberius::Query`, conversión explícita de `Decimal` a `Numeric`, y la limitación conocida del `NULL` sin tipo.

### Próximo paso recomendado

- Implementar `Etapa 4: MssqlRow y conversión de errores a OrmError`, usando `query_raw` como base para `fetch_one` y `fetch_all`.

### Sesión: `MssqlConnection` y configuración desde connection string

- Se confirmó nuevamente que el plan maestro no está en la raíz; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se tomó la tarea `Etapa 4: Implementar MssqlConnection y configuración desde connection string` como siguiente prioridad del backlog y se cerró tras validación del workspace.
- Se reemplazó el placeholder puro de `mssql-orm-tiberius` por una estructura inicial con módulos `config` y `connection`.
- Se añadió integración real con `tiberius` usando `tiberius = 0.12.3` con features `rustls`, `tds73`, `tokio` y `tokio-util`, más `tokio`, `tokio-util` y `futures-io` como soporte mínimo del adaptador.
- Se implementó `MssqlConnectionConfig::from_connection_string(&str) -> Result<Self, OrmError>` sobre `tiberius::Config::from_ado_string`, preservando el connection string original y exponiendo `addr()` para la conexión TCP.
- Se añadió validación propia para rechazar connection strings vacíos o que Tiberius acepte con host vacío (`server=`), evitando dejar configuración inválida pasar a la etapa de conexión.
- Se implementó `MssqlConnection<S>` con alias `TokioConnectionStream = Compat<TcpStream>`, junto con `connect`, `connect_with_config`, `config`, `client`, `client_mut` e `into_inner`.
- `MssqlConnection::connect` ya abre `tokio::net::TcpStream`, configura `TCP_NODELAY` y crea `tiberius::Client` real, pero sin adelantar todavía ejecución, binding de parámetros ni mapeo de filas.
- Se reexportaron `MssqlConnection`, `MssqlConnectionConfig` y `TokioConnectionStream` desde `crates/mssql-orm-tiberius/src/lib.rs`.
- Se añadieron pruebas unitarias para parseo válido de ADO connection strings, rechazo de configuración inválida y reexport del config desde la superficie de la crate.
- `Cargo.lock` se actualizó para registrar la incorporación de Tiberius y su árbol transitivo.
- Durante la validación apareció un ajuste necesario: `tiberius::Client<S>` exige bounds explícitos `AsyncRead + AsyncWrite + Unpin + Send` sobre `S`, por lo que se declararon en `MssqlConnection<S>` usando `futures-io`.
- Se validó el workspace con `cargo fmt --all --check`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 4 ya tiene configuración y conexión base sobre Tiberius, dejando lista la superficie necesaria para la siguiente tarea de `Executor` y binding de parámetros.

### Bloqueos

- No hubo bloqueos técnicos permanentes. Solo fue necesario endurecer la validación propia del connection string y explicitar los bounds genéricos exigidos por `tiberius::Client`.

### Próximo paso recomendado

- Implementar `Etapa 4: Executor sobre Tiberius con binding de parámetros`, consumiendo `CompiledQuery` sin mover lógica SQL fuera de `mssql-orm-sqlserver`.

### Sesión: Snapshot tests para SQL y orden de parámetros

- Se confirmó nuevamente que el plan maestro no está en la raíz; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se movió en `docs/tasks.md` la tarea `Etapa 3: Agregar snapshot tests para SQL y orden de parámetros` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió `insta = "1"` como `dev-dependency` en `crates/mssql-orm-sqlserver/Cargo.toml` para fijar el SQL compilado y el orden observable de parámetros con snapshots versionados.
- Se creó la prueba de integración `crates/mssql-orm-sqlserver/tests/compiler_snapshots.rs` con fixtures mínimas de entidad, modelos `Insertable`/`Changeset` y helper de render estable para `CompiledQuery`.
- Los snapshots nuevos cubren `select`, `insert`, `update`, `delete` y `count`, versionando tanto el SQL final como la secuencia exacta de parámetros `@P1..@Pn`.
- Se generaron y aceptaron los archivos `.snap` bajo `crates/mssql-orm-sqlserver/tests/snapshots/` usando `INSTA_UPDATE=always cargo test -p mssql-orm-sqlserver --test compiler_snapshots`.
- `Cargo.lock` se actualizó para registrar la nueva dependencia de test y su árbol transitivo.
- Durante la validación, `cargo fmt --all --check` detectó solo un ajuste menor de formato en el archivo nuevo de tests; se corrigió con `cargo fmt --all` y luego el workspace quedó limpio.
- Se validó el workspace con `cargo fmt --all --check`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 3 quedó consolidada con snapshots versionados del compilador SQL Server, reduciendo el riesgo de regresiones silenciosas en formato de SQL y orden de parámetros.

### Bloqueos

- No hubo bloqueos técnicos. Solo fue necesario descargar e incorporar la dependencia nueva de testing y aceptar los snapshots iniciales.

### Próximo paso recomendado

- Empezar `Etapa 4: Implementar MssqlConnection y configuración desde connection string`, manteniendo `mssql-orm-sqlserver` y `CompiledQuery` ya estabilizados.

### Sesión: Compilación SQL Server a `CompiledQuery`

- Se confirmó nuevamente que el plan maestro no está en la raíz; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se movió en `docs/tasks.md` la tarea `Etapa 3: Compilar select, insert, update, delete y count a SQL parametrizado @P1..@Pn` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió `crates/mssql-orm-sqlserver/src/compiler.rs` como primera implementación real del compilador SQL Server sobre el AST de `mssql-orm-query`.
- `SqlServerCompiler` ahora expone `compile_query`, `compile_select`, `compile_insert`, `compile_update`, `compile_delete` y `compile_count`, todos devolviendo `Result<CompiledQuery, OrmError>`.
- Se implementó un builder interno de parámetros para preservar el orden exacto de `@P1..@Pn` y garantizar que `params.len()` coincida con los placeholders emitidos.
- La compilación de `select` cubre proyección explícita o `*` por defecto, `WHERE`, `ORDER BY` y `OFFSET ... FETCH NEXT ...` usando parámetros para `offset` y `limit`.
- La compilación de `insert` y `update` emite `OUTPUT INSERTED.*` en línea con el plan maestro actual; `delete` y `count` se compilan sin adelantar responsabilidades de ejecución.
- La compilación soporta `Expr::Column`, `Expr::Value`, `Expr::Binary`, `Expr::Unary` y `Expr::Function`, además de `Predicate` con comparaciones, `LIKE`, nulabilidad y composición lógica.
- Se añadieron errores explícitos para combinaciones inválidas o ambiguas en esta etapa, por ejemplo paginación sin `ORDER BY`, `INSERT` sin valores, `UPDATE` sin cambios, funciones vacías y predicados lógicos sin hijos.
- Se agregaron pruebas unitarias en `mssql-orm-sqlserver` para `select`, `insert`, `update`, `delete`, `count`, orden de parámetros, entrada única mediante `Query`, expresiones/funciones y rutas de error.
- Durante la validación apareció una advertencia por `pub use compiler::*` innecesario en `lib.rs`; se eliminó y luego el workspace quedó limpio.
- Se validó el workspace con `cargo fmt --all --check`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 3 ya cuenta con compilación real del AST a SQL Server parametrizado y el contrato `CompiledQuery` quedó conectado de forma usable con el dialecto.

### Bloqueos

- No hubo bloqueos técnicos. Solo apareció una advertencia local de import no usado durante la primera pasada de validación y se corrigió en la misma sesión.

### Próximo paso recomendado

- Ejecutar `Etapa 3: Agregar snapshot tests para SQL y orden de parámetros` para fijar la salida del compilador antes de avanzar a la capa Tiberius.

### Sesión: Quoting seguro de identificadores SQL Server

- Se confirmó nuevamente que el plan maestro no está en la raíz; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se movió en `docs/tasks.md` la tarea `Etapa 3: Implementar quoting seguro de identificadores SQL Server` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se reemplazó el placeholder puro de `mssql-orm-sqlserver` por una primera capacidad real del dialecto mediante el módulo nuevo `crates/mssql-orm-sqlserver/src/quoting.rs`.
- Se implementó `quote_identifier(&str) -> Result<String, OrmError>` para producir identificadores entre corchetes, escapando `]` como `]]`.
- La validación del identificador rechaza nombre vacío, caracteres de control y el separador `.` dentro de una sola parte, forzando que schema y objeto se coticen por separado.
- Se añadieron helpers `quote_qualified_identifier`, `quote_table_ref` y `quote_column_ref` para reutilizar metadata del AST sin adelantar todavía la compilación completa de `select`, `insert`, `update`, `delete` ni `count`.
- Se reexportó la API de quoting desde `crates/mssql-orm-sqlserver/src/lib.rs` para que la siguiente tarea del compilador la consuma desde la superficie pública de la crate.
- Se agregaron pruebas unitarias para quoting simple, escape de `]`, rechazo de identificadores vacíos, rechazo de caracteres de control, rechazo de multipartes en la API de segmento único y quoting de `TableRef`/`ColumnRef`.
- Durante la validación, `cargo fmt --all --check` reportó únicamente ajustes de estilo en los archivos nuevos; se corrigieron con `cargo fmt --all` y luego el workspace quedó limpio.
- Se validó el workspace con `cargo fmt --all --check`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 3 ya tiene quoting seguro y reutilizable de identificadores SQL Server, dejando preparada la base inmediata para compilar el AST a SQL parametrizado `@P1..@Pn`.

### Bloqueos

- No hubo bloqueos técnicos. Solo apareció un ajuste de formato detectado por `rustfmt` en la primera pasada.

### Próximo paso recomendado

- Implementar `Etapa 3: Compilar select, insert, update, delete y count a SQL parametrizado @P1..@Pn` en `mssql-orm-sqlserver`, reutilizando los helpers de quoting recién introducidos.

## 2026-04-22

### Sesión: AST de queries y `CompiledQuery`

- Se confirmó nuevamente que el plan maestro no está en la raíz; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se movió en `docs/tasks.md` la tarea `Etapa 3: Implementar AST de queries y CompiledQuery` a `En Progreso` antes de validar el trabajo y luego a `Completadas` tras cerrar la implementación.
- Se reemplazó el placeholder de `mssql-orm-query` por una estructura real de módulos alineada con el árbol previsto en el plan: `expr`, `predicate`, `select`, `insert`, `update`, `delete`, `order` y `pagination`.
- Se implementaron `TableRef` y `ColumnRef`, incluyendo puente explícito desde `EntityColumn<E>` hacia el AST para reutilizar la metadata estática ya generada en Etapa 1.
- Se implementó el AST base `Expr` con variantes `Column`, `Value`, `Binary`, `Unary` y `Function`, junto con `BinaryOp` y `UnaryOp`.
- Se implementó `Predicate` con operadores de comparación, `LIKE`, nulabilidad y composición lógica, manteniéndolo todavía como representación estructural sin emitir SQL.
- Se implementaron `SelectQuery`, `CountQuery`, `InsertQuery`, `UpdateQuery` y `DeleteQuery` como operaciones del AST, con `filter` acumulativo, `order_by` y `Pagination`.
- `InsertQuery` y `UpdateQuery` consumen directamente `Insertable<E>` y `Changeset<E>`, dejando conectadas las etapas 2 y 3 sin mover responsabilidades a `sqlserver` ni `tiberius`.
- Se agregó `CompiledQuery { sql, params }` como contrato neutral compartido para la futura compilación SQL Server y la capa de ejecución.
- Se añadieron pruebas unitarias en `mssql-orm-query` para cubrir resolución de columnas desde entidades, composición de expresiones, composición de predicados, captura de `select/count/insert/update/delete`, paginación y preservación de orden de parámetros en `CompiledQuery`.
- Durante la validación se corrigieron dos detalles locales: se eliminó `Eq` de `CompiledQuery` porque `SqlValue` no puede implementarlo por contener `f64`, y se renombró el helper `Predicate::not` a `Predicate::negate` para satisfacer `clippy`.
- Se validó el workspace con `cargo fmt --all --check`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 3 ya tiene un AST utilizable y un contrato `CompiledQuery` estable, dejando a `mssql-orm-query` listo para que la siguiente tarea implemente quoting y compilación SQL Server en la crate correspondiente.

### Bloqueos

- No hubo bloqueos técnicos. Solo aparecieron ajustes menores de modelado y lint detectados por compilación y `clippy`.

### Próximo paso recomendado

- Ejecutar `Etapa 3: Implementar quoting seguro de identificadores SQL Server` en `mssql-orm-sqlserver` como base inmediata del compilador de `select`, `insert`, `update`, `delete` y `count`.

### Sesión: Pruebas de mapping de filas y valores persistibles

- Se confirmó otra vez que el plan maestro no está en la raíz; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se movió en `docs/tasks.md` la tarea `Etapa 2: Crear pruebas de mapping de filas y extracción de valores persistibles` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió la prueba de integración `crates/mssql-orm/tests/stage2_mapping.rs` para cubrir el uso público real de la API de Etapa 2.
- La nueva prueba define una entidad derivada `Customer`, modelos `NewCustomer` y `UpdateCustomer`, un `TestRow` neutral sobre `SqlValue` y un `CustomerRecord` con implementación manual de `FromRow`.
- Se cubrieron escenarios de éxito y error para `FromRow`: lectura de columnas requeridas, lectura de columna nullable con `NULL`, ausencia de columna requerida y mismatch de tipo en extracción tipada.
- Se cubrió la extracción de valores persistibles desde `#[derive(Insertable)]`, verificando orden estable de campos y conversión de `Option<T>` a `SqlValue::Null`.
- Se cubrió la semántica de `#[derive(Changeset)]`, verificando que solo se emitan cambios presentes y que `Some(None)` preserve la actualización explícita a `NULL`.
- Fue necesario añadir `#[allow(dead_code)]` solo sobre la entidad del test para mantener `cargo clippy -D warnings` limpio, ya que la struct se usa como portadora de metadata derivada y no se instancia directamente.
- Se validó el workspace con `cargo fmt --all --check`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 2 quedó cerrada con cobertura adicional sobre el recorrido actual de persistencia y mapeo, sin adelantar AST, compilación SQL ni integración con Tiberius.

### Bloqueos

- No hubo bloqueos técnicos. Solo apareció una advertencia de `dead_code` en la entidad del test de integración y se resolvió de forma local y explícita.

### Próximo paso recomendado

- Empezar `Etapa 3: Implementar AST de queries y CompiledQuery`, manteniendo el límite de que `mssql-orm-query` modele AST y parámetros sin generar SQL directo.

### Sesión: Derives `Insertable` y `Changeset`

- Se confirmó que el archivo del plan maestro no está en la raíz; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se movió en `docs/tasks.md` la tarea `Etapa 2: Implementar derives #[derive(Insertable)] y #[derive(Changeset)]` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se implementó en `crates/mssql-orm-macros` el derive real de `#[derive(Insertable)]`, con soporte para `#[orm(entity = MiEntidad)]`, structs con campos nombrados y override opcional `#[orm(column = "...")]` por campo.
- El derive `Insertable` genera `Vec<ColumnValue>` usando `SqlTypeMapping::to_sql_value` sobre clones de los campos y resuelve el nombre final de columna contra la metadata de la entidad objetivo.
- Se implementó en `crates/mssql-orm-macros` el derive real de `#[derive(Changeset)]`, también con `#[orm(entity = MiEntidad)]` y soporte opcional `#[orm(column = "...")]`.
- El derive `Changeset` exige `Option<T>` en el nivel externo de cada campo para preservar la semántica del plan: `None` omite la actualización, `Some(None)` produce `NULL` cuando el tipo interno es `Option<U>` y `Some(Some(valor))` persiste el valor indicado.
- Se actualizó `crates/mssql-orm/src/lib.rs` para reexportar en la `prelude` los macros `Insertable` y `Changeset`.
- Se añadieron pruebas unitarias en la crate pública para cubrir extracción de `values()` y `changes()` desde modelos derivados, incluyendo mapeo por nombre de columna explícito y el caso `Option<Option<T>>`.
- Se amplió `trybuild` con un caso válido para ambos derives y dos fallos esperados: ausencia de `#[orm(entity = ...)]` en `Insertable` y uso de un campo no `Option<_>` en `Changeset`.
- Se versionaron los snapshots `.stderr` nuevos de `trybuild` y se eliminó el directorio temporal `wip` generado durante la aceptación de errores de compilación.
- Se validó el workspace con `cargo fmt --all --check`, `cargo check --workspace`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 2 ya cuenta con derives funcionales para modelos de inserción y actualización, alineados con la metadata de entidades existente y sin adelantar responsabilidades de AST, compilación SQL ni ejecución.

### Bloqueos

- No hubo bloqueos técnicos; solo fue necesario fijar los snapshots `.stderr` nuevos de `trybuild` y ajustar una observación menor de Clippy sobre un borrow innecesario.

### Próximo paso recomendado

- Ejecutar la tarea `Etapa 2: Crear pruebas de mapping de filas y extracción de valores persistibles`, enfocándola en cobertura adicional de `FromRow`, `Insertable` y `Changeset` desde modelos derivados.

## 2026-04-21

### Sesión: Inicialización del sistema autónomo

- Se creó la carpeta `docs/` como base operativa del repositorio.
- Se creó `docs/instructions.md` con reglas de operación, flujo de trabajo, restricciones, gestión de tareas y criterios de calidad.
- Se creó `docs/tasks.md` como fuente única de verdad del trabajo pendiente.
- Se creó `docs/context.md` para conservar contexto transversal entre sesiones.

### Resultado

- El repositorio ya tiene una base documental mínima para trabajo autónomo con trazabilidad.

### Próximo paso recomendado

- Traducir el plan maestro del ORM a tareas ejecutables por etapas y priorizarlas en `docs/tasks.md`.

### Sesión: Ajuste de backlog desde el plan maestro

- Se actualizó `docs/tasks.md` para reflejar el roadmap del archivo `plan_orm_sqlserver_tiberius_code_first.md`.
- Las tareas pendientes quedaron reorganizadas por etapas, desde fundamentos del workspace hasta release y documentación pública.
- Se preservó `Completadas` para lo ya realizado en esta fase documental.

### Resultado

- El proyecto ya tiene un backlog operativo alineado con el plan técnico principal.

### Próximo paso recomendado

- Empezar la Etapa 0 creando el workspace Rust y la estructura inicial de crates.

### Sesión: Creación del workspace Rust base

- Se confirmó que el plan maestro no está en la raíz; la ruta real usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se creó el `Cargo.toml` raíz como workspace con las ocho crates base bajo `crates/`.
- Se generaron las crates `mssql-orm`, `mssql-orm-core`, `mssql-orm-macros`, `mssql-orm-query`, `mssql-orm-sqlserver`, `mssql-orm-tiberius`, `mssql-orm-migrate` y `mssql-orm-cli`.
- Se ajustaron los `Cargo.toml` internos para usar configuración compartida de workspace y dependencias mínimas coherentes con la arquitectura.
- Se convirtió `mssql-orm-macros` en crate `proc-macro` con derives placeholder vacíos para `Entity`, `DbContext`, `Insertable` y `Changeset`.
- Se reemplazó el código de plantilla por marcadores mínimos por crate para dejar explícitas sus responsabilidades sin adelantar funcionalidad de etapas posteriores.
- Se expuso una `prelude` mínima en la crate pública `mssql-orm` y se reexportaron las crates internas de infraestructura desde la API principal.
- Se validó el workspace con `cargo fmt --check`, `cargo check --workspace`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features`.

### Resultado

- El repositorio ya tiene un workspace Rust compilable, validado y alineado con la segmentación arquitectónica definida para el ORM.

### Bloqueos

- No hubo bloqueos técnicos para esta tarea.

### Próximo paso recomendado

- Implementar la tarea `Etapa 0: Configurar CI base con cargo check, cargo test, rustfmt y clippy`.

### Sesión: Consolidación de repositorio Git único

- Se registró en `docs/tasks.md` una tarea operativa para consolidar un único repositorio Git en la raíz.
- Se actualizó `docs/instructions.md` para exigir commit al cierre de una tarea completada y validada.
- Se añadió la regla operativa de mantener un único repositorio Git en la raíz del proyecto.
- Se creó `.gitignore` en la raíz para ignorar artefactos `target`.
- Se eliminaron los directorios `.git` anidados creados dentro de cada crate.
- Se inicializó un repositorio Git único en la raíz del proyecto.
- Se verificó que solo exista `./.git` y que el workspace siga compilando con `cargo check --workspace`.

### Resultado

- El proyecto quedó consolidado bajo un único repositorio Git raíz y la política de cierre con commit quedó documentada.

### Bloqueos

- No hubo bloqueos técnicos para esta tarea.

### Próximo paso recomendado

- Implementar la tarea `Etapa 0: Configurar CI base con cargo check, cargo test, rustfmt y clippy`.

### Sesión: Configuración de CI base

- Se confirmó nuevamente que el plan maestro no está en la raíz; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se movió en `docs/tasks.md` la tarea `Etapa 0: Configurar CI base con cargo check, cargo test, rustfmt y clippy` a `En Progreso` antes de iniciar la implementación y luego a `Completadas` tras validarla.
- Se creó `.github/workflows/ci.yml` con un workflow base de GitHub Actions para ejecutar `cargo fmt --all --check`, `cargo check --workspace`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- El workflow instala el toolchain estable de Rust con `rustfmt` y `clippy` y utiliza caché de dependencias para acelerar ejecuciones posteriores.
- Se validó localmente el mismo conjunto de chequeos definido en CI sobre el workspace actual.

### Resultado

- El repositorio quedó con CI base alineada con la Etapa 0 y con validaciones locales consistentes con el pipeline automatizado.

### Bloqueos

- No hubo bloqueos técnicos para esta tarea.

### Próximo paso recomendado

- Implementar la tarea `Etapa 0: Crear README principal, ADRs iniciales y documentación arquitectónica mínima`.

### Sesión: Base documental pública y arquitectónica

- Se tomó la siguiente tarea prioritaria de la Etapa 0: `Crear README principal, ADRs iniciales y documentación arquitectónica mínima`.
- Se creó `README.md` en la raíz con objetivo del proyecto, estado actual, arquitectura del workspace, restricciones y validación base.
- Se creó `docs/architecture/overview.md` para fijar el flujo arquitectónico esperado y los límites explícitos por crate antes de la Etapa 1.
- Se creó `docs/adr/0001-sql-server-first.md` para dejar formalizada la decisión de soportar solo SQL Server en esta fase.
- Se creó `docs/adr/0002-workspace-boundaries.md` para fijar la separación estricta por crates y sus responsabilidades.
- Se creó `docs/adr/0003-public-api-in-root-crate.md` para formalizar que la API pública se concentra en `mssql-orm`.
- Se validó que el workspace siga compilando con `cargo check --workspace`.

### Resultado

- El repositorio ya tiene documentación pública mínima y decisiones arquitectónicas explícitas para evitar improvisación al iniciar metadata y macros reales.

### Bloqueos

- No hubo bloqueos técnicos para esta tarea.

### Próximo paso recomendado

- Implementar la tarea `Etapa 0: Crear documentación de colaboración con IA en docs/ai/`.

### Sesión: Documentación de colaboración con IA

- Se creó `docs/ai/README.md` como guía base de colaboración para agentes de IA con fuente de verdad, límites de actuación, política de continuidad y criterios mínimos de validación.
- Se creó `docs/ai/session-template.md` con una plantilla de sesión para mantener el flujo de lectura, selección de tarea, ejecución, validación y cierre.
- Se creó `docs/ai/handover-checklist.md` como checklist de cierre para asegurar trazabilidad documental y commits limpios.
- Se movió en `docs/tasks.md` la tarea `Etapa 0: Crear documentación de colaboración con IA en docs/ai/` a `En Progreso` antes de implementarla y luego a `Completadas`.
- Se verificó consistencia del repositorio documental y se validó el workspace con `cargo check --workspace`.

### Resultado

- La Etapa 0 quedó cerrada con base operativa, CI, documentación pública, arquitectura explícita y guías concretas para continuidad de agentes.

### Bloqueos

- No hubo bloqueos técnicos para esta tarea.

### Próximo paso recomendado

- Empezar `Etapa 1: Implementar Entity trait y metadata base (EntityMetadata, ColumnMetadata, índices y foreign keys)` en `mssql-orm-core`.

### Sesión: Metadata base de entidades en core

- Se implementó en `crates/mssql-orm-core` el trait `Entity` con contrato estático `metadata() -> &'static EntityMetadata`.
- Se agregaron los tipos base de metadata: `EntityMetadata`, `ColumnMetadata`, `PrimaryKeyMetadata`, `IndexMetadata`, `IndexColumnMetadata`, `ForeignKeyMetadata`, `IdentityMetadata`, `ReferentialAction` y `SqlServerType`.
- Se añadieron helpers mínimos de lectura sobre metadata (`column`, `field`, `primary_key_columns`) y helpers de columna (`is_computed`, `is_generated_on_insert`).
- Se mejoró `OrmError` para implementar `Display` y `std::error::Error`, manteniéndolo todavía como error base simple.
- Se expusieron los contratos y tipos nuevos desde la `prelude` de `mssql-orm`, junto con el reexport del macro namespace.
- Se añadieron pruebas unitarias en `mssql-orm-core` y en la crate pública para verificar lookup de metadata, llaves primarias, índices, columnas generadas y exposición de la API.
- Se validó el workspace con `cargo fmt --all --check`, `cargo check --workspace`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 1 ya tiene contratos estables de metadata en `core`, listos para que `mssql-orm-macros` implemente `#[derive(Entity)]` sin introducir todavía SQL ni ejecución.

### Bloqueos

- No hubo bloqueos técnicos para esta tarea.

### Próximo paso recomendado

- Implementar `Etapa 1: #[derive(Entity)]` en `mssql-orm-macros`, consumiendo los tipos de metadata recién definidos.

### Sesión: Corrección de alineación contra el plan maestro

- Se revisó la implementación de metadata base contra `docs/plan_orm_sqlserver_tiberius_code_first.md`, tratándolo como fuente principal de verdad para contratos y shapes de tipos.
- Se corrigió `EntityMetadata::primary_key_columns()` para preservar el orden declarado en `PrimaryKeyMetadata`, en lugar del orden de `self.columns`.
- Se eliminó de `ColumnMetadata` el helper `is_generated_on_insert`, porque introducía semántica derivada no definida por el plan y potencialmente conflictiva con `insertable` y `default_sql`.
- Se ajustaron las pruebas de `mssql-orm-core` para cubrir orden de claves primarias compuestas y mantener solo helpers alineados con campos explícitos del plan.
- Se reforzó `docs/instructions.md` y `docs/ai/README.md` para dejar explícito que el plan maestro prevalece sobre inferencias locales cuando se definen contratos, tipos o responsabilidades.
- Se validó el workspace con `cargo fmt --all --check`, `cargo check --workspace`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La metadata base de entidades volvió a quedar alineada con el plan maestro y la documentación operativa reduce el riesgo de repetir desalineaciones similares.

### Bloqueos

- No hubo bloqueos técnicos para esta tarea.

### Próximo paso recomendado

- Implementar `Etapa 1: #[derive(Entity)]` en `mssql-orm-macros`, usando el plan maestro como referencia principal del shape de metadata generado.

### Sesión: Derive `Entity` funcional con metadata estática

- Se confirmó que el plan maestro no está en la raíz; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se movió en `docs/tasks.md` la tarea `Etapa 1: Implementar #[derive(Entity)] con parser de atributos #[orm(...)]` a `En Progreso` antes de editar y, tras validarla, a `Completadas`.
- Se completó en `crates/mssql-orm-macros` una implementación real de `#[derive(Entity)]` basada en `syn` y `quote`.
- El derive ahora genera `EntityMetadata` estática e implementa `mssql_orm::core::Entity` para structs con campos nombrados.
- Se soportaron en el parser los atributos de la etapa activa necesarios para materializar metadata: `table`, `schema`, `column`, `primary_key`, `identity`, `length`, `nullable`, `default_sql`, `index`, `unique`, además de `sql_type`, `precision`, `scale`, `computed_sql` y `rowversion` como soporte directo del shape ya definido en `core`.
- Se añadieron convenciones mínimas alineadas con el plan: `schema = "dbo"` por defecto, nombre de tabla en `snake_case` pluralizado, `id` como primary key por convención, `Option<T>` como nullable, `String -> nvarchar(255)` y `Decimal -> decimal(18,2)` cuando aplique.
- Se incorporaron validaciones tempranas del macro para rechazar entidades sin PK, `identity` sobre tipos no enteros y `rowversion` fuera de `Vec<u8>`.
- Se ajustó `crates/mssql-orm/src/lib.rs` para declarar `extern crate self as mssql_orm`, estabilizando la ruta generada por el macro tanto para consumidores como para pruebas internas.
- Se agregaron pruebas unitarias en la crate pública para verificar metadata derivada, convenciones por defecto, índices únicos y no únicos, flags `insertable`/`updatable`, `rowversion` y defaults.
- Se movió también a `Completadas` la tarea `Etapa 1: Soportar atributos base table, schema, primary_key, identity, length, nullable, default_sql, index y unique`, porque quedó cubierta por la implementación del derive y su validación.
- Se validó el workspace con `cargo fmt --all --check`, `cargo check --workspace`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 1 ya cuenta con un `#[derive(Entity)]` operativo que genera metadata estática usable desde la API pública, sin romper los límites entre `core`, `macros`, SQL ni ejecución.

### Bloqueos

- No hubo bloqueos técnicos al cerrar la tarea; la única corrección iterativa necesaria fue ajustar la convención de pluralización por defecto para nombres terminados en consonante + `y`.

### Próximo paso recomendado

- Implementar `Etapa 1: Generar columnas estáticas para el futuro query builder`.

### Sesión: Columnas estáticas para el query builder futuro

- Se movió en `docs/tasks.md` la tarea `Etapa 1: Generar columnas estáticas para el futuro query builder` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se incorporó en `crates/mssql-orm-core` el tipo `EntityColumn<E>` como símbolo estático de columna, desacoplado todavía del AST y de cualquier generación SQL.
- `EntityColumn<E>` expone `rust_field()`, `column_name()`, `entity_metadata()` y `metadata()`, reutilizando la metadata estática ya generada por `Entity`.
- Se actualizó `#[derive(Entity)]` en `crates/mssql-orm-macros` para generar asociados estáticos por campo con la forma esperada por el plan maestro, por ejemplo `Customer::email` y `Customer::created_at`.
- La generación se hizo como `impl` inherente con `#[allow(non_upper_case_globals)]`, de modo que los símbolos queden en minúsculas y usables desde la API prevista sin introducir warnings en la validación estricta.
- Se reexportó `EntityColumn` desde la `prelude` de `mssql-orm`.
- Se añadieron pruebas unitarias en `mssql-orm-core` y `mssql-orm` para verificar resolución de metadata desde `EntityColumn` y uso real de `Entity::campo` desde entidades derivadas.
- Se validó el workspace con `cargo fmt --all --check`, `cargo check --workspace`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 1 ya expone símbolos estáticos de columna alineados con la API objetivo del plan, dejando listo el soporte base para que una etapa posterior construya el query builder encima de ellos.

### Bloqueos

- No hubo bloqueos técnicos; solo fue necesario ajustar formato con `cargo fmt` antes de la validación final.

### Próximo paso recomendado

- Implementar `Etapa 1: Agregar pruebas trybuild para casos válidos e inválidos de entidades`.

### Sesión: Pruebas `trybuild` para derive de entidades

- Se movió en `docs/tasks.md` la tarea `Etapa 1: Agregar pruebas trybuild para casos válidos e inválidos de entidades` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió `trybuild` como `dev-dependency` en `crates/mssql-orm/Cargo.toml`.
- Se creó el harness [crates/mssql-orm/tests/trybuild.rs](/home/esteban94/Proyectos/Rust/mssql-orm/crates/mssql-orm/tests/trybuild.rs) para validar el derive `Entity` desde la crate pública `mssql-orm`, replicando el punto de integración real de un consumidor.
- Se añadieron fixtures UI en `crates/mssql-orm/tests/ui/` para un caso válido y tres inválidos ya soportados por el macro actual: entidad sin primary key, `identity` en tipo no entero y `rowversion` fuera de `Vec<u8>`.
- Se generaron y versionaron los snapshots `.stderr` de `trybuild` para fijar los mensajes de error de compilación esperados del macro.
- Se mantuvo el alcance acotado a validaciones ya implementadas; no se añadieron reglas nuevas ni se adelantó soporte de `foreign_key`, `Insertable`, `Changeset` ni AST de queries.
- Se validó el workspace con `cargo fmt --all --check`, `cargo check --workspace`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 1 quedó cerrada para el derive `Entity` actual, con cobertura de compilación positiva y negativa sobre la API pública del crate principal.

### Bloqueos

- No hubo bloqueos técnicos; la única preparación extra fue descargar `trybuild` y sus dependencias de desarrollo para ejecutar el harness.

### Próximo paso recomendado

- Empezar `Etapa 2: Implementar FromRow, Insertable, Changeset y SqlValue`.

### Sesión: Contratos base de mapping y valores persistibles

- Se movió en `docs/tasks.md` la tarea `Etapa 2: Implementar FromRow, Insertable, Changeset y SqlValue` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadieron en `crates/mssql-orm-core` los contratos `FromRow`, `Insertable<E>`, `Changeset<E>` y el enum `SqlValue`.
- Se incorporó también `ColumnValue` como par columna/valor persistible y el trait `Row` como abstracción neutra de lectura de filas, para evitar acoplar `core` al wrapper concreto de Tiberius que se implementará más adelante.
- `SqlValue` quedó con variantes base alineadas al plan actual: `Null`, `Bool`, `I32`, `I64`, `F64`, `String`, `Bytes`, `Uuid`, `Decimal`, `Date` y `DateTime`.
- Se añadieron dependencias en `mssql-orm-core` para `chrono`, `uuid` y `rust_decimal`, necesarias para materializar el contrato de `SqlValue` definido por el plan maestro.
- Se reexportaron los contratos nuevos desde la `prelude` de `mssql-orm`.
- Se agregaron pruebas unitarias en `mssql-orm-core` para mapping básico desde una fila fake y para extracción de `ColumnValue` desde implementaciones manuales de `Insertable` y `Changeset`.
- Se ajustó una prueba en la crate pública `mssql-orm` para verificar exposición de `ColumnValue` y `SqlValue` desde la API pública.
- Se validó el workspace con `cargo fmt --all --check`, `cargo check --workspace`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 2 ya tiene contratos base estables en `core` para leer filas de forma abstracta y representar valores persistibles, sin romper la separación arquitectónica respecto de `mssql-orm-tiberius`.

### Bloqueos

- No hubo bloqueos técnicos; la única consideración de diseño fue introducir el trait `Row` como abstracción intermedia para respetar que `core` no dependa del wrapper concreto `MssqlRow`.

### Próximo paso recomendado

- Implementar `Etapa 2: Definir mapeo base Rust -> SQL Server para tipos soportados`.

### Sesión: Mapeo base Rust -> SQL Server

- Se movió en `docs/tasks.md` la tarea `Etapa 2: Definir mapeo base Rust -> SQL Server para tipos soportados` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió en `crates/mssql-orm-core` el trait `SqlTypeMapping` como contrato base para relacionar tipos Rust con `SqlServerType`, `SqlValue` y metadata derivada (`DEFAULT_MAX_LENGTH`, `DEFAULT_PRECISION`, `DEFAULT_SCALE`).
- Se implementó `SqlTypeMapping` para los tipos base previstos en el plan actual: `bool`, `i32`, `i64`, `f64`, `String`, `Vec<u8>`, `uuid::Uuid`, `rust_decimal::Decimal`, `chrono::NaiveDate`, `chrono::NaiveDateTime` y `Option<T>`.
- Se añadieron helpers tipados `try_get_typed<T>()` y `get_required_typed<T>()` al trait `Row`, para que `FromRow` pueda apoyarse en el mapping base sin conocer detalles del wrapper de infraestructura.
- Se ajustó una prueba existente de `FromRow` para usar el mapping tipado ya introducido.
- Se reexportó `SqlTypeMapping` desde la `prelude` de `mssql-orm`.
- Se añadieron pruebas unitarias en `mssql-orm-core` para validar convenciones por defecto (`String -> nvarchar(255)`, `Decimal -> decimal(18,2)`, etc.) y roundtrip `Rust <-> SqlValue` sobre los tipos soportados.
- Se restringieron `rust_decimal` y `uuid` a configuraciones sin features por defecto, manteniendo solo el soporte mínimo necesario para estos contratos base.
- Se validó el workspace con `cargo fmt --all --check`, `cargo check --workspace`, `cargo test --workspace` y `cargo clippy --workspace --all-targets --all-features -- -D warnings`.

### Resultado

- La Etapa 2 ya tiene un mapping base explícito entre tipos Rust soportados, metadata SQL Server y valores persistibles, listo para que los derives de `Insertable` y `Changeset` se construyan sobre ese contrato.

### Bloqueos

- No hubo bloqueos técnicos; solo fue necesario corregir una importación faltante en las pruebas de `core` durante la iteración de validación.

### Próximo paso recomendado

- Implementar `Etapa 2: Implementar derives #[derive(Insertable)] y #[derive(Changeset)]`.

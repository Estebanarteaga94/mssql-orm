# Worklog

## 2026-04-23

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

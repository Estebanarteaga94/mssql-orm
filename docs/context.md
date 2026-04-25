# Contexto del Proyecto

## Estado Actual

El repositorio contiene como base principal el documento `docs/plan_orm_sqlserver_tiberius_code_first.md`, que describe la visión y roadmap para construir un ORM code-first en Rust para SQL Server usando Tiberius.

El backlog operativo de `docs/tasks.md` ya fue alineado con ese plan maestro y ahora representa la secuencia de trabajo recomendada por etapas.

Ya existe un workspace Rust inicial con crates separadas para `mssql-orm`, `core`, `macros`, `query`, `sqlserver`, `tiberius`, `migrate` y `cli`.
El control de versiones quedó consolidado en un único repositorio Git en la raíz; no deben existir repositorios anidados dentro de `crates/`.
También existe CI base en GitHub Actions para validar formato, compilación, pruebas y lint del workspace.
Ya existe documentación pública mínima en `README.md`, documentación arquitectónica en `docs/architecture/overview.md` y ADRs iniciales en `docs/adr/`.
Ya existe `docs/ai/` con guía de colaboración, plantilla de sesión y checklist de handoff para futuras sesiones autónomas.
`mssql-orm-core` ya contiene el contrato `Entity` y la metadata base de entidades, columnas, índices y foreign keys.
La metadata base fue re-alineada contra el plan maestro para preservar el orden de PK compuesto y evitar helpers con semántica no definida por el plan.

## Objetivo Técnico Actual

La Etapa 12 quedó cerrada con surface, persistencia, cobertura y límites documentados para el change tracking experimental. La Etapa 13 ya quedó cerrada también en migraciones avanzadas: índices compuestos, `computed columns`, foreign keys avanzadas, scripts idempotentes, `RenameColumn` explícito y `RenameTable` explícito ya están soportados dentro del pipeline de migraciones. La Etapa 14 también quedó cerrada: además de la surface operativa de producción (`timeouts`, `retry`, `tracing`, slow query, health, pool y wiring público desde pool`), el ejemplo web async `todo_app` ya tiene dominio, queries públicas, endpoints mínimos, wiring real con `MssqlPool` y validación reproducible contra SQL Server real. La Etapa 15 sigue en curso: ya quedaron cerradas la documentación pública base (`README.md`), el quickstart reproducible, la consolidación de ejemplos/guías operativas, la guía `code-first` (`docs/code-first.md`) y la guía de migraciones (`docs/migrations.md`). A raíz de la desalineación previa con el plan maestro se añadió backlog técnico nuevo para cerrar el gap; ya quedaron implementadas diez piezas fundacionales: `mssql-orm` expone `MigrationModelSource` con `entity_metadata()` derivado desde `DbContext`, `mssql-orm-migrate` puede serializar/deserializar `ModelSnapshot` en JSON, `mssql-orm-cli migration add` puede consumir un snapshot actual explícito con `--model-snapshot <Path>`, la CLI puede invocar un binario exportador del consumidor con `--snapshot-bin <BinName> --manifest-path <Path>`, `migration add` ya carga además el `model_snapshot.json` de la última migración local cuando dispone de snapshot actual para dejar explícito el par `previous/current`, la propia CLI ya ejecuta internamente `snapshot -> diff -> MigrationOperation -> DDL SQL Server` para calcular el plan observable de la migración, escribe automáticamente ese SQL compilado en `up.sql` dejando `model_snapshot.json` versionado con el estado actual, bloquea por defecto cambios destructivos salvo `--allow-destructive`, `examples/todo-app` ya cuenta con exportador de snapshot y script reproducible para generar migración inicial, migración incremental no-op y `database_update.sql`, y `mssql-orm-cli database update --execute` ya puede aplicar el script acumulado contra SQL Server usando `--connection-string`, `DATABASE_URL` o `MSSQL_ORM_TEST_CONNECTION_STRING`. El backlog de release sigue pendiente de las guías de query builder, transacciones, relaciones, API docs, changelog y validación final.

## Etapa 16 Propuesta: Entity Policies

Después de revisar el modelo actual y el dominio del ejemplo `todo_app`, quedó identificada una evolución natural del enfoque `code-first`: permitir que el usuario defina structs reutilizables para columnas transversales y que cada entidad declare qué políticas aplica. El caso inicial deseado es auditoría:

```rust
#[derive(Entity)]
#[orm(table = "todos", schema = "todo", audit = Audit)]
struct Todo {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    title: String,
}
```

La motivación viene de campos que hoy se repiten manualmente en entidades reales del repo, especialmente `created_at`, `created_by_user_id`, `version` y otros campos similares en `examples/todo-app/src/domain.rs`. La dirección correcta no es crear un sistema paralelo al modelo existente, sino extender la metadata `code-first`: las columnas generadas por una política deben terminar como `ColumnMetadata` normales dentro de `EntityMetadata.columns`. Así snapshots, diff, DDL SQL Server, migraciones y `DbContext` pueden seguir usando el pipeline existente sin rutas especiales.

El nombre conceptual elegido para esta línea es `Entity Policies`. Una policy representa una preocupación transversal de modelo que puede aportar columnas y, más adelante, comportamiento automático. Ejemplos candidatos:

- `audit = Audit`: columnas de auditoría como `created_at`, `created_by`, `updated_at`, `updated_by`.
- `timestamps = Timestamps`: variante reducida para `created_at` y `updated_at`.
- `concurrency = RowVersion`: forma declarativa sobre el soporte existente de `#[orm(rowversion)]`.
- `soft_delete = SoftDelete`: columnas y semántica de borrado lógico; cuando una entidad tenga esta policy, las rutas de borrado deben actualizar columnas como `deleted_at`/`deleted_by` en vez de emitir `DELETE` físico.
- `tenant = TenantScope`: columna `tenant_id` y filtros obligatorios de seguridad; cuando una entidad tenga esta policy, las rutas públicas de lectura y escritura deben aplicar automáticamente `tenant_id = current_tenant`.

El MVP de Etapa 16 debe ser conservador: implementar primero `audit = Audit` como generación de columnas y metadata. No debe autollenar valores todavía en `DbSet::insert`, `DbSet::update`, Active Record ni `save_changes`. Ese comportamiento futuro requiere un contrato separado tipo `AuditProvider`, porque toca rutas de persistencia, contexto por request, transacciones, tracking experimental y seguridad de datos.

La sintaxis preferida para el MVP es `#[orm(audit = Audit)]` a nivel de entidad. El usuario debería poder definir un struct de auditoría reutilizable con un derive o contrato explícito, por ejemplo `#[derive(AuditFields)]`, y ese struct debería usar atributos `#[orm(...)]` compatibles con columnas actuales: `column`, `length`, `nullable`, `default_sql`, `sql_type`, `precision`, `scale` y reglas equivalentes cuando apliquen. La implementación debe validar en compile-time que el struct tenga campos nombrados, tipos mapeables por `SqlTypeMapping`, columnas no duplicadas y atributos soportados.

Puntos de integración esperados:

- `mssql-orm-core`: contrato mínimo para que una policy exponga columnas reutilizables sin duplicar `ColumnMetadata`.
- `mssql-orm-macros`: parser de `#[orm(audit = Audit)]`, derive/contrato `AuditFields`, validaciones y expansión de columnas dentro del `EntityMetadata` generado.
- `mssql-orm-migrate`: idealmente no debería requerir cambios estructurales si las columnas auditables llegan como `ColumnMetadata`; sí requiere pruebas de snapshot/diff.
- `mssql-orm-sqlserver`: debería compilar DDL sin lógica especial si las columnas auditables son `ColumnSnapshot` normales; sí requiere snapshots SQL para defaults, tipos y nullability.
- `mssql-orm`: debe reexportar la surface pública necesaria en `prelude` y cubrirla con `trybuild` desde la perspectiva del consumidor.
- `examples/todo-app` o un fixture dedicado: debe mostrar una entidad real con `#[orm(audit = Audit)]` y validar que el exportador de snapshot captura esas columnas.

Hay una decisión sensible sobre si las columnas auditables deben existir como campos Rust visibles dentro del entity. Para el MVP se acepta que puedan ser solo metadata/schema si eso reduce el riesgo, pero debe quedar documentado explícitamente. Si se decide materializarlas en el struct Rust, entonces `FromRow`, símbolos de columna como `Todo::created_at`, inserciones, updates y ergonomía del usuario deben diseñarse con más cuidado. No conviene mezclar ambas decisiones sin pruebas.

La Etapa 16 ya fue agregada en `docs/tasks.md` con tareas pequeñas y verificables para evitar que una sesión futura deje la feature a medias. Las tareas separan diseño público, metadata, macros, validaciones compile-time, migraciones, DDL, ejemplo, documentación y validación local. También quedaron marcadas como `Etapa 16+` las extensiones que no deben colarse en el MVP: `AuditProvider`, autollenado, `timestamps`, `concurrency`, `soft_delete` y `tenant`.

El concepto público inicial de esta línea quedó documentado en `docs/entity-policies.md`. Ese documento fija que una `Entity Policy` es una pieza reutilizable de modelo `code-first`, declarada en compile-time sobre una entidad, que debe producir columnas normales dentro de `EntityMetadata.columns`. También deja explícito que el primer criterio técnico es evitar un segundo pipeline de esquema: snapshots, diff y DDL deben seguir consumiendo `ColumnMetadata`/`ColumnSnapshot` ordinarios.

Para `soft_delete`, el comportamiento esperado futuro debe quedar claro desde el diseño: si una entidad declara `#[orm(soft_delete = SoftDelete)]`, entonces `DbSet::delete(...)`, `entity.delete(&db)`, `DbSet::remove_tracked(...)` y `save_changes()` no deberían compilar ni ejecutar un `DELETE FROM ...` normal para esa entidad. En su lugar deben construir un `UPDATE` que marque la fila como eliminada lógicamente, por ejemplo asignando `deleted_at`, `deleted_by` o los campos definidos por el struct `SoftDelete`. Esta ruta debe seguir respetando primary key simple/compuesta según soporte vigente, `rowversion`, `ConcurrencyConflict`, transacciones y el pipeline existente de compilación SQL Server. También debe existir una decisión explícita sobre queries: por defecto las entidades con `soft_delete` deberían excluir filas con borrado lógico, y cualquier acceso a eliminadas debe requerir una API visible como `with_deleted()` o `only_deleted()`, no un bypass accidental.

Para `tenant`, el comportamiento esperado futuro es de seguridad, no solo comodidad. Si una entidad declara `#[orm(tenant = TenantScope)]`, toda ruta pública que lea o modifique esa entidad debe aplicar el tenant activo: `query().all()`, `first`, `count`, `find`, `update`, `delete`, Active Record y `save_changes()`. En ausencia de tenant activo, la política debe fallar cerrado por defecto para entidades tenant-scoped, en vez de ejecutar una consulta sin filtro. Los inserts deben recibir `tenant_id` desde el contexto o rechazar la operación si el usuario intenta insertar con un tenant distinto. Esta policy necesita pruebas dedicadas para asegurar que joins, query builder manual, tracking y helpers internos no puedan omitir el filtro por accidente.

## Dirección Arquitectónica Vigente

- El proyecto apunta a un workspace Rust con múltiples crates.
- La arquitectura propuesta separa `core`, `macros`, `query`, `sqlserver`, `tiberius`, `migrate` y `cli`.
- SQL Server es el objetivo inicial único.
- Tiberius debe quedar encapsulado como adaptador de infraestructura, no como núcleo del ORM.
- El MVP debe enfocarse en metadata, macros de entidad, CRUD básico, query builder simple, `DbContext`, `DbSet` y migraciones básicas.
- La crate pública `mssql-orm` centraliza la API expuesta y reexporta internals seleccionados.
- Para la Etapa 15, el usuario quiere que el `README.md` principal sea llamativo, didáctico y orientado a “vender” la librería: debe priorizar propuesta de valor, claridad de uso, quick wins y ejemplos atractivos, no solo inventario técnico de módulos.
- `docs/quickstart.md` ya existe como quickstart público reproducible para conexión, CRUD base y query builder; además quedó respaldado por el fixture `trybuild` `crates/mssql-orm/tests/ui/quickstart_public_valid.rs`, para que la documentación no derive hacia una API inválida.
- `docs/code-first.md` ya existe como guía pública del enfoque `code-first` actual y quedó respaldada por el fixture `trybuild` `crates/mssql-orm/tests/ui/code_first_public_valid.rs`, de modo que el discurso documental sobre derives, `DbContext` y `DbSet` quede anclado a la surface realmente compilable.
- `docs/migrations.md` ya existe como guía operativa de migraciones y deja explícito que la CLI actual `mssql-orm-cli` scaffolda, lista, genera `up.sql` desde snapshots reales y genera el script de `database update`. Por defecto imprime SQL a `stdout`; con `database update --execute` también puede aplicar el script usando el adaptador `mssql-orm-tiberius`.
- El plan maestro sí exige migraciones generadas automáticamente desde modelos Rust; para cerrar esa brecha el backlog ahora separa explícitamente las piezas faltantes: fuente de metadata del `DbContext`, snapshots persistibles, resolución de contexto del consumidor en CLI, generación automática de `up.sql` y validación end-to-end con `todo_app`.
- La primera de esas piezas ya quedó cubierta: la crate pública `mssql-orm` ahora expone el trait `MigrationModelSource`, y `#[derive(DbContext)]` implementa automáticamente `entity_metadata()` con las entidades declaradas en cada contexto, dejando una fuente estable de metadata para la futura CLI de migraciones automáticas.
- La segunda pieza también quedó cubierta en `mssql-orm-migrate`: `ModelSnapshot` y sus artefactos relacionados serializan/deserializan JSON estable mediante `to_json_pretty()` y `from_json(...)`; el filesystem de migraciones expone `read_model_snapshot(...)` y `write_model_snapshot(...)`, y `create_migration_scaffold(...)` ya escribe `model_snapshot.json` usando esa API en vez de un string placeholder.
- La crate pública `mssql-orm` ahora expone además `model_snapshot_from_source::<C>()` y `model_snapshot_json_from_source::<C>()`, para que un binario consumidor pueda exportar el snapshot actual directamente desde un `DbContext` que implemente `MigrationModelSource`.
- La CLI acepta `migration add <Name> --model-snapshot <Path>` como entrada explícita del snapshot actual del modelo y también `migration add <Name> --snapshot-bin <BinName> --manifest-path <Path>` para ejecutar un binario exportador del consumidor, capturar su `stdout` y versionar ese snapshot en la migración. Sin flags, `migration add` conserva el scaffold con snapshot vacío válido.
- `mssql-orm-migrate` ahora expone `latest_migration(...)` y `read_latest_model_snapshot(...)`; `mssql-orm-cli migration add` los usa para cargar el snapshot previo de la última migración local cuando ya tiene snapshot actual, y reporta ambos lados como base observable para el diff siguiente.
- `mssql-orm-cli migration add` ya ensambla internamente `diff_schema_and_table_operations`, `diff_column_operations`, `diff_relational_operations` y `SqlServerCompiler::compile_migration_operations(...)`, y ahora vuelca ese SQL compilado a `up.sql`. Cuando no hay operaciones, escribe `-- No schema changes detected.` como contenido explícito.
- `mssql-orm-cli migration add` bloquea por defecto planes destructivos antes de crear el scaffold. La detección actual cubre `DropTable`, `DropColumn`, reducción de longitud, cambio de tipo y nullable -> non-nullable sin `default_sql`; el bypass explícito es `--allow-destructive`.
- El artefacto editable MVP de migración queda definido como `up.sql`, `down.sql` y `model_snapshot.json`. La CLI lista esas rutas al crear una migración, `down.sql` queda como rollback manual y `migration.rs` se difiere explícitamente hasta diseñar una API Rust de migraciones que no duplique el pipeline actual.
- `diff_relational_operations(...)` ahora emite `CreateIndex` y `AddForeignKey` también para tablas nuevas, de modo que una migración inicial generada desde un snapshot completo no pierda índices ni relaciones.
- `examples/README.md` ya centraliza la guía de ejemplos disponibles y deja explícito que `todo-app` es el ejemplo ejecutable real presente en el árbol actual.
- `mssql-orm-core` ya define `Entity`, `EntityMetadata`, `ColumnMetadata`, `IndexMetadata`, `ForeignKeyMetadata`, `SqlServerType` y tipos auxiliares.
- `mssql-orm-core` ahora también expone helpers explícitos de metadata relacional sobre `ForeignKeyMetadata` y `EntityMetadata`, incluyendo búsqueda por nombre, por columna local y por tabla referenciada.
- El plan maestro prevalece explícitamente sobre helpers o inferencias locales cuando se definan contratos, campos de metadata o responsabilidades entre crates.
- `mssql-orm-macros` ya implementa un `#[derive(Entity)]` funcional sobre structs con campos nombrados, generando `EntityMetadata` estática, implementación del trait `Entity` y materialización `FromRow`.
- El derive soporta al menos los atributos base ya priorizados en la Etapa 1: `table`, `schema`, `primary_key`, `identity`, `length`, `nullable`, `default_sql`, `index` y `unique`.
- `mssql-orm-macros` ahora soporta `#[orm(foreign_key = "tabla.columna")]`, `#[orm(foreign_key = "schema.tabla.columna")]` y la sintaxis estructurada `#[orm(foreign_key(entity = Customer, column = id))]`.
- `mssql-orm-macros` ahora también soporta índices compuestos a nivel de entidad mediante `#[orm(index(name = "ix_...", columns(campo_a, campo_b)))]`, resolviendo esos campos hacia múltiples `IndexColumnMetadata` dentro de la metadata derivada.
- Sobre esos campos, el derive ya acepta además `#[orm(on_delete = "no action" | "cascade" | "set null")]`, generando `ForeignKeyMetadata` con `on_delete` configurable y `on_update = NoAction` en esta etapa.
- El derive valida en compile-time que `#[orm(on_delete = "set null")]` solo pueda usarse sobre columnas nullable.
- La sintaxis estructurada valida en compile-time la existencia de la columna de destino apoyándose en los símbolos generados por `#[derive(Entity)]` sobre la entidad referenciada, y no exige que esa columna sea primary key porque SQL Server también permite FKs hacia columnas no PK.
- El derive también cubre soporte directo para `column`, `sql_type`, `precision`, `scale`, `computed_sql` y `rowversion`, en línea con el shape de metadata ya definido en `core`.
- El derive también acepta ahora `#[orm(renamed_from = "...")]` sobre campos, dejando ese hint explícito disponible para el diff de migraciones sin inferencia automática de renombres.
- El derive también acepta ahora `#[orm(renamed_from = "...")]` a nivel de entidad, dejando un hint explícito para renombres de tabla dentro del mismo schema sin introducir inferencia automática de `RenameTable`.
- `examples/todo-app/` ya existe como crate aislada fuera del workspace principal; además de `TodoAppSettings`, `TodoAppState<Db>`, `build_app(...)`, `main.rs` y el perfil operativo base, ahora también define el dominio inicial del ejemplo en `src/domain.rs`.
- Ese dominio base del ejemplo ya cubre `todo.users`, `todo.todo_lists` y `todo.todo_items`, con relaciones `User -> TodoList`, `TodoList -> TodoItem` y referencias de auditoría `TodoItem -> User`.
- La crate del ejemplo reexporta `domain::User` como `TodoUser`, preservando nombres claros hacia el consumidor sin alterar la convención actual del derive para metadata.
- `examples/todo-app/src/db.rs` ya define `TodoAppDbContext` como contexto real del ejemplo, con `DbSet<User>`, `DbSet<TodoList>` y `DbSet<TodoItem>`.
- `examples/todo-app/src/queries.rs` ya define consultas reutilizables del ejemplo (`user_lists_page_query`, `list_items_page_query`, `open_items_preview_query`, `open_items_count_query`) mostrando uso real desde `db.todo_lists.query()...` y `db.todo_items.query()...`; los `SelectQuery` manuales quedaron reducidos a helpers internos de prueba.
- La cobertura del ejemplo ya fija AST y SQL compilado para filtros, ordenamientos, joins, paginación de página, preview con offset cero y conteo de ítems abiertos.
- `examples/todo-app/src/lib.rs` ya monta `GET /health` en `build_app(...)` y delega su ejecución a `DbContext::health_check()`, con respuestas HTTP mínimas `200 ok` y `503 database unavailable`.
- `examples/todo-app/src/http.rs` ya concentra los endpoints mínimos del ejemplo y su contrato de lectura (`TodoAppApi`), incluyendo DTOs serializables y handlers de lectura para listas e ítems.
- Esos handlers ya muestran uso real de `DbSet::find`, `DbSetQuery::all()` y `DbSetQuery::count()` desde el consumidor web del ejemplo.
- `examples/todo-app/src/lib.rs` ya expone `pool_builder_from_settings(...)`, `connect_pool(...)` y `state_from_pool(...)` como helpers explícitos del ejemplo para construir el contexto desde `MssqlPool`.
- `examples/todo-app/src/main.rs` ya usa `connect_pool(&settings).await?` y `TodoAppDbContext::from_pool(...)` cuando `pool-bb8` está activo; el fallback a `PendingTodoAppDbContext` quedó solo para builds sin ese feature.
- El dominio del ejemplo `todo_app` ya no necesita `impl FromRow` manuales: `#[derive(Entity)]` ahora materializa automáticamente `User`, `TodoList` y `TodoItem` desde filas, lo que también simplifica fixtures válidos de `trybuild` e integración pública.
- `examples/todo-app/scripts/smoke_setup.sql` ya deja un fixture operativo reproducible para validar el ejemplo contra SQL Server real en `tempdb`.
- `examples/todo-app/src/bin/model_snapshot.rs` exporta el `ModelSnapshot` real de `TodoAppDbContext`, y `examples/todo-app/scripts/migration_e2e.sh` valida la generación automática de migración inicial, migración incremental no-op y script `database_update.sql` en un directorio temporal. Las migraciones reales del ejemplo fueron aplicadas contra SQL Server local (`tempdb`) mediante `mssql-orm-cli database update --execute` usando `DATABASE_URL`, y una segunda ejecución validó el historial idempotente.
- La crate del ejemplo ya incluye una prueba ignorada `smoke_preview_query_runs_against_sql_server_fixture`, ejecutable con `DATABASE_URL`, para repetir parte del smoke real desde el propio consumidor.
- La generación automática de `FromRow` ya resuelve columnas nullable con la forma correcta `try_get_typed::<Option<T>>()?.flatten()` y mantiene `get_required_typed::<T>()` para campos no opcionales.
- `mssql-orm-tiberius` ahora soporta también `ColumnType::Intn` en `MssqlRow`, ampliando la lectura real de enteros SQL Server de anchura variable.
- La crate pública `mssql-orm` ahora también incluye un fixture `trybuild` específico del dominio `todo_app` que valida el uso público de `DbSetQuery` con `filter`, `order_by`, joins, `limit`, `take`, `paginate` y `count`.
- La validación del dominio dejó fijada una convención observable del macro: cuando se usa `#[orm(foreign_key(entity = Tipo, column = id))]`, el nombre generado del foreign key usa el nombre de tabla derivado del tipo Rust referenciado para el sufijo del constraint, aunque la tabla efectiva pueda estar sobrescrita con `#[orm(table = ...)]`.
- `mssql-orm-core` ya define `EntityColumn<E>` como símbolo estático de columna, y `#[derive(Entity)]` genera asociados como `Customer::email` para el query builder futuro.
- La crate pública `mssql-orm` ya contiene pruebas `trybuild` que cubren casos válidos de entidades con `foreign_key`, schema por defecto `dbo` para referencias abreviadas, la sintaxis estructurada y errores de compilación esperados para ausencia de PK, `identity` inválido, `rowversion` inválido, segmentos vacíos/formato inválido en `foreign_key` y columnas de destino inexistentes en el formato estructurado.
- La crate pública `mssql-orm` ahora también incluye una batería dedicada `stage9_relationship_metadata.rs` para fijar la metadata relacional generada por `#[derive(Entity)]`, incluyendo múltiples foreign keys, nombres generados y helpers de lookup sobre metadata.
- `mssql-orm-core` ya define `SqlValue`, `ColumnValue`, `Row`, `FromRow`, `Insertable<E>` y `Changeset<E>` como contratos base de mapping y persistencia.
- `mssql-orm-core` ya define `SqlTypeMapping` con implementaciones base para `bool`, `i32`, `i64`, `f64`, `String`, `Vec<u8>`, `Uuid`, `Decimal`, `NaiveDate`, `NaiveDateTime` y `Option<T>`, alineadas con las convenciones actuales del plan.
- `mssql-orm-macros` ya implementa `#[derive(Insertable)]` y `#[derive(Changeset)]` para structs con campos nombrados usando `#[orm(entity = MiEntidad)]`.
- `Insertable` soporta `#[orm(column = "...")]` por campo y produce `Vec<ColumnValue>` resolviendo el nombre final de columna contra la metadata de la entidad objetivo.
- `Changeset` exige `Option<T>` en el nivel externo de cada campo para mantener la semántica de omisión de cambios; esto permite también `Option<Option<T>>` para representar actualizaciones a `NULL`.
- La crate pública `mssql-orm` ya incluye una prueba de integración de Etapa 2 sobre un `Row` neutral, con cobertura de lectura tipada exitosa, ausencia de columna requerida, mismatch de tipo, `NULL` opcional y extracción de `ColumnValue` desde modelos `Insertable` y `Changeset`.
- `mssql-orm-query` ya dejó de ser un placeholder y ahora expone `Expr`, `Predicate`, `SelectQuery`, `CountQuery`, `InsertQuery`, `UpdateQuery`, `DeleteQuery`, `OrderBy`, `Pagination`, `TableRef`, `ColumnRef` y `CompiledQuery`.
- El AST de `mssql-orm-query` reutiliza `EntityColumn<E>` y metadata estática de `core` para construir referencias de tabla y columna sin generar SQL directo.
- `InsertQuery` y `UpdateQuery` ya se pueden construir desde `Insertable<E>` y `Changeset<E>`, conectando persistencia estructural con la futura compilación SQL Server.
- `mssql-orm-sqlserver` ya implementa quoting seguro de identificadores mediante `quote_identifier`, `quote_qualified_identifier`, `quote_table_ref` y `quote_column_ref`.
- El quoting actual usa corchetes SQL Server, escapa `]` como `]]` y rechaza identificadores vacíos, con caracteres de control o multipartes pasados como una sola cadena.
- `mssql-orm-sqlserver` ya implementa compilación de `select`, `insert`, `update`, `delete` y `count` a `CompiledQuery`, incluyendo placeholders `@P1..@Pn` y preservación de orden de parámetros.
- La compilación actual emite `OUTPUT INSERTED.*` para `insert` y `update`, usa `*` cuando `select` no tiene proyección explícita y exige `ORDER BY` antes de `OFFSET/FETCH`.
- `mssql-orm-sqlserver` ya cuenta con snapshots versionados para `select`, `insert`, `update`, `delete` y `count`, fijando el SQL generado y la secuencia observable de parámetros.
- La crate `mssql-orm-sqlserver` ahora usa `insta` solo como `dev-dependency` para congelar el contrato del compilador sin introducir dependencia runtime nueva.
- `mssql-orm-tiberius` ya integra la dependencia real `tiberius` y expone `MssqlConnectionConfig`, `MssqlConnection` y `TokioConnectionStream`.
- `MssqlConnectionConfig` ya parsea ADO connection strings mediante `tiberius::Config`, conserva el string original, rechaza entradas vacías o sin host usable y ahora también preserva `MssqlOperationalOptions` como contrato estable para preocupaciones de producción.
- `mssql-orm-tiberius` ahora expone además `MssqlOperationalOptions`, `MssqlTimeoutOptions`, `MssqlRetryOptions`, `MssqlTracingOptions`, `MssqlSlowQueryOptions`, `MssqlHealthCheckOptions` y `MssqlPoolOptions`, junto a enums auxiliares (`MssqlParameterLogMode`, `MssqlHealthCheckQuery`, `MssqlPoolBackend`) como surface explícita para las siguientes subtareas de Etapa 14.
- `mssql-orm-tiberius` ahora aplica `connect_timeout` al bootstrap del cliente y `query_timeout` a ejecución de queries y comandos transaccionales (`BEGIN`, `COMMIT`, `ROLLBACK`), manteniendo esa lógica estrictamente dentro del adaptador.
- `mssql-orm-tiberius` ahora también instrumenta conexión, queries y transacciones con `tracing`, usando spans `mssql_orm.connection`, `mssql_orm.query` y `mssql_orm.transaction`, y eventos estructurados para inicio/fin/error de queries, conexión y comandos transaccionales.
- La instrumentación actual registra `server_addr`, `operation`, `timeout_ms`, `param_count`, `sql`, `params_mode`, `params` y `duration_ms` como campos estables; los parámetros siguen redactados o deshabilitados por defecto y no se exponen valores sensibles.
- `MssqlConnection::connect` ya abre `TcpStream`, configura `TCP_NODELAY` e inicializa `tiberius::Client`, sin adelantar todavía ejecución de `CompiledQuery` ni mapeo de filas.
- `mssql-orm-tiberius` ya expone `ExecuteResult`, el trait `Executor` y los métodos `execute`/`query_raw` sobre `MssqlConnection<S>`.
- `mssql-orm-tiberius` ahora también expone `MssqlTransaction<'a, S>` y `MssqlConnection::begin_transaction()`, iniciando transacciones con `BEGIN TRANSACTION` y cerrándolas explícitamente mediante `commit()` o `rollback()`.
- La capa de ejecución del adaptador ahora comparte helpers internos entre conexión normal y transacción, por lo que `MssqlTransaction` también implementa `Executor` y puede reutilizar `execute`, `query_raw`, `fetch_one` y `fetch_all` sin duplicar binding ni mapeo.
- El adaptador ya prepara `CompiledQuery`, valida conteo de placeholders y realiza binding real de `SqlValue` hacia `tiberius::Query`.
- El binding de `Decimal` ya se resuelve a `tiberius::numeric::Numeric`; el caso `SqlValue::Null` sigue siendo provisional y hoy se envía como `Option::<String>::None`.
- `mssql-orm-tiberius` ya expone `MssqlRow<'a>` como wrapper sobre `tiberius::Row`, implementa el contrato neutral `Row` del core y convierte tipos soportados de SQL Server a `SqlValue`.
- El adaptador ya encapsula errores de Tiberius en `OrmError` mediante una capa interna de mapeo contextual, incluyendo lectura de filas, detección básica de deadlock y detalle del error original cuando falla la ejecución de una query.
- `MssqlConnection<S>` ya implementa `fetch_one<T: FromRow>` y `fetch_all<T: FromRow>` apoyándose en `query_raw`, `MssqlRow` y el contrato `FromRow` del core.
- `mssql-orm-tiberius` ya cuenta con pruebas de integración reales en `crates/mssql-orm-tiberius/tests/sqlserver_integration.rs`, activables mediante `MSSQL_ORM_TEST_CONNECTION_STRING`.
- Las pruebas reales usan tablas efímeras únicas en `tempdb.dbo` en lugar de `#temp tables`, porque la ejecución RPC usada por Tiberius no preserva tablas temporales locales entre llamadas separadas.
- La validación manual de esta sesión confirmó conectividad real con SQL Server local usando el login `sa`; la cadena original con `Database=test` no fue usable porque esa base no estaba accesible, así que la verificación se ejecutó contra `master`.
- La crate pública `mssql-orm` declara `extern crate self as mssql_orm` para que los macros puedan apuntar a una ruta estable tanto dentro del workspace como desde crates consumidoras.
- La crate pública `mssql-orm` ya expone `DbContext`, `DbSet`, `DbSetQuery`, `SharedConnection`, `connect_shared` y reexporta `tokio`, permitiendo que `#[derive(DbContext)]` genere métodos `connect`, `from_connection` y `from_shared_connection` sin depender de imports adicionales en el consumidor.
- La crate pública `mssql-orm` ahora también reexporta la surface operativa de producción y expone `connect_shared_with_options(...)` y `connect_shared_with_config(...)`, preservando compatibilidad con `connect_shared(...)`.
- `#[derive(DbContext)]` ahora genera también `connect_with_options(...)` y `connect_with_config(...)`, de modo que los consumidores puedan fijar configuración operativa sin abandonar la API derivada actual.
- `DbContext` ahora también expone `shared_connection()` y `transaction(...)`, y `#[derive(DbContext)]` genera el método inherente `db.transaction(|tx| async move { ... })` construyendo un contexto transaccional sobre la misma conexión compartida.
- La crate pública `mssql-orm` ahora también expone `DbContextEntitySet<E>`, y `#[derive(DbContext)]` implementa automáticamente ese trait para cada `DbSet<E>` del contexto, habilitando resolución tipada `DbContext -> DbSet<T>` para la futura capa Active Record.
- Como esa resolución sería ambigua con dos `DbSet` del mismo tipo de entidad en un mismo contexto, el derive `DbContext` ahora rechaza en compile-time contextos con múltiples `DbSet` para la misma entidad.
- La implementación pública actual abre `BEGIN TRANSACTION`, ejecuta el closure con un nuevo contexto del mismo tipo, hace `COMMIT` en `Ok` y `ROLLBACK` en `Err`, sin depender de `Drop` async.
- La crate pública `mssql-orm` ahora también expone el trait `EntityColumnPredicateExt` en su `prelude`, habilitando `eq`, `ne`, `gt`, `gte`, `lt`, `lte`, `is_null` e `is_not_null` directamente sobre `EntityColumn`.
- La misma extensión pública ahora también expone `contains`, `starts_with` y `ends_with`, reutilizando `Predicate::Like` con parámetros generados en la crate pública.
- La crate pública `mssql-orm` ahora también expone `EntityColumnOrderExt`, habilitando `asc()` y `desc()` directamente sobre `EntityColumn` y produciendo `OrderBy` del AST existente.
- `DbSet<T>` ya encapsula una conexión compartida sobre `tokio::sync::Mutex<MssqlConnection<_>>`, expone metadata de entidad y ahora también expone `query()` y `query_with(SelectQuery)` como base pública para ejecución de queries por entidad.
- `DbSet<T>` ahora también expone `find<K>()` para primary key simple, construyendo un `SelectQuery` filtrado desde la metadata de entidad y delegando la ejecución al runner base.
- `DbSet<T>` ahora también expone `insert<I>()`, compilando un `InsertQuery` desde `Insertable<E>` y materializando la entidad devuelta por `OUTPUT INSERTED.*`.
- `DbSet<T>` ahora también expone `update<K, C>() -> Result<Option<E>, OrmError>`, compilando un `UpdateQuery` desde `Changeset<E>` y materializando la fila actualizada cuando existe.
- `DbSet<T>` ahora también expone `delete<K>() -> Result<bool, OrmError>`, compilando un `DeleteQuery` por primary key simple y devolviendo si hubo al menos una fila afectada.
- La crate pública `mssql-orm` ahora también expone `PageRequest` como contrato estable de paginación explícita.
- La crate pública `mssql-orm` ahora también expone `PredicateCompositionExt`, habilitando `and`, `or` y `not` sobre `Predicate` sin introducir un DSL lógico paralelo.
- `DbSetQuery<T>` ya encapsula un `SelectQuery` y soporta `filter`, `order_by`, `limit`, `take`, `paginate`, `all`, `first` y `count`, reutilizando `SqlServerCompiler`, `fetch_one` y `fetch_all` sin mover ejecución ni generación SQL fuera de sus crates.
- La crate pública `mssql-orm` ahora también cuenta con una batería específica de pruebas públicas del query builder: una prueba de integración sobre la forma del AST y un caso `trybuild` que valida el encadenamiento desde código consumidor.
- La crate pública `mssql-orm` ahora también cuenta con snapshots del SQL generado desde el query builder público y con una prueba explícita de seguridad para confirmar que valores no confiables quedan parametrizados y no se interpolan en el SQL.
- `mssql-orm-migrate` ya dejó de ser solo un marker crate y ahora expone `ModelSnapshot`, `SchemaSnapshot`, `TableSnapshot`, `ColumnSnapshot`, `IndexSnapshot`, `IndexColumnSnapshot` y `ForeignKeySnapshot` como base del sistema de migraciones.
- El snapshot actual usa `String` y `Vec<_>` para ser persistible fuera de metadata estática, pero conserva el shape relevante de SQL Server (`SqlServerType`, `IdentityMetadata`, nullability, defaults, computed, rowversion, longitudes, precisión/escala, PK e índices).
- `TableSnapshot` conserva nombre de PK y columnas de PK además de columnas, índices y foreign keys, permitiendo que el pipeline de migraciones preserve ya la forma relacional relevante del modelo.
- `TableSnapshot` ahora también preserva `renamed_from` a nivel de tabla, habilitando renombres explícitos sin mezclar esa señal con creación/eliminación de tablas.
- `mssql-orm-migrate` ahora también implementa conversión directa desde metadata estática: `ColumnSnapshot: From<&ColumnMetadata>`, `IndexColumnSnapshot: From<&IndexColumnMetadata>`, `IndexSnapshot: From<&IndexMetadata>`, `ForeignKeySnapshot: From<&ForeignKeyMetadata>` y `TableSnapshot: From<&EntityMetadata>`.
- `ModelSnapshot::from_entities(&[&EntityMetadata])` ya agrupa entidades por schema usando orden determinista y ordena tablas por nombre dentro de cada schema, dejando una base estable para snapshots persistidos y futuros diffs.
- La conversión actual conserva el orden original de columnas, el nombre y columnas de primary key, los índices declarados y las foreign keys declaradas en metadata.
- `mssql-orm-migrate` ahora también expone `MigrationOperation` en un módulo separado, con payloads mínimos para `CreateSchema`, `DropSchema`, `CreateTable`, `DropTable`, `AddColumn`, `DropColumn`, `AlterColumn`, `CreateIndex`, `DropIndex`, `AddForeignKey` y `DropForeignKey`.
- Las operaciones de tabla reutilizan `TableSnapshot` completo y las de columna reutilizan `ColumnSnapshot`, evitando duplicar contratos antes de implementar el diff engine.
- Las operaciones relacionales nuevas reutilizan `IndexSnapshot` y `ForeignKeySnapshot`, de modo que el futuro DDL de Etapa 9 pueda compilarse sin volver a inferir shape desde metadata cruda.
- `MigrationOperation` ya expone helpers de lectura para `schema_name()` y `table_name()`, lo que simplifica ordenamiento y aserciones del futuro diff básico sin introducir aún generación SQL.
- `mssql-orm-migrate` ahora también expone `diff_schema_and_table_operations(previous, current)`, que compara `ModelSnapshot` y emite operaciones deterministas para `CreateSchema`, `CreateTable`, `DropTable` y `DropSchema`.
- El orden del diff actual es intencionalmente seguro para este alcance: primero crea schemas, luego tablas nuevas; después elimina tablas sobrantes y al final schemas vacíos/eliminados.
- El diff de schemas/tablas no intenta todavía detectar renombres ni cambios internos de columnas; esas responsabilidades quedan explícitamente para las siguientes subtareas de Etapa 7.
- El diff de schemas/tablas ahora puede emitir `RenameTable` cuando una tabla actual declara `renamed_from` y el nombre previo existe en el mismo schema; fuera de ese hint explícito sigue sin inferir renombres automáticamente.
- `mssql-orm-migrate` ahora también expone `diff_column_operations(previous, current)`, limitado a tablas compartidas entre ambos snapshots.
- El diff de columnas ya detecta `AddColumn`, `DropColumn` y `AlterColumn` comparando `ColumnSnapshot` completo y usando orden determinista por nombre de columna.
- Cuando cambia `computed_sql` o una columna pasa de regular a computada (o viceversa), el diff actual la modela como `DropColumn` seguido de `AddColumn`; `AlterColumn` sigue reservado a cambios básicos de tipo y nullability.
- Cuando una columna actual declara `renamed_from`, el diff puede emitir `RenameColumn` de forma explícita; si además cambia shape soportado, el rename se compone con `AlterColumn` posterior en lugar de degradar directamente a `DropColumn + AddColumn`.
- El diff de columnas ignora intencionalmente tablas nuevas o eliminadas, para no duplicar trabajo ya cubierto por `CreateTable`/`DropTable`; los renombres de tabla siguen como subtarea pendiente separada.
- El diff de columnas ya reutiliza tablas emparejadas por `RenameTable` explícito, de modo que un rename de tabla no rompa la detección posterior de `RenameColumn`, `AlterColumn`, índices o foreign keys sobre la misma entidad.
- `mssql-orm-migrate` ahora también expone `diff_relational_operations(previous, current)`, limitado a tablas compartidas entre ambos snapshots.
- El diff relacional detecta `CreateIndex`, `DropIndex`, `AddForeignKey` y `DropForeignKey`; cuando cambia la definición de un índice o de una foreign key existente, hoy la modela como `Drop...` seguido de `Create/Add...`.
- Ese contrato ya quedó cubierto también para foreign keys compuestas y para cambios de acciones referenciales (`NoAction`, `Cascade`, `SetNull`, `SetDefault`) en el pipeline de snapshots/diff.
- La cobertura del diff engine ya quedó centralizada en pruebas unitarias dedicadas dentro de `crates/mssql-orm-migrate/src/diff.rs`, en lugar de estar dispersa en `lib.rs`.
- Esa batería ya fija casos mínimos de orden seguro, no-op sobre snapshots iguales, altas/bajas de tablas, altas/bajas de columnas, alteraciones básicas y una composición completa de diff sobre snapshots mínimos.
- `lib.rs` quedó otra vez enfocado en reexports, boundaries y shape base de snapshots/operaciones, reduciendo ruido y duplicación en la capa pública de la crate.
- `mssql-orm-sqlserver` ahora compila `MigrationOperation` a DDL SQL Server mediante un módulo dedicado de migraciones, reutilizando `MigrationOperation` y `ColumnSnapshot`/`TableSnapshot` definidos en `mssql-orm-migrate`.
- La crate `mssql-orm-migrate` dejó de depender de `mssql-orm-sqlserver`; esa dependencia se invirtió para evitar un ciclo entre crates y respetar que la generación SQL pertenece a la capa SQL Server.
- La generación SQL actual cubre `CreateSchema`, `DropSchema`, `CreateTable`, `DropTable`, `AddColumn`, `DropColumn` y `AlterColumn`, además de la creación idempotente de `dbo.__mssql_orm_migrations`.
- La generación SQL actual también cubre `RenameColumn` mediante `sp_rename`, siempre que el diff lo reciba como operación explícita.
- La generación SQL actual también cubre `RenameTable` mediante `sp_rename ... 'OBJECT'` y sigue tratando el rename como operación explícita recibida desde el diff.
- `mssql-orm-sqlserver` ya compila `AddForeignKey` y `DropForeignKey` a DDL SQL Server básico usando `ALTER TABLE ... ADD/DROP CONSTRAINT`.
- `mssql-orm-sqlserver` ya compila foreign keys con `ON DELETE` y `ON UPDATE` para `NO ACTION`, `CASCADE`, `SET NULL` y `SET DEFAULT`.
- La compilación DDL de foreign keys ahora también valida cardinalidad mínima y que exista el mismo número de columnas locales y referenciadas antes de generar SQL.
- La surface pública actual sigue declarando foreign keys desde campos individuales; aunque snapshots/diff/DDL ya aceptan foreign keys compuestas, la sintaxis pública para derivarlas automáticamente no se amplió en esta sesión.
- `mssql-orm-sqlserver` ya compila `CreateIndex` y `DropIndex` a DDL SQL Server usando `CREATE [UNIQUE] INDEX ... ON ...` y `DROP INDEX ... ON ...`, preservando orden `ASC`/`DESC` desde el snapshot.
- `mssql-orm-sqlserver` ya compila columnas computadas en `CREATE TABLE` y `ALTER TABLE ... ADD [col] AS (...)`; los cambios sobre `computed_sql` siguen entrando por la estrategia de recreación del diff y no por `ALTER COLUMN`.
- `mssql-orm-query` ahora modela joins explícitos en `SelectQuery` mediante `JoinType`, `Join`, `join(...)`, `inner_join::<E>(...)` y `left_join::<E>(...)`, manteniendo el AST libre de SQL directo.
- `mssql-orm-sqlserver` ya compila joins explícitos a `INNER JOIN` y `LEFT JOIN` para el caso base sin aliases, preservando el orden de joins y de parámetros en el SQL parametrizado final.
- Mientras no exista aliasing en el AST, la compilación SQL Server rechaza explícitamente self-joins o joins repetidos sobre la misma tabla.
- La crate pública `mssql-orm` ya expone `DbSetQuery::join(...)`, `inner_join::<T>(...)` y `left_join::<T>(...)`, además de reexportar `Join` y `JoinType` desde la `prelude`.
- `mssql-orm-sqlserver` ahora también tiene snapshots dedicados para `SELECT` con joins y para DDL de foreign keys, y la crate pública `mssql-orm` cuenta con un snapshot adicional para joins compilados desde su query builder.
- Las operaciones de índices (`CreateIndex`, `DropIndex`) siguen rechazadas explícitamente en `mssql-orm-sqlserver`, porque su DDL todavía no forma parte del alcance activo.
- `AlterColumn` se limita intencionalmente a cambios básicos de tipo y nullability; defaults, computed columns, identity, PK y otros cambios que requieren operaciones dedicadas todavía retornan error explícito en esta etapa.
- `mssql-orm-migrate` ahora expone soporte mínimo de filesystem para migraciones: crear scaffolds, listar migraciones locales y construir un script SQL de `database update` a partir de `up.sql`.
- `mssql-orm-cli` ya implementa `migration add <Name>`, `migration list` y `database update`, delegando la lógica de scaffolding/listado/script al crate de migraciones y reutilizando la creación SQL de `__mssql_orm_migrations` desde `mssql-orm-sqlserver`.
- La CLI actual genera y lista migraciones locales y produce un script SQL acumulado para `database update`.
- `database update` divide `up.sql` en sentencias mínimas y ejecuta cada una mediante `EXEC(N'...')`, evitando el fallo detectado al validar migraciones reales con `CREATE SCHEMA` seguido de `CREATE TABLE`.
- Cada migración del script queda ahora encapsulada en un bloque idempotente con verificación de checksum, `BEGIN TRY/CATCH`, transacción explícita y `ROLLBACK` ante error; si el historial contiene el mismo `id` con checksum distinto, el script falla con `THROW` para no ocultar drift local.
- El script `database update` ahora también emite los `SET` de sesión requeridos por SQL Server para trabajar de forma fiable con índices sobre computed columns (`ANSI_NULLS`, `ANSI_PADDING`, `ANSI_WARNINGS`, `ARITHABORT`, `CONCAT_NULL_YIELDS_NULL`, `QUOTED_IDENTIFIER`, `NUMERIC_ROUNDABORT OFF`).
- Los ids de migración generados por `migration add` ahora usan resolución de nanosegundos para evitar colisiones y desorden léxico cuando se crean varias migraciones muy rápido en la misma sesión.
- La validación real ya se ejecutó contra SQL Server local (`tempdb`) usando `sqlcmd`: una migración inicial creó `qa_real_stage7.customers`, una migración incremental añadió `phone`, y la reaplicación del mismo script se mantuvo idempotente con exactamente dos filas en `dbo.__mssql_orm_migrations`.
- El artefacto temporal anterior `dbo.qa_1776961277_customers`, usado solo durante una validación intermedia, ya fue eliminado junto con sus filas de historial asociadas.
- La crate pública `mssql-orm` ya cuenta con una prueba de integración real en `crates/mssql-orm/tests/stage5_public_crud.rs` que valida `insert`, `find`, `query`, `update` y `delete` contra SQL Server.
- Esa prueba crea y limpia `dbo.mssql_orm_public_crud` dentro de la base activa del connection string y usa `MSSQL_ORM_TEST_CONNECTION_STRING` con skip limpio cuando no existe configuración.
- La misma prueba pública ahora acepta `KEEP_TEST_TABLES=1` para conservar `dbo.mssql_orm_public_crud` y facilitar inspección manual posterior en SQL Server.
- La misma prueba pública ahora también acepta `KEEP_TEST_ROWS=1` para conservar la tabla y dejar una fila final persistida, facilitando inspección manual con datos reales.
- La misma batería pública ahora también cubre `db.transaction(...)` contra SQL Server real, validando persistencia con `COMMIT` y reversión con `ROLLBACK`.
- El árbol actual de `examples/` expone de forma verificable `examples/todo-app/` como ejemplo ejecutable real; la guía índice quedó centralizada en `examples/README.md`.
- El worklog conserva referencias históricas a un ejemplo `basic-crud`, pero ese ejemplo no está presente en el árbol actual del repositorio y no debe seguir tratándose como artefacto disponible en la documentación vigente.
- `mssql-orm-sqlserver` ahora compila `CountQuery` con alias estable `AS [count]`, habilitando materialización consistente del conteo desde la crate pública.
- `mssql-orm-macros` ya implementa `#[derive(DbContext)]` para structs con campos `DbSet<Entidad>`, validando en compilación que el shape del contexto siga el contrato previsto.
- La crate pública `mssql-orm` ahora también expone `ActiveRecord`, implementado blanket sobre toda `Entity`; su superficie de Etapa 10 ya incluye `Entity::query(&db)`, `Entity::find(&db, id)`, `entity.delete(&db)` y `entity.save(&db)`, delegando estrictamente a `DbContextEntitySet<E>` y `DbSet<E>`.
- La cobertura de Active Record base ya quedó separada de la batería genérica: existe `tests/active_record_trybuild.rs` para contratos de compilación y `tests/stage10_public_active_record.rs` para roundtrip real de `query/find` contra SQL Server.
- Los fixtures `trybuild` de Active Record ya quedaron resintonizados con la toolchain actual: `DbContext` exige `FromRow` en los casos válidos con `DbSet<T>` y el caso `active_record_missing_entity_set` vuelve a aislar el error de `DbContextEntitySet<User>` ausente en lugar de fallar por precondiciones secundarias.
- `entity.delete(&db)` ya quedó implementado sobre Active Record reutilizando `DbSet::delete` a través de un helper oculto de PK simple generado por `#[derive(Entity)]`; para PK compuesta sigue retornando error explícito de etapa.
- `entity.save(&db)` ya quedó implementado sobre `&mut self` y sincroniza la instancia con la fila persistida devuelta por la base.
- `#[derive(Entity)]` ahora genera además contratos ocultos de persistencia para Active Record: valores insertables, cambios actualizables, sincronización desde la fila materializada y estrategia de persistencia basada en la PK simple.
- La estrategia actual de `save` es explícita y mínima: PK simple `identity` con valor `0` inserta y refresca la entidad; PK simple sin `identity` usa `find` por PK para decidir entre inserción y actualización; cualquier PK compuesta sigue rechazándose en esta etapa.
- `mssql-orm-core` ahora también expone `EntityMetadata::rowversion_column()` y `Changeset::concurrency_token()` para permitir que la concurrencia optimista se apoye en metadata y contracts ya presentes.
- `mssql-orm-core` ahora modela `OrmError` como enum estable con `Message(String)` y `ConcurrencyConflict`, manteniendo `OrmError::new(...)` como constructor de compatibilidad para errores simples y para mensajes dinámicos provenientes del driver.
- `#[derive(Changeset)]` ahora detecta campos mapeados a columnas `rowversion`: no los incluye en el `SET`, pero sí los usa como token de concurrencia para construir el `WHERE ... AND [version] = @Pn`.
- `DbSet::update(...)` ya soporta predicados de concurrencia optimista cuando el `Changeset` aporta token; si el token es viejo, la operación retorna `None` y no pisa datos silenciosamente.
- `DbSet::update(...)`, las rutas internas de borrado/update por `SqlValue` y Active Record ya elevan los mismatches reales de `rowversion` a `OrmError::ConcurrencyConflict` cuando la PK todavía existe.
- `ActiveRecord::save(&db)` y `entity.delete(&db)` también reutilizan `rowversion` cuando la entidad lo tiene y ahora propagan `OrmError::ConcurrencyConflict` en lugar de mensaje genérico o `false`.
- La `prelude` pública ya reexporta los derives `Entity`, `Insertable`, `Changeset` y `DbContext`, por lo que los tests de integración usan la misma superficie que usará un consumidor real.
- La operación del proyecto ahora exige realizar commit al cerrar una tarea completada y validada.
- El workflow `.github/workflows/ci.yml` es la automatización mínima vigente y replica las validaciones locales base del workspace.
- La arquitectura ya quedó documentada y respaldada por ADRs para SQL Server primero, separación estricta por crates y API pública concentrada en `mssql-orm`.
- La colaboración autónoma ya quedó formalizada en `docs/ai/`, por lo que las siguientes sesiones deben apoyarse en esa guía además de `docs/instructions.md`.

## Fuente de Verdad

- Plan maestro: `docs/plan_orm_sqlserver_tiberius_code_first.md`
- Operación del agente: `docs/instructions.md`
- Colaboración con IA: `docs/ai/`
- Trabajo pendiente: `docs/tasks.md`
- Historial de sesiones: `docs/worklog.md`
- Arquitectura y decisiones: `README.md`, `docs/architecture/overview.md`, `docs/adr/`

## Configuración Operativa Local

- Plantilla de connection string para validaciones reales e integraciones locales:
  `Server=localhost;Database=tempdb;User Id=<usuario>;Password=<password>;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;`
- Usarla como forma base para `MSSQL_ORM_TEST_CONNECTION_STRING` en pruebas reales y para `DATABASE_URL` en `examples/todo-app/`, reemplazando credenciales por valores propios del entorno local.
- Esta configuración debe mantenerse sin credenciales reales en la documentación; si cambia el entorno local, actualizar solo la plantilla o las notas operativas necesarias.

## Riesgos Inmediatos

- `SqlValue::Null` sigue siendo no tipado en el core, por lo que su binding actual en Tiberius es provisional y conviene revisarlo cuando exista suficiente contexto de tipo.
- La implementación actual de `db.transaction(...)` reutiliza la misma `SharedConnection`; por tanto, durante el closure debe asumirse uso lógico exclusivo de ese contexto/conexión y todavía no existe aislamiento adicional a nivel de pool o multiplexación.
- La surface de producción de Etapa 14 ya no es solo contractual: `connect_timeout`, `query_timeout`, `tracing`, `slow_query`, `health_check`, `retry`, `pool` y el wiring de `DbContext` desde pool ya alteran runtime del adaptador Tiberius y de la crate pública.
- `MssqlSlowQueryOptions` ya reutiliza exactamente la medición de duración de `trace_query(...)`: puede emitir `orm.query.slow` con `threshold_ms` y redacción configurable de parámetros, incluso si `MssqlTracingOptions.enabled` está apagado.
- `MssqlConnection::health_check()` y `DbContext::health_check()` ya ejecutan `SELECT 1 AS [health_check]` sobre la conexión activa, usando `health.timeout` cuando existe y fallback a `query_timeout` en caso contrario.
- `MssqlRetryOptions` ya se aplica solo a lecturas materializadas clasificadas como `select` (`fetch_one`, `fetch_all` y rutas públicas que dependen de ellas); no reintenta `execute`, `query_raw` ni operaciones dentro de `MssqlTransaction`.
- El pooling ya existe detrás del feature `pool-bb8` mediante `MssqlPool::builder()` y `MssqlPool::acquire() -> MssqlPooledConnection<'_>`; ahora ese ownership también puede encapsularse en `SharedConnection` para alimentar `DbContext`, pero la adquisición explícita desde `MssqlPool` sigue disponible para consumidores que no quieran pasar por la crate pública.
- `SharedConnection` ya no es un alias a `Arc<Mutex<MssqlConnection>>`; ahora es un wrapper público que puede representar conexión directa o pool, conservando el nombre/rol existente y permitiendo que `DbContext::from_shared_connection(...)` siga siendo el punto de entrada común para ambos casos.
- `#[derive(DbContext)]` ya expone `from_pool(pool)` bajo `pool-bb8`, mientras mantiene `from_connection(...)` y `connect*` para la ruta directa; la diferencia de ownership queda encapsulada en `SharedConnection`.
- La futura integración web async conviene construirla en varias subtareas testeables; el intento monolítico previo se revirtió para evitar dejar un ejemplo grande con cobertura insuficiente.
- `todo_app` debe entenderse como el ejemplo operativo realista que materializa la Etapa 14; sus relaciones, queries y wiring web forman parte del mismo objetivo, aunque convenga desarrollarlos en subtareas pequeñas y verificables.
- La validación real de `todo_app` ya quedó cerrada con fixture SQL reproducible, smoke HTTP manual y prueba ignorada de lectura contra `DATABASE_URL`; el riesgo inmediato ya no está en Etapa 14 sino en consolidar release/documentación pública de Etapa 15.
- El fixture SQL del ejemplo usa `NO ACTION` en `completed_by_user_id` en lugar de `SET NULL` para evitar `multiple cascade paths` en SQL Server dentro de un esquema de smoke compacto; esa diferencia está acotada al fixture operativo, no al dominio del ejemplo.
- La metadata relacional ya se genera automáticamente desde `#[orm(foreign_key = ...)]` y `#[orm(foreign_key(entity = ..., column = ...))]`, pero la validación compile-time actual de la variante estructurada depende del error nativo de símbolo inexistente cuando la columna referenciada no existe.
- La Etapa 9 quedó cubierta en metadata, DDL, joins y cobertura observable básica; la Etapa 10 también quedó cerrada con la surface completa de Active Record prevista para esta fase.
- La Etapa 11 quedó cerrada completamente: la infraestructura actual incorpora `rowversion` en update/delete/save y expresa los conflictos con un error público estable, sin mover compilación SQL fuera de `mssql-orm-sqlserver` ni ejecución fuera de `mssql-orm-tiberius`.
- La Etapa 12 ya no figura como tarea monolítica en el backlog; quedó dividida en entregables pequeños para evitar que una sola sesión mezcle modelado base, carga trackeada, wiring de contexto, persistencia y cobertura.
- La crate pública `mssql-orm` ahora expone `Tracked<T>` y `EntityState` como surface experimental mínima de tracking.
- `Tracked<T>` quedó definido como wrapper snapshot-based con `original`, `current` y `state`, y hoy expone constructores mínimos (`from_loaded`, `from_added`), accessors de lectura y acceso mutable observado (`current_mut`, `Deref`, `DerefMut`), además de `into_current()` por clon seguro del valor actual.
- La documentación del módulo de tracking deja explícitas las exclusiones vigentes de esta etapa: la surface sigue siendo experimental y no reemplaza la API explícita de `DbSet`/`ActiveRecord`.
- `DbSet::find_tracked(id)` ya está disponible para entidades con PK simple y reutiliza exactamente `find(...)` para cargar la fila y construir `Tracked::from_loaded(...)`.
- `DbSet::add_tracked(entity)` ya está disponible como entrada explícita para nuevas entidades en estado `Added`, registrándolas en el `TrackingRegistry` compartido sin saltarse la infraestructura CRUD existente.
- `DbSet::remove_tracked(&mut tracked)` ya está disponible como entrada explícita para marcar entidades trackeadas en estado `Deleted`; si el wrapper venía de `Added`, cancela la inserción pendiente sin emitir `DELETE` contra la base.
- El estado `Tracked<T>::state()` ya transiciona de `Unchanged` a `Modified` en cuanto se solicita acceso mutable a la entidad actual; en esta etapa no existe todavía diff estructural entre snapshots.
- `#[derive(DbContext)]` ahora crea un `TrackingRegistry` interno compartido por todos los `DbSet` del contexto derivado, y `find_tracked(...)` registra allí las entidades cargadas como base experimental para pasos posteriores.
- `#[derive(DbContext)]` ahora también genera `save_changes()`, que hoy persiste entidades trackeadas vivas en estado `Added`, `Modified` y `Deleted`, reutilizando `DbSet::insert`/`DbSet::update`/`DbSet::delete`.
- La base CRUD pública y el ejemplo ejecutable ya existen; el siguiente riesgo inmediato es introducir un query builder público que duplique o contradiga el AST y runner ya presentes.
- `find` todavía no soporta primary key compuesta; hoy falla explícitamente en ese caso y ese límite debe mantenerse documentado hasta que exista soporte dedicado.
- `update` tampoco soporta primary key compuesta en esta etapa y sigue retornando `Option<E>` para ausencia de fila, pero los mismatches detectados por `rowversion` ahora salen como `OrmError::ConcurrencyConflict`.
- `delete` tampoco soporta primary key compuesta en esta etapa y sigue retornando `bool` para ausencia de fila cuando no hay token de concurrencia; con `rowversion`, los mismatches también salen como `OrmError::ConcurrencyConflict`.
- `save` también queda limitado a PK simple; en PK con `identity` depende de la convención explícita `0 => insert`, y para PK natural simple usa una comprobación previa de existencia antes de decidir entre inserción o actualización.
- El futuro change tracking debe montarse sobre la infraestructura ya existente de `DbSet`, `save`, `delete`, `rowversion` y `ConcurrencyConflict`; no debe crear un segundo pipeline de persistencia.
- `Tracked<T>` y `save_changes` siguen siendo explícitamente experimentales y no deben reemplazar la API CRUD actual ni introducir reflexión/proxies tipo EF Core.
- El tracking ya observa acceso mutable local sobre el wrapper, mantiene referencias vivas a entidades trackeadas mientras el wrapper exista y `save_changes()` ya persiste `Added`, `Modified` y `Deleted`; sin embargo, al hacer `drop` del wrapper este deja de participar en la unidad de trabajo experimental.
- `save_changes()` actual cubre entidades `Added`, `Modified` y `Deleted`; el tracking sigue siendo explícito y no existe inferencia automática global de altas/bajas fuera del wrapper.
- `save_changes()` no persiste entidades `Unchanged`; si no hay wrappers vivos en estado pendiente, devuelve `0`.
- Si un wrapper trackeado se descarta antes de `save_changes()`, su registro interno se elimina y sus cambios dejan de participar en la persistencia experimental.
- Quitar una entidad que estaba en `Added` mediante `remove_tracked(...)` cancela la inserción pendiente localmente; no emite `DELETE` contra la base.
- El tracking experimental sigue limitado a entidades con primary key simple en las rutas que reutilizan `find`, `update`, `delete` o `save_changes()`.
- Las pruebas reales dependen de un connection string válido en `MSSQL_ORM_TEST_CONNECTION_STRING`; si apunta a una base inexistente, la validación falla antes de probar el adaptador.
- La validación real de Etapa 13 confirmó en SQL Server local la creación de computed columns, índices compuestos, foreign keys avanzadas y `RenameColumn`, además de la idempotencia por historial/checksum del script acumulado.
- Una validación real adicional confirmó también el comportamiento efectivo de las foreign keys sobre datos: `SET NULL`, `CASCADE`, `NO ACTION` y `SET DEFAULT` se observaron directamente en `tempdb`, no solo en metadata o DDL generado.
- `RenameTable` quedó validado localmente por cobertura unitaria, snapshots SQL y surface pública de macros; todavía no se hizo una corrida adicional contra SQL Server real específicamente para `sp_rename` de tablas porque la Etapa 13 ya contaba con validación real amplia sobre el pipeline de migraciones y esta subtarea no exigió infraestructura adicional.
- En SQL Server, `SET DEFAULT` sobre foreign keys requiere defaults válidos en las columnas locales; hoy esa precondición no se valida todavía de forma estructural antes de compilar el DDL.
- `crates/mssql-orm/tests/stage5_public_crud.rs` comparte nombres de tabla fijos entre tests; para evitar interferencia entre casos, su ejecución fiable sigue siendo serial (`-- --test-threads=1`) mientras no se aíslen los recursos por prueba.
- Si futuras sesiones empiezan a programar sin revisar `docs/`, se pierde trazabilidad.
- Como el repositorio raíz es nuevo, cualquier archivo ajeno al trabajo técnico debe revisarse antes de incluirlo en commits iniciales.

## Próximo Enfoque Recomendado

1. Ejecutar `Etapa 14: Validar el ejemplo web async todo_app contra SQL Server real con smoke test/documentación operativa reproducible`.
2. Solo después preparar la `Etapa 15` de release con documentación pública, quickstart, ejemplos completos y changelog.
3. Preservar el límite arquitectónico actual: `query` sigue sin generar SQL directo, `sqlserver` sigue siendo la única capa de compilación y `tiberius` la única capa de ejecución.

# Worklog

## 2026-04-23

### Sesión: descomposición de la Etapa 14 de producción

- Se volvió a tomar como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md`; la Etapa 14 del plan define explícitamente como entregables `pool opcional`, `timeouts`, `retry policy opcional`, `logging con tracing`, `slow query logs` y `health checks`, con `Definition of Done` ligada a uso en una API web async y ejemplo con Axum o Actix.
- Al contrastar ese alcance con `docs/tasks.md` se confirmó que la tarea única existente era demasiado grande y además omitía de forma explícita la `retry policy`, que sí aparece en el plan maestro.
- Se reemplazó esa entrada monolítica por subtareas ordenadas y verificables: definición de surface/configuración, timeouts, instrumentación con `tracing`, slow query logs, health checks, retry policy opcional, pooling opcional, wiring público de `DbContext` sobre pool y ejemplo web async final.
- La descomposición preserva dependencias técnicas: primero contratos y configuración, luego observabilidad y control de tiempo, después resiliencia/pooling y por último integración pública y ejemplo end-to-end.

### Resultado

- La Etapa 14 quedó preparada para ejecución incremental sin mezclar concerns de configuración, observabilidad, resiliencia, pooling y ejemplo web en una sola sesión.

### Validación

- No aplicó validación con `cargo`: en esta sesión solo se actualizó backlog y documentación operativa; no hubo cambios de código.

### Bloqueos

- No hubo bloqueos persistentes.
- La elección concreta de backend de pool y del framework web del ejemplo sigue pendiente para la subtarea correspondiente; esta sesión solo fijó el orden y el shape verificable del backlog.

### Próximo paso recomendado

- Tomar `Etapa 14: Definir surface y configuración operativa de producción para mssql-orm-tiberius y la crate pública`.

### Sesión: `RenameTable` explícito en snapshots, diff y DDL SQL Server

- Se volvió a tomar como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md`; la ruta pedida en la consigna original no existe en la raíz del repositorio.
- Se movió en `docs/tasks.md` la subtarea `Etapa 13: Soportar RenameTable explícito en snapshots, diff y DDL SQL Server` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `mssql-orm-core` ahora expone `EntityMetadata::renamed_from`, y `mssql-orm-macros` acepta `#[orm(renamed_from = "...")]` también a nivel de entidad para declarar renombres explícitos de tabla sin inferencia heurística.
- `mssql-orm-migrate` ahora preserva ese hint en `TableSnapshot`, incorpora `MigrationOperation::RenameTable` y hace que `diff_schema_and_table_operations` emita `RenameTable` cuando una tabla actual apunta explícitamente a un nombre previo dentro del mismo schema.
- El diff de columnas y el diff relacional ahora reutilizan esa misma correspondencia de tabla renombrada como contexto compartido, por lo que cambios posteriores de columnas, índices o foreign keys siguen comparándose contra la tabla previa correcta y no degradan el rename a `DropTable + CreateTable`.
- `mssql-orm-sqlserver` ahora compila `RenameTable` a `EXEC sp_rename ... 'OBJECT'`, y se añadieron cobertura unitaria y snapshot observable para ese SQL.
- La crate pública `mssql-orm` añadió un caso `trybuild` válido para fijar la nueva surface del derive con `#[orm(renamed_from = "...")]` a nivel de entidad.

### Resultado

- La Etapa 13 quedó cerrada también en renombres explícitos de tabla: metadata derivada, snapshot, diff y DDL SQL Server ya soportan `RenameTable` explícito dentro del mismo schema sin degradarlo a recreación destructiva de la tabla.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm-migrate --lib`
- `cargo test -p mssql-orm-sqlserver --lib migration`
- `cargo test -p mssql-orm-sqlserver --test migration_snapshots`
- `cargo test -p mssql-orm --test trybuild`

### Bloqueos

- No hubo bloqueos persistentes.
- El soporte actual de `RenameTable` es explícito y limitado a renombres dentro del mismo schema; mover tablas entre schemas sigue siendo responsabilidad de operaciones separadas (`CreateSchema`/`CreateTable`/`DropTable`) y no se infiere como rename.

### Próximo paso recomendado

- Empezar la Etapa 14 por `Implementar pooling opcional, timeouts, tracing, slow query logs y health checks`.

### Sesión: ampliación de validación real de Etapa 13 con foreign keys

- A pedido del usuario se amplió la validación real previa de Etapa 13 para no quedarse solo en la ejecución del script, sino revisar también el resultado efectivo dentro de SQL Server sobre datos reales.
- Se levantó un esquema temporal adicional `qa_stage13_fk_real_1776987291814399221` en `tempdb` con un escenario más completo:
  `customers` con PK compuesta y columna renombrada a `email_address`,
  `orders` con FK compuesta hacia `customers` (`NO ACTION` / `CASCADE` en update),
  `order_allocations` con computed column `line_total`, índice compuesto sobre esa computed column y FK compuesta hacia `customers` (`SET DEFAULT` / `CASCADE`),
  `order_notes` con FK a `orders` (`ON DELETE CASCADE`) y FK nullable a `users` (`ON DELETE SET NULL`).
- Se inspeccionó el resultado físico en catálogos de SQL Server (`sys.tables`, `sys.columns`, `sys.computed_columns`, `sys.indexes`, `sys.index_columns`, `sys.foreign_keys`) y se confirmó:
  existencia de las 5 tablas esperadas,
  rename efectivo de `email` a `email_address`,
  definición persistida de `line_total`,
  índice `ix_order_allocations_customer_line_total` con `customer_id ASC` y `line_total DESC`,
  foreign keys con acciones `SET_DEFAULT`, `SET_NULL`, `CASCADE` y `NO_ACTION` según lo esperado.
- Además se verificó comportamiento real sobre datos:
  al borrar `users.id = 10`, `order_notes.reviewer_id` pasó a `NULL` (`SET NULL`);
  al borrar `orders.id = 200`, la nota asociada se eliminó (`CASCADE`);
  el intento de borrar `customers.(1,1)` mientras seguía referenciado por `orders` falló como corresponde por la FK `NO ACTION`;
  tras eliminar primero `orders.id = 100`, borrar `customers.(1,1)` hizo que `order_allocations.(1000)` cambiara a `customer_id = 0, branch_id = 1` (`SET DEFAULT`);
  la computed column siguió materializando `45.00` tras el cambio de FK local, mostrando que el rename y las acciones referenciales no la degradaron.

### Resultado

- La validación real de Etapa 13 ya no cubre solo DDL y migración aplicada: también confirma semántica observable de foreign keys, rename de columna, computed columns e índices compuestos directamente sobre SQL Server.

### Validación

- Aplicación real de migraciones en `tempdb` con `mssql-orm-cli database update` y `sqlcmd`
- Consultas reales a catálogos `sys.*`
- Inserciones y borrados reales para observar `SET NULL`, `CASCADE`, `NO ACTION` y `SET DEFAULT`

### Bloqueos

- No hubo bloqueos persistentes.
- La validación mostró explícitamente la interacción entre FKs: una FK `NO ACTION` puede impedir observar `SET DEFAULT` en otra FK hasta liberar primero la referencia bloqueante, lo cual es comportamiento correcto de SQL Server.

### Próximo paso recomendado

- Implementar `Etapa 13: Soportar RenameTable explícito en snapshots, diff y DDL SQL Server`.

### Sesión: validación real de Etapa 13 contra SQL Server

- Se ejecutó una validación real de migraciones de Etapa 13 sobre SQL Server local (`tempdb`) usando `sqlcmd` y un proyecto temporal aislado fuera del repo.
- El escenario aplicado cubrió creación de schema, tabla con `computed column`, índice compuesto sobre esa computed column, foreign key compuesta con acciones referenciales avanzadas (`SET DEFAULT` / `CASCADE`) y una segunda migración con `RenameColumn` vía `sp_rename`.
- La primera corrida real expuso dos restricciones concretas de SQL Server que no estaban cubiertas todavía por la capa de script:
  `ON DELETE SET DEFAULT` exige defaults válidos en las columnas locales de la FK, por lo que el fixture temporal se corrigió para usar un caso relacional válido.
  La creación y uso de índices sobre computed columns exige ciertos `SET` de sesión (`QUOTED_IDENTIFIER`, `ANSI_NULLS`, etc.), y el script acumulado de `database update` no los emitía aún.
- Se corrigió `crates/mssql-orm-migrate/src/filesystem.rs` para que `database update` emita al inicio del script los `SET` requeridos por SQL Server (`ANSI_NULLS`, `ANSI_PADDING`, `ANSI_WARNINGS`, `ARITHABORT`, `CONCAT_NULL_YIELDS_NULL`, `QUOTED_IDENTIFIER`, `NUMERIC_ROUNDABORT OFF`).
- `crates/mssql-orm-cli/src/main.rs` actualizó su cobertura para fijar la presencia de esos `SET` en el SQL observable del comando `database update`.
- Tras el fix, la validación real confirmó:
  creación de `qa_stage13_real_1776986896364717782.customers` y `qa_stage13_real_1776986896364717782.order_allocations`,
  existencia de `line_total` como computed column con definición esperada,
  existencia de `ix_order_allocations_customer_line_total` con orden `customer_id ASC, line_total DESC`,
  existence de `fk_order_allocations_customer_branch_customers` con `DELETE = SET_DEFAULT` y `UPDATE = CASCADE`,
  rename efectivo de `email` a `email_address`,
  cálculo observable de `line_total = 45.00` tras insertar datos reales,
  reaplicación idempotente del mismo script con exactamente 2 filas en `dbo.__mssql_orm_migrations`,
  y fallo controlado por checksum mismatch (`THROW 50001`) al alterar localmente una migración ya aplicada.

### Resultado

- La Etapa 13 quedó validada contra SQL Server real en sus entregables ya implementados, y el generador de `database update` quedó endurecido para escenarios reales con índices sobre computed columns.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm-migrate --lib`
- `cargo test -p mssql-orm-cli`
- Ejecución real de `database update` contra `tempdb` con `sqlcmd`
- Consultas reales a `sys.tables`, `sys.columns`, `sys.computed_columns`, `sys.indexes`, `sys.index_columns`, `sys.foreign_keys` y `dbo.__mssql_orm_migrations`

### Bloqueos

- No hubo bloqueos persistentes.
- La validación real también dejó explícito que `SET DEFAULT` en foreign keys depende de defaults válidos en las columnas locales; hoy esa comprobación sigue siendo responsabilidad del SQL/fixture consumido y no una validación estructural previa del compilador.

### Próximo paso recomendado

- Implementar `Etapa 13: Soportar RenameTable explícito en snapshots, diff y DDL SQL Server`.

### Sesión: `RenameColumn` explícito con `#[orm(renamed_from = "...")]`

- Se volvió a tomar como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md`; la ruta pedida en la consigna original no existe en la raíz del repositorio.
- Al revisar el alcance real de renombres explícitos se confirmó que la subtarea original era demasiado grande para una sola sesión verificable, así que se descompuso operativamente en `RenameColumn` y `RenameTable` dentro de `docs/tasks.md` antes de continuar.
- Se movió en `docs/tasks.md` la nueva subtarea `Etapa 13: Soportar RenameColumn explícito con #[orm(renamed_from = \"...\")] en snapshots, diff y DDL SQL Server` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm-core/src/lib.rs` ahora incorpora `renamed_from` en `ColumnMetadata`, preservando el hint explícito de rename en la metadata derivada.
- `crates/mssql-orm-macros/src/lib.rs` ahora acepta `#[orm(renamed_from = \"old_name\")]` en campos de entidad y lo emite en la metadata pública generada por `#[derive(Entity)]`.
- `crates/mssql-orm-migrate/src/snapshot.rs` ahora preserva `renamed_from` en `ColumnSnapshot`, y `crates/mssql-orm-migrate/src/operation.rs`/`diff.rs` introducen `MigrationOperation::RenameColumn` con detección explícita basada en ese hint, sin inferir automáticamente que `drop + add` implique rename.
- El diff de columnas ahora emite `RenameColumn` cuando una columna actual apunta a un nombre previo mediante `renamed_from`; si además cambia shape soportado, emite `RenameColumn` seguido de `AlterColumn` en lugar de degradar el rename a `DropColumn + AddColumn`.
- `crates/mssql-orm-sqlserver/src/migration.rs` ahora compila `RenameColumn` a `EXEC sp_rename ... 'COLUMN'`, y `crates/mssql-orm-sqlserver/tests/migration_snapshots.rs` junto al snapshot `migration_snapshots__rename_column_migration_sql.snap` congelan ese SQL observable.
- `crates/mssql-orm/tests/trybuild.rs` y `crates/mssql-orm/tests/ui/entity_renamed_from_valid.rs` fijan la nueva surface pública del derive para consumidores reales.

### Resultado

- La mitad acotada de la subtarea de renombres quedó cerrada: el sistema ya soporta `RenameColumn` explícito de extremo a extremo en metadata derivada, snapshots, diff y DDL SQL Server, sin introducir inferencia riesgosa de renombres.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p mssql-orm-migrate --lib`
- `cargo test -p mssql-orm-sqlserver --lib migration`
- `cargo test -p mssql-orm-sqlserver --test migration_snapshots`
- `cargo test -p mssql-orm --test trybuild`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos persistentes.
- `RenameTable` sigue pendiente como subtarea separada; esta sesión no introdujo metadata ni diff explícito para renombres de tabla.

### Próximo paso recomendado

- Implementar `Etapa 13: Soportar RenameTable explícito en snapshots, diff y DDL SQL Server`.

### Sesión: scripts de migración idempotentes para SQL Server

- Se volvió a tomar como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md`; la ruta pedida en la consigna original no existe en la raíz del repositorio.
- Se movió en `docs/tasks.md` la subtarea `Etapa 13: Generar scripts de migración idempotentes para SQL Server` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm-migrate/src/filesystem.rs` ahora genera para cada migración un bloque idempotente más robusto: verifica checksum previo en `dbo.__mssql_orm_migrations`, falla con `THROW 50001` si detecta drift entre historial y contenido local, y ejecuta la migración dentro de `BEGIN TRY / BEGIN TRANSACTION / COMMIT` con `ROLLBACK` en `CATCH`.
- La misma capa mantiene la división de `up.sql` en sentencias mínimas mediante `EXEC(N'...')`, pero ahora evita emitir bloques `EXEC` vacíos cuando una migración solo contiene comentarios o whitespace.
- `crates/mssql-orm-cli/src/main.rs` actualizó su cobertura para fijar el nuevo contrato observable del comando `database update`, incluyendo checksum mismatch y transacción explícita por migración.

### Resultado

- La subtarea quedó cerrada: `database update` ahora produce scripts reejecutables más seguros para SQL Server, con salto por historial, verificación de checksum para evitar reaplicar migraciones alteradas y rollback explícito ante fallos parciales dentro de una migración.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm-migrate --lib`
- `cargo test -p mssql-orm-cli`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos persistentes.
- Esta sesión no implementó todavía `migration script --from --to` ni guards idempotentes por operación DDL individual; la robustez se concentra en el bloque por migración y en el historial/checksum.

### Próximo paso recomendado

- Implementar `Etapa 13: Soportar renombres explícitos de tablas y columnas sin degradar a drop + add`.

### Sesión: foreign keys avanzadas en snapshots, diff y DDL SQL Server

- Se volvió a tomar como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md`; la ruta pedida en la consigna original no existe en la raíz del repositorio.
- Se movió en `docs/tasks.md` la subtarea `Etapa 13: Completar foreign keys avanzadas en snapshots, diff y DDL SQL Server` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm-migrate/src/lib.rs` ahora fija mediante pruebas que `TableSnapshot::from(&EntityMetadata)` preserva foreign keys compuestas, múltiples columnas referenciadas y acciones referenciales distintas de `NoAction`.
- `crates/mssql-orm-migrate/src/diff.rs` añadió cobertura explícita para recrear foreign keys compuestas cuando cambia su definición, incluyendo cambios de acciones referenciales.
- `crates/mssql-orm-sqlserver/src/migration.rs` ahora compila `ReferentialAction::SetDefault` a `SET DEFAULT` en DDL SQL Server y valida que toda foreign key tenga al menos una columna local, al menos una columna referenciada y la misma cardinalidad en ambos lados.
- La misma capa SQL ahora tiene cobertura unitaria para foreign keys compuestas con `SET DEFAULT` y para el rechazo de definiciones con cardinalidad inválida.
- `crates/mssql-orm-sqlserver/tests/migration_snapshots.rs` y el snapshot `migration_snapshots__advanced_foreign_key_migration_sql.snap` ahora congelan el SQL observable de una foreign key compuesta con acciones referenciales avanzadas.

### Resultado

- La subtarea quedó cerrada para el pipeline de migraciones: snapshots, diff relacional y DDL SQL Server ya soportan foreign keys compuestas y `SET DEFAULT`, con validaciones explícitas del shape relacional.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p mssql-orm-migrate --lib`
- `cargo test -p mssql-orm-sqlserver --lib migration`
- `cargo test -p mssql-orm-sqlserver --test migration_snapshots`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos persistentes.
- La surface pública de `#[derive(Entity)]` sigue limitada a foreign keys declaradas por campo; esta sesión no introdujo sintaxis pública nueva para declarar foreign keys compuestas desde macros.

### Próximo paso recomendado

- Implementar `Etapa 13: Generar scripts de migración idempotentes para SQL Server`.

### Sesión: computed columns en snapshots, diff y DDL SQL Server

- Se tomó como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md`; la ruta pedida en la consigna (`plan_orm_sqlserver_tiberius_code_first.md`) no existe en la raíz del repositorio y se dejó esta constancia para evitar ambigüedad operativa.
- Se movió en `docs/tasks.md` la subtarea `Etapa 13: Soportar computed columns en snapshots, diff y DDL SQL Server` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm-migrate/src/diff.rs` ahora trata cualquier cambio en `computed_sql` como reemplazo estructural de la columna (`DropColumn` + `AddColumn`) en lugar de degradarlo a `AlterColumn`, preservando el límite actual de SQL Server para alteraciones simples y evitando prometer un `ALTER COLUMN` que la compilación no soporta en esta etapa.
- La misma batería de diff ahora cubre dos casos explícitos: cambio de expresión computada y transición entre columna regular y columna computada, fijando orden determinista de operaciones.
- `crates/mssql-orm-sqlserver/src/migration.rs` añadió cobertura unitaria para columnas computadas tanto en `CREATE TABLE` como en `ALTER TABLE ... ADD`, y mantiene el rechazo explícito de `AlterColumn` para cambios de `computed_sql`.
- `crates/mssql-orm-sqlserver/tests/migration_snapshots.rs` y el snapshot `migration_snapshots__computed_column_migration_sql.snap` ahora congelan el SQL observable para añadir y eliminar una columna computada mediante migraciones.

### Resultado

- La subtarea de `computed columns` quedó cerrada para el alcance activo: el snapshot ya preservaba `computed_sql`, el diff ahora genera operaciones ejecutables para cambios sobre columnas computadas y la capa SQL Server tiene cobertura observable para su DDL.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p mssql-orm-migrate --lib`
- `cargo test -p mssql-orm-sqlserver --lib migration`
- `cargo test -p mssql-orm-sqlserver --test migration_snapshots`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos persistentes.
- La estrategia actual para cambios de `computed_sql` es `drop + add`; todavía no existe soporte de renombre ni preservación de dependencias alrededor de columnas computadas complejas.

### Próximo paso recomendado

- Implementar `Etapa 13: Completar foreign keys avanzadas en snapshots, diff y DDL SQL Server`.

### Sesión: índices compuestos en metadata derivada, snapshots y diff

- Se volvió a tomar como fuente de verdad el plan maestro real en `docs/plan_orm_sqlserver_tiberius_code_first.md`, tomando como subtarea activa `Etapa 13: Soportar índices compuestos en snapshots, diff y DDL SQL Server`.
- Se movió en `docs/tasks.md` esa subtarea a `En Progreso` antes de editar y luego a `Completadas` tras validarla; además se corrigió el estado operativo de la tarea ya ejecutada de descomposición de Etapa 13.
- `crates/mssql-orm-macros/src/lib.rs` ahora soporta índices compuestos a nivel de entidad mediante `#[orm(index(name = \"ix_...\", columns(campo_a, campo_b)))]`, resolviendo esos campos hacia columnas reales de metadata y manteniendo intacto el soporte previo de índices simples por campo.
- La generación de metadata ahora produce `IndexMetadata` con múltiples `IndexColumnMetadata` cuando se usa esa sintaxis, dejando que snapshots y DDL reutilicen el mismo shape ya existente sin abrir un sistema paralelo.
- `crates/mssql-orm-migrate/src/diff.rs` ahora recrea índices cuando cambia su definición manteniendo el mismo nombre, en lugar de comparar solo presencia/ausencia; esto cierra el hueco real para índices compuestos en el diff relacional.
- `crates/mssql-orm-migrate/src/lib.rs` añadió cobertura unitaria para confirmar que `TableSnapshot::from(&EntityMetadata)` preserva índices compuestos y su orden/dirección.
- `crates/mssql-orm/src/lib.rs` y `crates/mssql-orm/tests/trybuild.rs` ahora fijan públicamente la nueva surface con un caso real de derive válido y aserciones sobre metadata compuesta.
- No fue necesario cambiar la compilación DDL de `mssql-orm-sqlserver`: ya soportaba múltiples `IndexColumnSnapshot`; la sesión añadió cobertura suficiente para congelar ese contrato en combinación con la nueva metadata derivada.

### Resultado

- La Etapa 13 ya soporta índices compuestos de extremo a extremo: metadata derivada, snapshot, diff relacional y compilación SQL Server.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p mssql-orm --lib`
- `cargo test -p mssql-orm --test trybuild`
- `cargo test -p mssql-orm-migrate --lib`
- `cargo test -p mssql-orm-sqlserver --lib`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- La nueva sintaxis de índices compuestos se limita por ahora a columnas en orden ascendente desde metadata derivada; la infraestructura de snapshot/DDL ya soporta `DESC`, pero esa configuración fina no se expuso todavía en atributos públicos en esta subtarea.

### Próximo paso recomendado

- Implementar `Etapa 13: Soportar computed columns en snapshots, diff y DDL SQL Server`.

### Sesión: descomposición de la Etapa 13 de migraciones avanzadas

- Se revisó nuevamente el backlog operativo en `docs/tasks.md` y se confirmó que la tarea amplia `Etapa 13: Soportar migraciones avanzadas: renombres, computed columns, FKs completas, índices compuestos y scripts idempotentes` era demasiado grande para una sola sesión sin mezclar varias capas del sistema de migraciones.
- Se reemplazó esa tarea amplia por subtareas verificables y ordenadas: descomposición operativa, índices compuestos, `computed columns`, foreign keys avanzadas, scripts idempotentes y renombres explícitos.
- El orden elegido prioriza cambios con menor ambigüedad primero y deja renombres al final, porque sin metadata explícita de rename el diff puede degradar fácilmente a `drop + add`, con mayor riesgo sobre el esquema.
- No se modificó código del workspace en esta sesión; el alcance fue exclusivamente de backlog y trazabilidad para preparar la entrada a la Etapa 13.

### Resultado

- La Etapa 13 quedó preparada para ejecución incremental, con subtareas suficientemente pequeñas como para implementarse con validación clara y menor riesgo arquitectónico.

### Validación

- No aplicó validación con `cargo`: en esta sesión solo se actualizó backlog y documentación operativa; no hubo cambios de código.

### Bloqueos

- No hubo bloqueos técnicos.
- La principal sensibilidad sigue siendo el diseño de renombres: debe resolverse con metadata/señal explícita y no con inferencia riesgosa desde el diff.

### Próximo paso recomendado

- Tomar `Etapa 13: Soportar índices compuestos en snapshots, diff y DDL SQL Server` como primera subtarea de implementación.

### Sesión: cierre de cobertura y límites del change tracking experimental

- Se volvió a tomar como fuente de verdad el plan maestro real en `docs/plan_orm_sqlserver_tiberius_code_first.md`, acotando la sesión a cerrar la última subtarea de Etapa 12 sin adelantar trabajo de Etapa 13.
- Se movió en `docs/tasks.md` la subtarea `Etapa 12: Agregar pruebas unitarias, integración y documentación de límites para la API experimental de change tracking` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm/src/tracking.rs` ahora documenta explícitamente la surface experimental vigente, sus entry points (`find_tracked`, `add_tracked`, `remove_tracked`, `save_changes`) y límites observables: wrappers vivos únicamente, ausencia de diff estructural, cancelación local de `Added` removidos, límite de PK simple y preservación de `ConcurrencyConflict`.
- `crates/mssql-orm/tests/stage5_public_crud.rs` añadió cobertura de integración real para dos contratos de límite que faltaban fijar: `save_changes()` devuelve `0` sobre entidades `Unchanged`, y wrappers descartados antes de guardar quedan fuera del unit of work experimental.
- `docs/context.md` ahora registra esos límites operativos de forma explícita para futuras sesiones: no-op sobre `Unchanged`, exclusión de wrappers descartados, cancelación local de `Added` eliminados antes de persistirse y alcance actual de PK simple.

### Resultado

- La Etapa 12 quedó cerrada completa: la API experimental de tracking ya tiene cobertura unitaria/integración suficiente para su alcance actual y deja documentados sus límites observables sin ambigüedad.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p mssql-orm --lib`
- `cargo test -p mssql-orm --test trybuild`
- `cargo test -p mssql-orm --test stage5_public_crud -- --test-threads=1`
- `cargo check --workspace`
- `cargo clippy -p mssql-orm --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- La API sigue siendo deliberadamente experimental; el cierre de Etapa 12 no cambia los límites ya explícitos sobre PK simple ni introduce tracking automático global.

### Próximo paso recomendado

- Iniciar `Etapa 13: Soportar migraciones avanzadas: renombres, computed columns, FKs completas, índices compuestos y scripts idempotentes`.

### Sesión: soporte experimental de `Deleted` con `remove_tracked(...)`

- Se volvió a tomar como fuente de verdad el plan maestro real en `docs/plan_orm_sqlserver_tiberius_code_first.md`, manteniendo esta sesión acotada a la subtarea de Etapa 12 para entidades `Deleted`.
- Se movió en `docs/tasks.md` la subtarea `Etapa 12: Soportar estado Deleted con remove(tracked) o equivalente explícito y persistencia vía delete` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm/src/context.rs` ahora expone `DbSet::remove_tracked(&mut Tracked<E>)`, que marca wrappers cargados como `Deleted` y, si el wrapper estaba en `Added`, cancela la inserción pendiente desregistrándolo del `TrackingRegistry`.
- El mismo módulo ahora implementa `DbSet::save_tracked_deleted()`, reutilizando la ruta existente de `delete` por PK simple y preservando `rowversion`/`OrmError::ConcurrencyConflict` mediante un helper interno específico para borrado trackeado.
- Tras un borrado exitoso, la entidad se desregistra del `TrackingRegistry` para evitar reintentos en `save_changes()` posteriores, manteniendo el wrapper vivo en estado observable `Deleted`.
- `crates/mssql-orm/src/tracking.rs` ahora conserva el `registration_id` en la vista interna `RegisteredTracked`, y añade helpers mínimos para `mark_deleted()` y `detach_registry()` sin cambiar la surface pública principal.
- `crates/mssql-orm-macros/src/lib.rs` ahora hace que `#[derive(DbContext)]` genere `save_changes()` en tres fases: `Added`, `Modified` y `Deleted`, siempre reutilizando la infraestructura CRUD ya cerrada.
- Se añadieron pruebas unitarias nuevas en `tracking.rs` y `context.rs` para fijar marcado a `Deleted`, cancelación de `Added` y desregistro explícito.
- `crates/mssql-orm/tests/stage5_public_crud.rs` ahora cubre borrado trackeado exitoso, cancelación de un `Added` removido antes de persistirse y conflicto real de `rowversion` durante `save_changes()` de una entidad `Deleted`.

### Resultado

- La Etapa 12 ya permite marcar entidades trackeadas para borrado mediante `DbSet::remove_tracked(...)` y persistirlas con `db.save_changes().await?`, sin duplicar la semántica de `delete` ni degradar la concurrencia optimista ya cerrada.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p mssql-orm --lib`
- `cargo test -p mssql-orm --test trybuild`
- `cargo test -p mssql-orm --test stage5_public_crud -- --test-threads=1`
- `cargo check --workspace`
- `cargo clippy -p mssql-orm --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- El borrado trackeado sigue limitado a entidades con PK simple, igual que la infraestructura CRUD subyacente; ese límite se preserva explícitamente en esta etapa.

### Próximo paso recomendado

- Implementar `Etapa 12: Agregar pruebas unitarias, integración y documentación de límites para la API experimental de change tracking`.

### Sesión: soporte experimental de `Added` con `add_tracked(...)`

- Se volvió a tomar como fuente de verdad el plan maestro real en `docs/plan_orm_sqlserver_tiberius_code_first.md`, manteniendo esta sesión acotada a la subtarea de Etapa 12 para entidades `Added`, sin adelantar todavía soporte de `Deleted`.
- Se movió en `docs/tasks.md` la subtarea `Etapa 12: Soportar estado Added con add(tracked) o equivalente explícito y persistencia vía insert` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm/src/context.rs` ahora expone `DbSet::add_tracked(entity)`, que construye `Tracked::from_added(...)`, lo registra en el `TrackingRegistry` compartido del contexto y deja explícita la entrada de nuevas entidades al pipeline experimental.
- El mismo módulo ahora implementa `DbSet::save_tracked_added()` reutilizando `insert_entity(...)`; al persistir correctamente, sincroniza el wrapper vivo con la fila materializada devuelta por SQL Server y lo deja en estado `Unchanged`.
- `crates/mssql-orm-macros/src/lib.rs` ahora hace que `#[derive(DbContext)]` genere `save_changes()` en dos fases por `DbSet`: primero persiste entidades `Added` y luego `Modified`, preservando la reutilización de la infraestructura CRUD ya existente.
- `crates/mssql-orm/src/tracking.rs` añadió cobertura unitaria para fijar que el registro interno expone entidades `Added` con el estado observable correcto.
- `crates/mssql-orm/tests/stage5_public_crud.rs` añadió una integración pública real que verifica `add_tracked(...)`, persistencia vía `db.save_changes().await?`, refresco de identidad y transición `Added -> Unchanged`.

### Resultado

- La Etapa 12 ya permite registrar nuevas entidades mediante `DbSet::add_tracked(...)` y persistirlas con `db.save_changes().await?`, reutilizando `insert` y manteniendo el wrapper sincronizado con la fila devuelta por la base.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p mssql-orm --lib`
- `cargo test -p mssql-orm --test trybuild`
- `cargo test -p mssql-orm --test stage5_public_crud -- --test-threads=1`
- `cargo check --workspace`
- `cargo clippy -p mssql-orm --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- El tracking experimental sigue dependiendo de que el wrapper `Tracked<T>` permanezca vivo; si se hace `drop` antes de `save_changes()`, la entidad `Added` se desregistra y deja de participar en la persistencia, igual que las `Modified`.

### Próximo paso recomendado

- Implementar `Etapa 12: Soportar estado Deleted con remove(tracked) o equivalente explícito y persistencia vía delete`.

### Sesión: `save_changes()` experimental para entidades `Modified`

- Se mantuvo como fuente de verdad el plan maestro real en `docs/plan_orm_sqlserver_tiberius_code_first.md`, acotando esta sesión a `save_changes()` solo para entidades `Modified`, sin adelantar todavía soporte de `Added` o `Deleted`.
- Se movió en `docs/tasks.md` la subtarea `Etapa 12: Implementar save_changes() para entidades Modified, reutilizando DbSet::update y preservando rowversion/ConcurrencyConflict` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm/src/tracking.rs` dejó de registrar solo metadata estática y ahora mantiene referencias estables a wrappers `Tracked<T>` vivos mediante almacenamiento heap-stable; además limpia automáticamente sus entradas del registro al hacer `drop` del wrapper.
- `Tracked<T>` preserva la surface observable ya fijada (`original`, `current`, `state`, `current_mut`, `Deref`, `DerefMut`), pero ahora `into_current()` devuelve un clon del valor actual para evitar romper seguridad al combinar `Drop` con el registro interno.
- `crates/mssql-orm/src/context.rs` ahora implementa `DbSet::save_tracked_modified()` como primitive interna que recorre las entidades trackeadas vivas del tipo correspondiente, filtra las que están en `Modified`, ejecuta `update` reutilizando la infraestructura existente y sincroniza el snapshot del wrapper a `Unchanged` cuando la persistencia tiene éxito.
- `crates/mssql-orm-macros/src/lib.rs` ahora genera `save_changes()` en `#[derive(DbContext)]`, sumando los resultados de cada `DbSet` derivado y devolviendo la cantidad total de entidades `Modified` persistidas.
- La semántica de concurrencia se preservó: si una entidad trackeada con `rowversion` queda stale, `save_changes()` propaga `OrmError::ConcurrencyConflict` y deja el wrapper en estado `Modified`, sin sobreescribir el snapshot local.
- Se añadieron integraciones nuevas en `crates/mssql-orm/tests/stage5_public_crud.rs` para cubrir `save_changes()` exitoso sobre una entidad trackeada y el conflicto real de `rowversion` al guardar un wrapper stale.
- Se ajustaron fixtures de compilación válidos (`dbcontext_valid.rs`, `query_builder_public_valid.rs`) para que las entidades de prueba implementen `FromRow`, porque `#[derive(DbContext)]` ahora expone también `save_changes()` sobre la crate pública.

### Resultado

- La Etapa 12 ya permite persistir entidades `Modified` cargadas vía `find_tracked(...)` usando `db.save_changes().await?`, manteniendo `rowversion` y `ConcurrencyConflict` alineados con la infraestructura ya cerrada en la Etapa 11.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p mssql-orm --lib`
- `cargo check --workspace`
- `cargo clippy -p mssql-orm --all-targets --all-features -- -D warnings`
- `cargo test -p mssql-orm --test trybuild`
- `MSSQL_ORM_TEST_CONNECTION_STRING='Server=localhost;Database=tempdb;User Id=SA;Password=Ea.930318;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;' cargo test -p mssql-orm --test stage5_public_crud -- --test-threads=1`

### Bloqueos

- No hubo bloqueos persistentes.
- `save_changes()` actual solo opera sobre wrappers `Tracked<T>` que siguen vivos; si un wrapper se descarta, su entrada se elimina del registro y deja de participar en la persistencia experimental, lo cual es consistente con el diseño actual pero debe mantenerse explícito mientras no exista una unidad de trabajo más rica.

### Próximo paso recomendado

- Implementar `Etapa 12: Soportar estado Added con add(tracked) o equivalente explícito y persistencia vía insert`.

### Sesión: colección interna mínima de entidades trackeadas en `DbContext`

- Se mantuvo como fuente de verdad el plan maestro real en `docs/plan_orm_sqlserver_tiberius_code_first.md` y se acotó la subtarea a introducir una colección interna compartida, sin adelantar todavía `save_changes()`, `add` o `remove`.
- Se movió en `docs/tasks.md` la subtarea `Etapa 12: Introducir colección interna mínima de entidades trackeadas dentro de DbContext experimental sin romper la API explícita existente` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm/src/tracking.rs` ahora define la infraestructura oculta `TrackingRegistry`, `TrackingRegistryHandle` y `TrackedEntityRegistration`, con una colección protegida por `Mutex` para registrar entidades cargadas experimentalmente.
- `crates/mssql-orm/src/context.rs` ahora hace que cada `DbSet` mantenga un `TrackingRegistryHandle`; `DbSet::find_tracked(...)` registra automáticamente las entidades cargadas en ese registro interno además de devolver `Tracked<E>`.
- La trait `DbContext` ahora expone el método oculto `tracking_registry()`, y `#[derive(DbContext)]` en `crates/mssql-orm-macros/src/lib.rs` construye un único registro compartido por todos los `DbSet` del contexto derivado mediante `DbSet::with_tracking_registry(...)`.
- La colección añadida en esta sesión es deliberadamente mínima: registra la carga de entidades por tipo y estado inicial, pero todavía no sincroniza mutaciones vivas del wrapper con el registro ni persiste cambios.
- Se añadieron pruebas unitarias del registro en `tracking.rs` y una integración pública nueva en `crates/mssql-orm/tests/stage5_public_crud.rs` que valida que dos `DbSet` distintos dentro del mismo `DbContext` derivado comparten el mismo registro y acumulan entradas al usar `find_tracked(...)`.

### Resultado

- La Etapa 12 ya cuenta con una colección interna común a nivel de `DbContext` derivado, suficiente como base experimental para montar `save_changes()` sobre entidades `Modified`.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p mssql-orm --lib`
- `cargo check --workspace`
- `cargo clippy -p mssql-orm --all-targets --all-features -- -D warnings`
- `MSSQL_ORM_TEST_CONNECTION_STRING='Server=localhost;Database=tempdb;User Id=SA;Password=Ea.930318;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;' cargo test -p mssql-orm --test stage5_public_crud -- --test-threads=1`

### Bloqueos

- No hubo bloqueos persistentes.
- El registro actual conserva únicamente registros de carga (`entity_rust_name`, `state` inicial); todavía no mantiene referencias vivas compartidas al contenido mutable de `Tracked<T>`, por lo que `save_changes()` deberá introducir ese acoplamiento con cuidado y sin romper la surface existente.

### Próximo paso recomendado

- Implementar `Etapa 12: save_changes() para entidades Modified, reutilizando DbSet::update y preservando rowversion/ConcurrencyConflict`.

### Sesión: transición `Unchanged -> Modified` en `Tracked<T>`

- Se volvió a tomar como fuente de verdad el plan maestro real en `docs/plan_orm_sqlserver_tiberius_code_first.md`, manteniendo la subtarea limitada a la mutabilidad observada del wrapper `Tracked<T>`.
- Se movió en `docs/tasks.md` la subtarea `Etapa 12: Detectar transición Unchanged -> Modified al mutar Tracked<T> sin exigir todavía tracking automático global` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm/src/tracking.rs` ahora expone `Tracked::current_mut()` y además implementa `Deref`/`DerefMut` hacia la entidad actual para permitir el uso previsto por el plan (`tracked.campo = ...`).
- La transición de estado quedó deliberadamente mínima: cualquier acceso mutable a una entidad `Unchanged` la marca como `Modified`; estados `Added` y `Deleted` no se reescriben automáticamente en esta subtarea.
- No se añadió todavía comparación estructural entre `original` y `current`; en esta fase el wrapper considera “potencialmente modificada” a la entidad desde el momento en que se pide acceso mutable.
- Se añadieron pruebas unitarias del módulo para fijar tres contratos: mutación vía `DerefMut`, mutación vía `current_mut()` y preservación del estado `Added` para entidades nuevas.
- Se amplió `crates/mssql-orm/tests/stage5_public_crud.rs` para validar con una entidad pública real que `find_tracked(...)` devuelve un wrapper inicialmente `Unchanged`, que conserva `original`, y que tras mutar `tracked.name` el estado observable pasa a `Modified`.

### Resultado

- La Etapa 12 ya permite mutar `Tracked<T>` de forma idiomática y deja marcada la entidad como `Modified`, preparando el terreno para la futura colección interna de trackeados y `save_changes()`.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p mssql-orm --lib`
- `cargo check --workspace`
- `cargo clippy -p mssql-orm --all-targets --all-features -- -D warnings`
- `MSSQL_ORM_TEST_CONNECTION_STRING='Server=localhost;Database=tempdb;User Id=SA;Password=Ea.930318;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;' cargo test -p mssql-orm --test stage5_public_crud -- --test-threads=1`

### Bloqueos

- No hubo bloqueos persistentes.
- La transición actual se activa con acceso mutable, no con diff estructural real; ese refinamiento queda fuera del alcance de esta subtarea y deberá evaluarse solo si más adelante aporta valor para `save_changes()`.

### Próximo paso recomendado

- Implementar `Etapa 12: Introducir colección interna mínima de entidades trackeadas dentro de DbContext experimental sin romper la API explícita existente`.

### Sesión: `DbSet::find_tracked(id)` sobre PK simple

- Se confirmó que el plan maestro real del repositorio no está en la raíz sino en `docs/plan_orm_sqlserver_tiberius_code_first.md`, y se usó esa ruta como fuente de verdad para esta subtarea.
- Se movió en `docs/tasks.md` la subtarea `Etapa 12: Exponer DbSet::find_tracked(id) para PK simple reutilizando find y snapshot inicial` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm/src/context.rs` ahora expone `DbSet::find_tracked(...)` como wrapper explícito sobre `DbSet::find(...)`, limitado a entidades `Clone + FromRow + Send` y retornando `Option<Tracked<E>>` construido con `Tracked::from_loaded(...)`.
- La implementación no introduce todavía colección interna de tracking, dirty detection, `save_changes()` ni nuevas rutas de persistencia; la carga trackeada sigue montada completamente sobre la infraestructura CRUD existente.
- Se actualizó `crates/mssql-orm/src/tracking.rs` para quitar de la documentación del módulo la exclusión `find_tracked(...)`, manteniendo explícitos los límites que siguen pendientes.
- Se añadió cobertura unitaria en `crates/mssql-orm/src/context.rs` para fijar que `find_tracked(...)` reutiliza el mismo camino de error/conexión de `find(...)`.
- Se amplió `crates/mssql-orm/tests/stage5_public_crud.rs` con una validación pública real contra SQL Server que verifica que `find_tracked(...)` devuelve `Tracked::from_loaded(...)` sobre una entidad recién insertada.
- Como ajuste documental de consistencia, se retiró de `docs/tasks.md` una tarea pendiente duplicada sobre `Tracked<T>` que ya estaba cubierta por la subtarea completada de surface mínima.

### Resultado

- La Etapa 12 ya permite cargar entidades como `Tracked<T>` por PK simple desde `DbSet`, dejando lista la base para la próxima subtarea de transición `Unchanged -> Modified`.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p mssql-orm --lib`
- `cargo check --workspace`
- `cargo clippy -p mssql-orm --all-targets --all-features -- -D warnings`
- `MSSQL_ORM_TEST_CONNECTION_STRING='Server=localhost;Database=tempdb;User Id=SA;Password=Ea.930318;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;' cargo test -p mssql-orm --test stage5_public_crud -- --test-threads=1`

### Bloqueos

- No hubo bloqueos persistentes.
- La suite `stage5_public_crud` comparte tablas fijas entre tests; cuando se ejecuta en paralelo puede producir fallos cruzados no relacionados con esta subtarea, por lo que en esta sesión se validó en serial con `--test-threads=1`.

### Próximo paso recomendado

- Implementar `Etapa 12: Detectar transición Unchanged -> Modified al mutar Tracked<T> sin exigir todavía tracking automático global`.

### Sesión: surface experimental mínima de change tracking

- Se tomó la primera subtarea de la Etapa 12 y se movió en `docs/tasks.md` a `En Progreso` antes de editar, usando como referencia el plan maestro real en `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se añadió `crates/mssql-orm/src/tracking.rs` como módulo nuevo de la crate pública, definiendo `EntityState` (`Unchanged`, `Added`, `Modified`, `Deleted`) y `Tracked<T>` como wrapper snapshot-based con `original`, `current` y `state`.
- La surface nueva quedó intencionalmente mínima: `Tracked::from_loaded(...)`, `Tracked::from_added(...)`, accessors de lectura (`original`, `current`, `state`) y `into_current()`, sin introducir todavía `find_tracked`, `save_changes`, registro en `DbContext` ni detección automática de dirty state.
- El módulo incluye documentación explícita de límites y exclusiones para evitar ambigüedad en sesiones futuras: no hay tracking registry, no hay `save_changes`, no hay dirty detection automática y la API explícita existente de `DbSet`/`ActiveRecord` sigue siendo la principal.
- `crates/mssql-orm/src/lib.rs` ahora reexporta `Tracked` y `EntityState` en la raíz pública y en la `prelude`, dejando fijada desde ahora la surface observable del experimento.
- Se añadieron pruebas unitarias del módulo nuevo y una prueba adicional en la crate pública para fijar la disponibilidad de la surface desde la `prelude`.

### Resultado

- La Etapa 12 ya tiene definida y validada la surface pública mínima sobre la que podrán montarse `find_tracked`, la transición a `Modified` y el futuro `save_changes`.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --lib`
- `cargo check --workspace`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- Esta subtarea no implementa aún mutabilidad observada ni wiring de contexto; eso queda explícitamente para las siguientes subtareas del backlog.

### Próximo paso recomendado

- Implementar `Etapa 12: Exponer DbSet::find_tracked(id) para PK simple reutilizando find y snapshot inicial`.

### Sesión: descomposición de la Etapa 12 de change tracking

- Se confirmó nuevamente que el plan maestro real del repositorio está en `docs/plan_orm_sqlserver_tiberius_code_first.md`, y se usó esa ruta para revisar el alcance real de `Tracked<T>`, `EntityState`, `find_tracked`, `add`, `remove` y `save_changes`.
- Se concluyó que la tarea amplia `Etapa 12: Implementar change tracking experimental con Tracked<T> y save_changes` era demasiado grande para una sola sesión sin riesgo de mezclar contratos base, wiring de contexto, persistencia y cobertura en un único cambio difícil de validar.
- Se reemplazó en `docs/tasks.md` la tarea amplia de Etapa 12 por subtareas ordenadas y verificables: definición de surface mínima, `find_tracked`, contrato de `Tracked<T>`, transición a `Modified`, colección interna trackeada en `DbContext`, `save_changes` para `Modified`, soporte de `Added`, soporte de `Deleted` y cobertura/documentación experimental.
- La descomposición deja explícita una progresión segura: primero modelar y fijar límites, luego cargar entidades trackeadas, después persistir `Modified`, y recién más tarde incorporar `Added/Deleted`.

### Resultado

- La Etapa 12 quedó preparada para ejecución incremental, con backlog suficientemente detallado como para implementarse en sesiones pequeñas sin perder coherencia arquitectónica.

### Validación

- No aplicó validación con `cargo`: en esta sesión solo se actualizó backlog y documentación operativa; no hubo cambios de código.

### Bloqueos

- No hubo bloqueos técnicos.
- La principal sensibilidad sigue siendo arquitectónica: el tracking no debe duplicar la semántica CRUD ya existente ni introducir estado implícito opaco fuera de la crate pública.

### Próximo paso recomendado

- Empezar por `Etapa 12: Definir surface experimental mínima de change tracking (Tracked<T>, EntityState, límites y exclusiones explícitas)`.

### Sesión: `OrmError::ConcurrencyConflict` para conflictos de actualización y borrado

- Se confirmó nuevamente que el plan maestro real del repositorio está en `docs/plan_orm_sqlserver_tiberius_code_first.md`, y se usó como referencia para cerrar la segunda subtarea de la Etapa 11.
- Se movió en `docs/tasks.md` la subtarea `Etapa 11: Retornar OrmError::ConcurrencyConflict en conflictos de actualización o borrado` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm-core/src/lib.rs` ahora modela `OrmError` como enum estable con `Message(&'static str)` y `ConcurrencyConflict`, preservando `OrmError::new(...)` para el resto del workspace y alineando la surface con el shape previsto por el plan.
- `crates/mssql-orm/src/context.rs` ahora distingue entre “no hubo fila” y “hubo conflicto de concurrencia”: cuando `update` o las rutas internas de update/delete operan con token `rowversion`, no afectan filas y la PK todavía existe, se promueve el resultado a `OrmError::ConcurrencyConflict`.
- `crates/mssql-orm/src/active_record.rs` dejó de exponer un mensaje ad hoc para el mismatch de `rowversion`; `save(&db)` y `delete(&db)` ahora propagan `OrmError::ConcurrencyConflict` desde `DbSet`.
- Se actualizaron `crates/mssql-orm/tests/stage5_public_crud.rs` y `crates/mssql-orm/tests/stage10_public_active_record.rs` para fijar la nueva semántica observable: stale update y stale delete ya no se ven como `None`, `false` o mensaje genérico, sino como `OrmError::ConcurrencyConflict`.

### Resultado

- La Etapa 11 quedó cerrada: el ORM ya evita overwrites silenciosos con `rowversion` y además expresa esos conflictos con un error público estable.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm-core --lib`
- `cargo test -p mssql-orm --lib`
- `cargo test -p mssql-orm --test stage5_public_crud`
- `cargo test -p mssql-orm --test stage10_public_active_record`
- `cargo check --workspace`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- La conversión a `ConcurrencyConflict` se activa solo cuando realmente existe token `rowversion`; operaciones sin token siguen preservando su contrato previo (`Option`/`bool`/mensajes existentes).

### Próximo paso recomendado

- Iniciar la Etapa 12 con el diseño de `Tracked<T>` y `save_changes`, reutilizando la semántica de conflicto ya fijada en la Etapa 11.

### Sesión: soporte de concurrencia optimista con `rowversion`

- Se confirmó nuevamente que el plan maestro real del repositorio está en `docs/plan_orm_sqlserver_tiberius_code_first.md`, y se tomó esa ruta como fuente de verdad para la primera subtarea de la Etapa 11.
- Se movió en `docs/tasks.md` la subtarea `Etapa 11: Implementar soporte de concurrencia optimista con rowversion` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm-core/src/lib.rs` ahora expone `EntityMetadata::rowversion_column()` y `Changeset::concurrency_token()` con default neutro, para que la concurrencia optimista pueda montarse sobre metadata y contracts ya existentes sin abrir un sistema paralelo.
- `crates/mssql-orm-macros/src/lib.rs` ahora hace dos cosas relevantes para concurrencia: `#[derive(Entity)]` genera extracción automática del token `rowversion` desde la entidad, y `#[derive(Changeset)]` detecta campos `rowversion` para usarlos como token de concurrencia sin intentar incluirlos dentro del `SET`.
- `crates/mssql-orm/src/context.rs` ahora agrega el predicado `AND [rowversion] = @Pn` en `DbSet::update(...)` cuando el `Changeset` aporta token, y en las rutas internas de `delete/save` usadas por Active Record cuando la entidad tiene columna `rowversion`.
- `crates/mssql-orm/src/active_record.rs` ahora hace que `save(&db)` y `delete(&db)` reutilicen también el token `rowversion` de la entidad; `save(&db)` devuelve por ahora un `OrmError` genérico cuando detecta mismatch en una actualización protegida, dejando el mapeo a `OrmError::ConcurrencyConflict` para la subtarea siguiente del backlog.
- Se ampliaron las pruebas unitarias de `DbSet` para fijar la forma exacta de los predicados con PK + rowversion, y se añadieron integraciones reales en `crates/mssql-orm/tests/stage5_public_crud.rs` y `crates/mssql-orm/tests/stage10_public_active_record.rs` para validar que un segundo update/delete con token viejo deja de afectar filas.

### Resultado

- La Etapa 11 ya quedó iniciada con soporte real de concurrencia optimista basado en `rowversion`, sin cambiar todavía el tipo de error público de conflicto.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --lib`
- `cargo test -p mssql-orm --test stage5_public_crud`
- `cargo test -p mssql-orm --test stage10_public_active_record`
- `cargo check --workspace`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- El soporte de `rowversion` ya evita overwrite silencioso, pero la surface pública todavía expresa el conflicto como `None`, `false` o `OrmError` genérico según la ruta; el cierre semántico correcto queda pendiente para `OrmError::ConcurrencyConflict`.

### Próximo paso recomendado

- Implementar `Etapa 11: Retornar OrmError::ConcurrencyConflict en conflictos de actualización o borrado`.

### Sesión: `entity.save(&db)` para Active Record

- Se confirmó nuevamente que el plan maestro real del repositorio está en `docs/plan_orm_sqlserver_tiberius_code_first.md`, y se tomó esa ruta como fuente de verdad para cerrar la última subtarea pendiente de la Etapa 10.
- Se movió en `docs/tasks.md` la subtarea `Etapa 10: Diseñar e implementar entity.save(&db) sobre Active Record con estrategia explícita de PK y persistencia` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm/src/active_record.rs` con `save(&db)` sobre `&mut self`, manteniendo la API Active Record como capa de conveniencia encima de `DbSet` y sincronizando la instancia con la fila materializada devuelta por la base.
- Se introdujeron los contratos ocultos `EntityPersist` y `EntityPersistMode`, y `crates/mssql-orm-macros/src/lib.rs` ahora los implementa automáticamente para `#[derive(Entity)]`, generando extracción de valores insertables, cambios actualizables y estrategia de persistencia por PK simple.
- La estrategia aplicada quedó explícita en el macro: para PK simple con `identity`, `id == 0` se trata como inserción y cualquier otro valor como actualización; para PK simple no `identity`, `save` realiza `insert-or-update` apoyándose en `DbSet::find`, `DbSet::insert` y `DbSet::update` sin compilar SQL fuera de la crate pública.
- `crates/mssql-orm/src/context.rs` se amplió solo con helpers internos basados en `ColumnValue` para buscar, insertar y actualizar por `SqlValue`, evitando duplicar el pipeline de compilación SQL Server y ejecución Tiberius ya existente.
- Se añadió `crates/mssql-orm/tests/ui/active_record_save_public_valid.rs`, se extendió `crates/mssql-orm/tests/active_record_trybuild.rs` y `crates/mssql-orm/tests/stage10_public_active_record.rs` ahora cubre roundtrip real de `save` tanto en alta como en actualización.

### Resultado

- La Etapa 10 quedó cerrada: `ActiveRecord` ya expone `query`, `find`, `delete` y `save`, siempre montado sobre `DbSet` y sin introducir una ruta paralela de compilación o ejecución.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --lib active_record`
- `cargo test -p mssql-orm --test active_record_trybuild`
- `cargo test -p mssql-orm --test stage10_public_active_record`
- `cargo check --workspace`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- `save`, igual que `find`, `update` y `delete`, sigue limitado a primary key simple; para PK compuesta retorna error explícito de etapa.
- La heurística `identity == 0 => insert` quedó restringida a PK enteras con `identity`; si más adelante se quiere soportar estados más ricos o detached entities, eso debe resolverse en la Etapa 12 con tracking explícito y no ampliando heurísticas implícitas.

### Próximo paso recomendado

- Empezar la Etapa 11 implementando soporte de concurrencia optimista con `rowversion` sobre la ruta de actualización ya existente.

### Sesión: `entity.delete(&db)` para Active Record

- Se confirmó nuevamente que el plan maestro real del repositorio está en `docs/plan_orm_sqlserver_tiberius_code_first.md`, y se tomó esa ruta como referencia para la subtarea de borrado Active Record.
- Se movió en `docs/tasks.md` la subtarea `Etapa 10: Diseñar e implementar entity.delete(&db) sobre Active Record sin duplicar la lógica de DbSet` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió en `crates/mssql-orm/src/active_record.rs` el método `delete(&db)` sobre `ActiveRecord`, delegando a `DbSet::delete_by_sql_value(...)` y manteniendo toda la ejecución real dentro de la capa ya existente.
- Se introdujo el helper oculto `EntityPrimaryKey` en la crate pública y `crates/mssql-orm-macros/src/lib.rs` ahora implementa ese contrato automáticamente para `#[derive(Entity)]`, extrayendo la PK simple como `SqlValue` y rechazando PK compuesta con error explícito de etapa.
- `crates/mssql-orm/src/context.rs` ahora también expone internamente la ruta `delete_by_sql_value(...)`, reutilizando la misma compilación SQL y el mismo contrato de borrado por PK ya existente en `DbSet`.
- Se amplió `crates/mssql-orm/tests/active_record_trybuild.rs` con `active_record_delete_public_valid.rs` y se extendió `crates/mssql-orm/tests/stage10_public_active_record.rs` con una integración real que valida borrado exitoso y borrado repetido devolviendo `false`.
- Durante la validación se corrigió además la ruta de conexión requerida para evitar `panic` en `DbSet` desconectado durante tests unitarios, devolviendo `OrmError` en las operaciones async que realmente necesitan conexión.

### Resultado

- La Etapa 10 ya soporta `entity.delete(&db)` sobre Active Record para entidades con PK simple, reutilizando `DbSet` y sin introducir una segunda ruta de ejecución o borrado.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --lib`
- `cargo test -p mssql-orm --test active_record_trybuild`
- `cargo test -p mssql-orm --test stage10_public_active_record`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- `entity.delete(&db)` mantiene el mismo límite funcional que `DbSet::delete`: hoy solo soporta PK simple; entidades con PK compuesta siguen recibiendo error explícito de etapa.

### Próximo paso recomendado

- Implementar `Etapa 10: Diseñar e implementar entity.save(&db) sobre Active Record con estrategia explícita de PK y persistencia`.

### Sesión: Cobertura dedicada para Active Record base

- Se confirmó nuevamente que el plan maestro real del repositorio está en `docs/plan_orm_sqlserver_tiberius_code_first.md`, y se usó esa ruta como referencia para cerrar la subtarea de cobertura de Active Record.
- Se movió en `docs/tasks.md` la subtarea `Etapa 10: Agregar pruebas unitarias, trybuild e integración dedicadas para la capa Active Record base` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se retiró `active_record_public_valid.rs` de la batería `trybuild` genérica y se creó `crates/mssql-orm/tests/active_record_trybuild.rs` como suite dedicada de Active Record.
- Se añadió `crates/mssql-orm/tests/ui/active_record_missing_entity_set.rs` y su `.stderr` para fijar el error de compilación cuando un contexto no implementa `DbContextEntitySet<User>` y aun así se intenta usar `User::query(&db)`.
- Se añadió `crates/mssql-orm/tests/stage10_public_active_record.rs` con integración pública dedicada sobre SQL Server real, cubriendo roundtrip de `ActiveRecord::query(&db)` y `ActiveRecord::find(&db, id)`, además del caso `None` para filas inexistentes.
- La cobertura unitaria de `crates/mssql-orm/src/active_record.rs` se mantuvo como batería interna mínima de la surface, y esta sesión completó la parte separada de `trybuild` e integración pública requerida por el backlog.

### Resultado

- La capa base de Active Record ya quedó protegida por cobertura dedicada de compilación e integración, separada de la batería general del resto de la crate pública.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test active_record_trybuild`
- `cargo test -p mssql-orm --test stage10_public_active_record`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- El siguiente frente de Active Record ya no es cobertura sino diseño de mutación de instancia; `entity.delete(&db)` debe montarse sobre `DbSet::delete` sin introducir otra ruta de ejecución ni resolver PKs por heurística opaca.

### Próximo paso recomendado

- Implementar `Etapa 10: Diseñar e implementar entity.delete(&db) sobre Active Record sin duplicar la lógica de DbSet`.

### Sesión: Trait `ActiveRecord` base sobre `DbSet`

- Se confirmó nuevamente que el plan maestro real del repositorio está en `docs/plan_orm_sqlserver_tiberius_code_first.md`, y se tomó esa ruta como referencia para la subtarea de `ActiveRecord`.
- Se movió en `docs/tasks.md` la subtarea `Etapa 10: Implementar trait ActiveRecord base con Entity::query(&db) y Entity::find(&db, id) sobre DbSet` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió `crates/mssql-orm/src/active_record.rs` con el trait público `ActiveRecord`, implementado blanket para toda `Entity`.
- La surface mínima nueva expone `Entity::query(&db)` y `Entity::find(&db, id)`, reutilizando exclusivamente `DbContextEntitySet<E>` y `DbSet<E>`; no se introdujo conexión global ni otro runner.
- Se actualizó `crates/mssql-orm/src/lib.rs` para reexportar `ActiveRecord` en la API pública y en la `prelude`.
- Se amplió `crates/mssql-orm/tests/trybuild.rs` y se añadió `crates/mssql-orm/tests/ui/active_record_public_valid.rs` para fijar por compilación que un consumidor real puede escribir `User::query(&db)` y `User::find(&db, 1_i64)`.
- También se añadieron pruebas unitarias internas en `crates/mssql-orm/src/active_record.rs` para asegurar que `query` delega al `DbSet` tipado y que `find` conserva el contrato de la capa existente.

### Resultado

- La Etapa 10 ya tiene la capa mínima de Active Record exigida por el plan maestro para `query/find`, montada estrictamente encima de `DbSet` y sin abrir todavía el frente más delicado de `save/delete`.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --lib`
- `cargo test -p mssql-orm --test trybuild`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- La cobertura añadida en esta sesión es suficiente para fijar la surface base, pero todavía conviene agregar una batería dedicada de pruebas públicas/integración antes de avanzar a `save/delete`.

### Próximo paso recomendado

- Implementar `Etapa 10: Agregar pruebas unitarias, trybuild e integración dedicadas para la capa Active Record base`.

### Sesión: Acceso tipado `DbContext -> DbSet<T>` para Active Record

- Se movió en `docs/tasks.md` la subtarea `Etapa 10: Exponer acceso tipado DbContext -> DbSet<T> para habilitar Active Record sobre la crate pública` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió en `crates/mssql-orm/src/context.rs` el nuevo trait público `DbContextEntitySet<E>`, definido como contrato mínimo para resolver un `DbSet<E>` desde cualquier contexto sin introducir reflexión ni conexión global.
- `crates/mssql-orm-macros/src/lib.rs` ahora hace que `#[derive(DbContext)]` implemente automáticamente `DbContextEntitySet<E>` por cada campo `DbSet<E>` del contexto.
- Para evitar ambigüedad en la futura API Active Record, el derive ahora rechaza en compile-time contextos que declaren múltiples `DbSet` para la misma entidad.
- Se actualizaron `crates/mssql-orm/src/lib.rs`, `crates/mssql-orm/tests/ui/dbcontext_valid.rs` y `crates/mssql-orm/tests/trybuild.rs`, y se añadió `crates/mssql-orm/tests/ui/dbcontext_duplicate_entity_set.rs` con su `.stderr` para fijar el contrato nuevo.
- También se añadieron pruebas unitarias internas en la crate pública para verificar el trait nuevo en un contexto mínimo desconectado y su reexport desde la `prelude`.

### Resultado

- La Etapa 10 ya tiene la base técnica necesaria para que `ActiveRecord` pueda resolver `DbSet<T>` desde `DbContext` de forma tipada, reutilizando la infraestructura existente de `DbSet` en lugar de crear otra capa de wiring.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --lib`
- `cargo test -p mssql-orm --test trybuild`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- La restricción nueva de un solo `DbSet<E>` por contexto es deliberada para evitar ambigüedad en Active Record; si más adelante se necesita relajarla, habrá que introducir un mecanismo explícito de selección y no inferencia implícita por tipo.

### Próximo paso recomendado

- Implementar `Etapa 10: Implementar trait ActiveRecord base con Entity::query(&db) y Entity::find(&db, id) sobre DbSet`.

### Sesión: División de la Etapa 10 de Active Record

- Se revisó la Etapa 10 contra la implementación actual de `DbSet`, `DbContext` y el plan maestro real en `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se concluyó que la tarea amplia `Implementar capa opcional Active Record sobre DbSet` era demasiado grande para una sola sesión sin riesgo de dejar contratos incompletos para `save` y `delete`.
- Se reemplazó esa tarea en `docs/tasks.md` por subtareas verificables: acceso tipado `DbContext -> DbSet<T>`, trait `ActiveRecord` base para `query/find`, cobertura de pruebas, `entity.delete(&db)` y `entity.save(&db)`.
- Se actualizó `docs/context.md` para dejar explícito que la siguiente sesión debe empezar por el acceso tipado `DbContext -> DbSet<T>` y que `save/delete` quedan diferidos hasta definir mejor PK y persistencia de instancias.

### Resultado

- El backlog de Etapa 10 quedó descompuesto en entregables pequeños y trazables, reduciendo el riesgo de dejar Active Record a medio implementar.

### Validación

- No aplicó validación con `cargo` porque en esta sesión solo se reestructuró el backlog y la documentación operativa; no hubo cambios de código.

### Bloqueos

- No hubo bloqueos técnicos.
- `entity.save(&db)` sigue siendo la parte más delicada de la Etapa 10 porque hoy la crate pública no tiene todavía un contrato explícito para extraer PK y distinguir persistencia de instancia sin introducir duplicación o acoplamiento indebido.

### Próximo paso recomendado

- Implementar `Etapa 10: Exponer acceso tipado DbContext -> DbSet<T> para habilitar Active Record sobre la crate pública`.

### Sesión: Sintaxis estructurada para `foreign_key`

- Se confirmó que el plan maestro real del repositorio está en `docs/plan_orm_sqlserver_tiberius_code_first.md`, y se tomó esa ruta como fuente de verdad junto con `docs/instructions.md`, `docs/tasks.md`, `docs/worklog.md` y `docs/context.md`.
- Se movió en `docs/tasks.md` la subtarea `Etapa 9: Rediseñar foreign_key hacia sintaxis estructurada #[orm(foreign_key(entity = Customer, column = id))] con validación en compile-time, sin exigir que la columna de destino sea primary key` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm-macros/src/lib.rs` para soportar `#[orm(foreign_key(entity = Customer, column = id))]` además de la sintaxis string previa, manteniendo compatibilidad con `tabla.columna` y `schema.tabla.columna`.
- El derive `Entity` ahora genera `__MSSQL_ORM_ENTITY_SCHEMA` y `__MSSQL_ORM_ENTITY_TABLE` sobre cada entidad derivada, y reutiliza `Customer::id` como validación compile-time mínima para resolver schema, tabla y columna de la referencia estructurada sin exigir primary key.
- Se actualizaron `crates/mssql-orm/tests/stage9_relationship_metadata.rs` y `crates/mssql-orm/tests/trybuild.rs`, y se añadieron `crates/mssql-orm/tests/ui/entity_foreign_key_structured_valid.rs` y `crates/mssql-orm/tests/ui/entity_foreign_key_structured_missing_column.rs` con sus expectativas `.stderr`.
- Durante la validación apareció un error de borrow parcial en `foreign_key.name`; se corrigió antes de relanzar pruebas y se ajustó también el snapshot `trybuild` del mensaje de error para formato inválido legacy.

### Resultado

- La Etapa 9 quedó cerrada también para el rediseño de `foreign_key`: el derive soporta la forma estructurada, valida la columna de destino en compile-time y mantiene compatibilidad con el formato string existente.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test stage9_relationship_metadata`
- `cargo test -p mssql-orm --test trybuild`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- La validación compile-time de la variante estructurada todavía depende del error nativo de símbolo inexistente cuando la columna referenciada no existe; ese nivel de diagnóstico es suficiente para esta etapa y no justifica introducir una capa adicional de reflexión o registro global.

### Próximo paso recomendado

- Implementar `Etapa 10: Implementar capa opcional Active Record sobre DbSet`.

### Sesión: Cobertura de integración y snapshots para joins y foreign keys

- Se movió en `docs/tasks.md` la subtarea `Etapa 9: Agregar pruebas de integración y snapshots para joins y foreign keys` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se amplió `crates/mssql-orm-sqlserver/tests/compiler_snapshots.rs` con un snapshot adicional `compiled_select_with_join`, fijando el SQL y el orden de parámetros para un `SELECT` con `INNER JOIN`.
- Se añadió `crates/mssql-orm-sqlserver/tests/migration_snapshots.rs` con el snapshot `foreign_key_migration_sql`, fijando el DDL observable de `AddForeignKey` y `DropForeignKey` con `ON DELETE CASCADE`.
- Se extendió `crates/mssql-orm/tests/stage6_public_query_builder_snapshots.rs` con el snapshot `public_query_builder_compiled_join_select`, cubriendo la compilación SQL desde la surface pública mínima de joins.
- Se materializaron y versionaron los snapshots nuevos bajo `crates/mssql-orm-sqlserver/tests/snapshots/` y `crates/mssql-orm/tests/snapshots/`.
- Durante la validación se detectó un gap menor en imports para el snapshot público de joins; se corrigió importando `Expr` y `Predicate` desde `mssql_orm::query`.

### Resultado

- La Etapa 9 ya tiene cobertura observable adicional para joins y foreign keys tanto en la capa SQL Server como en la crate pública, reduciendo riesgo de regresiones silenciosas en SQL y DDL.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm-sqlserver --test compiler_snapshots`
- `cargo test -p mssql-orm-sqlserver --test migration_snapshots`
- `cargo test -p mssql-orm --test stage6_public_query_builder_snapshots`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- La ausencia de aliases sigue limitando joins repetidos sobre la misma tabla; la cobertura añadida se mantuvo dentro del caso base ya soportado.

### Próximo paso recomendado

- Implementar `Etapa 9: Rediseñar foreign_key hacia sintaxis estructurada #[orm(foreign_key(entity = Customer, column = id))] con validación en compile-time, sin exigir que la columna de destino sea primary key`.

### Sesión: Surface pública mínima para joins explícitos

- Se movió en `docs/tasks.md` la subtarea `Etapa 9: Exponer joins explícitos mínimos en la crate pública` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm/src/dbset_query.rs` para que `DbSetQuery` exponga `join(...)`, `inner_join::<T>(...)` y `left_join::<T>(...)`, delegando directamente al `SelectQuery` interno sin crear un AST paralelo en la crate pública.
- `crates/mssql-orm/src/lib.rs` ahora reexporta también `Join` y `JoinType` en la `prelude`, de modo que el consumidor tenga acceso al shape público mínimo de joins desde la crate principal.
- Se ampliaron las pruebas internas de `DbSetQuery` para fijar que los nuevos helpers construyen el `SelectQuery` esperado y conservan la tabla de destino y el tipo de join.
- Se actualizó `crates/mssql-orm/tests/stage6_public_query_builder.rs` para cubrir joins explícitos en el AST observable desde la crate pública y `crates/mssql-orm/tests/ui/query_builder_public_valid.rs` para fijar por compilación que un consumidor puede escribir `db.users.query().inner_join::<Order>(...)` y `left_join::<Order>(...)`.
- La verificación pública de joins columna-columna usa `Predicate::eq(Expr::from(...), Expr::from(...))`, manteniendo sin cambios las extensiones tipadas de columnas que siguen modeladas para comparaciones contra valores.

### Resultado

- La crate pública `mssql-orm` ya expone joins explícitos mínimos sobre `DbSetQuery`, apoyándose en el AST y la compilación SQL Server ya existentes y sin adelantar todavía aliases ni eager loading.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test stage6_public_query_builder`
- `cargo test -p mssql-orm --test trybuild`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- La API pública mínima todavía no resuelve joins repetidos sobre la misma tabla, porque esa limitación sigue determinada por la ausencia de aliases en el AST base.

### Próximo paso recomendado

- Implementar `Etapa 9: Agregar pruebas de integración y snapshots para joins y foreign keys`.

### Sesión: Compilación SQL Server de joins explícitos

- Se movió en `docs/tasks.md` la subtarea `Etapa 9: Compilar joins explícitos a SQL Server parametrizado` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm-sqlserver/src/compiler.rs` para compilar `SelectQuery::joins` a `INNER JOIN` y `LEFT JOIN`, reutilizando `quote_table_ref`, `Predicate` y el mismo `ParameterBuilder` ya usado por filtros y paginación.
- La compilación preserva orden de joins y orden global de parámetros, de modo que valores usados en condiciones `ON`, `WHERE` y `OFFSET/FETCH` siguen numerándose en secuencia `@P1..@Pn`.
- Dado que el AST todavía no soporta aliases, la compilación ahora rechaza explícitamente joins repetidos sobre la misma tabla o self-joins con el error `SQL Server join compilation requires unique tables until alias support exists`.
- Se añadieron pruebas unitarias en `mssql-orm-sqlserver` para cubrir compilación de joins explícitos y rechazo de tablas duplicadas sin aliasing.
- Esta sesión no agregó aún surface pública nueva ni snapshots dedicados de joins; eso queda para las subtareas posteriores ya separadas en el backlog.

### Resultado

- La Etapa 9 ya cuenta con joins explícitos compilables en la crate SQL Server para el caso mínimo soportado actualmente: joins entre tablas distintas sin aliasing.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm-sqlserver`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- La ausencia de aliases en el AST impide todavía soportar self-joins o múltiples joins sobre la misma tabla; ese límite quedó documentado y validado con error explícito.

### Próximo paso recomendado

- Implementar `Etapa 9: Exponer joins explícitos mínimos en la crate pública`.

### Sesión: Joins explícitos en el AST de queries

- Se movió en `docs/tasks.md` la subtarea `Etapa 9: Incorporar joins explícitos al AST de mssql-orm-query` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió `crates/mssql-orm-query/src/join.rs` con los tipos `JoinType` y `Join`, modelando joins explícitos como parte del AST sin introducir generación SQL en la crate `query`.
- `crates/mssql-orm-query/src/select.rs` ahora expone `SelectQuery::joins`, `join(...)`, `inner_join::<E>(...)` y `left_join::<E>(...)`, manteniendo la condición de join en términos de `Predicate`.
- `crates/mssql-orm-query/src/lib.rs` ahora reexporta `Join` y `JoinType`, y su batería de pruebas incluye casos específicos que fijan el shape del AST para joins internos y left joins sobre entidades explícitas.
- Para no dejar una semántica silenciosamente incorrecta en la siguiente capa, `crates/mssql-orm-sqlserver/src/compiler.rs` ahora rechaza explícitamente `SelectQuery` con joins no vacíos mediante el error `SQL Server join compilation is not supported in this stage`.
- Se actualizó la batería de pruebas de `mssql-orm-sqlserver` para fijar ese rechazo explícito hasta la siguiente subtarea dedicada a compilación SQL de joins.

### Resultado

- La Etapa 9 ya tiene joins explícitos modelados en el AST de `mssql-orm-query`, con contratos estables y sin adelantar todavía su compilación SQL ni la API pública fluente.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm-query`
- `cargo test -p mssql-orm-sqlserver`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- La compilación SQL Server de joins sigue fuera del alcance de esta sesión y queda rechazada explícitamente para evitar pérdida silenciosa de semántica.

### Próximo paso recomendado

- Implementar `Etapa 9: Compilar joins explícitos a SQL Server parametrizado`.

### Sesión: DDL SQL Server para índices de migración

- Se movió en `docs/tasks.md` la subtarea `Etapa 9: Implementar DDL SQL Server para CreateIndex y DropIndex en migraciones` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm-sqlserver/src/migration.rs` para compilar `MigrationOperation::CreateIndex` a `CREATE INDEX` y `CREATE UNIQUE INDEX` sobre tablas calificadas por schema.
- La misma capa ahora compila `MigrationOperation::DropIndex` a `DROP INDEX ... ON ...`, manteniendo toda la generación DDL de índices dentro de la crate SQL Server.
- La compilación de índices reutiliza `IndexSnapshot` e `IndexColumnSnapshot`, preservando orden de columnas y dirección `ASC`/`DESC` a partir del snapshot ya producido por metadata/diff.
- Se añadió validación explícita para rechazar índices sin columnas, evitando generar DDL inválido desde snapshots corruptos o incompletos.
- Se actualizaron las pruebas unitarias de `mssql-orm-sqlserver` para cubrir índices normales, únicos, compuestos con orden mixto y rechazo de índices vacíos.

### Resultado

- La capa SQL Server ya cubre todo el DDL relacional básico pendiente de Etapa 9 para migraciones: foreign keys con acciones referenciales iniciales e índices simples/compuestos.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm-sqlserver`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- El único ajuste necesario durante la validación fue importar `CreateIndex` en el módulo de tests de `mssql-orm-sqlserver`; quedó corregido en la misma sesión.

### Próximo paso recomendado

- Implementar `Etapa 9: Incorporar joins explícitos al AST de mssql-orm-query`.

### Sesión: Delete behavior inicial para foreign keys

- Se confirmó otra vez que el plan maestro usado como fuente de verdad está en `docs/plan_orm_sqlserver_tiberius_code_first.md`, no en la raíz del repositorio.
- Se movió en `docs/tasks.md` la subtarea `Etapa 9: Soportar delete behavior inicial (no action, cascade, set null) en metadata y DDL` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm-macros/src/lib.rs` para aceptar `#[orm(on_delete = "no action" | "cascade" | "set null")]` en campos con `foreign_key`, manteniendo `on_update` en `NoAction` dentro del alcance actual.
- El derive `Entity` ahora emite `ForeignKeyMetadata::on_delete` configurable y rechaza en compile-time `on_delete = "set null"` cuando la columna local no es nullable.
- Se amplió `crates/mssql-orm/tests/stage9_relationship_metadata.rs` para fijar metadata derivada con `Cascade` y `SetNull`, y se añadió el caso `trybuild` `entity_foreign_key_set_null_requires_nullable`.
- Se actualizó `crates/mssql-orm-sqlserver/src/migration.rs` para compilar `AddForeignKey` con `ON DELETE` y `ON UPDATE` usando `NO ACTION`, `CASCADE` y `SET NULL`, rechazando todavía `SET DEFAULT` con error explícito de etapa.
- Se añadieron pruebas unitarias en la crate SQL Server para renderizado explícito de `NO ACTION`, `CASCADE`, `SET NULL` y rechazo de `SET DEFAULT`.
- Se registró en `docs/tasks.md` una nueva subtarea pendiente: `Etapa 9: Implementar DDL SQL Server para CreateIndex y DropIndex en migraciones`, porque esa parte sigue rechazada explícitamente y era un hueco real no trazado en el backlog.
- `Cargo.lock` se sincronizó con los manifests actuales del workspace durante la validación, incorporando dependencias ya declaradas que no estaban reflejadas en el lockfile versionado.

### Resultado

- La Etapa 9 ya soporta `delete behavior` inicial de foreign keys tanto en metadata derivada como en DDL SQL Server, con validación temprana para el caso `set null` sobre columnas no nullable.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test stage9_relationship_metadata`
- `cargo test -p mssql-orm-sqlserver`
- `cargo test -p mssql-orm --test trybuild`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- `CreateIndex` y `DropIndex` siguen rechazados explícitamente en `mssql-orm-sqlserver`; por eso se añadió la subtarea dedicada al backlog en esta misma sesión.

### Próximo paso recomendado

- Implementar `Etapa 9: Implementar DDL SQL Server para CreateIndex y DropIndex en migraciones`.

### Sesión: DDL SQL Server base para foreign keys

- Se movió en `docs/tasks.md` la subtarea `Etapa 9: Generar DDL SQL Server para crear y eliminar foreign keys` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se actualizó `crates/mssql-orm-sqlserver/src/migration.rs` para compilar `AddForeignKey` a `ALTER TABLE ... ADD CONSTRAINT ... FOREIGN KEY ... REFERENCES ...`.
- La misma capa ahora compila `DropForeignKey` a `ALTER TABLE ... DROP CONSTRAINT ...`, reutilizando quoting seguro de identificadores y nombres multipartes ya existentes en la crate SQL Server.
- Para no adelantar la subtarea de `delete behavior`, la compilación de foreign keys ahora rechaza explícitamente acciones referenciales distintas de `NoAction` con error claro de etapa.
- `CreateIndex` y `DropIndex` permanecen rechazadas explícitamente, porque su DDL sigue fuera del alcance de esta sesión.
- Se añadieron pruebas unitarias en `crates/mssql-orm-sqlserver/src/migration.rs` para `AddForeignKey`, `DropForeignKey` y rechazo de acciones `Cascade` antes de la subtarea dedicada.

### Resultado

- La crate SQL Server ya puede generar DDL básico de creación y eliminación de foreign keys a partir de las operaciones emitidas por el diff relacional, sin mezclar todavía soporte de `cascade`/`set null` ni DDL de índices.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm-sqlserver`
- `cargo test -p mssql-orm-migrate`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- `Cargo.lock` sigue con cambios previos ajenos a esta sesión y no fue modificado como parte del trabajo.

### Próximo paso recomendado

- Implementar `Etapa 9: Soportar delete behavior inicial (no action, cascade, set null) en metadata y DDL`.

### Sesión: Snapshots y diff de migraciones para relaciones

- Se movió en `docs/tasks.md` la subtarea `Etapa 9: Extender snapshots y diff de migraciones para foreign keys e índices asociados` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm-migrate/src/snapshot.rs` con `ForeignKeySnapshot`, `TableSnapshot::foreign_keys`, lookup por nombre y conversión automática desde `ForeignKeyMetadata`.
- `TableSnapshot::from(&EntityMetadata)` ahora conserva también foreign keys derivadas, además de columnas, primary key e índices.
- Se amplió `crates/mssql-orm-migrate/src/operation.rs` con operaciones explícitas `CreateIndex`, `DropIndex`, `AddForeignKey` y `DropForeignKey`, manteniendo la responsabilidad de generación SQL fuera de esta subtarea.
- Se extendió `crates/mssql-orm-migrate/src/diff.rs` con `diff_relational_operations(previous, current)`, cubriendo altas/bajas de índices, altas/bajas de foreign keys y recreación de foreign keys cuando cambia su definición.
- Se reforzaron las pruebas unitarias de `mssql-orm-migrate` para snapshots con foreign keys, surface de nuevas operaciones y diffs relacionales sobre snapshots compartidos.
- Se actualizó `crates/mssql-orm-sqlserver/src/migration.rs` para rechazar explícitamente operaciones de índices y foreign keys con error claro hasta implementar el DDL específico en la siguiente subtarea.

### Resultado

- El sistema de migraciones ya puede serializar metadata relacional en snapshots y detectar cambios de índices/FKs como operaciones explícitas, dejando lista la base para implementar el DDL SQL Server sin redefinir contratos.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm-migrate`
- `cargo test -p mssql-orm-sqlserver`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- `Cargo.lock` sigue con cambios previos ajenos a esta sesión y no fue modificado como parte del trabajo.

### Próximo paso recomendado

- Implementar `Etapa 9: Generar DDL SQL Server para crear y eliminar foreign keys`.

### Sesión: Cobertura de pruebas para metadata relacional

- Se confirmó nuevamente que el plan maestro usado como fuente de verdad está en `docs/plan_orm_sqlserver_tiberius_code_first.md`, no en la raíz del repositorio.
- Se movió en `docs/tasks.md` la subtarea `Etapa 9: Agregar pruebas trybuild y unitarias de metadata de relaciones` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se amplió `crates/mssql-orm/tests/trybuild.rs` con un caso válido adicional y un caso inválido adicional centrados en `foreign_key`.
- Se añadió `crates/mssql-orm/tests/ui/entity_foreign_key_default_schema_valid.rs` para fijar por compilación y runtime mínimo que `#[orm(foreign_key = "customers.id")]` usa schema `dbo` por defecto, respeta `#[orm(column = "...")]` como columna local y genera el nombre esperado de foreign key.
- Se añadió `crates/mssql-orm/tests/ui/entity_foreign_key_empty_segment.rs` y su `.stderr` para rechazar explícitamente segmentos vacíos como `crm..id`.
- Se añadió `crates/mssql-orm/tests/stage9_relationship_metadata.rs` con pruebas dedicadas de metadata relacional derivada, cubriendo múltiples foreign keys, nombres generados, schema por defecto, acciones referenciales por defecto y helpers `foreign_key`, `foreign_keys_for_column` y `foreign_keys_referencing`.

### Resultado

- La Etapa 9 ahora tiene una batería de pruebas específica para metadata de relaciones, separada de los casos generales de entidades y suficiente para fijar el contrato observable antes de avanzar a snapshots, DDL y joins.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test stage9_relationship_metadata`
- `cargo test -p mssql-orm --test trybuild`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- `Cargo.lock` ya tenía cambios previos ajenos a esta sesión y no fue alterado por el trabajo realizado.

### Próximo paso recomendado

- Implementar `Etapa 9: Extender snapshots y diff de migraciones para foreign keys e índices asociados`.

### Sesión: Derive de `foreign_key` en `Entity`

- Se movió en `docs/tasks.md` la subtarea `Etapa 9: Soportar atributos foreign_key en #[derive(Entity)] y generar metadata correspondiente` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm-macros/src/lib.rs` para que `parse_field_config` acepte `#[orm(foreign_key = "...")]` en formato `tabla.columna` o `schema.tabla.columna`.
- `#[derive(Entity)]` ahora genera `ForeignKeyMetadata` automáticamente para los campos marcados con `foreign_key`, usando la columna local derivada y `ReferentialAction::NoAction` por defecto en esta etapa.
- Cuando el usuario omite el schema de destino, el derive asume `dbo`, alineado con la convención actual del proyecto para SQL Server.
- Se amplió `crates/mssql-orm/tests/ui/entity_valid.rs` para fijar por compilación y runtime mínimo que la metadata derivada ya incluye foreign keys.
- Se añadió `crates/mssql-orm/tests/ui/entity_foreign_key_invalid_format.rs` y su `.stderr` para rechazar formatos inválidos de `foreign_key`.

### Resultado

- El derive `Entity` ya puede generar metadata de relaciones uno-a-muchos a partir del atributo `foreign_key`, dejando lista la base para una batería más específica de pruebas y para la posterior integración con migraciones.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test trybuild`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- Esta sesión no implementó todavía índices asociados, `delete behavior` configurable ni DDL/migraciones de foreign keys; esos entregables siguen en el backlog separado de Etapa 9.

### Próximo paso recomendado

- Implementar `Etapa 9: Agregar pruebas trybuild y unitarias de metadata de relaciones`.

### Sesión: Metadata base de relaciones uno-a-muchos

- Se movió en `docs/tasks.md` la subtarea `Etapa 9: Extender metadata base para relaciones y foreign keys uno-a-muchos` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se reforzó `crates/mssql-orm-core/src/lib.rs` manteniendo `ForeignKeyMetadata` como contrato base, pero agregando un constructor `const` y helpers explícitos para consultas de metadata de relaciones.
- `ForeignKeyMetadata` ahora expone `new(...)`, `references_table(...)` e `includes_column(...)`, permitiendo que macros, migraciones y futuras capas de joins reutilicen el mismo shape sin duplicar lógica auxiliar.
- `EntityMetadata` ahora también expone `foreign_key(name)`, `foreign_keys_for_column(column_name)` y `foreign_keys_referencing(schema, table)` como surface base para resolver relaciones uno-a-muchos desde metadata estática.
- Se ampliaron las pruebas unitarias de `mssql-orm-core` para fijar búsqueda por nombre, filtrado por columna local y filtrado por tabla referenciada.

### Resultado

- La base de metadata de relaciones quedó más explícita y utilizable sin alterar todavía macros, AST de joins ni generación SQL; eso deja una base estable para la siguiente subtarea del derive.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm-core`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- El atributo `#[orm(foreign_key = ...)]` todavía no está implementado en `#[derive(Entity)]`; esa parte quedó explícitamente fuera del alcance de esta sesión.

### Próximo paso recomendado

- Implementar `Etapa 9: Soportar atributos foreign_key en #[derive(Entity)] y generar metadata correspondiente`.

### Sesión: Pruebas reales de commit y rollback

- Se movió en `docs/tasks.md` la subtarea `Etapa 8: Agregar pruebas de commit y rollback` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm/tests/stage5_public_crud.rs` con dos pruebas de integración reales adicionales sobre la API pública `db.transaction(...)`.
- `public_dbcontext_transaction_commits_on_ok` verifica que una inserción realizada dentro del closure transaccional queda persistida y visible luego del `COMMIT`.
- `public_dbcontext_transaction_rolls_back_on_err` fuerza un `Err` dentro del closure y valida que la fila insertada no permanezca en la tabla tras el `ROLLBACK`.
- Ambas pruebas reutilizan la misma tabla real `dbo.mssql_orm_public_crud` y el mismo setup/cleanup ya existente, evitando introducir otro fixture paralelo para la Etapa 8.

### Resultado

- La Etapa 8 quedó cerrada de extremo a extremo: infraestructura transaccional en el adaptador, exposición pública de `db.transaction(...)` y pruebas reales de commit/rollback sobre SQL Server.

### Validación

- `cargo fmt --all`
- `cargo test --test stage5_public_crud`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.

### Próximo paso recomendado

- Iniciar `Etapa 9: Implementar metadata de relaciones, foreign keys, joins explícitos e índices asociados`.

### Sesión: Exposición pública de `db.transaction(...)`

- Se movió en `docs/tasks.md` la subtarea `Etapa 8: Exponer db.transaction(...) en la crate pública reutilizando la infraestructura transaccional` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm/src/context.rs` para que `DbContext` exponga `shared_connection()` y un método por defecto `transaction(...)` que:
  inicia `BEGIN TRANSACTION`,
  ejecuta el closure con un nuevo contexto construido sobre la misma conexión compartida,
  hace `COMMIT` en `Ok`,
  y hace `ROLLBACK` en `Err`.
- Se actualizó `crates/mssql-orm-macros/src/lib.rs` para que `#[derive(DbContext)]` implemente `shared_connection()` y genere además el método inherente `transaction(...)`, manteniendo la experiencia de uso `db.transaction(|tx| async move { ... })`.
- Se amplió `crates/mssql-orm-tiberius/src/transaction.rs` con helpers reutilizables de scope (`begin_transaction_scope`, `commit_transaction_scope`, `rollback_transaction_scope`) y `crates/mssql-orm-tiberius/src/connection.rs` ahora expone wrappers públicos mínimos para que la crate pública no tenga que emitir SQL ni tocar Tiberius directamente.
- Se actualizó `crates/mssql-orm/tests/ui/dbcontext_valid.rs` para fijar por compilación que la surface pública del derive ahora incluye `transaction(...)`.

### Resultado

- La crate pública `mssql-orm` ya expone `db.transaction(...)` alineado con el plan maestro, sin mover responsabilidades de ejecución fuera del adaptador Tiberius.

### Validación

- `cargo fmt --all`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- Aún no existen pruebas específicas de commit y rollback sobre SQL Server real para esta API; quedaron como siguiente subtarea explícita del backlog.

### Próximo paso recomendado

- Implementar `Etapa 8: Agregar pruebas de commit y rollback`.

### Sesión: Infraestructura transaccional base en el adaptador Tiberius

- Se detectó que el archivo del plan maestro no estaba en la raíz pedida inicialmente; la ruta real usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- La tarea original de Etapa 8 se dividió en `docs/tasks.md` para mantener entregables pequeños y verificables: infraestructura transaccional del adaptador, exposición pública de `db.transaction(...)` y pruebas explícitas de commit/rollback.
- Se movió a `En Progreso` y luego a `Completadas` la subtarea `Etapa 8: Implementar infraestructura transaccional en mssql-orm-tiberius con BEGIN, COMMIT y ROLLBACK`.
- Se añadió `crates/mssql-orm-tiberius/src/transaction.rs` con `MssqlTransaction<'a, S>`, inicio explícito de transacción y cierre explícito mediante `commit()` y `rollback()`, sin depender de `Drop` async.
- `MssqlConnection<S>` ahora expone `begin_transaction()`, devolviendo el wrapper transaccional sobre el mismo `Client<S>`.
- Se refactorizó `crates/mssql-orm-tiberius/src/executor.rs` para compartir helpers internos de ejecución parametrizada (`execute`, `query_raw`, `fetch_one`, `fetch_all`) entre conexión normal y transacción, y se implementó `Executor` también para `MssqlTransaction`.
- `crates/mssql-orm-tiberius/src/lib.rs` ahora reexporta `MssqlTransaction`, alineando la boundary pública del adaptador con la arquitectura definida en el plan.

### Resultado

- El adaptador Tiberius ya dispone de una infraestructura transaccional explícita y reutilizable, lista para que la siguiente subtarea exponga `db.transaction(...)` en la crate pública sobre esta base.

### Validación

- `cargo fmt --all`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos persistentes.
- Todavía no existe la API pública `db.transaction(...)`; esa capa quedó separada como siguiente subtarea para no mezclar infraestructura interna con surface pública en la misma sesión.

### Próximo paso recomendado

- Implementar `Etapa 8: Exponer db.transaction(...) en la crate pública reutilizando la infraestructura transaccional`.

### Sesión: Revalidación local de migraciones creadas en la raíz

- A petición del usuario, se repitió la validación real de migraciones creando temporalmente `./migrations/` en la raíz del repositorio para inspeccionar resultados locales en vez de usar un directorio temporal externo.
- La primera repetición detectó un segundo gap real: dos llamadas consecutivas a `migration add` podían producir ids con el mismo segundo base, dejando el orden final dependiente del slug y no del orden real de creación.
- Se corrigió `crates/mssql-orm-migrate/src/filesystem.rs` para generar ids con resolución de nanosegundos, eliminando la colisión observada durante la prueba.
- Tras el fix, se recrearon dos migraciones locales en secuencia (`QaCreateCustomers` y `QaAddPhone`), se generó `database update`, se aplicó el script en `tempdb` y se verificó de nuevo la tabla `qa_real_stage7.customers`, la columna incremental `phone` y la idempotencia del historial.
- Al finalizar, se eliminó otra vez `./migrations/` de la raíz para no dejar artefactos de validación dentro del repositorio.

### Resultado

- La validación local en raíz también quedó correcta y confirmó tanto el fix de batching en `database update` como el fix de orden/colisión en ids de migración.

### Validación

- `cargo test -q -p mssql-orm-migrate -p mssql-orm-cli`
- `cargo run -q --manifest-path crates/mssql-orm-cli/Cargo.toml -- migration add QaCreateCustomers`
- `cargo run -q --manifest-path crates/mssql-orm-cli/Cargo.toml -- migration add QaAddPhone`
- `cargo run -q --manifest-path crates/mssql-orm-cli/Cargo.toml -- database update`
- `sqlcmd -S localhost -U SA -P 'Ea.930318' -d tempdb -C -b -i /tmp/mssql_orm_stage7_retry.sql`
- Consultas `sqlcmd` sobre `sys.tables`, `sys.columns` y `dbo.__mssql_orm_migrations`

### Próximo paso recomendado

- Continuar con `Etapa 8: transacciones con commit en Ok y rollback en Err`.

### Sesión: Validación real de migraciones sobre SQL Server

- Se movió en `docs/tasks.md` la subtarea `Etapa 7: Validar migraciones iniciales e incrementales contra SQL Server real` a `En Progreso` antes de ejecutar la validación y luego a `Completadas` tras cerrarla.
- Se ejecutó una validación real con `sqlcmd` contra `tempdb`, usando un proyecto temporal de migraciones creado con la CLI mínima del workspace.
- La primera validación expuso un gap real en `database update`: el script envolvía todo `up.sql` en un único `EXEC(N'...')`, lo que falló al intentar ejecutar `CREATE SCHEMA` seguido de `CREATE TABLE` en la misma batch dinámica.
- Se corrigió `crates/mssql-orm-migrate/src/filesystem.rs` para dividir `up.sql` en sentencias mínimas y emitir un `EXEC(N'...')` por sentencia, manteniendo la inserción idempotente en `dbo.__mssql_orm_migrations`.
- Después del fix, se repitió la validación real completa: una migración inicial creó `qa_real_stage7.customers`, una migración incremental añadió la columna `phone`, y la reaplicación del mismo script no duplicó historial ni reejecutó cambios previos.
- Durante la sesión se detectó y eliminó un artefacto temporal previo de validación (`dbo.qa_1776961277_customers`) junto con sus filas de historial, para dejar `tempdb` consistente con la validación final correcta.

### Resultado

- La Etapa 7 quedó validada de extremo a extremo: scaffolding local, script `database update`, creación de tabla de historial, migración inicial, migración incremental e idempotencia del script acumulado sobre SQL Server real.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo run -q --manifest-path crates/mssql-orm-cli/Cargo.toml -- migration add CreateCustomers`
- `cargo run -q --manifest-path crates/mssql-orm-cli/Cargo.toml -- migration add AddPhone`
- `cargo run -q --manifest-path crates/mssql-orm-cli/Cargo.toml -- database update`
- `sqlcmd -S localhost -U SA -P 'Ea.930318' -d tempdb -C -b -i <script.sql>`
- Consultas `sqlcmd` sobre `sys.tables`, `sys.columns` y `dbo.__mssql_orm_migrations` para verificar creación inicial, cambio incremental e idempotencia

### Bloqueos

- No hubo bloqueos persistentes; el único gap detectado (`CREATE SCHEMA` dentro de una única batch dinámica) se corrigió en la misma sesión.

### Próximo paso recomendado

- Implementar `Etapa 8: transacciones con commit en Ok y rollback en Err`.

### Sesión: CLI mínima de migraciones

- Se movió en `docs/tasks.md` la subtarea `Etapa 7: Implementar CLI mínima con migration add, database update y migration list` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `mssql-orm-migrate` con `crates/mssql-orm-migrate/src/filesystem.rs`, agregando helpers para crear scaffolds de migración, listar migraciones locales y construir un script SQL acumulado de `database update`.
- `migration add` ahora crea `migrations/<timestamp>_<slug>/` con `up.sql`, `down.sql` y `model_snapshot.json`.
- `migration list` ahora enumera directorios de migración ordenados por id.
- `database update` ahora genera un script SQL acumulado que incluye la creación de `dbo.__mssql_orm_migrations` y un bloque `IF NOT EXISTS ... BEGIN ... INSERT INTO __mssql_orm_migrations ... END` por cada migración local.
- Se reemplazó el placeholder de `crates/mssql-orm-cli/src/main.rs` por una CLI mínima real, con parser simple de argumentos y delegación hacia `mssql-orm-migrate` y `mssql-orm-sqlserver`.
- Se añadió además la dependencia de `mssql-orm-sqlserver` en la CLI para reutilizar la compilación de la tabla de historial y no duplicar SQL allí.
- Se agregaron pruebas unitarias tanto en `mssql-orm-migrate` como en `mssql-orm-cli` para scaffolding, listado, construcción del script y parseo/ejecución mínima de comandos.

### Resultado

- El workspace ya dispone de una CLI mínima funcional para crear migraciones locales, listarlas y generar un script de actualización SQL Server sin volver a introducir lógica duplicada fuera de las crates correctas.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos para esta subtarea.

### Próximo paso recomendado

- Implementar `Etapa 7: Validar migraciones iniciales e incrementales contra SQL Server real`.

### Sesión: Generación SQL de migraciones e historial base

- Se movió en `docs/tasks.md` la subtarea `Etapa 7: Implementar generación SQL y tabla __mssql_orm_migrations` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió `crates/mssql-orm-sqlserver/src/migration.rs` para compilar `MigrationOperation` a DDL SQL Server y para exponer el SQL idempotente de creación de `dbo.__mssql_orm_migrations`.
- La implementación actual compila `CreateSchema`, `DropSchema`, `CreateTable`, `DropTable`, `AddColumn`, `DropColumn` y `AlterColumn` a sentencias SQL Server concretas.
- `CreateTable` reutiliza `TableSnapshot` completo para emitir columnas y primary key; `AddColumn` y `AlterColumn` reutilizan `ColumnSnapshot` para renderizar el tipo SQL Server, identidad, nullability, defaults y rowversion cuando aplica.
- `AlterColumn` quedó acotado intencionalmente a cambios básicos de tipo y nullability; cambios de default, computed SQL, identity, PK o rowversion siguen rechazándose con error explícito hasta que existan operaciones dedicadas.
- Fue necesario invertir una dependencia entre crates: `mssql-orm-migrate` ya no depende de `mssql-orm-sqlserver`, y `mssql-orm-sqlserver` ahora depende de `mssql-orm-migrate`, eliminando un ciclo que violaba la separación de responsabilidades.
- Se añadieron pruebas unitarias en `crates/mssql-orm-sqlserver/src/migration.rs` para el SQL de operaciones base, la tabla `__mssql_orm_migrations`, un `AlterColumn` soportado y el rechazo explícito de un `AlterColumn` no soportado.

### Resultado

- El workspace ya dispone de una ruta completa y verificable desde snapshots/diff/operaciones hasta SQL Server DDL, incluyendo la tabla interna de historial de migraciones.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos para esta subtarea.

### Próximo paso recomendado

- Implementar `Etapa 7: Implementar CLI mínima con migration add, database update y migration list`.

### Sesión: Batería unitaria dedicada del diff engine

- Se movió en `docs/tasks.md` la subtarea `Etapa 7: Agregar pruebas unitarias del diff engine sobre snapshots mínimos` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se consolidaron las pruebas del diff engine directamente en `crates/mssql-orm-migrate/src/diff.rs`, en un módulo `#[cfg(test)]` dedicado a esa responsabilidad.
- Se añadieron helpers mínimos de snapshots para construir schemas, tablas y columnas sin depender de metadata más amplia de la crate pública.
- La batería dedicada fija orden seguro de operaciones sobre schemas/tablas, detección de altas/bajas de columnas, alteraciones básicas, no-op sobre snapshots iguales y un caso combinado de diff completo sobre snapshots mínimos.
- Se retiró de `crates/mssql-orm-migrate/src/lib.rs` la cobertura de diff que había quedado mezclada allí, manteniendo ese archivo centrado en reexports, boundaries y contratos base.

### Resultado

- El diff engine de Etapa 7 ya quedó cubierto por una batería unitaria específica, más mantenible y con mejor trazabilidad para futuras iteraciones del sistema de migraciones.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos para esta subtarea.

### Próximo paso recomendado

- Implementar `Etapa 7: Implementar generación SQL y tabla __mssql_orm_migrations`.

### Sesión: Diff engine básico de columnas

- Se movió en `docs/tasks.md` la subtarea `Etapa 7: Implementar diff engine para columnas nuevas, eliminadas y alteraciones básicas` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm-migrate/src/diff.rs` con la función pública `diff_column_operations(previous, current)`.
- La implementación solo compara columnas de tablas presentes en ambos snapshots, para evitar duplicar trabajo ya cubierto por `CreateTable` y `DropTable`.
- El diff de columnas emite `AddColumn`, `DropColumn` y `AlterColumn` usando orden determinista por nombre de columna y comparación directa de `ColumnSnapshot`.
- Se añadieron pruebas unitarias en `crates/mssql-orm-migrate/src/lib.rs` para cubrir alta/baja de columnas, alteraciones básicas y el caso donde no debe emitirse nada porque la tabla es nueva o fue eliminada.

### Resultado

- `mssql-orm-migrate` ya cuenta con el diff básico completo del MVP inicial sobre snapshots: schemas, tablas y columnas.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos para esta subtarea.

### Próximo paso recomendado

- Implementar `Etapa 7: Agregar pruebas unitarias del diff engine sobre snapshots mínimos`, consolidando escenarios y orden estable del diff completo.

### Sesión: Diff engine básico de schemas y tablas

- Se movió en `docs/tasks.md` la subtarea `Etapa 7: Implementar diff engine para creación y eliminación de schemas y tablas` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se creó `crates/mssql-orm-migrate/src/diff.rs` con la función pública `diff_schema_and_table_operations(previous, current)`.
- La implementación compara `ModelSnapshot` con mapas ordenados (`BTreeMap`) para emitir una secuencia determinista de operaciones sobre schemas y tablas.
- El orden de salida quedó fijado para este MVP como: `CreateSchema` antes de `CreateTable`, y `DropTable` antes de `DropSchema`, evitando secuencias inválidas al aplicar operaciones.
- Se agregaron pruebas unitarias en `crates/mssql-orm-migrate/src/lib.rs` para cubrir creación/eliminación de schema completo, altas/bajas de tablas en schema existente y el caso sin cambios.

### Resultado

- `mssql-orm-migrate` ya puede producir el primer diff funcional del sistema de migraciones para schemas y tablas, sin adelantar todavía diff de columnas ni generación SQL.

### Validación

- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos para esta subtarea.

### Próximo paso recomendado

- Implementar `Etapa 7: Implementar diff engine para columnas nuevas, eliminadas y alteraciones básicas`.

### Sesión: Definición de `MigrationOperation` base

- Se movió en `docs/tasks.md` la subtarea `Etapa 7: Definir MigrationOperation y payloads básicos para schema, tabla y columna` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se creó `crates/mssql-orm-migrate/src/operation.rs` para separar el contrato de operaciones de migración del modelo de snapshots.
- Se definió `MigrationOperation` con el subset mínimo necesario para el MVP inmediato: `CreateSchema`, `DropSchema`, `CreateTable`, `DropTable`, `AddColumn`, `DropColumn` y `AlterColumn`.
- Los payloads de tabla reutilizan `TableSnapshot` completo y los payloads de columna reutilizan `ColumnSnapshot`, evitando duplicar shape mientras el diff engine aún no existe.
- Se añadieron helpers `schema_name()` y `table_name()` en `MigrationOperation` para facilitar ordenamiento, inspección y aserciones en el futuro diff engine.
- Se agregaron pruebas unitarias en `crates/mssql-orm-migrate/src/lib.rs` para fijar la superficie mínima de operaciones y la preservación explícita de `previous` y `next` en `AlterColumn`.

### Resultado

- `mssql-orm-migrate` ya tiene el contrato mínimo de operaciones sobre el que puede apoyarse el diff engine de Etapa 7 sin introducir aún SQL, CLI ni features avanzadas.

### Validación

- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos para esta subtarea.

### Próximo paso recomendado

- Implementar `Etapa 7: Implementar diff engine para creación y eliminación de schemas y tablas`, emitiendo operaciones ordenadas y deterministas.

### Sesión: Conversión desde metadata hacia `ModelSnapshot`

- Se movió en `docs/tasks.md` la subtarea `Etapa 7: Implementar conversión desde metadata de entidades hacia ModelSnapshot` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se extendió `crates/mssql-orm-migrate/src/snapshot.rs` con conversiones directas desde metadata estática del core: `ColumnSnapshot: From<&ColumnMetadata>`, `IndexColumnSnapshot: From<&IndexColumnMetadata>`, `IndexSnapshot: From<&IndexMetadata>` y `TableSnapshot: From<&EntityMetadata>`.
- Se añadió `ModelSnapshot::from_entities(&[&EntityMetadata])`, agrupando entidades por schema con `BTreeMap` y ordenando tablas por nombre para obtener snapshots deterministas e independientes del orden de entrada.
- La conversión preserva orden de columnas, nombre y columnas de primary key e índices declarados, sin adelantar todavía foreign keys, operaciones de migración ni diff engine.
- Se añadieron pruebas unitarias nuevas en `crates/mssql-orm-migrate/src/lib.rs` para fijar la conversión de `EntityMetadata -> TableSnapshot` y la agrupación/orden determinista de `ModelSnapshot`.

### Resultado

- `mssql-orm-migrate` ya puede materializar snapshots mínimos a partir de metadata code-first real del workspace, dejando lista la base para definir `MigrationOperation` y luego el diff engine.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos para esta subtarea.

### Próximo paso recomendado

- Implementar `Etapa 7: Definir MigrationOperation y payloads básicos para schema, tabla y columna`, alineando el shape mínimo con los snapshots ya fijados.

### Sesión: Definición de `ModelSnapshot` base para migraciones

- Se revisó la ruta real del plan maestro y se confirmó que la fuente de verdad vigente es `docs/plan_orm_sqlserver_tiberius_code_first.md`, no un archivo en la raíz.
- Se movió en `docs/tasks.md` la subtarea `Etapa 7: Definir ModelSnapshot y snapshots mínimos de schema, tabla, columna e índice` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se creó `crates/mssql-orm-migrate/src/snapshot.rs` con los tipos públicos `ModelSnapshot`, `SchemaSnapshot`, `TableSnapshot`, `ColumnSnapshot`, `IndexSnapshot` e `IndexColumnSnapshot`.
- El contrato de snapshot se definió con `String` y `Vec<_>` para que pueda persistirse fuera de metadata estática, pero preservando el shape SQL Server ya fijado en `core`: `SqlServerType`, `IdentityMetadata`, nullability, PK, defaults, computed SQL, rowversion, longitudes y precisión/escala.
- `TableSnapshot` retiene además `primary_key_name` y `primary_key_columns` para no perder información estructural necesaria en la siguiente subtarea de conversión desde metadata.
- Se actualizó `crates/mssql-orm-migrate/src/lib.rs` para reexportar el módulo de snapshots y se añadieron pruebas unitarias que fijan lookups por schema/tabla/columna/índice y la preservación de shape específico de SQL Server.

### Resultado

- `mssql-orm-migrate` ya tiene una base estructural real para migraciones code-first y dejó de ser únicamente un marker crate.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Bloqueos

- No hubo bloqueos para esta subtarea.

### Próximo paso recomendado

- Implementar `Etapa 7: Implementar conversión desde metadata de entidades hacia ModelSnapshot`, reutilizando directamente `EntityMetadata`, `ColumnMetadata` e `IndexMetadata` de `mssql-orm-core`.

### Sesión: Desglose detallado de la Etapa 7

- Se revisó el estado actual de `mssql-orm-migrate` y se confirmó que la tarea original de Etapa 7 seguía siendo demasiado amplia para ejecutarla como una sola unidad verificable.
- Se reestructuró `docs/tasks.md` para dividir la Etapa 7 en subtareas concretas y secuenciales: definición de `ModelSnapshot`, conversión desde metadata, definición de `MigrationOperation`, diff de schemas/tablas, diff de columnas y pruebas unitarias del diff engine.
- Se mantuvieron como tareas posteriores separadas la generación SQL de migraciones, la tabla `__mssql_orm_migrations`, la CLI y la validación real contra SQL Server.
- Se actualizó `docs/context.md` para fijar como próximo foco la primera subtarea concreta de migraciones, en lugar de la etapa completa.

### Resultado

- La Etapa 7 quedó descompuesta en entregables pequeños, trazables y cerrables, evitando arrancar implementación sobre una tarea demasiado ambigua.

### Validación

- No se ejecutaron validaciones de Cargo porque esta sesión solo modificó documentación operativa.
- Se verificó manualmente la consistencia del backlog y del nuevo foco operativo en `docs/tasks.md` y `docs/context.md`.

### Próximo paso recomendado

- Mover a `En Progreso` la subtarea `Etapa 7: Definir ModelSnapshot y snapshots mínimos de schema, tabla, columna e índice` e implementarla primero.

### Sesión: Snapshots y seguridad de parámetros del query builder público

- Se movió en `docs/tasks.md` la subtarea `Etapa 6: Agregar pruebas snapshot y de seguridad de parámetros para el query builder público` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió `insta` como `dev-dependency` de `crates/mssql-orm` para congelar el contrato observable del builder público sin afectar dependencias runtime.
- Se creó `crates/mssql-orm/tests/stage6_public_query_builder_snapshots.rs` para compilar queries construidas desde la superficie pública y fijar tanto el SQL generado como el orden de parámetros.
- Se añadió el snapshot `crates/mssql-orm/tests/snapshots/stage6_public_query_builder_snapshots__public_query_builder_compiled_select.snap`.
- Se añadió además una prueba explícita de seguridad que verifica que un valor malicioso no aparece interpolado en el SQL generado y que solo viaja en `compiled.params`, preservando además el orden de parámetros para filtro y paginación.
- Durante la validación, `insta` generó inicialmente un `.snap.new`; se revisó el contenido, se materializó el snapshot definitivo y se eliminó el archivo temporal antes de repetir la validación completa.

### Resultado

- La Etapa 6 quedó cerrada con cobertura pública completa: API fluida, pruebas unitarias del AST y snapshots/seguridad de parámetros sobre el SQL compilado desde el query builder público.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Próximo paso recomendado

- Iniciar `Etapa 7: Implementar ModelSnapshot, diff engine y operaciones básicas de migración`.

### Sesión: Pruebas unitarias públicas del query builder

- Se movió en `docs/tasks.md` la subtarea `Etapa 6: Agregar pruebas unitarias de la API pública del query builder y de la forma del AST generado` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió `crates/mssql-orm/tests/stage6_public_query_builder.rs` como prueba de integración pública enfocada en la forma del AST generado desde la superficie soportada.
- Esa prueba valida composición de predicados, ordenamiento y paginación pública mediante `EntityColumnPredicateExt`, `EntityColumnOrderExt`, `PredicateCompositionExt` y `PageRequest`.
- Se añadió `crates/mssql-orm/tests/ui/query_builder_public_valid.rs` para verificar con `trybuild` que un consumidor puede encadenar `query().filter(...).order_by(...).limit(...).paginate(...)` usando solo la API pública.
- Se actualizó `crates/mssql-orm/tests/trybuild.rs` para incluir el nuevo caso `pass` del query builder público.
- La cobertura nueva no introduce runtime extra ni depende de SQL Server real; se limita a validar contratos públicos y la forma observable del AST.

### Resultado

- La Etapa 6 ya cuenta con una batería pública específica que fija la sintaxis soportada del query builder y la estructura del AST resultante desde la perspectiva de un consumidor.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Próximo paso recomendado

- Implementar `Etapa 6: Agregar pruebas snapshot y de seguridad de parámetros para el query builder público`, compilando queries públicos a SQL Server y fijando tanto SQL como orden de parámetros.

### Sesión: Composición lógica pública de predicados

- Se movió en `docs/tasks.md` la subtarea `Etapa 6: Implementar composición lógica pública de predicados (and, or, not)` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió `crates/mssql-orm/src/predicate_composition.rs` como capa pública de composición lógica sobre `Predicate`.
- La implementación expone el trait `PredicateCompositionExt` con `and`, `or` y `not`, evitando introducir un AST alterno o mover composición al `core`.
- `and` y `or` hacen flatten de grupos lógicos existentes para evitar estructuras redundantes del tipo `And([And([...]), ...])` u `Or([Or([...]), ...])`.
- Se reexportó `PredicateCompositionExt` desde `mssql-orm` y desde la `prelude`, y se añadió cobertura unitaria específica junto con una prueba de superficie pública en `crates/mssql-orm/src/lib.rs`.
- No fue necesario modificar `mssql-orm-query` ni `mssql-orm-sqlserver`, porque el AST y la compilación ya soportaban lógica booleana; esta subtarea solo la hizo accesible desde la API pública.

### Resultado

- La superficie pública del query builder ya soporta composición lógica explícita de predicados, completando la base funcional principal de Etapa 6 sin romper límites entre crates.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Próximo paso recomendado

- Ejecutar la tarea `Etapa 6: Agregar pruebas unitarias de la API pública del query builder y de la forma del AST generado`, consolidando en una sola batería la superficie pública ya expuesta.

### Sesión: Paginación pública con `PageRequest`

- Se movió en `docs/tasks.md` la subtarea `Etapa 6: Exponer paginación pública en DbSetQuery con request explícito y contrato estable` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió `crates/mssql-orm/src/page_request.rs` con el contrato público `PageRequest { page, page_size }`.
- `PageRequest` expone `new(page, page_size)` y la conversión estable a `Pagination`, fijando en la crate pública el request explícito descrito por el plan maestro.
- Se extendió `crates/mssql-orm/src/dbset_query.rs` para exponer `DbSetQuery::paginate(PageRequest)`, reutilizando `SelectQuery::paginate` y `Pagination::page`.
- Se reexportó `PageRequest` desde `mssql-orm` y desde la `prelude`, y se añadió cobertura unitaria tanto para la conversión `PageRequest -> Pagination` como para el `SelectQuery` generado por `DbSetQuery::paginate`.
- Se eligió explícitamente no implementar en esta subtarea la variante `paginate(1, 20)` porque el backlog pedía un request explícito y contrato estable; esa sobrecarga queda fuera del alcance actual.

### Resultado

- La crate pública ya soporta paginación explícita y tipada sobre `DbSetQuery`, alineada con la forma `PageRequest` del plan maestro y sin introducir un segundo contrato de paginación.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`

### Próximo paso recomendado

- Implementar `Etapa 6: composición lógica pública de predicados (and, or, not)` sin introducir un AST paralelo.

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

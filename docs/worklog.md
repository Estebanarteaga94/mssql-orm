# Worklog

## 2026-04-25

### Sesión: verificación solicitada de rutas internas para `soft_delete`

- Se revisó nuevamente la tarea `Etapa 16+: Agregar rutas internas sin filtro implícito de soft_delete para comprobaciones de existencia y ConcurrencyConflict sin exponer bypass público accidental`.
- Se confirmó que el plan maestro no está en la raíz con el nombre solicitado; la ruta real vigente es `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- `docs/tasks.md` ya mantiene la tarea en `Completadas`, por lo que no se movió de estado.
- La implementación actual sostiene el cierre: `DbSet::query_with_internal_visibility(...)` construye `DbSetQuery` con `with_deleted()`, `find_by_sql_value_internal(...)` y `exists_by_sql_value_internal(...)` quedan encapsulados como helpers internos, y los checks de `ConcurrencyConflict` en update/delete/update por valores crudos usan esa existencia física real.
- `ActiveRecord::save(...)` también usa `exists_by_sql_value_internal(...)` para decidir insert-or-update sin abrir un bypass público.
- La surface pública conserva el comportamiento esperado: `DbSet::find(...)` usa la query pública con filtro implícito de `soft_delete`; `DbSetQuery` solo expone los escapes explícitos `with_deleted()` y `only_deleted()`.

### Resultado

- La tarea está completada y validada en el estado actual del repositorio.

### Validación

- `cargo test -p mssql-orm dbset_internal_query_visibility_bypasses_soft_delete_filter --lib -- --nocapture`
- `cargo test -p mssql-orm active_record_find_reuses_dbset_error_contract --lib -- --nocapture`
- `cargo test -p mssql-orm --test stage16_soft_delete_runtime public_soft_delete_with_rowversion_reports_concurrency_conflict_without_deleting -- --nocapture`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- La prueba runtime de SQL Server se compiló y se auto-saltó porque `MSSQL_ORM_TEST_CONNECTION_STRING` no está definido en este entorno.

### Próximo paso recomendado

- Continuar con `Etapa 16+: Diseñar tenant = TenantScope para que toda query, find, update, delete, Active Record y save_changes() sobre entidades tenant-scoped agregue automáticamente el filtro tenant_id = current_tenant cuando exista un tenant activo en el contexto`.

### Sesión: evaluación de `tenant = TenantScope`

- Se ejecutó la tarea `Etapa 16+: Evaluar tenant = TenantScope como feature de seguridad con filtros obligatorios, inserción automática de tenant_id y validación de que no existan rutas de query que omitan el tenant por accidente`.
- Se revisó el plan maestro en `docs/plan_orm_sqlserver_tiberius_code_first.md`, el backlog, `docs/context.md`, `docs/entity-policies.md` y las rutas reales de `DbSet`, `DbSetQuery`, Active Record, `SharedConnection` y tracking.
- Se movió la tarea a `En Progreso` antes de editar y a `Completadas` después de validar.
- Se documentó en `docs/entity-policies.md` que `tenant = TenantScope` es viable, pero debe implementarse como feature de seguridad fail-closed, no como simple columna generada.
- La evaluación fija que las rutas públicas tenant-scoped deben fallar antes de compilar SQL si no hay tenant activo, y deben agregar `tenant_id = current_tenant` en lecturas, updates y deletes.
- Se dejó explícito que los helpers internos no deben ignorar tenant para existencia o `ConcurrencyConflict`; a diferencia de `soft_delete`, tenant no es visibilidad configurable sino frontera de seguridad.
- Se definió el comportamiento esperado de inserts: autollenar `tenant_id` desde el contexto, rechazar valores distintos al tenant activo y fallar si falta tenant.
- Se actualizó `docs/context.md` con la decisión y el siguiente foco recomendado.

### Resultado

- `tenant = TenantScope` queda aprobado conceptualmente como backlog `Etapa 16+`, condicionado a diseñar primero el contrato de tenant activo y su propagación por `DbContext`/`SharedConnection` o provider dedicado.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- No se ejecutaron pruebas runtime porque la tarea fue de evaluación documental y no modificó código ni fixtures.

### Próximo paso recomendado

- Ejecutar `Etapa 16+: Diseñar tenant = TenantScope para que toda query, find, update, delete, Active Record y save_changes() sobre entidades tenant-scoped agregue automáticamente el filtro tenant_id = current_tenant cuando exista un tenant activo en el contexto`.

### Sesión: guardrail final de seguridad para `soft_delete`

- Se ejecutó la tarea `Etapa 16+: Cubrir soft_delete con prueba de seguridad final agregada que reúna metadata, SQL compilado y migraciones ya cubiertas, más rutas runtime públicas, para evitar regresiones de borrado físico accidental`.
- Se movió la tarea a `En Progreso` antes de editar y a `Completadas` después de validar.
- Se actualizó la cobertura de metadata y migraciones para usar `#[derive(SoftDeleteFields)]` como forma pública principal, en vez de `impl EntityPolicy` manual.
- Se agregó el test unitario `soft_delete_security_guardrail_keeps_schema_and_delete_paths_logical` en `crates/mssql-orm/src/context.rs`.
- El guardrail verifica en un solo punto que una entidad con `soft_delete` produce columnas ordinarias en `ModelSnapshot`, que activar la policy se ve como `AddColumn`, que el pipeline de migraciones crea la tabla por el camino normal y que la ruta interna de delete compila a `UPDATE`, no a `DELETE`.
- Se ejecutó también la cobertura runtime compilable de `stage16_soft_delete_runtime`; en este entorno los smokes reales se auto-saltaron porque `MSSQL_ORM_TEST_CONNECTION_STRING` no está definido.

### Resultado

- La cobertura de `soft_delete` queda cerrada localmente para API pública de campos, metadata, snapshot/diff/DDL, SQL compilado y rutas runtime de alto riesgo.
- No queda tarea pendiente específica de `soft_delete` en el backlog vivo; el próximo foco pasa a `tenant`.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm soft_delete_security_guardrail_keeps_schema_and_delete_paths_logical --lib -- --nocapture`
- `cargo test -p mssql-orm --test stage16_entity_policies soft_delete -- --nocapture`
- `cargo test -p mssql-orm --test stage16_audit_migrations soft_delete -- --nocapture`
- `cargo test -p mssql-orm --test trybuild entity_derive_ui -- --nocapture`
- `cargo test -p mssql-orm --test stage16_soft_delete_runtime -- --nocapture`
- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- Para evidencia contra SQL Server real hace falta definir `MSSQL_ORM_TEST_CONNECTION_STRING`; sin esa variable, los smokes reales compilan y se auto-saltan siguiendo el patrón existente del repositorio.

### Próximo paso recomendado

- Ejecutar `Etapa 16+: Evaluar tenant = TenantScope como feature de seguridad con filtros obligatorios, inserción automática de tenant_id y validación de que no existan rutas de query que omitan el tenant por accidente`.

### Sesión: derive público `SoftDeleteFields`

- Se corrigió una brecha de ergonomía señalada durante la revisión de `soft_delete`: aunque `#[orm(soft_delete = SoftDelete)]` ya consumía un `EntityPolicy`, el camino público esperado es que el usuario defina un struct de columnas y derive la policy, igual que con `AuditFields`.
- Se agregó `#[derive(SoftDeleteFields)]` en `mssql-orm-macros`, reutilizando la generación de columnas de policies pero con `POLICY_NAME = "soft_delete"`.
- Los campos de `SoftDeleteFields` soportan los mismos atributos estructurales de columna ya usados por policies (`column`, `length`, `nullable`, `default_sql`, `renamed_from`, `sql_type`, `precision`, `scale`, `insertable`, `updatable`).
- Los defaults de `SoftDeleteFields` son específicos de borrado lógico: `insertable = false` y `updatable = true`, para que columnas como `deleted_at` y `deleted_by` no participen en inserts normales pero sí puedan ser asignadas por la ruta de soft delete.
- Se reexportó `SoftDeleteFields` desde `mssql_orm::prelude::*`.
- Se actualizó el fixture público `entity_soft_delete_attr_valid.rs` para usar `#[derive(SoftDeleteFields)]` en vez de implementar `EntityPolicy` manualmente.
- Se agregó el fixture `soft_delete_fields_valid.rs` para validar directamente el contrato generado.
- Se actualizó `docs/entity-policies.md` y `docs/context.md` para dejar explícita la forma pública esperada.
- Se movió la subtarea a `Completadas` tras validar.

### Resultado

- La API de `soft_delete` ya permite el uso esperado:

```rust
#[derive(SoftDeleteFields)]
struct SoftDelete {
    deleted_at: Option<String>,
    deleted_by: Option<String>,
}

#[derive(Entity)]
#[orm(soft_delete = SoftDelete)]
struct Todo { /* ... */ }
```

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test trybuild entity_derive_ui -- --nocapture`
- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Ejecutar la tarea agregada final de seguridad de `soft_delete`, ahora usando `SoftDeleteFields` como forma pública en los fixtures que correspondan.

### Sesión: cobertura runtime de alto riesgo para `soft_delete`

- Se descompuso la tarea amplia `Etapa 16+: Cubrir soft_delete con pruebas de metadata, SQL compilado, CRUD público, Active Record, change tracking, ConcurrencyConflict y migraciones...` en una subtarea runtime concreta y una tarea paraguas final de seguridad.
- Se movió a `En Progreso` la subtarea `Etapa 16+: Cubrir soft_delete en rutas runtime públicas de alto riesgo: DbSet::delete, Active Record delete, change tracking save_changes() con Deleted y ConcurrencyConflict con rowversion`.
- Se extendió `crates/mssql-orm/tests/stage16_soft_delete_runtime.rs` con una entidad `VersionedSoftDeleteUser` que combina `soft_delete` y `#[orm(rowversion)]`.
- El smoke opcional contra SQL Server real ahora cubre cuatro rutas públicas: `DbSet::delete(...)`, `entity.delete(&db)` por Active Record, `remove_tracked(...)` + `save_changes()` en estado `Deleted`, y conflicto de concurrencia al intentar soft-delete con un `rowversion` stale.
- Las pruebas verifican que la fila sigue existiendo físicamente con `deleted_at` poblado cuando el borrado lógico procede, que las lecturas públicas normales la ocultan y que un `ConcurrencyConflict` no marca la fila como eliminada.
- Se mantuvo el comportamiento de auto-skip cuando `MSSQL_ORM_TEST_CONNECTION_STRING` no está definido, consistente con las pruebas reales existentes del repositorio.
- Se movió la subtarea runtime a `Completadas` tras validar.

### Resultado

- Las rutas públicas de mayor riesgo de `soft_delete` ya tienen cobertura compilable y ejecutable contra SQL Server real cuando el entorno aporta connection string.
- La tarea paraguas final queda pendiente para agregar una prueba o checklist de seguridad agregada que reúna metadata, SQL compilado, migraciones y runtime, evitando que la cobertura quede dispersa.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test stage16_soft_delete_runtime -- --nocapture`
- `cargo fmt --all --check`
- `cargo test -p mssql-orm dbset_delete_compiled_query --lib -- --nocapture`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- No se ejecutó SQL Server real porque `MSSQL_ORM_TEST_CONNECTION_STRING` no está definido en este entorno; los cuatro tests de integración compilaron y se auto-saltaron según el patrón existente.

### Próximo paso recomendado

- Ejecutar `Etapa 16+: Cubrir soft_delete con prueba de seguridad final agregada que reúna metadata, SQL compilado y migraciones ya cubiertas, más rutas runtime públicas, para evitar regresiones de borrado físico accidental`.

### Sesión: `soft_delete` en snapshots, diff y DDL

- Se ejecutó la subtarea `Etapa 16+: Integrar soft_delete en snapshots, diff y DDL como columnas ordinarias sin abrir un segundo pipeline de esquema`.
- Se movió la tarea a `En Progreso` antes de editar y a `Completadas` después de validar.
- Se confirmó que el plan maestro no está en la raíz con el nombre solicitado; la ruta operativa sigue siendo `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- La implementación no requirió lógica nueva de migraciones: `#[orm(soft_delete = SoftDelete)]` ya expande columnas de la policy como `ColumnMetadata` ordinarias, igual que `audit`.
- Se agregó cobertura en `crates/mssql-orm/tests/stage16_entity_policies.rs` para confirmar que `ModelSnapshot::from_entities(...)` incluye `deleted_at` y `deleted_by`, preserva flags de persistencia, tipo, longitud y roundtrip JSON.
- Se agregó cobertura en `crates/mssql-orm/tests/stage16_audit_migrations.rs` para confirmar que una entidad nueva con `soft_delete` genera `CREATE TABLE` con columnas de borrado lógico, que activar la policy sobre una tabla existente emite `AddColumn` y que quitarla emite `DropColumn`.
- Se agregó snapshot SQL en `crates/mssql-orm-sqlserver/tests/migration_snapshots.rs` y `crates/mssql-orm-sqlserver/tests/snapshots/migration_snapshots__soft_delete_column_migration_sql.snap` para fijar el DDL SQL Server de `ALTER TABLE ... ADD` sobre columnas `soft_delete`.
- Se actualizó `docs/context.md` con el estado operativo: `soft_delete` ya queda cubierto en metadata, snapshots, diff y DDL sin pipeline de esquema especial.

### Resultado

- `soft_delete` queda integrado y validado como columnas ordinarias de schema. La tarea amplia pendiente pasa a ser cobertura integral de runtime/CRUD/Active Record/tracking/ConcurrencyConflict/migraciones para asegurar que ninguna ruta siga haciendo borrado físico por accidente.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p mssql-orm --test stage16_entity_policies model_snapshot_includes_soft_delete_columns_without_special_pipeline -- --nocapture`
- `cargo test -p mssql-orm --test stage16_audit_migrations soft_delete -- --nocapture`
- `cargo test -p mssql-orm-sqlserver --test migration_snapshots snapshots_soft_delete_column_migration_sql -- --nocapture`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- `cargo-insta` no está instalado en este entorno; el snapshot nuevo se revisó y aceptó manualmente.

### Próximo paso recomendado

- Ejecutar `Etapa 16+: Cubrir soft_delete con pruebas de metadata, SQL compilado, CRUD público, Active Record, change tracking, ConcurrencyConflict y migraciones para evitar que alguna ruta siga haciendo borrado físico por accidente`.

### Sesión: revisión de cierre de rutas internas para `soft_delete`

- Se revisó si la subtarea `Etapa 16+: Agregar rutas internas sin filtro implícito de soft_delete para comprobaciones de existencia y ConcurrencyConflict sin exponer bypass público accidental` estaba realmente completada.
- Se confirmó que el plan maestro no está en la raíz con el nombre solicitado; la ruta operativa sigue siendo `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- La tarea ya figura en `docs/tasks.md` como completada y el código lo respalda: `DbSet` tiene `query_with_internal_visibility(...)`, `find_by_sql_value_internal(...)` y `exists_by_sql_value_internal(...)` como rutas internas que fuerzan `with_deleted()`.
- Los checks de `ConcurrencyConflict` en `DbSet::update(...)`, `DbSet::delete_by_sql_value(...)` y `update_entity_values_by_sql_value(...)` usan `exists_by_sql_value_internal(...)`, por lo que no dependen del filtro público por defecto de `soft_delete`.
- `ActiveRecord::save(...)` también usa `exists_by_sql_value_internal(...)` para decidir existencia real en modo insert-or-update, sin exponer una API pública de bypass.
- La ruta pública `DbSet::find(...)` sigue usando `query_with(...).first()` y por tanto conserva la visibilidad pública por defecto; `DbSetQuery` solo expone los escapes explícitos `with_deleted()` y `only_deleted()`.
- Se detectó que `docs/tasks.md` ya tenía un cambio pendiente ajeno a esta revisión, limitado a mover el encabezado `## Pendientes`; no se modificó ese archivo en esta sesión.

### Resultado

- La tarea revisada puede considerarse completada: hay implementación interna, cobertura unitaria focal y trazabilidad previa en `docs/worklog.md`.

### Validación

- `cargo test -p mssql-orm dbset_internal_query_visibility_bypasses_soft_delete_filter --lib -- --nocapture`
- `cargo test -p mssql-orm active_record_find_reuses_dbset_error_contract --lib -- --nocapture`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Continuar con `Etapa 16+: Integrar soft_delete en snapshots, diff y DDL como columnas ordinarias sin abrir un segundo pipeline de esquema`.

### Sesión: evaluación de `concurrency = RowVersion`

- Se ejecutó la subtarea `Etapa 16+: Evaluar concurrency = RowVersion como política declarativa sobre el soporte existente de #[orm(rowversion)], sin romper ConcurrencyConflict`.
- Se confirmó que el plan maestro no está en la raíz con el nombre solicitado; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se revisó `docs/instructions.md`, `docs/tasks.md`, `docs/worklog.md`, `docs/context.md`, `docs/entity-policies.md`, el plan maestro y las rutas reales de concurrencia en `crates/mssql-orm/src/context.rs`, `active_record.rs`, `lib.rs` y pruebas asociadas.
- Se movió la tarea a `En Progreso` antes de editar y a `Completadas` después de validar.
- Se actualizó `docs/entity-policies.md` con la decisión: no implementar `concurrency = RowVersion` como `Entity Policy`.
- La razón técnica quedó documentada: la concurrencia optimista necesita un campo Rust visible `#[orm(rowversion)] version: Vec<u8>` para materializar el token devuelto por SQL Server y reutilizarlo en `Changeset::concurrency_token()`, `EntityPersist::concurrency_token()`, Active Record y `save_changes()`.
- Se actualizó `docs/context.md` para reflejar que `#[orm(rowversion)]` sigue siendo la API canónica y que `OrmError::ConcurrencyConflict` debe preservarse en las rutas actuales.
- Se actualizó `docs/tasks.md`.

### Resultado

- La concurrencia optimista queda explícitamente fuera de `Entity Policies`. No hay API nueva pendiente para `concurrency = RowVersion`; el siguiente foco del backlog vivo pasa a `soft_delete = SoftDelete`.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- No se ejecutó `cargo test --workspace` porque la tarea fue de evaluación/diseño documental y no cambió código, macros ni fixtures.

### Próximo paso recomendado

- Ejecutar `Etapa 16+: Evaluar soft_delete = SoftDelete como cambio semántico explícito de delete, entity.delete(&db), queries por defecto y migraciones, documentando sus riesgos antes de implementarlo`.

### Sesión: retirar policy temporal del roadmap

- Se atendió la decisión de producto de no implementar una policy temporal separada porque `audit = Audit` ya puede cubrir el caso con un struct reducido que solo declare `created_at` y `updated_at`.
- Se retiró esa línea del backlog vivo en `docs/tasks.md`.
- Se actualizó `docs/entity-policies.md` para presentar `audit = Audit` como el mecanismo único de columnas generadas del MVP y eliminar la sección de diseño futuro de la policy temporal.
- Se actualizó `README.md` y `docs/context.md` para no listar esa policy como diseño futuro ni como próximo enfoque recomendado.
- Se mantuvo el código sin cambios.

### Resultado

- El roadmap vivo queda simplificado: no hay implementación pendiente para una policy temporal separada; el siguiente foco vuelve a `concurrency = RowVersion`, `soft_delete`, `tenant` o `AuditProvider` según prioridad.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- No se ejecutó `cargo test --workspace` porque el cambio fue documental y no modificó código ni fixtures.

### Próximo paso recomendado

- Ejecutar `Etapa 16+: Evaluar concurrency = RowVersion como política declarativa sobre el soporte existente de #[orm(rowversion)], sin romper ConcurrencyConflict`.

### Sesión: evaluación de `timestamps = Timestamps`

- Se ejecutó la subtarea `Etapa 16+: Evaluar timestamps = Timestamps como política separada o alias simplificado de audit, evitando solapamientos de columnas con audit`.
- Se confirmó que el plan maestro no está en la raíz con el nombre solicitado; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se revisó `docs/instructions.md`, `docs/tasks.md`, `docs/worklog.md`, `docs/context.md`, `docs/entity-policies.md`, el plan maestro y la implementación actual de `AuditFields`/`#[orm(audit = ...)]` en `crates/mssql-orm-macros/src/lib.rs`.
- Se movió la tarea a `En Progreso` antes de editar y a `Completadas` después de validar.
- Se actualizó `docs/entity-policies.md` con la decisión: `timestamps = Timestamps` debe ser una policy separada de columnas generadas, no un alias de `audit`.
- La decisión fija un derive futuro `#[derive(TimestampFields)]`, consumo explícito con `#[orm(timestamps = Timestamps)]`, reutilización de `EntityPolicy`, ausencia de autollenado runtime, ausencia de campos Rust visibles/símbolos asociados y validaciones obligatorias de colisión contra columnas propias y `audit = Audit`.
- Se descompuso el backlog en subtareas verificables para diseño, parser de entidad, derive, colisiones y cobertura de `timestamps`.
- Se actualizó `docs/context.md` con el nuevo estado operativo y el siguiente paso recomendado.

### Resultado

- `timestamps = Timestamps` queda decidido como extensión `Etapa 16+` separada de `audit`, con contrato esperado y backlog listo para implementación posterior sin introducir todavía API compilable.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- No se ejecutó `cargo test --workspace` porque la tarea fue de evaluación/diseño documental y no cambió código, macros ni fixtures.

### Próximo paso recomendado

- Ejecutar `Etapa 16+: Diseñar TimestampFields como derive dedicado para timestamps = Timestamps, reutilizando EntityPolicy sin introducir alias implícito de audit`.

### Sesión: mutación futura de `Vec<ColumnValue>` por `AuditProvider`

- Se ejecutó la subtarea `Etapa 16+: Definir cómo AuditProvider debe modificar Vec<ColumnValue> en insert/update sin duplicar la lógica existente de Insertable, Changeset, EntityPersist, Active Record ni change tracking`.
- Se confirmó que el plan maestro no está en la raíz; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se revisó `docs/instructions.md`, `docs/tasks.md`, `docs/worklog.md`, `docs/context.md`, `docs/entity-policies.md`, el plan maestro y las rutas reales de persistencia en `crates/mssql-orm/src/context.rs`, `active_record.rs` y `tracking.rs`.
- Se movió la tarea a `En Progreso` antes de editar y a `Completadas` después de validar.
- Se actualizó `docs/entity-policies.md` con la sección `Mutacion de Vec<ColumnValue>`.
- El diseño fija una única transformación interna futura en `mssql-orm` para recibir valores explícitos de `Insertable`, `Changeset` o `EntityPersist`, completar columnas auditables faltantes y devolver valores normalizados antes de construir `InsertQuery` o `UpdateQuery`.
- La transformación debe preservar precedencia de valores explícitos del usuario, respetar `insertable`/`updatable`, detectar columnas duplicadas como error, no tocar concurrencia/rowversion y no inferir auditoría por nombres mágicos.
- Se dejó explícito que `DbSet::insert`, `DbSet::update`, Active Record y `save_changes()` deben converger en esa transformación mediante `insert_entity_values(...)` y `update_entity_values_by_sql_value(...)`.
- Se registró que una implementación runtime posterior necesitará un contrato auxiliar generado por `#[orm(audit = Audit)]` para exponer el slice de columnas auditables, sin alterar snapshots, diff ni DDL.
- Se actualizó `docs/context.md` con el nuevo estado operativo.
- Se actualizó `docs/tasks.md`.

### Resultado

- Quedó definido el punto y algoritmo de mutación futura de `Vec<ColumnValue>` por `AuditProvider`, sin implementar autollenado runtime ni cambiar APIs actuales.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- No se ejecutó `cargo test --workspace` porque la tarea fue documental y no modificó código, macros ni fixtures.

### Próximo paso recomendado

- Ejecutar `Etapa 16+: Evaluar timestamps = Timestamps como política separada o alias simplificado de audit, evitando solapamientos de columnas con audit`.

### Sesión: revisión de acoplamiento de `AuditProvider`

- Se revisó el diseño documental de `AuditProvider` contra la implementación actual de persistencia.
- Se confirmó que el commit previo fue documental y no introdujo código runtime ni dependencias nuevas.
- Se contrastó el diseño con los contratos existentes: `Insertable`, `Changeset`, `EntityPersist`, `DbSet`, `RawInsertable`, `RawChangeset`, `DbContext::transaction`, `mssql-orm-query`, `mssql-orm-sqlserver` y `mssql-orm-tiberius`.
- Se agregó en `docs/entity-policies.md` la sección `Acoplamiento con la implementacion actual`.
- La sección fija que el autollenado futuro debe vivir en la capa pública de persistencia de `mssql-orm`, donde convergen `DbSet::insert`, `DbSet::update`, Active Record y `save_changes()`.
- También deja explícito que los derives deben seguir siendo conversiones puras, `query` no debe conocer policies, `sqlserver` no debe distinguir origen de valores y `tiberius` no debe interpretar metadata de auditoría.
- Se actualizó `docs/context.md` con este criterio.

### Resultado

- El diseño de `AuditProvider` queda acoplado al proyecto en términos de puntos reales de integración, sin romper límites entre crates ni adelantar implementación runtime.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Ejecutar `Etapa 16+: Definir cómo AuditProvider debe modificar Vec<ColumnValue> en insert/update sin duplicar la lógica existente de Insertable, Changeset, EntityPersist, Active Record ni change tracking`.

### Sesión: diseño conceptual de `AuditProvider`

- Se ejecutó la subtarea `Etapa 16+: Diseñar AuditProvider para autollenado futuro, incluyendo now, usuario actual, valores por request, integración con DbContext y comportamiento dentro de transacciones`.
- Se confirmó que el plan maestro no está en la raíz; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se revisó `docs/instructions.md`, `docs/tasks.md`, `docs/worklog.md`, `docs/context.md`, `docs/entity-policies.md`, `docs/code-first.md`, `docs/api.md`, `README.md` y el plan maestro.
- Se movió la tarea a `En Progreso` antes de editar y a `Completadas` después de validar.
- Se agregó en `docs/entity-policies.md` la sección `Diseno futuro de AuditProvider`.
- El diseño separa metadata y runtime: `audit = Audit` sigue declarando columnas, mientras `AuditProvider` solo resolvería valores para columnas auditables existentes.
- Se documentó un shape conceptual con `AuditContext`, `AuditOperation`, `AuditRequestValues` y `AuditProvider`.
- Se definieron reglas esperadas para `now`, usuario actual, valores por request, integración con `DbContext` y herencia dentro de `db.transaction(...)`.
- Se dejó explícito que no se implementa autollenado runtime en esta sesión y que la mutación de `Vec<ColumnValue>` queda para la siguiente tarea.
- Se actualizó `docs/context.md` con el nuevo entendimiento operativo y el próximo foco recomendado.
- Se actualizó `docs/tasks.md`.

### Resultado

- `AuditProvider` queda diseñado a nivel conceptual para autollenado futuro, sin cambios de código ni modificación de rutas runtime de persistencia.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- No se ejecutó `cargo test --workspace` porque la tarea fue documental y no cambió código ni fixtures.

### Próximo paso recomendado

- Ejecutar `Etapa 16+: Definir cómo AuditProvider debe modificar Vec<ColumnValue> en insert/update sin duplicar la lógica existente de Insertable, Changeset, EntityPersist, Active Record ni change tracking`.

### Sesión: validación local mínima de cierre de Etapa 16

- Se ejecutó la subtarea `Etapa 16: Ejecutar validación local mínima antes de cerrar: cargo fmt --all --check, cargo check --workspace, tests trybuild afectados y pruebas unitarias de core, macros, migrate y sqlserver relacionadas`.
- Se confirmó que el plan maestro no está en la raíz; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se revisó `docs/instructions.md`, `docs/tasks.md`, `docs/worklog.md`, `docs/context.md` y el plan maestro.
- Se movió la tarea a `En Progreso` antes de ejecutar validaciones y a `Completadas` después de que todas pasaran.
- Se ejecutó el set de validación local mínima definido para cerrar Etapa 16: formato, check de workspace, `trybuild` afectado, pruebas de metadata/persistencia de policies, pruebas de migraciones auditadas y pruebas unitarias de `core`, `macros`, `migrate` y `sqlserver`.
- Se actualizó `docs/context.md` para reflejar que Etapa 16 queda cerrada y que `AuditProvider`, `timestamps`, `concurrency = RowVersion`, `soft_delete` y `tenant` permanecen como backlog `Etapa 16+`.

### Resultado

- Etapa 16 queda cerrada localmente: el MVP de `Entity Policies` para auditoría de metadata/schema está documentado, cubierto y validado.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm --test trybuild entity_derive_ui`
- `cargo test -p mssql-orm --test stage16_entity_policies`
- `cargo test -p mssql-orm --test stage16_audit_migrations`
- `cargo test -p mssql-orm-core`
- `cargo test -p mssql-orm-macros`
- `cargo test -p mssql-orm-migrate`
- `cargo test -p mssql-orm-sqlserver`

### Bloqueos

- No hubo bloqueos técnicos.
- No se ejecutó `cargo test --workspace` completo porque la tarea pedía validación local mínima enfocada y ya se cubrieron los tests afectados por Etapa 16 junto con las crates relacionadas.

### Próximo paso recomendado

- Si se continúa con `Etapa 16+`, empezar por diseñar `AuditProvider` para autollenado futuro sin modificar todavía rutas runtime de persistencia.

### Sesión: cierre de contexto de Etapa 16

- Se ejecutó la subtarea `Etapa 16: Actualizar docs/context.md al cerrar la etapa con decisiones reales, límites, tests ejecutados y cualquier tradeoff de API pública`.
- Se confirmó nuevamente que el plan maestro no está en la raíz; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se revisó `docs/instructions.md`, `docs/tasks.md`, `docs/worklog.md`, `docs/context.md`, `docs/entity-policies.md`, `README.md` y el plan maestro.
- Se agregó en `docs/context.md` un cierre operativo explícito para Etapa 16.
- El cierre fija la decisión real: `Entity Policies` queda disponible como evolución `code-first`, pero el único MVP implementado es `audit = Audit` como metadata/schema mediante `#[derive(AuditFields)]` y `#[orm(audit = Audit)]`.
- Se documentó el tradeoff público: las columnas auditables no son campos Rust visibles, no generan símbolos asociados como `Todo::created_at`, no se materializan en `FromRow` y no se autollenan en `Insertable`, `Changeset`, Active Record ni `save_changes()`.
- Se registraron en contexto las validaciones relevantes ya ejecutadas durante la etapa: `trybuild`, `stage16_entity_policies`, `stage16_audit_migrations`, validaciones del ejemplo `todo-app`, exportador `model_snapshot`, `migration_e2e.sh`, `cargo fmt --all --check` y `cargo check --workspace`.
- Se actualizó el próximo enfoque recomendado hacia la validación local mínima de cierre de Etapa 16.
- Se actualizó `docs/tasks.md` moviendo la tarea a `Completadas`.

### Resultado

- `docs/context.md` ya refleja el estado operativo de cierre de Etapa 16, con decisiones, límites, validaciones y extensiones diferidas.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- No se ejecutó `cargo test --workspace` porque la tarea fue documental; la validación amplia queda como subtarea pendiente explícita de cierre.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Ejecutar validación local mínima antes de cerrar: cargo fmt --all --check, cargo check --workspace, tests trybuild afectados y pruebas unitarias de core, macros, migrate y sqlserver relacionadas`.

### Sesión: README y roadmap de Entity Policies

- Se ejecutó la subtarea `Etapa 16: Actualizar README.md y/o documentación de roadmap para presentar Entity Policies como evolución code-first, aclarando qué está implementado y qué queda diferido`.
- Se confirmó que el plan maestro no está en la raíz; la ruta operativa usada como fuente de verdad fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se revisó `README.md`, `docs/entity-policies.md`, `docs/context.md`, `docs/tasks.md`, `docs/worklog.md` y el plan maestro.
- Se actualizó `README.md` para incluir `Entity Policies` dentro de la surface disponible, enlazar `docs/entity-policies.md`, agregar una sección dedicada y corregir el estado real: Etapa 15 cerrada y Etapa 16 en cierre.
- La sección nueva aclara que `#[derive(AuditFields)]` y `#[orm(audit = Audit)]` ya están implementados como columnas de metadata/schema, integradas con snapshots, diff, DDL y migraciones.
- También deja explícitamente diferidos el autollenado runtime, campos Rust visibles, símbolos como `Todo::created_at`, `timestamps`, `soft_delete`, `tenant` y `AuditProvider`.
- Se actualizó `docs/entity-policies.md` para retirar la advertencia obsoleta de que la feature no estaba disponible todavía.
- Se actualizó `docs/context.md` con el estado operativo posterior a esta documentación pública.
- Se actualizó `docs/tasks.md` moviendo la tarea a `Completadas`.

### Resultado

- El README y la documentación de roadmap ya presentan `Entity Policies` como evolución code-first disponible para auditoría de metadata/schema, sin prometer comportamiento runtime fuera del MVP.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- No se ejecutó `cargo test --workspace` porque la tarea fue documental y no modificó código ni fixtures.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Actualizar docs/context.md al cerrar la etapa con decisiones reales, límites, tests ejecutados y cualquier tradeoff de API pública`.

### Sesión: documentar auditoría en code-first

- Se ejecutó la subtarea `Etapa 16: Actualizar docs/code-first.md con la sintaxis #[orm(audit = Audit)], límites del MVP y ejemplo compilable respaldado por fixture trybuild`.
- Se revisó `docs/code-first.md`, `docs/entity-policies.md` y los fixtures `crates/mssql-orm/tests/ui/entity_audit_public_valid.rs` y `audit_fields_valid.rs`.
- Se agregó `audit` a la lista de atributos soportados por `#[derive(Entity)]`.
- Se agregó una sección de `Entity Policies` en `docs/code-first.md` con ejemplo de `#[derive(AuditFields)]` y uso de `#[orm(audit = Audit)]`.
- La guía deja explícito que las columnas auditables se expanden como `ColumnMetadata` normales para snapshots, diff y DDL.
- La guía enlaza el ejemplo con el fixture compilable `entity_audit_public_valid.rs`, que usa únicamente `mssql_orm::prelude::*`.
- Se documentaron los límites del MVP: sin campos Rust visibles, sin símbolos como `Todo::created_at`, sin autollenado runtime y con colisiones rechazadas en compile-time.
- Se actualizó `docs/tasks.md`, `docs/worklog.md` y `docs/context.md`.

### Resultado

- La guía code-first pública ya documenta la sintaxis `#[orm(audit = Audit)]` y sus límites reales, alineada con cobertura `trybuild`.

### Validación

- `cargo test -p mssql-orm --test trybuild entity_derive_ui`
- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Actualizar README.md y/o documentación de roadmap para presentar Entity Policies como evolución code-first, aclarando qué está implementado y qué queda diferido`.

### Sesión: auditoría sin autollenado runtime en MVP

- Se ejecutó la subtarea `Etapa 16: Mantener fuera del MVP el autollenado de created_by, updated_by, created_at y updated_at desde DbSet::insert, DbSet::update, Active Record y save_changes`.
- Se revisaron las rutas de persistencia de `DbSet`, Active Record, `EntityPersist` y `save_changes`.
- No se agregó `AuditProvider`, hooks runtime ni modificación de `ColumnValue` en las rutas de escritura.
- Se amplió `crates/mssql-orm/tests/stage16_entity_policies.rs` con cobertura explícita para una entidad auditada.
- La prueba `insertable_and_changeset_do_not_auto_fill_audit_columns` confirma que `Insertable` y `Changeset`, usados por `DbSet::insert` y `DbSet::update`, solo producen columnas explícitas del payload (`name`, `status`) y no columnas auditables.
- La prueba `entity_persist_for_active_record_and_save_changes_does_not_auto_fill_audit_columns` confirma que `EntityPersist`, usado por Active Record y `save_changes`, solo produce campos reales de la entidad y no columnas auditables generadas por `AuditFields`.
- Se actualizó `docs/tasks.md`, `docs/worklog.md` y `docs/context.md`.

### Resultado

- El límite del MVP queda cubierto por pruebas: `audit = Audit` aporta metadata/schema, pero no autollenado runtime en inserciones, actualizaciones, Active Record ni tracking.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test stage16_entity_policies`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm --test stage16_audit_migrations`
- `cargo test -p mssql-orm --test trybuild entity_derive_ui`

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Actualizar docs/code-first.md con la sintaxis #[orm(audit = Audit)], límites del MVP y ejemplo compilable respaldado por fixture trybuild`.

### Sesión: validación snapshot-bin con columnas auditables en todo-app

- Se ejecutó la subtarea `Etapa 16: Agregar binario/exportador de snapshot del ejemplo actualizado y validar que migration add --snapshot-bin ... capture columnas auditables en model_snapshot.json`.
- Se revisó el exportador existente `examples/todo-app/src/bin/model_snapshot.rs`, el script `examples/todo-app/scripts/migration_e2e.sh` y la documentación del ejemplo.
- No fue necesario crear un binario nuevo: `model_snapshot` ya exporta `TodoAppDbContext`, que ahora incluye `audit_events: DbSet<AuditEvent>`.
- Se reforzó `migration_e2e.sh` para validar que la migración inicial generada por `migration add --snapshot-bin model_snapshot --manifest-path examples/todo-app/Cargo.toml` contenga en `model_snapshot.json` la tabla `audit_events` y las columnas `created_at`, `created_by_user_id`, `updated_at` y `updated_by`.
- El mismo script valida que `up.sql` contenga `CREATE TABLE [todo].[audit_events]`.
- Se actualizó `examples/todo-app/README.md` para dejar documentada esa validación.
- Se actualizó `docs/tasks.md`, `docs/worklog.md` y `docs/context.md`.

### Resultado

- El flujo reproducible del ejemplo ahora prueba que `migration add --snapshot-bin ...` captura columnas auditables dentro del `model_snapshot.json` versionado.

### Validación

- `examples/todo-app/scripts/migration_e2e.sh`
- `cargo fmt --manifest-path examples/todo-app/Cargo.toml --all --check`
- `cargo check --manifest-path examples/todo-app/Cargo.toml`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- `migration_e2e.sh` omitió la aplicación real con `sqlcmd` porque `MSSQL_ORM_SQLCMD_SERVER`, `MSSQL_ORM_SQLCMD_USER` y `MSSQL_ORM_SQLCMD_PASSWORD` no están configuradas; la tarea solo requería validar el snapshot y artefactos locales generados.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Mantener fuera del MVP el autollenado de created_by, updated_by, created_at y updated_at desde DbSet::insert, DbSet::update, Active Record y save_changes`.

### Sesión: entidad auditada en todo-app

- Se ejecutó la subtarea `Etapa 16: Actualizar el ejemplo todo-app o agregar fixture dedicado para mostrar al menos una entidad con #[orm(audit = Audit)] sin degradar el smoke existente`.
- Se revisó `docs/entity-policies.md`, el dominio del ejemplo, `TodoAppDbContext`, el exportador `model_snapshot` y el fixture SQL de smoke.
- Se decidió no aplicar `audit = TodoAudit` sobre `User`, `TodoList` ni `TodoItem` porque esas entidades ya tienen campos manuales como `created_at`; hacerlo activaría correctamente la validación de colisiones del macro.
- Se agregó `TodoAudit` con `#[derive(AuditFields)]` en `examples/todo-app/src/domain.rs`.
- Se agregó la entidad `AuditEvent` con `#[orm(table = "audit_events", schema = "todo", audit = TodoAudit)]`.
- Se incorporó `audit_events: DbSet<AuditEvent>` en `TodoAppDbContext`.
- Se reexportaron `AuditEvent` y `TodoAudit` desde la librería del ejemplo.
- Se agregó cobertura unitaria de metadata para confirmar que las columnas auditables se expanden en orden estable dentro de `AuditEvent::metadata()`.
- Se actualizó `examples/todo-app/scripts/smoke_setup.sql` para crear `todo.audit_events` sin insertar datos ni tocar endpoints existentes.
- Se actualizó `examples/todo-app/README.md` para documentar la entidad auditada del ejemplo.
- Se confirmó con el exportador `model_snapshot` que `audit_events`, `created_by_user_id` y `updated_by` aparecen en el snapshot generado.
- Se actualizó `docs/tasks.md`, `docs/worklog.md` y `docs/context.md`.

### Resultado

- `todo-app` ahora muestra una entidad code-first con `#[orm(audit = TodoAudit)]` sin cambiar los endpoints HTTP ni las queries de smoke existentes.
- El autollenado runtime de auditoría sigue fuera del MVP; la entidad muestra columnas de metadata/schema.

### Validación

- `cargo fmt --manifest-path examples/todo-app/Cargo.toml --all`
- `cargo fmt --manifest-path examples/todo-app/Cargo.toml --all --check`
- `cargo check --manifest-path examples/todo-app/Cargo.toml`
- `cargo test --manifest-path examples/todo-app/Cargo.toml`
- `cargo test --manifest-path examples/todo-app/Cargo.toml domain::tests::audit_event_metadata_expands_reusable_audit_policy_columns`
- `cargo clippy --manifest-path examples/todo-app/Cargo.toml --all-targets --all-features`
- `cargo check --workspace`
- `cargo run --manifest-path examples/todo-app/Cargo.toml --bin model_snapshot`

### Bloqueos

- No hubo bloqueos técnicos.
- `cargo clippy --manifest-path examples/todo-app/Cargo.toml --all-targets --all-features` terminó con código 0, pero mantiene advertencias no bloqueantes preexistentes en el ejemplo (`manual_async_fn`, `bool_assert_comparison`, `iter_overeager_cloned`).
- No se ejecutó smoke real contra SQL Server porque esta subtarea no requería `DATABASE_URL`; el fixture SQL quedó actualizado para que el smoke manual siga reflejando el modelo.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Agregar binario/exportador de snapshot del ejemplo actualizado y validar que migration add --snapshot-bin ... capture columnas auditables en model_snapshot.json`.

### Sesión: validación final de release Etapa 15

- Se ejecutó la subtarea `Etapa 15: Ejecutar validación final de release sobre workspace y ejemplos documentados`.
- Se confirmó que el plan maestro no está en la raíz; la ruta real vigente es `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- La tarea ya estaba movida a `En Progreso` al iniciar la sesión, por lo que se continuó desde ese estado.
- Se revisó la documentación operativa, el README principal, el índice de ejemplos y el `Cargo.toml` independiente de `examples/todo-app`.
- La primera corrida de `cargo test --workspace` falló en tres fixtures `trybuild` válidos porque los binarios generados bajo `target/tests/trybuild` eran archivos de ceros y el sistema respondió `cannot execute binary file`.
- Se limpió solo el directorio generado `target/tests/trybuild` y se repitió `cargo test -p mssql-orm --test trybuild`; el test pasó, confirmando que el fallo era un artefacto corrupto de build y no un problema de fuente.
- Se aplicó `cargo fmt --manifest-path examples/todo-app/Cargo.toml --all` porque el ejemplo tenía un ajuste pendiente de formato en `examples/todo-app/src/lib.rs`.
- Se validó el exportador `examples/todo-app` con `cargo run --manifest-path examples/todo-app/Cargo.toml --bin model_snapshot`.
- Se ejecutó `examples/todo-app/scripts/migration_e2e.sh`, que generó migración inicial, migración incremental no-op y `database_update.sql` en un directorio temporal.
- El apply real con `sqlcmd` del script del ejemplo se omitió automáticamente porque `MSSQL_ORM_SQLCMD_SERVER`, `MSSQL_ORM_SQLCMD_USER` y `MSSQL_ORM_SQLCMD_PASSWORD` no están configuradas.
- Se actualizó `docs/tasks.md`, `docs/worklog.md` y `docs/context.md`.

### Resultado

- La validación final de release de Etapa 15 queda cerrada sobre el workspace y el ejemplo documentado `todo-app`.
- No se introdujeron cambios funcionales; el único cambio de código fue formato en el ejemplo.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm --test trybuild` tras limpiar `target/tests/trybuild`
- `cargo test --workspace`
- `cargo clippy --workspace --all-targets --all-features`
- `cargo fmt --manifest-path examples/todo-app/Cargo.toml --all --check`
- `cargo check --manifest-path examples/todo-app/Cargo.toml`
- `cargo test --manifest-path examples/todo-app/Cargo.toml`
- `cargo clippy --manifest-path examples/todo-app/Cargo.toml --all-targets --all-features`
- `cargo run --manifest-path examples/todo-app/Cargo.toml --bin model_snapshot`
- `examples/todo-app/scripts/migration_e2e.sh`

### Bloqueos

- No hubo bloqueos permanentes.
- `cargo clippy --workspace --all-targets --all-features` terminó con código 0, pero mantiene advertencias preexistentes en `mssql-orm-migrate/src/diff.rs` (`collapsible_if`) y `mssql-orm/src/context.rs` (`large_enum_variant`).
- `cargo clippy --manifest-path examples/todo-app/Cargo.toml --all-targets --all-features` terminó con código 0, pero reportó advertencias no bloqueantes en el ejemplo (`manual_async_fn`, `bool_assert_comparison`, `iter_overeager_cloned`).
- La prueba ignorada `smoke_preview_query_runs_against_sql_server_fixture` del ejemplo no se ejecutó porque requiere `DATABASE_URL` y fixture SQL Server explícitos.
- `migration_e2e.sh` no aplicó el script contra SQL Server por falta de variables `MSSQL_ORM_SQLCMD_*`.

### Próximo paso recomendado

- Continuar con `Etapa 16: Actualizar el ejemplo todo-app o agregar fixture dedicado para mostrar al menos una entidad con #[orm(audit = Audit)] sin degradar el smoke existente`.

### Sesión: bloqueo de transacciones sobre pool

- Se ejecutó la subtarea `Etapa 15+: Bloquear db.transaction(...) sobre SharedConnection::Pool hasta pinnear una conexión física durante todo el closure transaccional`.
- Se confirmó que el plan maestro está en `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se revisó la implementación de `SharedConnection`, `DbContext::transaction(...)`, `MssqlPool` y la guía pública de transacciones.
- Se agregó una clasificación interna de `SharedConnection` para distinguir conexión directa y pool.
- `DbContext::transaction(...)` ahora valida esa clasificación antes de emitir `BEGIN TRANSACTION`.
- Cuando el contexto usa `SharedConnection::Pool`, la API devuelve `OrmError` con un mensaje accionable y no inicia una transacción parcial.
- Se agregaron pruebas unitarias para fijar que la ruta directa sigue aceptando transacciones y que la ruta pooled queda rechazada bajo `pool-bb8`.
- Se actualizó `docs/transactions.md`, `docs/tasks.md` y `docs/context.md`.

### Resultado

- La combinación insegura `db.transaction(...)` sobre contextos creados desde pool queda bloqueada explícitamente hasta que exista pinning de conexión física durante todo el closure.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm direct_shared_connections_support_transactions`
- `cargo test -p mssql-orm --features pool-bb8 pooled_shared_connections_reject_transactions_until_pinned`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo check --workspace --all-features`
- `cargo test -p mssql-orm context::tests::direct_shared_connections_support_transactions`
- `cargo test -p mssql-orm --features pool-bb8 context::tests::pooled_shared_connections_reject_transactions_until_pinned`
- `cargo clippy --workspace --all-targets --all-features`

### Bloqueos

- No hubo bloqueos técnicos.
- No se ejecutó `cargo test --workspace` completo porque el cambio fue acotado a la guardia de transacciones y se validó con compilación completa más pruebas enfocadas, incluyendo el feature `pool-bb8`.
- `cargo clippy --workspace --all-targets --all-features` terminó con código 0, pero mantiene advertencias preexistentes no relacionadas en `mssql-orm-migrate/src/diff.rs` (`collapsible_if`) y `mssql-orm/src/context.rs` (`large_enum_variant`).

### Próximo paso recomendado

- Ejecutar `Etapa 15: Ejecutar validación final de release sobre workspace y ejemplos documentados`.

### Sesión: changelog inicial del release

- Se ejecutó la subtarea `Etapa 15: Preparar changelog inicial del release con surface disponible y exclusiones explícitas`.
- Se revisó la version del workspace (`0.1.0`), `docs/api.md`, `README.md`, `docs/tasks.md` y el estado operativo de Etapas 12 a 16.
- Se agregó `CHANGELOG.md` con entrada `0.1.0 - Unreleased`.
- El changelog enumera la surface disponible: workspace modular, derives, metadata, `DbContext`, `DbSet`, CRUD, query builder, SQL Server compiler, adaptador Tiberius, Active Record, concurrencia, tracking experimental, migraciones, CLI, ejemplo `todo-app` y documentación pública.
- Se agregaron secciones dedicadas a `Entity Policies`, exclusiones explícitas, validación conocida y documentación de referencia.
- Se enlazó `CHANGELOG.md` desde `README.md`.
- Se actualizó `docs/tasks.md` y `docs/context.md`.

### Resultado

- El release inicial ya tiene changelog operativo con la surface disponible y los límites explícitos del estado actual.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- No se ejecutó `cargo test --workspace` porque la tarea fue documental; la validación final completa queda como siguiente subtarea explícita de Etapa 15.

### Próximo paso recomendado

- Ejecutar `Etapa 15: Ejecutar validación final de release sobre workspace y ejemplos documentados`.

### Sesión: inventario de API pública de la crate raíz

- Se ejecutó la subtarea `Etapa 15: Consolidar API docs mínimas y surface pública publicada por la crate raíz`.
- Se revisó `crates/mssql-orm/src/lib.rs`, la `prelude`, los reexports de crates internas y las pruebas unitarias de exposición pública.
- Se agregó `docs/api.md` como inventario mínimo de API pública publicada por `mssql-orm`.
- La guía separa la ruta recomendada `mssql_orm::prelude::*` de módulos avanzados (`core`, `query`, `migrate`, `sqlserver`, `tiberius`) y enumera derives, contratos de modelo, `DbContext`, `DbSet`, query builder, Active Record, tracking experimental, migraciones, configuración operacional, Entity Policies y exclusiones explícitas.
- Se enlazó la nueva guía desde `README.md`, `docs/code-first.md`, `docs/quickstart.md`, `docs/query-builder.md`, `docs/relationships.md`, `docs/transactions.md` y `docs/migrations.md`.
- Se actualizó `docs/tasks.md` y `docs/context.md`.

### Resultado

- La surface pública de la crate raíz queda documentada como índice mínimo de release, con límites explícitos y enlaces hacia las guías de uso específicas.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm exposes_public_prelude`
- `cargo test -p mssql-orm exposes_operational_configuration_surface`

### Bloqueos

- No hubo bloqueos técnicos.
- No se ejecutó `cargo test --workspace` porque la tarea fue documental y se validó con compilación completa más pruebas enfocadas de exposición pública.

### Próximo paso recomendado

- Ejecutar `Etapa 15: Preparar changelog inicial del release con surface disponible y exclusiones explícitas`.

### Sesión: guía pública de relaciones y joins explícitos

- Se ejecutó la subtarea `Etapa 15: Preparar guía de relaciones y joins explícitos con foreign keys derivadas`.
- Se revisó la implementación real de `foreign_key` en `#[derive(Entity)]`, los helpers de `ForeignKeyMetadata`, el AST de joins, la compilación SQL Server y el dominio/queries de `examples/todo-app`.
- Se agregó `docs/relationships.md` como guía pública de relaciones code-first y joins explícitos.
- La guía documenta la sintaxis estructurada `#[orm(foreign_key(entity = User, column = id))]`, la sintaxis string legacy, nombres generados o explícitos de constraints, `on_delete`, metadata derivada, migraciones y DDL.
- También documenta cómo escribir `inner_join::<T>(...)` y `left_join::<T>(...)` con `Predicate::eq(Expr::from(...), Expr::from(...))`.
- Se dejaron explícitos los límites actuales: no hay navigation properties, lazy/eager loading, aliases, joins inferidos, proyecciones parciales públicas ni foreign keys compuestas derivadas automáticamente desde atributos públicos.
- Se enlazó la nueva guía desde `README.md`, `docs/code-first.md` y `docs/query-builder.md`.
- Se actualizó `docs/tasks.md` y `docs/context.md`.

### Resultado

- La guía pública de relaciones queda disponible y alineada con la API real de metadata relacional, migraciones y joins explícitos.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm --test stage9_relationship_metadata`
- `cargo test -p mssql-orm --test stage6_public_query_builder`

### Bloqueos

- No hubo bloqueos técnicos.
- No se ejecutó `cargo test --workspace` porque la tarea fue documental y se validó con compilación completa más pruebas enfocadas de relaciones y query builder público.

### Próximo paso recomendado

- Ejecutar `Etapa 15: Consolidar API docs mínimas y surface pública publicada por la crate raíz`.

### Sesión: guía pública de transacciones runtime

- Se ejecutó la subtarea `Etapa 15: Preparar guía de transacciones y límites operativos de db.transaction(...)`.
- Se revisó la implementación real en `crates/mssql-orm/src/context.rs`, el derive `DbContext`, el adaptador transaccional de `mssql-orm-tiberius` y las pruebas públicas de commit/rollback.
- Se agregó `docs/transactions.md` como guía pública de `db.transaction(|tx| async move { ... })`.
- La guía documenta el contrato actual: `BEGIN TRANSACTION`, commit si el closure devuelve `Ok`, rollback si devuelve `Err`, uso obligatorio del contexto `tx`, operaciones soportadas, errores, timeouts, tracing y retry deshabilitado dentro de transacciones.
- Se dejó explícito que no hay savepoints, transacciones anidadas, rollback en `Drop`, rollback automático ante panic ni transacciones distribuidas.
- Se detectó y documentó un límite importante con `pool-bb8`: `db.transaction(...)` no pinnea todavía una conexión física del pool durante todo el closure, por lo que no debe prometerse como soportado sobre `from_pool(...)`.
- Se agregó en `docs/tasks.md` una tarea técnica pendiente para corregir o bloquear `db.transaction(...)` sobre `SharedConnection::Pool`.
- Se enlazó la nueva guía desde `README.md` y desde `docs/code-first.md`.
- Se actualizó `docs/tasks.md` y `docs/context.md`.

### Resultado

- La guía de transacciones queda disponible y alineada con el comportamiento real de la API pública, incluyendo sus garantías y exclusiones operativas.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm --test trybuild entity_derive_ui`
- `cargo test -p mssql-orm-tiberius transaction`

### Bloqueos

- No hubo bloqueos técnicos para la tarea documental.
- No se ejecutó `cargo test --workspace` porque la tarea fue documental y se validó con compilación completa más pruebas enfocadas de surface pública y capa transaccional.
- Queda como riesgo/tarea pendiente corregir o bloquear `db.transaction(...)` sobre pool hasta pinnear conexión física durante todo el closure.

### Próximo paso recomendado

- Ejecutar `Etapa 15: Preparar guía de relaciones y joins explícitos con foreign keys derivadas`.

### Sesión: guía pública del query builder

- Se ejecutó la subtarea `Etapa 15: Preparar guía del query builder público (filter, order_by, joins, take, paginate, count)`.
- Se confirmó que el plan maestro no está en la raíz; la ruta real usada fue `docs/plan_orm_sqlserver_tiberius_code_first.md`.
- Se agregó `docs/query-builder.md` como guía pública de la surface actual de `DbSetQuery<T>`.
- La guía cubre entrada por `DbSet::query()`, filtros tipados, composición lógica, ordenamiento, `limit`/`take`, `PageRequest`, joins explícitos, `all`, `first`, `count` e inspección del AST.
- Se documentó explícitamente que el query builder público construye AST y no SQL directo, preservando que la compilación pertenece a `mssql-orm-sqlserver` y la ejecución a Tiberius.
- Se dejó registrado el límite actual de `count()`: conserva `from` y filtros de la entidad base, pero no traslada joins, ordenamiento ni paginación al `CountQuery` interno.
- Se enlazó la nueva guía desde `README.md` y desde `docs/code-first.md`.
- Se actualizó `docs/tasks.md` y `docs/context.md`.

### Resultado

- La guía pública del query builder queda disponible y alineada con la API real validada por los fixtures `stage6` y `trybuild`.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm --test stage6_public_query_builder`
- `cargo test -p mssql-orm --test trybuild entity_derive_ui`

### Bloqueos

- No hubo bloqueos técnicos.
- No se ejecutó `cargo test --workspace` porque la tarea fue documental y se validó con compilación completa del workspace más pruebas enfocadas del query builder público.

### Próximo paso recomendado

- Ejecutar `Etapa 15: Preparar guía de transacciones y límites operativos de db.transaction(...)`.

### Sesión: snapshot DDL SQL Server para columnas auditables

- Se ejecutó la subtarea `Etapa 16: Validar el SQL Server DDL generado para columnas auditables con defaults como SYSUTCDATETIME(), longitudes nvarchar, nullability y tipos fecha compatibles`.
- Se agregó en `crates/mssql-orm-sqlserver/tests/migration_snapshots.rs` el snapshot test `snapshots_audit_column_migration_sql`.
- La prueba compila operaciones `AddColumn` para `created_at`, `created_by_user_id`, `updated_at` y `updated_by` usando `SqlServerCompiler::compile_migration_operations(...)`.
- Se agregó el snapshot `crates/mssql-orm-sqlserver/tests/snapshots/migration_snapshots__audit_column_migration_sql.snap`.
- El snapshot fija el DDL SQL Server esperado: `datetime2 NOT NULL DEFAULT SYSUTCDATETIME()`, `bigint NULL`, `datetime2 NULL DEFAULT SYSUTCDATETIME()` y `nvarchar(120) NULL`.
- No se modificó la lógica productiva del compilador; la tarea cerró con cobertura observable de la salida DDL existente.
- Se actualizó `docs/tasks.md` y `docs/context.md`.

### Resultado

- El DDL SQL Server para columnas auditables queda validado por snapshot en la crate dueña de compilación SQL Server.

### Validación

- `cargo fmt --all`
- `INSTA_UPDATE=always cargo test -p mssql-orm-sqlserver --test migration_snapshots snapshots_audit_column_migration_sql`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm-sqlserver --test migration_snapshots`
- `cargo test -p mssql-orm --test stage16_audit_migrations`
- `cargo clippy --workspace --all-targets --all-features`
- `cargo test --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- `cargo clippy --workspace --all-targets --all-features` terminó con código 0, pero mantiene advertencias preexistentes no relacionadas en `mssql-orm-migrate/src/diff.rs` (`collapsible_if`) y `mssql-orm/src/context.rs` (`large_enum_variant`).

### Próximo paso recomendado

- Ejecutar `Etapa 16: Actualizar el ejemplo todo-app o agregar fixture dedicado para mostrar al menos una entidad con #[orm(audit = Audit)] sin degradar el smoke existente`.

### Sesión: bloqueo destructivo al quitar auditoría

- Se ejecutó la subtarea `Etapa 16: Agregar pruebas de diff donde quitar audit = Audit sea detectado como destructivo por la CLI cuando produzca DropColumn`.
- Se amplió `crates/mssql-orm/tests/stage16_audit_migrations.rs` con una prueba que compara `AuditedEntity` como snapshot previo contra `PlainEntity` como snapshot actual.
- La prueba valida que `diff_column_operations(...)` emite exactamente cuatro `DropColumn`: `created_at`, `created_by_user_id`, `updated_at` y `updated_by`.
- Se agregó en `crates/mssql-orm-cli/src/main.rs` el helper de test `audited_entity_snapshot(...)` para modelar una tabla auditada/sin auditar con `ModelSnapshot` serializable.
- Se agregó la prueba CLI `run_migration_add_blocks_removing_audit_policy_by_default`, que carga el snapshot previo auditado desde `model_snapshot.json`, usa un snapshot actual sin auditoría y verifica que `migration add` falle antes de crear migración.
- El bloqueo observado es el existente de cambios destructivos: `Error: destructive migration detected` con `Operation: DropColumn audit.audited_entities.created_at`.
- Se actualizó `docs/tasks.md` y `docs/context.md`.

### Resultado

- Quitar `#[orm(audit = Audit)]` de una tabla existente queda cubierto como cambio destructivo: el diff genera `DropColumn` y la CLI lo bloquea por defecto salvo uso explícito de `--allow-destructive`.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test stage16_audit_migrations`
- `cargo test -p mssql-orm-cli run_migration_add_blocks_removing_audit_policy_by_default`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm-cli`
- `cargo test -p mssql-orm-migrate diff`
- `cargo clippy --workspace --all-targets --all-features`
- `cargo test --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- `cargo clippy --workspace --all-targets --all-features` terminó con código 0, pero mantiene advertencias preexistentes no relacionadas en `mssql-orm-migrate/src/diff.rs` (`collapsible_if`) y `mssql-orm/src/context.rs` (`large_enum_variant`).

### Próximo paso recomendado

- Ejecutar `Etapa 16: Validar el SQL Server DDL generado para columnas auditables con defaults como SYSUTCDATETIME(), longitudes nvarchar, nullability y tipos fecha compatibles`.

### Sesión: diff al activar auditoría sobre tabla existente

- Se ejecutó la subtarea `Etapa 16: Agregar pruebas de diff donde activar audit = Audit sobre una tabla existente genere AddColumn para cada columna auditable esperada`.
- Se amplió `crates/mssql-orm/tests/stage16_audit_migrations.rs` con `PlainEntity`, una entidad sin auditoría que apunta a la misma tabla que `AuditedEntity`.
- La prueba nueva compara `ModelSnapshot::from_entities(&[PlainEntity::metadata()])` contra `ModelSnapshot::from_entities(&[AuditedEntity::metadata()])`.
- Se validó que `diff_column_operations(...)` emite exactamente cuatro operaciones `AddColumn`, una por cada columna auditable: `created_at`, `created_by_user_id`, `updated_at` y `updated_by`.
- La prueba fija además shape de columnas agregadas: tipo SQL Server, default SQL, nullability, longitud `nvarchar` y flags `insertable`/`updatable`.
- No se introdujeron cambios productivos; el diff funciona porque las columnas auditables participan como `ColumnSnapshot` ordinarias en la tabla compartida.
- Se actualizó `docs/tasks.md` y `docs/context.md`.

### Resultado

- Activar `#[orm(audit = Audit)]` sobre una tabla existente queda cubierto como migración incremental no destructiva basada en `AddColumn` por cada columna auditable.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test stage16_audit_migrations`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm-migrate diff`
- `cargo test -p mssql-orm-sqlserver migration`
- `cargo clippy --workspace --all-targets --all-features`
- `cargo test --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- `cargo clippy --workspace --all-targets --all-features` terminó con código 0, pero mantiene advertencias preexistentes no relacionadas en `mssql-orm-migrate/src/diff.rs` (`collapsible_if`) y `mssql-orm/src/context.rs` (`large_enum_variant`).

### Próximo paso recomendado

- Ejecutar `Etapa 16: Agregar pruebas de diff donde quitar audit = Audit sea detectado como destructivo por la CLI cuando produzca DropColumn`.

### Sesión: migración `CREATE TABLE` para entidad auditada

- Se ejecutó la subtarea `Etapa 16: Agregar pruebas de migraciones donde una entidad nueva con audit = Audit genere CREATE TABLE con columnas auditables`.
- Se agregó `crates/mssql-orm/tests/stage16_audit_migrations.rs` como prueba de integración pública de Etapa 16.
- La prueba define un `Audit` con `#[derive(AuditFields)]` y una entidad `AuditedEntity` con `#[orm(audit = Audit)]`, usando la misma surface pública que un consumidor.
- El flujo validado es completo: `EntityMetadata` derivada -> `ModelSnapshot::from_entities(...)` -> `diff_schema_and_table_operations(...)` -> `MigrationOperation::CreateSchema/CreateTable` -> `SqlServerCompiler::compile_migration_operations(...)`.
- El SQL esperado confirma `CREATE TABLE [audit].[audited_entities]` con columnas propias y auditables: `created_at`, `created_by_user_id`, `updated_at` y `updated_by`, además de PK e identity.
- No se introdujo lógica especial para auditoría en migraciones: la prueba pasa porque las columnas auditables ya llegan como `ColumnMetadata`/`ColumnSnapshot` ordinarias.
- Se actualizó `docs/tasks.md` y `docs/context.md`.

### Resultado

- Una entidad nueva con `#[orm(audit = Audit)]` ya tiene cobertura de migración para creación de tabla, confirmando que el pipeline code-first genera `CREATE TABLE` con columnas auditables.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test stage16_audit_migrations`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm --test stage16_entity_policies`
- `cargo test -p mssql-orm-migrate diff`
- `cargo test -p mssql-orm-sqlserver migration`
- `cargo clippy --workspace --all-targets --all-features`
- `cargo test --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- `cargo clippy --workspace --all-targets --all-features` terminó con código 0, pero mantiene advertencias preexistentes no relacionadas en `mssql-orm-migrate/src/diff.rs` (`collapsible_if`) y `mssql-orm/src/context.rs` (`large_enum_variant`).

### Próximo paso recomendado

- Ejecutar `Etapa 16: Agregar pruebas de diff donde activar audit = Audit sobre una tabla existente genere AddColumn para cada columna auditable esperada`.

### Sesión: snapshot de entidades auditadas

- Se ejecutó la subtarea `Etapa 16: Confirmar que ModelSnapshot::from_entities(...) incluye columnas auditables sin cambios especiales en snapshot, serialización JSON ni orden determinista`.
- Se amplió `crates/mssql-orm/tests/stage16_entity_policies.rs` con una entidad adicional del mismo schema para fijar el orden determinista de tablas dentro de `ModelSnapshot`.
- Se agregó la prueba `model_snapshot_includes_audit_columns_without_special_pipeline`, que construye el snapshot desde `AuditedEntity::metadata()` y confirma que las columnas auditables aparecen como `ColumnSnapshot` normales.
- La prueba cubre orden de columnas, primary key, tipos SQL, defaults, nullability, `insertable`, `updatable`, longitud `nvarchar` y columna auditable renombrada (`created_by_user_id`).
- La prueba valida `to_json_pretty()` y `from_json(...)`, incluyendo roundtrip completo y presencia observable de columnas/defaults auditables en JSON.
- No se introdujeron cambios productivos: el pipeline existente `EntityMetadata -> TableSnapshot -> ModelSnapshot -> JSON` ya soportaba auditoría porque consume `ColumnMetadata`.
- Se actualizó `docs/tasks.md` y `docs/context.md`.

### Resultado

- `ModelSnapshot::from_entities(...)` queda cubierto para entidades auditadas sin lógica especial de policies: las columnas generadas por `AuditFields` entran al snapshot, se serializan/deserializan y mantienen orden determinista.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test stage16_entity_policies`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm-migrate snapshot`
- `cargo clippy --workspace --all-targets --all-features`
- `cargo test --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- `cargo clippy --workspace --all-targets --all-features` terminó con código 0, pero mantiene advertencias preexistentes no relacionadas en `mssql-orm-migrate/src/diff.rs` (`collapsible_if`) y `mssql-orm/src/context.rs` (`large_enum_variant`).

### Próximo paso recomendado

- Ejecutar `Etapa 16: Agregar pruebas de migraciones donde una entidad nueva con audit = Audit genere CREATE TABLE con columnas auditables`.

### Sesión: cobertura unitaria de metadata de auditoría

- Se ejecutó la subtarea `Etapa 16: Agregar pruebas unitarias de metadata para confirmar schema, table, columnas propias, columnas auditables, defaults, nullability, insertable/updatable y orden estable`.
- El plan maestro requerido por la sesión está en `docs/plan_orm_sqlserver_tiberius_code_first.md`; no existe el archivo homónimo en la raíz.
- Se amplió `crates/mssql-orm/tests/stage16_entity_policies.rs` para fijar explícitamente `rust_name`, `schema`, `table`, primary key, ausencia de índices/foreign keys y orden completo de columnas.
- La fixture auditada ahora cubre columnas propias `id`, `name`, `status` y columnas de policy `created_at`, `created_by_user_id`, `updated_at`, `updated_by`.
- La prueba confirma tipos SQL, defaults, nullability, longitud, identity, `insertable` y `updatable` tanto en columnas propias como en columnas generadas por `AuditFields`.
- Se preservó el límite MVP: no hubo cambios productivos ni autollenado runtime; la sesión solo reforzó cobertura observable de metadata.
- Se actualizó `docs/tasks.md` y `docs/context.md`.

### Resultado

- La metadata generada por `#[orm(audit = Audit)]` queda cubierta por pruebas unitarias/integración pública para el contrato pedido por el backlog: schema, table, columnas propias, columnas auditables, defaults, nullability, flags de persistencia y orden estable.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test stage16_entity_policies`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm-core`
- `cargo test -p mssql-orm-macros`
- `cargo clippy --workspace --all-targets --all-features`
- `cargo test --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- `cargo clippy --workspace --all-targets --all-features` terminó con código 0, pero reportó advertencias preexistentes no relacionadas en `mssql-orm-migrate/src/diff.rs` (`collapsible_if`) y `mssql-orm/src/context.rs` (`large_enum_variant`).

### Próximo paso recomendado

- Ejecutar `Etapa 16: Confirmar que ModelSnapshot::from_entities(...) incluye columnas auditables sin cambios especiales en snapshot, serialización JSON ni orden determinista`.

### Sesión: cobertura `trybuild` de auditoría inválida

- Se ejecutó la subtarea `Etapa 16: Cubrir errores trybuild para auditoría inválida: tipo inexistente, struct sin campos nombrados, atributo no soportado, columna duplicada y tipo sin mapping SQL soportado`.
- Ya existía cobertura para `AuditFields` sobre struct sin campos nombrados, atributo no soportado, `column = ""` y tipo sin `SqlTypeMapping`.
- Se agregó validación explícita en `#[derive(AuditFields)]` para rechazar columnas duplicadas dentro del struct de auditoría.
- Se agregó el fixture inválido `audit_fields_duplicate_column.rs` y su `.stderr` esperado.
- Se agregó el fixture inválido `entity_audit_missing_type.rs` y su `.stderr` esperado para `#[orm(audit = MissingAudit)]`.
- Se registraron los nuevos fixtures en `crates/mssql-orm/tests/trybuild.rs`.
- Se actualizó `docs/entity-policies.md`, `docs/context.md` y `docs/tasks.md`.

### Resultado

- Los errores de auditoría inválida enumerados en el backlog quedan cubiertos por `trybuild`: tipo inexistente, struct sin campos nombrados, atributo no soportado, columna vacía/duplicada y tipo sin mapping SQL.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test trybuild`
- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Agregar pruebas unitarias de metadata para confirmar schema, table, columnas propias, columnas auditables, defaults, nullability, insertable/updatable y orden estable`.

### Sesión: fixture público válido para `#[orm(audit = Audit)]`

- Se ejecutó la subtarea `Etapa 16: Cubrir #[orm(audit = Audit)] con tests trybuild válidos en la crate pública usando únicamente la API reexportada por mssql-orm::prelude`.
- Se agregó el fixture `crates/mssql-orm/tests/ui/entity_audit_public_valid.rs`.
- El fixture usa solo `use mssql_orm::prelude::*` para la API del ORM y cubre `AuditFields`, `Entity`, `EntityPolicy`, `Row`, `SqlValue`, `OrmError` y `FromRow` desde el punto de vista de un consumidor.
- El fixture valida en compilación una entidad con `#[orm(audit = Audit)]`, metadata expandida, `COLUMN_NAMES` y materialización desde una fila que incluye columnas auditables extra.
- Se registró el fixture en `crates/mssql-orm/tests/trybuild.rs`.
- Se actualizó `docs/entity-policies.md`, `docs/context.md` y `docs/tasks.md`.

### Resultado

- El camino público válido de auditoría ya tiene cobertura `trybuild` explícita usando únicamente la `prelude` de la crate pública.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test trybuild`
- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Cubrir errores trybuild para auditoría inválida: tipo inexistente, struct sin campos nombrados, atributo no soportado, columna duplicada y tipo sin mapping SQL soportado`.

### Sesión: materialización `FromRow` de entidades auditadas

- Se ejecutó la subtarea `Etapa 16: Asegurar que FromRow generado pueda materializar entidades con auditoría; si el MVP no agrega campos Rust visibles al entity, documentar que esas columnas son solo metadata/schema en esta etapa`.
- Se amplió `crates/mssql-orm/tests/stage16_entity_policies.rs` con un `TestRow` neutral y dos casos de materialización.
- Una entidad con `#[orm(audit = Audit)]` ahora queda cubierta al materializarse desde una fila que solo trae columnas propias reales (`id`, `name`).
- También queda cubierta la fila que trae columnas auditables extra (`created_at`, `updated_by`); el `FromRow` generado las ignora porque no existen campos Rust visibles donde asignarlas.
- Se documentó en `docs/entity-policies.md` y `docs/context.md` que las columnas auditables son metadata/schema en este MVP y no estado Rust visible.
- Se actualizó `docs/tasks.md`.

### Resultado

- El contrato MVP de auditoría queda probado para `FromRow`: las entidades auditadas se materializan correctamente usando solo sus campos propios, con o sin columnas auditables presentes en la fila.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test stage16_entity_policies`
- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- Materializar columnas auditables como campos Rust visibles queda diferido porque requiere rediseñar el shape del entity, símbolos de columna y persistencia/autollenado.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Cubrir #[orm(audit = Audit)] con tests trybuild válidos en la crate pública usando únicamente la API reexportada por mssql-orm::prelude`.

### Sesión: documentar símbolos de columnas auditables en el MVP

- Se ejecutó la subtarea `Etapa 16: Generar símbolos de columna asociados para columnas auditables cuando sea posible, o documentar claramente si el MVP no los expone como Todo::created_at`.
- Se revisó `#[derive(Entity)]` y se confirmó que los símbolos `EntityColumn` se generan solo desde campos Rust declarados directamente en la entidad.
- Se decidió no generar `Todo::created_at` para columnas aportadas por `#[orm(audit = Audit)]` en el MVP, porque el macro de entidad solo recibe el path de la policy y no debe inferir campos de otro derive para crear API de query.
- Se agregó el fixture `trybuild` inválido `entity_audit_column_symbol_unavailable.rs`, que fija que `AuditedEntity::created_at` no existe cuando `created_at` viene de `AuditFields`.
- Se documentó la decisión en `docs/entity-policies.md` y `docs/context.md`.
- Se actualizó `docs/tasks.md`.

### Resultado

- El límite del MVP quedó explícito y cubierto: las columnas auditables participan en metadata/schema, snapshots, diff y DDL, pero no en el DSL tipado de columnas asociadas del entity.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test trybuild`
- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- Generar símbolos asociados para columnas de policies queda diferido hasta diseñar cómo se relaciona con campos Rust visibles, `FromRow`, query builder y autollenado.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Asegurar que FromRow generado pueda materializar entidades con auditoría; si el MVP no agrega campos Rust visibles al entity, documentar que esas columnas son solo metadata/schema en esta etapa`.

### Sesión: rechazar policies audit duplicadas

- Se ejecutó la subtarea `Etapa 16: Validar que una entidad no pueda declarar dos políticas que generen la misma columna, dejando preparado el caso futuro de audit + timestamps`.
- En el MVP actual solo existe una policy compilable a nivel de entidad: `audit = Audit`; `timestamps` sigue diferida y no se introdujo sintaxis nueva.
- `parse_entity_config(...)` ahora rechaza una segunda declaración `audit` en `#[derive(Entity)]` en vez de sobrescribir silenciosamente la primera.
- El mensaje compile-time deja explícito que `Entity` solo soporta una policy `audit` y que múltiples policies con columnas solapadas deben rechazarse explícitamente.
- Se agregó el fixture `trybuild` inválido `entity_duplicate_audit_policy.rs`, modelando el caso futuro `audit` + `timestamps` con dos structs `AuditFields` que producirían `created_at`.
- Se actualizó `docs/entity-policies.md`, `docs/context.md` y `docs/tasks.md`.

### Resultado

- Una entidad ya no puede declarar dos policies `audit`; esto evita el caso más cercano de doble policy en el MVP y conserva `COLUMN_NAMES` como base para validar solapamientos cuando exista una segunda policy real.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test trybuild`
- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- No se implementó `timestamps = Timestamps`; sigue diferido por diseño y por backlog.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Generar símbolos de columna asociados para columnas auditables cuando sea posible, o documentar claramente si el MVP no los expone como Todo::created_at`.

### Sesión: generar `down.sql` reversible cuando el payload lo permite

- Se completó la subtarea `Etapa 7+: Evaluar generación reversible de down.sql cuando las operaciones de migración conserven payload suficiente para invertir cambios de forma segura`.
- La tarea ya estaba en `En Progreso` al iniciar esta sesión y existían cambios parciales en `mssql-orm-cli` y `mssql-orm-migrate`; se continuó sobre ese trabajo sin revertirlo.
- `mssql-orm-cli migration add` ahora calcula operaciones inversas en orden inverso del plan y compila esas operaciones con `SqlServerCompiler` para poblar `down.sql` cuando todo el plan es reversible.
- Las operaciones reversibles actuales son `CreateSchema`, `DropSchema`, `CreateTable`, `RenameTable`, `RenameColumn`, `AddColumn`, `AlterColumn`, `CreateIndex` y `AddForeignKey`.
- Las operaciones que eliminan objetos sin conservar payload de reconstrucción (`DropTable`, `DropColumn`, `DropIndex`, `DropForeignKey`) dejan `down.sql` como plantilla manual y la salida de la CLI reporta `down.sql: manual (...)` con la primera operación no reversible.
- `mssql-orm-migrate` expone `write_migration_down_sql(...)` con el mismo formato de statements que `up.sql`.
- Se agregó cobertura en `mssql-orm-cli` para `down.sql` generado en una migración reversible y para rollback manual al permitir un cambio destructivo con `--allow-destructive`.
- Se agregó cobertura en `mssql-orm-migrate` para escribir `down.sql` generado.
- Se actualizó `docs/migrations.md`, `docs/context.md` y `docs/tasks.md`.

### Resultado

- `migration add` ya produce `down.sql` útil para planes reversibles, sin prometer ejecución automática de rollback.
- Los casos sin payload suficiente siguen siendo explícitamente manuales y revisables.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm-cli`
- `cargo test -p mssql-orm-migrate filesystem`
- `cargo fmt --all --check`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- No se ejecutó `cargo test --workspace` ni `cargo clippy --workspace --all-targets --all-features` en esta sesión para mantener la validación proporcional al alcance; el cambio afecta CLI, filesystem de migraciones y documentación.

### Próximo paso recomendado

- Continuar con las tareas pendientes de Etapa 15 de documentación pública o volver a Etapa 16 para validar colisiones entre varias policies antes de introducir `timestamps`.

### Sesión: validar colisiones entre columnas auditables y columnas propias

- Se ejecutó la subtarea `Etapa 16: Validar colisiones entre columnas generadas por auditoría y campos propios de la entidad, fallando en compile-time con un mensaje accionable`.
- El plan maestro requerido por la sesión está en `docs/plan_orm_sqlserver_tiberius_code_first.md`; no existe el archivo homónimo en la raíz.
- `EntityPolicy` ahora expone `COLUMN_NAMES` como contrato constante y conserva compatibilidad mediante valor por defecto `&[]`.
- `mssql-orm-core` agregó `column_name_exists(...)` como helper `const fn` para permitir validaciones de nombres de columna durante compilación.
- `#[derive(AuditFields)]` ahora genera `COLUMN_NAMES` a partir de los nombres finales de columna, respetando `#[orm(column = "...")]`.
- `#[derive(Entity)]` ahora emite aserciones constantes para entidades con `#[orm(audit = Audit)]`; si una columna propia colisiona con una columna auditable, la compilación falla nombrando la columna duplicada y sugiriendo renombrar mediante `#[orm(column = "...")]`.
- Se agregó el fixture `trybuild` inválido `entity_audit_column_collision.rs` y su `.stderr` esperado.
- Se actualizó `docs/entity-policies.md`, `docs/context.md` y `docs/tasks.md`.

### Resultado

- Las colisiones entre columnas propias de una entidad y columnas aportadas por `AuditFields` ya fallan en compile-time antes de construir `EntityMetadata`.
- La validación de colisiones entre varias policies queda pendiente para la siguiente subtarea antes de introducir policies simultáneas como `timestamps`.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm --test trybuild`
- `cargo test -p mssql-orm-core`
- `cargo test -p mssql-orm-macros`
- `cargo test -p mssql-orm --test stage16_entity_policies`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets --all-features`

### Bloqueos

- No hubo bloqueos técnicos.
- `cargo clippy --workspace --all-targets --all-features` terminó con código 0, pero reportó advertencias preexistentes no relacionadas en `mssql-orm-migrate/src/diff.rs` (`collapsible_if`) y `mssql-orm/src/context.rs` (`large_enum_variant`).
- Se detectó que `crates/mssql-orm/wip/` ya contenía archivos previos no relacionados (`.gitignore` y `dbcontext_invalid_field_type.stderr`); no se modificaron.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Validar que una entidad no pueda declarar dos políticas que generen la misma columna, dejando preparado el caso futuro de audit + timestamps`.

### Sesión: expandir columnas auditables en metadata de entidad

- Se ejecutó la subtarea `Etapa 16: Hacer que #[orm(audit = Audit)] expanda las columnas auditables dentro de EntityMetadata.columns en orden estable y documentado`.
- El plan maestro requerido por la sesión está en `docs/plan_orm_sqlserver_tiberius_code_first.md`; no existe el archivo homónimo en la raíz.
- `#[derive(Entity)]` ahora usa el path de `audit = Audit` para exigir que el tipo implemente `EntityPolicy` y anexar sus columnas a la metadata de la entidad.
- Para entidades con auditoría, la metadata se construye una vez con `OnceLock`, combinando columnas propias y columnas de policy en un slice estático.
- El orden estable queda documentado como columnas propias de la entidad en orden de declaración Rust, seguidas por columnas de `AuditFields` en orden de declaración Rust.
- Las entidades sin `audit` conservan el camino anterior de `static EntityMetadata`.
- Se actualizó el fixture `trybuild` `entity_audit_attr_valid.rs` para esperar la columna `created_at`.
- Se agregó la prueba ejecutable `stage16_entity_policies.rs` para verificar schema, table, columnas propias, columnas auditables, defaults, nullability, flags `insertable`/`updatable` y orden estable.
- Se actualizó `docs/entity-policies.md`, `docs/context.md` y `docs/tasks.md`.

### Resultado

- `#[orm(audit = Audit)]` ya expande columnas auditables como `ColumnMetadata` normales dentro de `EntityMetadata.columns`.
- En este corte las columnas auditables son metadata/schema: no generan campos Rust visibles en la entidad ni símbolos asociados como `Todo::created_at`.

### Validación

- `cargo fmt --all --check`
- `cargo test -p mssql-orm --test stage16_entity_policies`
- `cargo test -p mssql-orm --test trybuild`
- `cargo test -p mssql-orm-macros`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- La validación de colisiones entre columnas propias y auditables queda pendiente para la siguiente subtarea.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Validar colisiones entre columnas generadas por auditoría y campos propios de la entidad, fallando en compile-time con un mensaje accionable`.

### Sesión: aceptar `#[orm(audit = Audit)]` en `Entity`

- Se ejecutó la subtarea `Etapa 16: Extender #[derive(Entity)] para aceptar #[orm(audit = Audit)] a nivel de entidad sin afectar entidades existentes que no declaran auditoría`.
- `parse_entity_config(...)` ahora acepta `audit = <Path>` a nivel de entidad y lo guarda en la configuración del derive.
- En este corte el atributo se parsea pero no se consume para expandir columnas; esa expansión queda en la subtarea siguiente.
- Se agregó el fixture `tests/ui/entity_audit_attr_valid.rs`, que valida que una entidad con `#[orm(audit = Audit)]` compila usando solo `mssql_orm::prelude::*`.
- El fixture fija explícitamente que, por ahora, `EntityMetadata.columns` conserva solo las columnas propias de la entidad y no incluye todavía columnas auditables.
- Se actualizó `docs/context.md` y `docs/entity-policies.md` con el estado real.

### Resultado

- La sintaxis pública `#[orm(audit = Audit)]` ya es aceptada por `#[derive(Entity)]` sin alterar entidades existentes ni adelantar la expansión de columnas.

### Validación

- `cargo fmt --all --check`
- `cargo test -p mssql-orm --test trybuild`
- `cargo test -p mssql-orm-macros`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- La expansión real de columnas auditables dentro de `EntityMetadata.columns` queda pendiente.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Hacer que #[orm(audit = Audit)] expanda las columnas auditables dentro de EntityMetadata.columns en orden estable y documentado`.

### Sesión: validar compile-time `AuditFields`

- Se ejecutó la subtarea `Etapa 16: Agregar validaciones compile-time para AuditFields: solo structs con campos nombrados, tipos con SqlTypeMapping, atributos #[orm(...)] permitidos, nombres de columnas no vacíos y errores claros en casos inválidos`.
- Se agregó validación explícita para rechazar `#[orm(column = "")]` en campos de `AuditFields`.
- Se agregó fixture válido `tests/ui/audit_fields_valid.rs` usando únicamente `mssql_orm::prelude::*`.
- Se agregaron fixtures `trybuild` inválidos para struct tuple, atributo no soportado, columna vacía y tipo sin `SqlTypeMapping`.
- Se versionaron los `.stderr` esperados de esos cuatro casos inválidos.
- El caso de tipo no soportado se valida por el bound generado contra `SqlTypeMapping`, preservando soporte para tipos custom que implementen el trait.
- Se actualizó `docs/context.md` con la cobertura compile-time actual.

### Resultado

- `AuditFields` ya tiene cobertura compile-time mínima para el contrato documentado antes de integrarse con `#[derive(Entity)]`.

### Validación

- `cargo fmt --all --check`
- `cargo test -p mssql-orm --test trybuild`
- `cargo test -p mssql-orm-macros`
- `cargo check --workspace`
- `cargo test -p mssql-orm`

### Bloqueos

- No hubo bloqueos técnicos.
- Los casos de auditoría inválida ligados a `#[orm(audit = Audit)]` sobre entidades quedan para las tareas posteriores donde se implemente esa integración.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Extender #[derive(Entity)] para aceptar #[orm(audit = Audit)] a nivel de entidad sin afectar entidades existentes que no declaran auditoría`.

### Sesión: implementar `#[derive(AuditFields)]`

- Se ejecutó la subtarea `Etapa 16: Implementar #[derive(AuditFields)] o contrato equivalente para convertir un struct de auditoría definido por el usuario en metadata reutilizable`.
- Se agregó `#[proc_macro_derive(AuditFields, attributes(orm))]` en `mssql-orm-macros`.
- El derive implementa `mssql_orm::core::EntityPolicy` para el struct de auditoría, con `POLICY_NAME = "audit"` y `columns() -> &'static [ColumnMetadata]`.
- Cada campo nombrado se convierte en `ColumnMetadata` normal, usando `rust_field`, `column_name`, `renamed_from`, `sql_type`, nullability, defaults, `max_length`, `precision`, `scale`, `insertable` y `updatable`.
- Para el tipo SQL y defaults de longitud/precisión/escala, el derive usa las constantes de `SqlTypeMapping`, permitiendo que tipos custom con mapping propio puedan participar.
- Se agregó parsing inicial de atributos permitidos para campos auditables: `column`, `length`, `nullable`, `default_sql`, `renamed_from`, `sql_type`, `precision`, `scale`, `insertable` y `updatable`.
- La crate pública `mssql-orm` reexporta `AuditFields` desde `prelude`.
- Se agregó cobertura en la crate pública para validar que `#[derive(AuditFields)]` produce metadata observable desde `mssql_orm::prelude::*`.
- Se actualizó `docs/entity-policies.md` y `docs/context.md` con el estado real.

### Resultado

- Ya existe el derive base `AuditFields` y puede convertir un struct de auditoría definido por el usuario en metadata reusable como `EntityPolicy`.
- La integración con entidades mediante `#[orm(audit = Audit)]` sigue pendiente y queda para las siguientes tareas.

### Validación

- `cargo fmt --all --check`
- `cargo test -p mssql-orm-macros`
- `cargo test -p mssql-orm-core`
- `cargo test -p mssql-orm`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- Las validaciones compile-time exhaustivas de `AuditFields` quedan en la subtarea siguiente del backlog.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Agregar validaciones compile-time para AuditFields: solo structs con campos nombrados, tipos con SqlTypeMapping, atributos #[orm(...)] permitidos, nombres de columnas no vacíos y errores claros en casos inválidos`.

### Sesión: definir shape de struct de auditoría

- Se ejecutó la subtarea `Etapa 16: Definir el shape esperado de un struct de auditoría de usuario, incluyendo columnas, tipos soportados, nullability, defaults SQL y reglas para campos no insertables/updatables`.
- Se amplió `docs/entity-policies.md` con la sección `Shape de AuditFields`.
- Quedó definido que el struct de auditoría debe usar `#[derive(AuditFields)]` sobre un struct con campos nombrados.
- Cada campo del struct de auditoría debe generar `ColumnMetadata` normal, usando el nombre del campo como `rust_field` y `column_name` por defecto, con `#[orm(column = "...")]` como override explícito.
- Se documentaron tipos soportados mediante `SqlTypeMapping`, incluyendo `Option<T>` para nullability.
- Se definieron atributos permitidos para campos auditables: `column`, `length`, `nullable`, `default_sql`, `sql_type`, `precision`, `scale`, `renamed_from`, `insertable` y `updatable`.
- Se definieron atributos rechazados para campos auditables: `primary_key`, `identity`, `computed_sql`, `rowversion`, `index`, `unique`, `foreign_key` y `on_delete`.
- Se fijó que `insertable` y `updatable` son flags de metadata; no implican autollenado runtime en el MVP.
- Se actualizó `docs/context.md` con el shape objetivo de `AuditFields`.

### Resultado

- El contrato esperado para structs de auditoría de usuario quedó listo para guiar la implementación del derive `AuditFields` y sus validaciones compile-time.

### Validación

- `cargo fmt --all --check`

### Bloqueos

- No hubo bloqueos técnicos.
- Esta sesión fue documental; no se ejecutaron pruebas Rust porque no hubo cambios de código.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Implementar #[derive(AuditFields)] o contrato equivalente para convertir un struct de auditoría definido por el usuario en metadata reutilizable`.

### Sesión: decidir sintaxis MVP de auditoría

- Se ejecutó la subtarea `Etapa 16: Decidir y documentar la sintaxis MVP soportada para auditoría a nivel de entidad, priorizando #[orm(audit = Audit)] sobre alternativas implícitas o runtime`.
- Se actualizó `docs/entity-policies.md` para fijar `#[orm(audit = Audit)]` a nivel de entidad como sintaxis canónica del MVP.
- Se documentó que el lado derecho debe ser un path Rust visible desde el scope del derive, por ejemplo `Audit`, `crate::model::Audit` o `common::Audit`.
- Se dejaron explícitamente rechazadas las variantes `#[orm(audit)]`, `#[orm(audit = "Audit")]`, `#[orm(audit(Audit))]`, llamadas runtime, providers y configuración inline de columnas en la entidad.
- Se confirmó que entidades sin `audit` deben conservar el comportamiento actual y que cada entidad solo podrá declarar una policy `audit` en el MVP.
- Se actualizó `docs/context.md` con la decisión sintáctica.

### Resultado

- La sintaxis pública objetivo para auditoría quedó decidida y lista para que las siguientes tareas implementen el parser macro y el contrato `AuditFields`.

### Validación

- `cargo fmt --all --check`

### Bloqueos

- No hubo bloqueos técnicos.
- Esta sesión fue documental; no se ejecutaron pruebas Rust porque no hubo cambios de código.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Definir el shape esperado de un struct de auditoría de usuario, incluyendo columnas, tipos soportados, nullability, defaults SQL y reglas para campos no insertables/updatables`.

### Sesión: definir contrato core de metadata para policies

- Se ejecutó la subtarea `Etapa 16: Definir el contrato de metadata para políticas reutilizables en mssql-orm-core, preservando que snapshots, diff y DDL sigan consumiendo columnas normales (ColumnMetadata) sin crear un segundo pipeline de esquema`.
- Se agregó en `mssql-orm-core` el contenedor `EntityPolicyMetadata { name, columns }`.
- Se agregó el trait neutral `EntityPolicy`, con `POLICY_NAME`, `columns() -> &'static [ColumnMetadata]` y helper `metadata()`.
- El contrato queda deliberadamente centrado en columnas normales; no agrega una lista de policies a `EntityMetadata` ni crea metadata paralela para snapshots, diff o DDL.
- Se agregó cobertura unitaria en `mssql-orm-core` para confirmar que una policy reusable expone `ColumnMetadata` ordinario con defaults, nullability e insertable/updatable.
- La crate pública `mssql-orm` reexporta `EntityPolicy` y `EntityPolicyMetadata` desde `prelude`, con cobertura de exposición pública.
- Se documentó el contrato en `docs/entity-policies.md` y se actualizó `docs/context.md` con el estado real.

### Resultado

- El core ya tiene el contrato mínimo para que las próximas tareas implementen `AuditFields` y la expansión desde `#[derive(Entity)]` sin inventar un segundo pipeline de esquema.

### Validación

- `cargo fmt --all --check`
- `cargo test -p mssql-orm-core`
- `cargo test -p mssql-orm`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- Aún no existe derive `AuditFields` ni parser de `#[orm(audit = Audit)]`; siguen como tareas posteriores del backlog.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Decidir y documentar la sintaxis MVP soportada para auditoría a nivel de entidad, priorizando #[orm(audit = Audit)] sobre alternativas implícitas o runtime`.

### Sesión: documentar alcance inicial de `Entity Policies`

- Se ejecutó la subtarea `Etapa 16: Documentar explícitamente el alcance inicial de Entity Policies: audit y timestamps como columnas generadas; soft_delete, tenant y comportamiento automático quedan diferidos hasta tener contrato estable`.
- Se amplió `docs/entity-policies.md` con una matriz de alcance para `audit`, `timestamps`, `soft_delete`, `tenant` y autollenado runtime.
- Quedó fijado que `audit = Audit` es la única policy que entra al MVP de implementación.
- `timestamps = Timestamps` queda reconocida como policy candidata de columnas generadas, pero diferida hasta decidir si será una policy separada, un alias o una convención encima del mismo contrato de metadata.
- `soft_delete = SoftDelete`, `tenant = TenantScope`, `AuditProvider` y todo autollenado quedan fuera del MVP porque requieren contratos runtime y afectan rutas públicas de escritura, borrado, consulta, seguridad o tracking.
- Se documentó qué significa "columnas generadas": columnas normales con metadata completa, orden estable, participación en snapshots/diff/DDL y validación de colisiones en compile-time, sin inyección automática de valores.
- Se actualizó `docs/context.md` con el alcance inicial vigente.

### Resultado

- El alcance inicial de Etapa 16 quedó explícito y separa claramente el MVP de metadata/DDL de las futuras policies con comportamiento automático.

### Validación

- `cargo fmt --all --check`

### Bloqueos

- No hubo bloqueos técnicos.
- Esta sesión fue documental; no se ejecutaron pruebas Rust porque no hubo cambios de código.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Definir el contrato de metadata para políticas reutilizables en mssql-orm-core, preservando que snapshots, diff y DDL sigan consumiendo columnas normales (ColumnMetadata) sin crear un segundo pipeline de esquema`.

### Sesión: diseñar concepto público de `Entity Policies`

- Se ejecutó la subtarea `Etapa 16: Diseñar el concepto público de Entity Policies para reutilizar columnas y comportamiento transversal sin romper el enfoque code-first actual`.
- El plan maestro se confirmó en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md`; no existe el archivo homónimo en la raíz del repositorio.
- Se revisó el shape real de `ColumnMetadata`, `EntityMetadata` y `ColumnSnapshot` para mantener el diseño alineado con el pipeline existente.
- Se agregó `docs/entity-policies.md` como documento de diseño público inicial de Etapa 16.
- El documento define `Entity Policies` como piezas reutilizables de modelo `code-first`, declaradas en compile-time sobre entidades, cuyo primer objetivo es aportar columnas transversales sin crear un segundo pipeline de esquema.
- Quedó fijado que las columnas generadas por una policy deben terminar como `ColumnMetadata` normales dentro de `EntityMetadata.columns`, para que snapshots, diff, DDL SQL Server y CLI sigan usando las rutas existentes.
- Se dejaron explícitamente fuera del primer corte los comportamientos runtime como autollenado de auditoría, filtros de `soft_delete`, reemplazo de `DELETE` físico, filtros obligatorios de tenant e inserción automática de `tenant_id`.
- Se actualizó `docs/context.md` con la ubicación del documento y la decisión conceptual principal.

### Resultado

- La Etapa 16 ya tiene un concepto público documentado y verificable para `Entity Policies`, sin introducir todavía cambios de código ni prometer como implementada una API que aún está en diseño.

### Validación

- `cargo fmt --all --check`

### Bloqueos

- No hubo bloqueos técnicos.
- Esta sesión fue documental por diseño; no se ejecutaron pruebas Rust porque no hubo cambios de código.

### Próximo paso recomendado

- Ejecutar `Etapa 16: Documentar explícitamente el alcance inicial de Entity Policies: audit y timestamps como columnas generadas; soft_delete, tenant y comportamiento automático quedan diferidos hasta tener contrato estable`.

## 2026-04-24

### Sesión: limpiar `todo-app` y ejecutar migraciones desde cero

- A petición del usuario, se limpió la base local solo para el fixture `todo-app`: se eliminaron las tablas `todo.todo_items`, `todo.todo_lists`, `todo.users`, el schema `todo` y las filas de historial de las tres migraciones del ejemplo en `dbo.__mssql_orm_migrations`.
- La limpieza fue preparatoria; la aplicación de migraciones se ejecutó después mediante `mssql-orm-cli database update --execute` desde `examples/todo-app`.
- El primer intento previo había fallado porque ya existía `todo.todo_items` sin que el historial local de migraciones estuviera alineado, por lo que el script intentaba crear la tabla inicial otra vez.
- Para hacer observable ese tipo de fallo, se ajustó `mssql-orm-core::OrmError` para almacenar `Message(String)` y se mejoró el mapeo de errores de `mssql-orm-tiberius` para incluir el detalle original del driver cuando falla una query.
- Tras limpiar el fixture, `database update --execute` creó desde cero el schema `todo`, las tablas `todo_items`, `todo_lists`, `users`, la columna incremental `todo_lists.description` y las tres filas de historial.
- Se reejecutó el mismo comando `database update --execute` y terminó correctamente, validando el comportamiento idempotente.
- Se verificó con `sqlcmd` que existen las tres tablas del schema `todo`, que `todo.todo_lists.description` existe como `nvarchar(500)` nullable y que `dbo.__mssql_orm_migrations` contiene los tres ids esperados.

### Resultado

- Las migraciones de `examples/todo-app` quedaron aplicadas desde cero por la CLI contra SQL Server local (`tempdb`).
- El fixture real quedó en estado inspeccionable con schema `todo`, tablas creadas, columna incremental aplicada e historial consistente.

### Validación

- `cargo build -p mssql-orm-cli`
- `DATABASE_URL=<redacted> ../../target/debug/mssql-orm-cli database update --execute` desde `examples/todo-app`
- Reejecución del mismo `database update --execute` para validar idempotencia
- Verificación `sqlcmd` de tablas `todo.*`, columna `todo_lists.description` e historial `dbo.__mssql_orm_migrations`
- `cargo fmt --all --check`
- `cargo test -p mssql-orm-core`
- `cargo test -p mssql-orm-tiberius`
- `cargo test -p mssql-orm-cli`

### Bloqueos

- No hubo bloqueos técnicos después de limpiar el fixture desalineado.
- No se registró la cadena de conexión real en documentación para evitar persistir credenciales.

### Próximo paso recomendado

- Resolver la historia de baselining para objetos existentes sin historial, o documentar explícitamente que `database update --execute` espera una base limpia o un historial ya alineado para la migración inicial.

### Sesión: aplicar migraciones de `todo-app` mediante CLI

- Se tomó la tarea `Etapa 7+: Aplicar contra SQL Server real el script generado desde examples/todo-app para validar creación desde cero e historial idempotente con DATABASE_URL`.
- El plan maestro se confirmó en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md`; no existe el archivo homónimo en la raíz del repositorio.
- Se reconoció la validación histórica con `KEEP_TEST_ROWS`/`KEEP_TEST_TABLES` en pruebas reales, pero esta sesión se centró en aplicar migraciones a través de la CLI, como pidió el usuario.
- Se cerró la implementación de `mssql-orm-cli database update --execute`, manteniendo el modo por defecto de imprimir SQL a `stdout`.
- `--execute` usa `mssql-orm-tiberius::MssqlConnection` para aplicar el mismo script acumulado generado por `database update`.
- La conexión se resuelve desde `--connection-string`, `DATABASE_URL` o `MSSQL_ORM_TEST_CONNECTION_STRING`, sin hardcodear credenciales en archivos del repositorio.
- Se aplicaron las migraciones generadas de `examples/todo-app` contra SQL Server local (`tempdb`) mediante `mssql-orm-cli database update --execute`.
- Se repitió el mismo comando y volvió a terminar correctamente, validando el salto idempotente por historial `dbo.__mssql_orm_migrations`.
- Se actualizó `docs/migrations.md` para documentar el nuevo modo de ejecución directa por CLI y mantener explícito el modo de revisión por script.
- Se actualizó `docs/context.md` con el nuevo estado operativo de migraciones.

### Resultado

- Las migraciones locales de `examples/todo-app` quedaron aplicadas por CLI: creación inicial del schema `todo`, migración incremental no-op y migración `AddTodoListDescription`.
- El script acumulado conserva creación de `dbo.__mssql_orm_migrations`, checksums, transacción por migración y reejecución idempotente.

### Validación

- `cargo test -p mssql-orm-cli`
- `cargo build -p mssql-orm-cli`
- `DATABASE_URL=<redacted> ../../target/debug/mssql-orm-cli database update --execute` desde `examples/todo-app`
- Reejecución del mismo `database update --execute` para validar idempotencia
- `cargo fmt --all --check`
- `cargo test --manifest-path examples/todo-app/Cargo.toml`
- `cargo clippy -p mssql-orm-cli --all-targets -- -D warnings` se ejecutó, pero no quedó en verde por warnings preexistentes en `mssql-orm-migrate/src/diff.rs`

### Bloqueos

- No hubo bloqueos técnicos.
- No se registró la cadena de conexión real en documentación para evitar persistir credenciales.
- `cargo clippy -p mssql-orm-cli --all-targets -- -D warnings` falla por cuatro warnings `collapsible_if` preexistentes en `mssql-orm-migrate/src/diff.rs`; no se corrigieron en esta sesión por estar fuera del alcance de aplicar migraciones por CLI.

### Próximo paso recomendado

- Preparar la guía pública del query builder o cerrar la validación final de release sobre workspace y ejemplos documentados.

### Sesión: validar generación automática reproducible con `todo-app`

- Se tomó la tarea `Etapa 7+: Validar end-to-end la generación automática con un consumidor real (examples/todo-app) creando base desde cero y migración incremental reproducible`.
- Como el entorno no tiene `DATABASE_URL` ni `MSSQL_ORM_TEST_CONNECTION_STRING`, y tampoco variables `MSSQL_ORM_SQLCMD_*`, la tarea se dividió: esta sesión completa la generación reproducible con el consumidor real y deja pendiente la aplicación contra SQL Server real.
- Se añadió `examples/todo-app/src/bin/model_snapshot.rs` para exportar el `ModelSnapshot` actual de `TodoAppDbContext` usando la API pública `model_snapshot_json_from_source`.
- Se añadió `examples/todo-app/scripts/migration_e2e.sh`, que construye la CLI, crea una migración inicial desde el snapshot del `todo-app`, crea una segunda migración incremental no-op y genera `database_update.sql` en un directorio temporal.
- La validación inicial reveló que `diff_relational_operations(...)` no emitía índices ni foreign keys para tablas nuevas, por lo que una migración inicial perdía parte de la metadata relacional.
- Se corrigió `mssql-orm-migrate` para emitir `CreateIndex` y `AddForeignKey` para tablas nuevas, tanto en schemas nuevos como en schemas existentes.
- Se agregó cobertura unitaria para fijar índices y foreign keys sobre tablas nuevas dentro del diff relacional.
- Se ajustó el dominio de `todo-app`: `completed_by_user_id` usa `NO ACTION` en vez de `SET NULL`, alineándolo con el fixture existente y evitando la combinación que SQL Server rechaza por múltiples rutas de cascada.
- Se actualizó `examples/todo-app/README.md` con el exportador de snapshot y el script reproducible de migraciones.

### Resultado

- `examples/todo-app/scripts/migration_e2e.sh` genera una migración inicial con schema, tres tablas, tres índices y cuatro foreign keys, una migración incremental no-op con `-- No schema changes detected.`, y un `database_update.sql` acumulado con historial idempotente.
- La aplicación real con `sqlcmd` quedó pendiente por falta de variables de conexión en el entorno.

### Validación

- `cargo fmt --all --check`
- `cargo run --manifest-path examples/todo-app/Cargo.toml --bin model_snapshot`
- `examples/todo-app/scripts/migration_e2e.sh`
- Verificación manual con `rg` sobre el `up.sql` y `database_update.sql` generados en `/tmp/mssql-orm-todo-migrations.8NYeZp`
- `cargo test -p mssql-orm-migrate`
- `cargo test --manifest-path examples/todo-app/Cargo.toml`
- `cargo test -p mssql-orm-cli`
- `cargo check --workspace`
- `cargo clippy -p mssql-orm-migrate -p mssql-orm-cli --all-targets`
- `cargo test --workspace`

### Bloqueos

- No se pudo aplicar el script contra SQL Server real porque el entorno no tiene configuradas variables de conexión (`DATABASE_URL`, `MSSQL_ORM_TEST_CONNECTION_STRING` ni `MSSQL_ORM_SQLCMD_SERVER/USER/PASSWORD`).
- `cargo clippy -p mssql-orm-migrate -p mssql-orm-cli --all-targets` terminó con código 0, pero volvió a reportar warnings preexistentes `collapsible_if` en `crates/mssql-orm-migrate/src/diff.rs`; no se corrigieron por estar fuera del alcance.

### Próximo paso recomendado

- Ejecutar `Etapa 7+: Aplicar contra SQL Server real el script generado desde examples/todo-app para validar creación desde cero e historial idempotente con DATABASE_URL`.

### Sesión: consolidar artefacto editable MVP de migración

- Se ejecutó la subtarea `Etapa 7+: Consolidar el artefacto editable MVP de migration add con up.sql, down.sql, model_snapshot.json y migration.rs explícitamente diferido`, derivada de la tarea grande sobre artefacto editable real.
- El plan maestro se confirmó en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md`; no existe el archivo homónimo en la raíz del repositorio.
- El artefacto editable MVP queda definido como `up.sql`, `down.sql` y `model_snapshot.json`.
- `migration.rs` queda diferido explícitamente para evitar introducir una segunda API de migraciones antes de diseñarla contra el pipeline actual de snapshots, diff y DDL SQL Server.
- `MigrationScaffold` ahora expone helpers de rutas para `up.sql`, `down.sql` y `model_snapshot.json`.
- La plantilla inicial de `down.sql` ahora declara que es rollback manual y que el MVP actual no lo ejecuta automáticamente.
- `mssql-orm-cli migration add` ahora imprime las rutas de los artefactos generados y marca `migration.rs` como diferido para el MVP.
- Se añadió una tarea futura para evaluar generación reversible de `down.sql` cuando las operaciones conserven payload suficiente para invertir cambios de forma segura.
- Se actualizaron `docs/migrations.md` y `docs/context.md` para reflejar el contrato operativo vigente.

### Resultado

- `migration add` deja un artefacto editable más explícito y trazable sin adelantar una API Rust de migraciones que aún no está diseñada.

### Validación

- `cargo fmt --all --check`
- `cargo test -p mssql-orm-migrate`
- `cargo test -p mssql-orm-cli`
- `cargo check --workspace`
- `cargo clippy -p mssql-orm-migrate -p mssql-orm-cli --all-targets`

### Bloqueos

- No hubo bloqueos técnicos.
- `cargo clippy -p mssql-orm-migrate -p mssql-orm-cli --all-targets` terminó con código 0, pero volvió a reportar warnings preexistentes `collapsible_if` en `crates/mssql-orm-migrate/src/diff.rs`; no se corrigieron por estar fuera del alcance.

### Próximo paso recomendado

- Ejecutar `Etapa 7+: Validar end-to-end la generación automática con un consumidor real (examples/todo-app) creando base desde cero y migración incremental reproducible`.

### Sesión: bloquear cambios destructivos en `migration add`

- Se ejecutó la subtarea `Etapa 7+: Detectar cambios destructivos en migration add y bloquear por defecto salvo confirmación/flag explícita alineada con el plan`.
- El plan maestro se confirmó en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md`; no existe el archivo homónimo en la raíz del repositorio.
- `mssql-orm-cli migration add` ahora evalúa el `MigrationPlan` generado antes de crear el scaffold de la nueva migración.
- Si el plan contiene un cambio destructivo, la CLI aborta por defecto con un mensaje explícito e indica usar `--allow-destructive` o editar la migración manualmente.
- Se añadió el flag explícito `--allow-destructive` para permitir generar el artefacto editable cuando el usuario ya revisó el impacto.
- La detección cubre `DropTable`, `DropColumn`, reducción de longitud, cambio de tipo y conversión de nullable a non-nullable sin `default_sql`, alineado con la sección de cambios destructivos del plan.
- Se actualizó `docs/migrations.md` para documentar el nuevo bloqueo por defecto y el bypass explícito.

### Resultado

- `migration add` ya no materializa automáticamente `up.sql` con drops o alteraciones peligrosas salvo confirmación explícita por flag.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo test -p mssql-orm-cli`
- `cargo check --workspace`
- `cargo clippy -p mssql-orm-cli --all-targets`
- `cargo test --workspace`

### Bloqueos

- No hubo bloqueos técnicos.
- `cargo clippy -p mssql-orm-cli --all-targets` terminó con código 0, pero volvió a reportar warnings preexistentes `collapsible_if` en `crates/mssql-orm-migrate/src/diff.rs`; no se corrigieron por estar fuera del alcance de esta subtarea.

### Próximo paso recomendado

- Ejecutar `Etapa 7+: Generar artefacto editable de migración real (manteniendo up.sql, down.sql y snapshot; decidir si migration.rs entra en alcance MVP o queda diferido con límite explícito)`.

### Sesión: generar `up.sql` automático desde el plan compilado

- Se ejecutó la subtarea `Etapa 7+: Generar up.sql automáticamente desde operaciones compiladas y versionar model_snapshot.json con el estado actual del modelo`.
- `mssql-orm-migrate` ahora expone `write_migration_up_sql(...)` para persistir el SQL compilado del plan de migración.
- `mssql-orm-cli migration add` ahora escribe `up.sql` automáticamente cuando dispone de `MigrationPlan`.
- El contenido de `up.sql` se genera a partir del SQL observable de `SqlServerCompiler::compile_migration_operations(...)`; cuando no hay operaciones, se escribe explícitamente `-- No schema changes detected.`.
- `model_snapshot.json` ya venía versionándose con el snapshot actual; esta sesión completa ese artefacto dejando también `up.sql` materializado en el scaffold.
- Se añadieron pruebas en `mssql-orm-migrate` para escritura de `up.sql` con sentencias reales y con caso no-op, y se extendieron pruebas de la CLI para validar el contenido del archivo generado.

### Resultado

- `migration add` ya no solo scaffolda y calcula el plan: ahora deja una migración utilizable con `up.sql` generado automáticamente y `model_snapshot.json` sincronizado con el snapshot actual.

### Validación

- `cargo fmt --all --check`
- `cargo test -p mssql-orm-migrate`
- `cargo test -p mssql-orm-cli`

### Bloqueos

- No hubo bloqueos técnicos.
- Esta sesión no introduce todavía guards sobre cambios destructivos; el plan puede generar `DropTable` o `DropColumn` y volcarlos a `up.sql` mientras la detección/bloqueo siga pendiente como subtarea separada.

### Próximo paso recomendado

- Ejecutar `Etapa 7+: Detectar cambios destructivos en migration add y bloquear por defecto salvo confirmación/flag explícita alineada con el plan`.

### Sesión: integrar diff y compilación SQL en `migration add`

- Se ejecutó la subtarea `Etapa 7+: Integrar el pipeline completo snapshot -> diff -> MigrationOperation -> DDL SQL Server dentro de migration add`.
- `mssql-orm-cli` ahora construye un `MigrationPlan` interno cuando dispone de snapshot actual.
- Ese plan encadena `diff_schema_and_table_operations(...)`, `diff_column_operations(...)`, `diff_relational_operations(...)` y `SqlServerCompiler::compile_migration_operations(...)`.
- `migration add` sigue scaffoldando la migración, pero ahora además reporta `Planned operations` y `Compiled SQL statements` como salida observable del pipeline completo.
- Se añadieron pruebas nuevas en la CLI para el helper `build_migration_plan(...)` y para fijar los conteos del plan en flujos con snapshot actual, snapshot previo local y exportador desde binario consumidor.

### Resultado

- La CLI ya tiene enlazadas todas las piezas estructurales del pipeline de migraciones automáticas hasta el SQL compilado.
- El siguiente paso ya no es de integración de crates sino de materialización: escribir ese SQL en `up.sql`.

### Validación

- `cargo fmt --all --check`
- `cargo test -p mssql-orm-cli`

### Bloqueos

- No hubo bloqueos técnicos.
- Esta sesión no escribió todavía el SQL compilado en `up.sql`; eso queda como la siguiente subtarea explícita del backlog.

### Próximo paso recomendado

- Ejecutar `Etapa 7+: Generar up.sql automáticamente desde operaciones compiladas y versionar model_snapshot.json con el estado actual del modelo`.

### Sesión: cargar snapshot previo local en `migration add`

- Se ejecutó la subtarea `Etapa 7+: Cargar el snapshot previo de la última migración local y generar el snapshot actual desde metadata derivada del modelo`.
- `mssql-orm-migrate` ahora expone `latest_migration(...)` y `read_latest_model_snapshot(...)` para localizar la última migración local y cargar su `model_snapshot.json`.
- `mssql-orm-cli migration add` ahora usa esos helpers cuando ya dispone de un snapshot actual real, ya sea por `--model-snapshot` o por `--snapshot-bin`.
- La salida de `migration add` ahora deja trazado el par base del próximo paso de diff: `Previous snapshot: ...` y `Current snapshot: ...`.
- Se añadieron pruebas unitarias en `mssql-orm-migrate` para orden léxico de la última migración y lectura del snapshot previo, y una prueba nueva en la CLI para verificar que `migration add` carga el snapshot de la última migración local.

### Resultado

- La CLI ya no solo sabe producir el snapshot actual desde el consumidor: también sabe recuperar el snapshot previo local que servirá como lado izquierdo del próximo diff.

### Validación

- `cargo fmt --all --check`
- `cargo test -p mssql-orm-migrate`
- `cargo test -p mssql-orm-cli`

### Bloqueos

- No hubo bloqueos técnicos.
- Esta sesión no integró todavía `diff_schema_and_table_operations`, `diff_column_operations` ni `diff_relational_operations` dentro de `migration add`; ese ensamblaje queda como siguiente subtarea separada.

### Próximo paso recomendado

- Ejecutar `Etapa 7+: Integrar el pipeline completo snapshot -> diff -> MigrationOperation -> DDL SQL Server dentro de migration add`.

### Sesión: exportación de snapshot desde `MigrationModelSource` vía binario consumidor

- Se ejecutó la subtarea `Etapa 7+: Resolver carga/exportación del contexto Rust consumidor desde mssql-orm-cli para generar el ModelSnapshot actual directamente desde MigrationModelSource`.
- La crate pública `mssql-orm` ahora expone `model_snapshot_from_source::<C>()` y `model_snapshot_json_from_source::<C>()`, ambos construidos sobre `MigrationModelSource`.
- `mssql-orm-cli` ahora acepta `migration add <Name> --snapshot-bin <BinName> --manifest-path <Path>`.
- Esa ruta hace que la CLI ejecute `cargo run --quiet --bin <BinName>` sobre el manifest indicado, capture `stdout`, deserialice el `ModelSnapshot` JSON y lo use como snapshot actual de la migración.
- Se preservó el flujo previo `--model-snapshot <Path>` como fallback explícito y ambos modos quedaron marcados como mutuamente excluyentes.
- La cobertura de la CLI ahora incluye una prueba real con un proyecto fixture temporal que depende de la crate pública, deriva `DbContext` y exporta el snapshot usando `model_snapshot_json_from_source::<AppDbContext>()`.
- `docs/migrations.md` ahora documenta el flujo de exportador explícito desde binario consumidor.

### Resultado

- La CLI ya puede obtener el snapshot actual directamente desde código Rust del consumidor sin acoplarse a Tiberius ni cargar tipos dinámicamente dentro del propio proceso.
- La selección concreta del `DbContext` sigue siendo responsabilidad explícita del binario exportador, lo cual mantiene el contrato simple y testeable.

### Validación

- `cargo fmt --all`
- `cargo test -p mssql-orm-cli`
- `cargo test -p mssql-orm --lib`

### Bloqueos

- No hubo bloqueos técnicos.
- Esta sesión no ejecutó todavía el diff contra la última migración local ni la generación automática de `up.sql`; esas piezas siguen como subtareas separadas del backlog.

### Próximo paso recomendado

- Ejecutar `Etapa 7+: Cargar el snapshot previo de la última migración local y generar el snapshot actual desde metadata derivada del modelo`.

### Sesión: entrada explícita de snapshot actual en `migration add`

- Se tomó la subtarea `Etapa 7+: Resolver en mssql-orm-cli el contexto objetivo del consumidor para migration add y obtener desde él el snapshot actual del modelo`.
- Durante la implementación se acotó la tarea: cargar dinámicamente el `DbContext` Rust consumidor y ejecutar `MigrationModelSource` desde la CLI requiere una pieza de exportación/carga separada, por lo que el backlog quedó dividido.
- Se completó la parte verificable inmediata: `migration add <Name> --model-snapshot <Path>` permite pasar a la CLI un `ModelSnapshot` actual explícito.
- `mssql-orm-cli` ahora parsea el flag opcional `--model-snapshot`, resuelve rutas relativas contra el root del proyecto y falla con mensaje contextual si no puede leer o deserializar el snapshot.
- `mssql-orm-migrate` ahora expone `create_migration_scaffold_with_snapshot(...)` para crear scaffolds versionando un snapshot provisto.
- Se agregó cobertura en la CLI para parsing del nuevo flag y para comprobar que el `model_snapshot.json` de la migración contiene el snapshot de entrada.
- `docs/migrations.md` documenta el nuevo uso explícito y deja claro que la carga automática del `DbContext` consumidor sigue pendiente.

### Resultado

- `migration add` ya puede obtener y versionar un snapshot actual real cuando el consumidor o un fixture lo provee como JSON.
- La carga/exportación automática del contexto Rust consumidor queda como subtarea pendiente separada y explícita en `docs/tasks.md`.

### Validación

- `cargo fmt --all --check`
- `cargo test -p mssql-orm-cli`
- `cargo test -p mssql-orm-migrate`
- `cargo check --workspace`
- `cargo clippy -p mssql-orm-cli --all-targets && cargo clippy -p mssql-orm-migrate --all-targets`

### Bloqueos

- No hubo bloqueos técnicos para el alcance completado.
- `cargo clippy -p mssql-orm-migrate --all-targets` terminó con código 0, pero volvió a reportar warnings preexistentes `collapsible_if` en `crates/mssql-orm-migrate/src/diff.rs`; no se corrigieron porque son limpieza ajena a la tarea.

### Próximo paso recomendado

- Ejecutar `Etapa 7+: Resolver carga/exportación del contexto Rust consumidor desde mssql-orm-cli para generar el ModelSnapshot actual directamente desde MigrationModelSource`.

### Sesión: serializar snapshots de modelo para migraciones

- Se ejecutó la subtarea `Etapa 7+: Serializar y deserializar ModelSnapshot y artefactos relacionados para reemplazar el model_snapshot.json placeholder actual`.
- El plan maestro se encontró en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md`; no existe el archivo homónimo en la raíz del repositorio.
- `mssql-orm-migrate` ahora depende directamente de `serde` y `serde_json` para persistir snapshots del modelo.
- `ModelSnapshot`, `SchemaSnapshot`, `TableSnapshot`, `ColumnSnapshot`, `IndexSnapshot`, `IndexColumnSnapshot` y `ForeignKeySnapshot` ahora serializan y deserializan JSON.
- Se añadieron `ModelSnapshot::to_json_pretty()` y `ModelSnapshot::from_json(...)`, con codificación explícita para tipos SQL Server, identity y acciones referenciales sin exigir `serde` en `mssql-orm-core`.
- El filesystem de migraciones ahora expone `write_model_snapshot(...)` y `read_model_snapshot(...)`.
- `create_migration_scaffold(...)` ya escribe `model_snapshot.json` mediante la API de snapshot, reemplazando el placeholder manual.
- Se agregaron pruebas de roundtrip JSON y lectura/escritura del artefacto `model_snapshot.json`.

### Resultado

- La crate de migraciones ya tiene una base persistible real para versionar snapshots de modelo y para que las siguientes subtareas puedan cargar el snapshot previo y comparar contra el modelo actual.

### Validación

- `cargo fmt --all --check`
- `cargo test -p mssql-orm-migrate`
- `cargo check --workspace`
- `cargo clippy --workspace --all-targets --all-features`

### Bloqueos

- No hubo bloqueos técnicos.
- `cargo clippy --workspace --all-targets --all-features` terminó con código 0, pero reportó warnings preexistentes no corregidos en esta sesión: `collapsible_if` en `crates/mssql-orm-migrate/src/diff.rs` y `large_enum_variant` en `crates/mssql-orm/src/context.rs`.
- No se realizó commit al cierre porque el worktree ya tenía cambios previos no originados en esta sesión (`mssql-orm-macros`, crate pública, fixture `dbcontext_valid` y documentación relacionada con `MigrationModelSource`), además de un artefacto no trackeado en `examples/todo-app/database_update.sql`; mezclar esos cambios con esta tarea en un commit automático perdería trazabilidad.

### Próximo paso recomendado

- Ejecutar `Etapa 7+: Resolver en mssql-orm-cli el contexto objetivo del consumidor para migration add y obtener desde él el snapshot actual del modelo`.

## 2026-04-23

### Sesión: exponer metadata de entidades desde `DbContext` para migraciones

- Se ejecutó la subtarea `Etapa 7+: Exponer desde #[derive(DbContext)] una fuente estable de metadata de entidades para migraciones (entity_metadata() o equivalente) sin acoplar core a la CLI`.
- La crate pública `mssql-orm` ahora expone el trait `MigrationModelSource`.
- `#[derive(DbContext)]` ahora implementa automáticamente ese trait y publica `entity_metadata()` para el conjunto de entidades declarado en el contexto.
- La implementación se resolvió con inicialización perezosa (`OnceLock`) para evitar exigir `const fn` sobre `Entity::metadata()` y mantener una referencia `'static` estable.
- Se amplió la cobertura pública en `crates/mssql-orm/src/lib.rs` y en `crates/mssql-orm/tests/ui/dbcontext_valid.rs` para fijar el nuevo contrato tanto en tests internos como desde `trybuild`.

### Resultado

- El repo ya tiene la primera pieza estructural necesaria para que la CLI pueda pedirle al consumidor su modelo actual sin reflexión runtime ni acoplar `core` a `mssql-orm-cli`.

### Validación

- `cargo fmt --all --check`
- `cargo test -p mssql-orm --lib`
- `cargo test -p mssql-orm --test trybuild`

### Bloqueos

- No hubo bloqueos permanentes.
- Durante la implementación apareció un problema de lifetime/const-eval al devolver metadata desde el derive; se corrigió reemplazando el `static` directo por `OnceLock`.

### Próximo paso recomendado

- Ejecutar `Etapa 7+: Serializar y deserializar ModelSnapshot y artefactos relacionados para reemplazar el model_snapshot.json placeholder actual`.

### Sesión: descomponer backlog para migraciones automáticas desde el modelo

- Se revisó el plan maestro en `docs/plan_orm_sqlserver_tiberius_code_first.md` y se confirmó la desalineación actual: el plan exige que `migration add` genere migraciones automáticamente desde structs Rust, mientras que la CLI vigente solo scaffolda archivos y genera el script acumulado de `database update`.
- Se ejecutó la tarea operativa `Operativo: Descomponer la generación automática de migraciones desde DbContext hacia subtareas verificables alineadas con el plan maestro`.
- `docs/tasks.md` ahora incluye una secuencia explícita de trabajo para cerrar ese gap sin mezclarlo con la Etapa 15 de documentación: exponer metadata desde `DbContext`, persistir snapshots reales, resolver el contexto del consumidor en CLI, integrar `snapshot -> diff -> DDL`, bloquear cambios destructivos por defecto y validar el flujo completo con `examples/todo-app`.
- También se actualizó `docs/context.md` para dejar trazada esa brecha entre la surface pública actual y el comportamiento esperado por el plan.

### Resultado

- El repositorio ya tiene backlog estructurado y detallado para llevar la CLI de migraciones desde scaffold manual hasta generación automática desde el modelo Rust, con subtareas verificables y orden lógico de implementación.

### Validación

- Revisión manual de consistencia entre `docs/plan_orm_sqlserver_tiberius_code_first.md`, `docs/tasks.md` y `docs/context.md`.

### Bloqueos

- No hubo bloqueos técnicos.
- No aplican validaciones `cargo` en esta sesión porque el cambio fue exclusivamente documental y de backlog.

### Próximo paso recomendado

- Mantener el foco inmediato de release en `Etapa 15: Preparar guía del query builder público (filter, order_by, joins, take, paginate, count)` y retomar luego la primera subtarea estructural de migraciones automáticas.

### Sesión: preparar guía pública de migraciones

- Se ejecutó la subtarea `Etapa 15: Preparar guía de migraciones (migration add, migration list, database update) y límites explícitos`.
- Se añadió `docs/migrations.md` con una guía centrada en cómo trabajar bien con la CLI actual de migraciones.
- La guía explica el flujo recomendado real: cambiar entidades, crear scaffold con `migration add`, editar `up.sql` y `down.sql`, generar el script con `database update`, revisarlo y aplicarlo externamente con una herramienta como `sqlcmd`.
- También deja explícitos los límites de la surface actual: la CLI no ejecuta el SQL directamente, no consume `down.sql`, no expone downgrade público y todavía no genera `up.sql` automáticamente desde las entidades.
- `README.md` ahora enlaza la nueva guía de migraciones como parte del surface documental público del release.

### Resultado

- El repositorio ya tiene una guía de migraciones práctica y honesta, enfocada en disciplina operativa real y no en promesas futuras del roadmap.

### Validación

- `cargo fmt --all --check`
- `cargo test -p mssql-orm-cli`

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Ejecutar `Etapa 15: Preparar guía del query builder público (filter, order_by, joins, take, paginate, count)`.

### Sesión: preparar guía `code-first` pública

- Se ejecutó la subtarea `Etapa 15: Preparar guía code-first alineada con derives, DbContext, DbSet y límites actuales`.
- Se añadió `docs/code-first.md` como guía pública específica del enfoque `code-first` actual del proyecto.
- La guía deja explícito qué piezas componen hoy esa experiencia: `#[derive(Entity)]`, `#[derive(Insertable)]`, `#[derive(Changeset)]`, `#[derive(DbContext)]`, `DbSet<T>`, relaciones por `foreign_key` y los límites vigentes de la surface.
- Se evitó documentar como disponible lo que todavía es roadmap: la guía aclara que no existe aún una capa de fluent configuration pública y que el soporte sigue centrado en SQL Server y primary keys simples para la ruta base de `DbSet`.
- `README.md` ahora enlaza también la nueva guía `code-first` para que forme parte del surface documental público del release.
- Para mantener la documentación honesta respecto a la API real, se añadió `crates/mssql-orm/tests/ui/code_first_public_valid.rs` y se registró en `crates/mssql-orm/tests/trybuild.rs`.

### Resultado

- El repositorio ya tiene una guía `code-first` pública y verificable, separada del quickstart y alineada con la surface real de derives, `DbContext` y `DbSet`.

### Validación

- `cargo fmt --all --check`
- `cargo test -p mssql-orm --test trybuild`

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Ejecutar `Etapa 15: Preparar guía de migraciones (migration add, migration list, database update) y límites explícitos`.

### Sesión: re-alinear la descomposición de Etapa 15 con el plan maestro

- Se retomó como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md`; no existe una copia vigente en la raíz del repositorio.
- Se ejecutó la tarea operativa `Operativo: Descomponer la Etapa 15 de release en subtareas verificables y ordenadas`.
- `docs/tasks.md` quedó corregido porque la descomposición anterior era incompleta frente a los entregables de Etapa 15 definidos en el plan maestro.
- Se agregaron subtareas pendientes explícitas para las guías todavía no representadas en el backlog: `code-first`, migraciones, query builder, transacciones, relaciones y `API docs`.
- También se actualizó `docs/context.md` para reflejar que la Etapa 15 sigue abierta y que el próximo foco ya no es solo el changelog, sino primero completar esas guías faltantes del release.

### Resultado

- La Etapa 15 volvió a quedar trazada de forma consistente entre plan maestro, backlog y contexto operativo.
- El backlog ahora cubre todos los entregables documentales del release mencionados en el plan, en lugar de una versión reducida a README, quickstart, ejemplos, changelog y validación final.

### Validación

- Revisión manual de consistencia entre `docs/plan_orm_sqlserver_tiberius_code_first.md`, `docs/tasks.md` y `docs/context.md`.
- Verificación de que `docs/tasks.md` mantiene exactamente las secciones `Pendientes`, `En Progreso` y `Completadas`.

### Bloqueos

- No hubo bloqueos técnicos.
- No aplican validaciones `cargo` en esta sesión porque el cambio fue exclusivamente documental y no tocó código Rust ni manifests.

### Próximo paso recomendado

- Ejecutar `Etapa 15: Preparar guía code-first alineada con derives, DbContext, DbSet y límites actuales`.

### Sesión: sanitizar credenciales en documentación

- Se retiraron cadenas de conexión y credenciales reales de la documentación vigente y del historial operativo.
- `README.md`, `docs/quickstart.md`, `examples/README.md`, `examples/todo-app/README.md`, `docs/context.md` y `docs/worklog.md` ahora usan placeholders como `<usuario>` y `<password>` en lugar de valores reales.
- La documentación mantiene la forma de los comandos y connection strings, pero ya no expone secretos del entorno local.

### Resultado

- La documentación quedó apta para compartirse sin revelar credenciales reales usadas durante validaciones locales.

### Validación

- Búsqueda global en `README.md`, `docs/` y `examples/` para confirmar que no quedaran cadenas sensibles previas.

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Continuar con `Etapa 15: Preparar changelog inicial del release con surface disponible y exclusiones explícitas`.

### Sesión: consolidar ejemplos y guías operativas del release

- Se ejecutó la subtarea `Etapa 15: Consolidar ejemplos ejecutables y guías de uso (todo_app, variables de entorno y smoke local, y resolver la inconsistencia documental de basic-crud)`.
- Se añadió `examples/README.md` como índice público del árbol `examples/`, dejando claro que el ejemplo ejecutable real disponible hoy es `todo-app`.
- `README.md` principal ahora enlaza ese índice de ejemplos.
- `examples/todo-app/README.md` ahora también enlaza el índice general de `examples/`.
- `docs/context.md` se mantuvo alineado con el árbol real: `todo-app` figura como ejemplo actual disponible y `basic-crud` queda solo como referencia histórica en el `worklog`.
- Se preservó la información operativa útil del ejemplo real: variables de entorno, comando de ejecución y smoke local con `sqlcmd`.

### Resultado

- La documentación vigente del release ya es consistente con el árbol real del repositorio: `todo-app` es el ejemplo disponible y `basic-crud` ya no aparece como artefacto ejecutable actual.

### Validación

- `cargo fmt --all --check`
- `cargo test --manifest-path examples/todo-app/Cargo.toml`
- Revisión manual de `README.md`, `examples/README.md`, `examples/todo-app/README.md` y `docs/context.md`

### Bloqueos

- No hubo bloqueos técnicos.
- Las referencias a `basic-crud` se preservan en el historial (`docs/worklog.md`) porque siguen siendo parte del registro de sesiones; la corrección se aplicó sobre la documentación vigente y no sobre el histórico.

### Próximo paso recomendado

- Ejecutar `Etapa 15: Preparar changelog inicial del release con surface disponible y exclusiones explícitas`.

### Sesión: quickstart reproducible para la API pública

- Se ejecutó la subtarea `Etapa 15: Preparar quickstart reproducible para conexión, CRUD base y query builder público`.
- Se añadió [docs/quickstart.md](/home/esteban94/Proyectos/Rust/mssql-orm/docs/quickstart.md) con una guía paso a paso para:
  preparar una tabla SQL Server,
  crear un proyecto Rust,
  declarar `Entity`, `Insertable`, `Changeset` y `DbContext`,
  conectar con `DbContext::connect(...)`,
  usar `insert`, `find`, `query().filter().order_by().take().all()`, `update` y `delete`.
- El `README.md` principal ahora enlaza ese quickstart y ya no presenta `basic-crud` como ejemplo ejecutable existente en el árbol actual.
- Para mantener la guía honesta respecto a la API real, se añadió `crates/mssql-orm/tests/ui/quickstart_public_valid.rs` y se registró en `crates/mssql-orm/tests/trybuild.rs`; así el snippet público del quickstart queda cubierto por compilación.
- Durante la revisión apareció una inconsistencia documental relevante: `docs/` todavía menciona `examples/basic-crud/`, pero ese ejemplo ya no existe en el árbol actual. Esa corrección queda explicitada como parte de la siguiente subtarea de consolidación de ejemplos.

### Resultado

- El repositorio ya tiene un quickstart reproducible y validado por `trybuild`, alineado con la surface pública actual.

### Validación

- `cargo fmt --all --check`
- `cargo test -p mssql-orm --test trybuild`

### Bloqueos

- No hubo bloqueos técnicos.
- Quedó detectada una inconsistencia documental sobre `basic-crud`; no bloquea el quickstart, pero sí debe resolverse en la subtarea siguiente de ejemplos y guías operativas.

### Próximo paso recomendado

- Ejecutar `Etapa 15: Consolidar ejemplos ejecutables y guías de uso (todo_app, variables de entorno y smoke local, y resolver la inconsistencia documental de basic-crud)`.

### Sesión: rehacer el `README` principal como landing pública

- Se ejecutó la subtarea `Etapa 15: Consolidar documentación pública base del release (README, alcance actual, estado por etapas y límites explícitos)`.
- El `README.md` principal se reescribió completo para reflejar el estado real del repositorio y funcionar como presentación pública de la librería, con tono más comercial/didáctico como pidió el usuario.
- El nuevo `README` ahora prioriza propuesta de valor, capacidades ya implementadas, ejemplo corto de uso, arquitectura por crates, ejemplos incluidos, estado real del roadmap, límites explícitos y comandos de validación.
- También se eliminaron afirmaciones viejas que ya no eran ciertas, como la idea de que el repo seguía en etapa de placeholders fundacionales.

### Resultado

- El repositorio ya tiene un `README` principal coherente con el código actual y orientado a “vender” la librería sin inventar capacidades no implementadas.

### Validación

- `cargo fmt --all --check`
- Revisión manual del contenido final de `README.md`

### Bloqueos

- No hubo bloqueos técnicos.
- Esta subtarea no introdujo cambios de código, solo documentación pública principal.

### Próximo paso recomendado

- Ejecutar `Etapa 15: Preparar quickstart reproducible para conexión, CRUD base y query builder público`.

### Sesión: registrar criterio editorial del `README` principal

- El usuario dejó un criterio adicional para la Etapa 15: el `README.md` principal del repositorio debe ser llamativo y didáctico, con tono de presentación pública de la librería, no solo como referencia técnica interna.
- Se registró esa preferencia en `docs/context.md` para que las próximas sesiones de release/documentación la tomen como requisito editorial explícito.

### Resultado

- El backlog de release mantiene el mismo scope técnico, pero ahora con una guía clara sobre el tono y objetivo del `README` principal.

### Validación

- Revisión manual de consistencia entre la preferencia del usuario y el foco actual de la Etapa 15.

### Bloqueos

- No hubo bloqueos técnicos.
- Esta sesión solo deja trazabilidad editorial; no introduce cambios de código.

### Próximo paso recomendado

- Ejecutar `Etapa 15: Consolidar documentación pública base del release (README, alcance actual, estado por etapas y límites explícitos)` aplicando ese enfoque más comercial y didáctico.

### Sesión: descomponer la Etapa 15 de release

- Se revisó el backlog tras cerrar Etapa 14 y la re-alineación de `FromRow`.
- La tarea monolítica `Etapa 15: Preparar release con documentación pública, quickstart, ejemplos completos y changelog` se dividió en subtareas pequeñas y verificables dentro de `docs/tasks.md`.
- La descomposición quedó separada en cinco entregables concretos: consolidación de documentación pública base, quickstart reproducible, consolidación de ejemplos ejecutables, changelog inicial y validación final de release.
- También se añadió una tarea operativa explícita para reflejar que la propia descomposición de Etapa 15 ya quedó hecha y trazada en el backlog.

### Resultado

- La Etapa 15 ya no depende de una tarea amplia y ambigua; el backlog quedó listo para ejecutarse por partes sin mezclar documentación, ejemplos, changelog y validación final en una sola sesión.

### Validación

- Revisión manual de consistencia de `docs/tasks.md` contra el estado actual del repositorio y el objetivo de release.

### Bloqueos

- No hubo bloqueos técnicos.
- Esta sesión solo reorganizó backlog; no introdujo cambios de código.

### Próximo paso recomendado

- Empezar por `Etapa 15: Consolidar documentación pública base del release (README, alcance actual, estado por etapas y límites explícitos)`.

### Sesión: re-alinear `FromRow` con el plan maestro

- Se retomó como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md` y se ejecutó la tarea operativa pendiente: `Re-alinear el row mapping con el plan maestro generando FromRow automático desde derive y retirando boilerplate manual de ejemplos/tests`.
- `crates/mssql-orm-macros/src/lib.rs` ahora hace que `#[derive(Entity)]` emita también `impl ::mssql_orm::core::FromRow`, reutilizando `get_required_typed::<T>()` para campos no opcionales y `try_get_typed::<Option<T>>()?.flatten()` para campos `Option<T>`.
- Se retiraron implementaciones manuales de `FromRow` ya redundantes en `examples/todo-app`, en tests de integración de la crate pública (`stage5_public_crud`, `stage10_public_active_record`) y en fixtures válidos de `trybuild`.
- Se añadió cobertura directa en `crates/mssql-orm/tests/stage2_mapping.rs` para comprobar que una entidad derivada materializa correctamente columnas requeridas y nullable sin impl manual adicional.
- Durante la validación solo fue necesario actualizar un snapshot `trybuild` (`active_record_missing_entity_set.stderr`) por cambio de numeración de líneas y limpiar un import de prueba que dejó de usarse.

### Resultado

- El repositorio ya quedó alineado con el plan maestro en este punto: `#[derive(Entity)]` genera `FromRow` automáticamente y el boilerplate manual desapareció de consumidores reales representativos.

### Validación

- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test --workspace`
- `cargo test --manifest-path examples/todo-app/Cargo.toml`

### Bloqueos

- No hubo bloqueos funcionales persistentes.
- La tarea requirió ajustar fixtures `trybuild` existentes porque el derive nuevo redujo código y movió líneas de error observables.

### Próximo paso recomendado

- Empezar `Etapa 15: Preparar release con documentación pública, quickstart, ejemplos completos y changelog`.

### Sesión: registrar desalineación de `FromRow` contra el plan maestro

- Se revisó el plan maestro y luego el código real de `mssql-orm-macros` para verificar si `FromRow` ya se estaba generando desde derives de entidad.
- La revisión confirmó una desalineación concreta: el plan dice que los derives de entidad deben generar `FromRow`, pero el código actual de `#[derive(Entity)]` no emite ese impl; hoy solo genera metadata, columnas estáticas y helpers de persistencia/PK.
- También quedó confirmado que `#[derive(DbContext)]` todavía exige `FromRow` en los bounds de las entidades y que por eso siguen existiendo implementaciones manuales en `examples/todo-app` y en fixtures válidos de `trybuild`.
- Se registró una tarea operativa nueva en `docs/tasks.md` para re-alinear esa parte del repo con el plan maestro sin mezclarla todavía con la Etapa 15 de release.

### Resultado

- El backlog y el contexto ahora dejan explícita una deuda estructural real del proyecto: falta generación automática de `FromRow`.

### Validación

- Revisión de `docs/plan_orm_sqlserver_tiberius_code_first.md`
- Revisión de `crates/mssql-orm-macros/src/lib.rs`
- Revisión de usos manuales de `FromRow` en `examples/` y `crates/mssql-orm/tests/ui/`

### Bloqueos

- No hubo bloqueos técnicos.
- Esta sesión solo dejó trazabilidad documental; no introdujo cambios de código.

### Próximo paso recomendado

- Ejecutar la tarea operativa de re-alineación e introducir generación automática de `FromRow` sin romper `Entity`, `DbContext`, Active Record ni los fixtures públicos existentes.

### Sesión: validación real de `todo_app` contra SQL Server

- Se retomó como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md` y se ejecutó la subtarea prioritaria de Etapa 14: `Validar el ejemplo web async todo_app contra SQL Server real con smoke test/documentación operativa reproducible`.
- Se añadió `examples/todo-app/scripts/smoke_setup.sql` como fixture reproducible para `tempdb`, creando `todo.users`, `todo.todo_lists` y `todo.todo_items` con datos mínimos para el smoke del ejemplo.
- Ese fixture deja documentado un matiz operativo de SQL Server: la combinación `users -> todo_lists ON DELETE CASCADE`, `todo_lists -> todo_items ON DELETE CASCADE` y `todo_items.completed_by_user_id -> users ON DELETE SET NULL` cae en `multiple cascade paths`, así que el script de smoke usa `NO ACTION` para esa FK porque la validación aquí es de lectura y no de borrado.
- La validación real se ejecutó con `sqlcmd` sobre SQL Server local (`localhost`, `tempdb`) y luego con el binario del ejemplo usando una `DATABASE_URL` local propia del entorno.
- El smoke HTTP verificó con `curl` las rutas reales del ejemplo:
  `GET /health`,
  `GET /todo-lists/10`,
  `GET /users/7/todo-lists?page=1&page_size=20`,
  `GET /todo-lists/10/items/preview?limit=2`,
  `GET /todo-lists/10/open-items/count`.
- Durante la validación apareció un fallo real del ejemplo, no de la documentación: `TodoItem::from_row` estaba leyendo columnas nullable con `try_get_typed::<i64>` y `try_get_typed::<String>`, lo que fallaba contra filas con `NULL`. Se corrigió en `examples/todo-app/src/domain.rs` usando `try_get_typed::<Option<_>>()?.flatten()`, y se sincronizó el fixture público de compile-time en `crates/mssql-orm/tests/ui/query_builder_todo_app_valid.rs`.
- Además se dejó una prueba ignorada pero ejecutable del propio ejemplo (`smoke_preview_query_runs_against_sql_server_fixture`) que usa `DATABASE_URL`, `connect_pool(...)` y `open_items_preview_query(...)` para repetir el smoke de lectura sin pasar por HTTP.
- En paralelo quedó incorporado en `crates/mssql-orm-tiberius/src/row.rs` el soporte de `ColumnType::Intn`, ampliando la lectura real del adaptador Tiberius para enteros SQL Server de anchura variable.

### Resultado

- La Etapa 14 quedó cerrada end-to-end: `todo_app` ya no solo compila y se prueba en local, sino que además quedó validado contra SQL Server real con fixture reproducible, smoke HTTP observable y prueba ignorada reutilizable desde el propio ejemplo.

### Validación

- `sqlcmd -S localhost -U '<usuario>' -P '<password>' -d tempdb -C -b -i examples/todo-app/scripts/smoke_setup.sql`
- `DATABASE_URL='Server=localhost;Database=tempdb;User Id=<usuario>;Password=<password>;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;' cargo test --manifest-path examples/todo-app/Cargo.toml smoke_preview_query_runs_against_sql_server_fixture -- --ignored --nocapture`
- `cargo test --manifest-path examples/todo-app/Cargo.toml`
- Smoke HTTP manual con `cargo run --manifest-path examples/todo-app/Cargo.toml` y `curl` sobre `/health`, `/todo-lists/10`, `/users/7/todo-lists?page=1&page_size=20`, `/todo-lists/10/items/preview?limit=2` y `/todo-lists/10/open-items/count`

### Bloqueos

- No quedaron bloqueos funcionales abiertos.
- La fixture de smoke no replica exactamente el `ON DELETE SET NULL` del dominio porque SQL Server rechaza esa combinación concreta de cascadas en un esquema tan compacto; esa diferencia quedó documentada de forma explícita y acotada al fixture operativo.

### Próximo paso recomendado

- Empezar `Etapa 15: Preparar release con documentación pública, quickstart, ejemplos completos y changelog`.

### Sesión: wiring con `MssqlPool` en el ejemplo `todo_app`

- Se retomó como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md` y se ejecutó la subtarea prioritaria de Etapa 14: `Integrar MssqlPool y DbContext::from_pool(...) en el ejemplo web async todo_app con coverage feature-gated del wiring del consumidor`.
- `examples/todo-app/src/lib.rs` ahora expone helpers explícitos feature-gated para el consumidor del ejemplo: `pool_builder_from_settings(...)`, `connect_pool(...)` y `state_from_pool(...)`.
- Ese wiring reutiliza exactamente la surface pública ya existente: `MssqlPool::builder().with_pool_options(...)`, `MssqlPoolBuilder::connect_with_config(...)` y `TodoAppDbContext::from_pool(...)`; no se introdujeron atajos nuevos fuera del ejemplo.
- `examples/todo-app/src/main.rs` ahora usa `connect_pool(&settings).await?` y `state_from_pool(pool, settings.clone())` cuando `pool-bb8` está activo, dejando el fallback a `PendingTodoAppDbContext` solo para builds sin ese feature.
- La cobertura del ejemplo ahora fija dos puntos importantes del wiring del consumidor bajo `pool-bb8`: que el builder hereda exactamente `settings.operational_options.pool` y que el ejemplo expone `TodoAppDbContext::from_pool`, `state_from_pool` y `connect_pool` como entrada pública coherente.

### Resultado

- El ejemplo `todo_app` ya usa la ruta real de pooling del ORM en su `main.rs` y dejó de depender de wiring ficticio para el caso normal con `pool-bb8`.

### Validación

- `cargo fmt --all --check` en `examples/todo-app/`
- `cargo check --manifest-path examples/todo-app/Cargo.toml`
- `cargo test --manifest-path examples/todo-app/Cargo.toml`

### Bloqueos

- No hubo bloqueos funcionales persistentes.
- Todavía no se ejecutó el ejemplo contra SQL Server real en esta subtarea; eso queda como el siguiente paso explícito del backlog.

### Próximo paso recomendado

- Implementar `Etapa 14: Validar el ejemplo web async todo_app contra SQL Server real con smoke test/documentación operativa reproducible`.

### Sesión: endpoints mínimos de lectura para `todo_app`

- Se retomó como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md` y se ejecutó la subtarea prioritaria de Etapa 14: `Implementar endpoints mínimos del todo_app usando DbSet y cubrir la lógica HTTP con pruebas unitarias o de servicio local`.
- Se implementó `examples/todo-app/src/http.rs` como módulo HTTP específico del ejemplo, con DTOs mínimos serializables, parámetros de query (`PageParams`, `PreviewParams`) y handlers para:
  `GET /todo-lists/{list_id}`,
  `GET /users/{user_id}/todo-lists`,
  `GET /todo-lists/{list_id}/items/preview`,
  `GET /todo-lists/{list_id}/open-items/count`.
- El módulo define además el trait de lectura `TodoAppApi`; su implementación real sobre `TodoAppDbContext` usa `DbSet::find`, `DbSetQuery::all()` y `DbSetQuery::count()` mediante la surface pública del ORM y las queries del ejemplo ya definidas.
- `examples/todo-app/src/lib.rs` ahora registra esas rutas en `build_app(...)` y hace que el ejemplo compile también con `PendingTodoAppDbContext`, retornando errores explícitos mientras el wiring real con pool sigue pendiente.
- Se añadió `serde` como dependencia directa del ejemplo para modelar payloads/params HTTP de forma explícita y estable; no afecta el workspace principal ni mueve responsabilidades arquitectónicas fuera del ejemplo consumidor.
- Las pruebas unitarias del ejemplo ahora cubren los handlers mínimos con un `FakeDbContext` propio del ejemplo, verificando `404` para listas inexistentes, filtrado de listas archivadas, preview limitado de ítems abiertos y conteo JSON de ítems abiertos.

### Resultado

- El ejemplo `todo_app` ya muestra un uso real del ORM desde HTTP: `find`, `query`, `all` y `count` quedan visibles detrás de endpoints mínimos del consumidor, no solo en tests o helpers internos.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo fmt --all --check` en `examples/todo-app/`
- `cargo check --workspace`
- `cargo check --manifest-path examples/todo-app/Cargo.toml`
- `cargo test --manifest-path examples/todo-app/Cargo.toml`

### Bloqueos

- No hubo bloqueos funcionales persistentes.
- Los endpoints quedan montados sobre `PendingTodoAppDbContext` en `main.rs` hasta integrar el wiring real con `MssqlPool`; eso es deliberado para no adelantar la siguiente subtarea del backlog.

### Próximo paso recomendado

- Implementar `Etapa 14: Integrar MssqlPool y DbContext::from_pool(...) en el ejemplo web async todo_app con coverage feature-gated del wiring del consumidor`.

### Sesión: health check HTTP real en el ejemplo `todo_app`

- Se retomó como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md` y se ejecutó la subtarea prioritaria de Etapa 14: `Implementar endpoint de health check del ejemplo web async reutilizando DbContext::health_check() y cubrirlo con pruebas de handler`.
- Se mantuvo la implementación dentro del ejemplo real, no como fixture aislado de tests: `examples/todo-app/src/db.rs` ahora define `TodoAppDbContext` derivado con `DbSet<User>`, `DbSet<TodoList>` y `DbSet<TodoItem>`.
- `examples/todo-app/src/lib.rs` ahora registra `GET /health` en `build_app(...)` y expone `health_check_handler(...)`, que delega en `state.db.health_check().await` y traduce el resultado a `200 ok` o `503 database unavailable`.
- Para no adelantar todavía la subtarea de pool, `examples/todo-app/src/main.rs` usa temporalmente `PendingTodoAppDbContext`; esto deja el ejemplo compilable y expone el endpoint HTTP real, pero sigue haciendo explícito que el wiring de base de datos queda pendiente para la siguiente fase.
- Como `#[derive(DbContext)]` exige que las entidades puedan mapearse desde filas, `examples/todo-app/src/domain.rs` ahora implementa `FromRow` para `User`, `TodoList` y `TodoItem`, preparando además el terreno para los endpoints CRUD mínimos que siguen en el backlog.
- A raíz del feedback del usuario, `examples/todo-app/src/queries.rs` dejó de exponer construcción manual de `SelectQuery` como surface del ejemplo y ahora muestra uso real desde consumidor: `db.todo_lists.query()...`, `db.todo_items.query()...`, manteniendo los helpers AST solo para pruebas internas.
- La cobertura del ejemplo ahora incluye pruebas unitarias del handler de health check con un `FakeDbContext` que devuelve `Ok(())` o error, además de seguir validando dominio, queries y arranque.

### Resultado

- El ejemplo `todo_app` ya expone un endpoint HTTP real de health check y muestra consultas idiomáticas desde `DbContext`, listo para que la siguiente subtarea agregue endpoints mínimos sobre la misma base.

### Validación

- `cargo fmt --all --check`
- `cargo fmt --all --check` en `examples/todo-app/`
- `cargo check --workspace`
- `cargo check --manifest-path examples/todo-app/Cargo.toml`
- `cargo test --manifest-path examples/todo-app/Cargo.toml`

### Bloqueos

- No hubo bloqueos funcionales persistentes.
- El endpoint `/health` queda montado sobre `PendingTodoAppDbContext` en `main.rs` hasta integrar el wiring real con `MssqlPool`; eso es deliberado para no adelantar la siguiente subtarea del backlog.

### Próximo paso recomendado

- Implementar `Etapa 14: Implementar endpoints mínimos del todo_app usando DbSet y cubrir la lógica HTTP con pruebas unitarias o de servicio local`.

### Sesión: consultas públicas del dominio `todo_app`

- Se retomó como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md` y se ejecutó la subtarea prioritaria de Etapa 14: `Cubrir consultas públicas usadas por todo_app para filter, order_by, joins, limit, take, paginate y count`.
- Se movió en `docs/tasks.md` la subtarea a `En Progreso` antes de editar y, tras validarla, se dejó cerrada y se avanzó la siguiente subtarea a `En Progreso`.
- Se añadió `examples/todo-app/src/queries.rs` como módulo reutilizable del ejemplo, con queries puras para `user_lists_page_query`, `list_items_page_query`, `open_items_preview_query` y `open_items_count_query`.
- Ese módulo cubre el shape real que el ejemplo podrá reutilizar después en handlers/servicios: filtros por owner/lista, ordenamiento por `title` y `position`, joins entre `todo_items`, `todo_lists` y `users`, paginación explícita para páginas y previews, y conteo de ítems abiertos.
- Las pruebas unitarias del ejemplo ahora validan tanto el AST esperado como el SQL Server compilado para las consultas principales de listado y conteo, sin mover compilación SQL fuera de `mssql-orm-sqlserver`.
- Se añadió el fixture `crates/mssql-orm/tests/ui/query_builder_todo_app_valid.rs` y su registro en `crates/mssql-orm/tests/trybuild.rs` para fijar en compile-time el uso público del query builder del ejemplo, incluyendo `filter`, `order_by`, `inner_join`, `left_join`, `limit`, `take`, `paginate` y `count`.
- Durante la validación apareció un detalle de Rust, no del ORM: una closure `async` que devolvía `query.count().await` sobre `&DbContext` introducía un problema de lifetime en `trybuild`. Se sustituyó por una `async fn` explícita para dejar la cobertura estable.

### Resultado

- La Etapa 14 ya tiene consultas públicas reutilizables para el ejemplo `todo_app`, con cobertura observable sobre AST, SQL compilado y type-check del query builder público usado por consumidores.

### Validación

- `cargo fmt --all --check`
- `cargo fmt --all --check` en `examples/todo-app/`
- `cargo check --workspace`
- `cargo check --manifest-path examples/todo-app/Cargo.toml`
- `cargo test --manifest-path examples/todo-app/Cargo.toml`
- `cargo test -p mssql-orm --test trybuild`

### Bloqueos

- No hubo bloqueos funcionales persistentes.
- Durante la validación hubo esperas breves por file locks de `cargo` al ejecutar verificaciones concurrentes.

### Próximo paso recomendado

- Implementar `Etapa 14: Implementar endpoint de health check del ejemplo web async reutilizando DbContext::health_check() y cubrirlo con pruebas de handler`.

### Sesión: dominio base de `todo_app` con metadata relacional

- Se retomó como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md` y se ejecutó la subtarea prioritaria de Etapa 14: `Definir el dominio todo_app (users, todo_lists, todo_items) y cubrir metadata/relaciones entre tablas con coverage unitaria y trybuild`.
- Se movió en `docs/tasks.md` la subtarea a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió `examples/todo-app/src/domain.rs` con el dominio base del ejemplo: `User`, `TodoList` y `TodoItem`, modelados con `#[derive(Entity)]`, índices, `rowversion`, defaults SQL y relaciones uno-a-muchos entre usuarios, listas e ítems.
- `examples/todo-app/src/lib.rs` ahora expone ese dominio desde la crate del ejemplo, reexportando `User` como `TodoUser` para mantener una surface semántica clara hacia consumidores del ejemplo.
- La cobertura unitaria del ejemplo ahora verifica schema, tabla, primary key, defaults, `rowversion`, índices y foreign keys del dominio, incluyendo `CASCADE` para `owner_user_id`/`list_id` y `SET NULL` para `completed_by_user_id`.
- Se añadió el fixture `crates/mssql-orm/tests/ui/entity_todo_app_domain_valid.rs` y su registro en `crates/mssql-orm/tests/trybuild.rs` para fijar en compile-time la forma válida del dominio y de sus relaciones estructuradas.
- Durante la validación apareció una regla operativa relevante del derive: en `foreign_key(entity = ..., column = ...)`, el nombre generado del constraint usa el nombre de tabla derivado del tipo Rust referenciado. Se ajustó el dominio para respetar esa convención y se dejó cubierta por tests.
- `examples/todo-app/README.md` quedó actualizado para reflejar que el ejemplo ya tiene dominio base definido y que la siguiente subtarea pasa a ser la cobertura de consultas públicas.

### Resultado

- La Etapa 14 ya tiene el dominio base real del ejemplo `todo_app`, con metadata relacional observable y validada tanto por pruebas unitarias del ejemplo como por `trybuild` en la crate pública.

### Validación

- `cargo fmt --all --check`
- `cargo fmt --all --check` en `examples/todo-app/`
- `cargo check --workspace`
- `cargo check --manifest-path examples/todo-app/Cargo.toml`
- `cargo test --manifest-path examples/todo-app/Cargo.toml`
- `cargo test -p mssql-orm --test trybuild`

### Bloqueos

- No hubo bloqueos funcionales persistentes.
- Durante la validación hubo esperas breves por file locks de `cargo` al ejecutar varias verificaciones en paralelo.

### Próximo paso recomendado

- Implementar `Etapa 14: Cubrir consultas públicas usadas por todo_app para filter, order_by, joins, limit, take, paginate y count`.

### Sesión: base del ejemplo web async realista `todo_app`

- Se retomó como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md` y se ejecutó la subtarea siguiente de Etapa 14: `Definir el ejemplo web async realista (todo_app) y cubrir su configuración/arranque con pruebas unitarias sin depender todavía de servidor HTTP real`.
- Se movió en `docs/tasks.md` la subtarea a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se creó la crate aislada `examples/todo-app/`, fuera del workspace principal, como base del ejemplo web async realista de Etapa 14.
- `examples/todo-app/src/lib.rs` ahora fija `TodoAppSettings`, `default_operational_options()`, `TodoAppState<Db>` y `build_app(...)`, dejando establecida la shape de configuración, state y arranque sin introducir aún dominio, endpoints ni wiring real de base de datos.
- La configuración operativa del ejemplo ya queda explícita y alineada con la surface productiva existente: `connect_timeout`, `query_timeout`, `acquire_timeout`, retry acotado, tracing, slow query, health check y pool `bb8`.
- `examples/todo-app/src/main.rs` ahora levanta un `Router` vacío sobre `axum`, inicializa tracing desde `RUST_LOG` y hace bind del listener con `APP_ADDR`, preservando que esta subtarea solo cubra arranque/configuración y no endpoints funcionales todavía.
- `examples/todo-app/README.md` documenta el alcance actual del ejemplo, las variables de entorno y el orden de las siguientes extensiones de Etapa 14.
- Se añadieron seis pruebas unitarias en la propia crate del ejemplo para cubrir: obligatoriedad de `DATABASE_URL`, defaults de `APP_ADDR`/`RUST_LOG`, overrides explícitos, perfil fijo de `MssqlOperationalOptions`, propagación de opciones a `MssqlConnectionConfig` y construcción de `AppState`/`Router` sin servidor HTTP real.

### Resultado

- La Etapa 14 ya tiene una base real y compilable del ejemplo web async `todo_app`, con configuración y arranque definidos y cubiertos unitariamente, lista para extenderse con dominio, queries, health check, endpoints y wiring con pool.

### Validación

- `cargo fmt --all --check`
- `cargo fmt --all --check` en `examples/todo-app/`
- `cargo check --workspace`
- `cargo check --manifest-path examples/todo-app/Cargo.toml`
- `cargo test --manifest-path examples/todo-app/Cargo.toml`

### Bloqueos

- No hubo bloqueos funcionales persistentes.
- Durante la validación hubo esperas breves por file locks de `cargo` mientras resolvía/compilaba dependencias del ejemplo aislado.

### Próximo paso recomendado

- Implementar `Etapa 14: Definir el dominio todo_app (users, todo_lists, todo_items) y cubrir metadata/relaciones entre tablas con coverage unitaria y trybuild`.

### Sesión: reencauzar `todo_app` al objetivo original de Etapa 14

- El usuario aclaró que la tarea original seguía siendo `Etapa 14: Crear ejemplo de integración con framework web async usando pool, health check y configuración operativa real` y que `todo_app` era una sugerencia para hacer ese ejemplo más realista, no para cambiar de objetivo.
- Se corrigió `docs/tasks.md` para que `todo_app` vuelva a ser la forma concreta del ejemplo web async de Etapa 14, en lugar de quedar desplazado como iniciativa separada o posterior.
- El backlog de Etapa 14 quedó reordenado alrededor de ese ejemplo realista: shape/configuración, dominio y relaciones, consultas usadas por el ejemplo, endpoint de health check, endpoints mínimos, wiring con pool y validación real contra SQL Server.

### Resultado

- La Etapa 14 recuperó su sentido original: construir un ejemplo de integración web async usando pool, health check y configuración operativa real, ahora con `todo_app` como materialización más realista de ese objetivo.

### Validación

- Revisión manual de consistencia de `docs/tasks.md` con la instrucción original del usuario y el estado operativo actual.

### Bloqueos

- No hubo bloqueos técnicos.
- Esta sesión corrige el backlog; no introduce todavía código nuevo del ejemplo.

### Próximo paso recomendado

- Empezar por `Etapa 14: Definir el ejemplo web async realista (todo_app) y cubrir su configuración/arranque con pruebas unitarias sin depender todavía de servidor HTTP real`.

### Sesión: aclarar que `todo_app` es ejemplo, no suite de tests

- El usuario aclaró que `todo_app` debe usarse como ejemplo futuro y no como nombre/vehículo de la suite de pruebas.
- Se descartaron los cambios no confirmados que intentaban modelar `todo_app` como tests (`stage14_todo_app_relationship_metadata.rs`, fixture `trybuild` asociado y su registro en `tests/trybuild.rs`).
- `docs/tasks.md` quedó corregido para separar responsabilidades: las pruebas de relaciones entre tablas y de consultas públicas permanecen como tareas técnicas independientes, mientras `todo_app` pasa a ser un entregable posterior de ejemplo end-to-end.

### Resultado

- El backlog ya no mezcla el ejemplo `todo_app` con la estrategia de testing; ahora el ejemplo queda como referencia funcional futura y la cobertura de relaciones/queries se planifica por separado.

### Validación

- Revisión de consistencia del árbol de cambios y de `docs/tasks.md` tras descartar los tests no confirmados.

### Bloqueos

- No hubo bloqueos técnicos.
- Esta sesión corrige dirección de trabajo; no introduce todavía nueva cobertura ni el ejemplo.

### Próximo paso recomendado

- Retomar por `Etapa 14: Agregar pruebas dedicadas de relaciones entre tablas con coverage unitaria y trybuild, independientes del ejemplo final`.

### Sesión: ampliar backlog hacia `todo_app` con relaciones y queries

- A petición del usuario se amplió el backlog pendiente del ejemplo web async para que no sea solo wiring HTTP/pool, sino también una referencia útil de dominio y cobertura.
- `docs/tasks.md` ahora registra explícitamente un futuro ejemplo `todo_app` y separa dos entregables previos al código del ejemplo: modelado de relaciones entre tablas (`users`, `todo_lists`, `todo_items`) y cobertura de consultas públicas para `filter`, `order_by`, joins, `limit`, `take`, `paginate` y `count`.
- Esta ampliación deja claro que el ejemplo futuro debe servir también como vehículo de pruebas para relaciones, queries y paginación, no solo como demo superficial de `axum` + pool.

### Resultado

- El backlog de Etapa 14 ya incorpora explícitamente el objetivo de un `todo_app` como ejemplo futuro y fija que antes de implementarlo debe existir cobertura dedicada para relaciones entre tablas y para la API pública de consultas/paginación.

### Validación

- Revisión manual de consistencia de `docs/tasks.md` y alineación con el foco actual de Etapa 14.

### Bloqueos

- No hubo bloqueos técnicos.
- Esta sesión solo reordena backlog; no introduce todavía pruebas ni código del ejemplo.

### Próximo paso recomendado

- Completar la tarea operativa actual moviendo el backlog nuevo a estado cerrado y luego arrancar por `Etapa 14: Definir el dominio del ejemplo todo_app (users, todo_lists, todo_items) y cubrir metadata/relaciones entre tablas con pruebas unitarias y trybuild`.

### Sesión: revertir ejemplo web async monolítico y descomponerlo

- A petición explícita del usuario se revirtió la última tarea `Etapa 14: Crear ejemplo de integración con framework web async usando pool, health check y configuración operativa real`, porque el resultado no dejó el nivel de granularidad ni de testeo deseado.
- La reversión se hizo con `git revert 379b5e7` para preservar trazabilidad y evitar reescritura destructiva del historial; el repositorio volvió así al estado funcional posterior a `ca8c94f`.
- Tras el revert, `docs/tasks.md` dejó de tratar el ejemplo web async como una sola pieza monolítica y ahora lo divide en cinco subtareas verificables: shape/configuración, endpoint de health check, endpoints CRUD mínimos, wiring con pool y validación real contra SQL Server.
- La descomposición nueva prioriza cobertura incremental: primero pruebas unitarias puras, luego pruebas de handlers/servicio local, después wiring feature-gated y por último validación real contra SQL Server.

### Resultado

- El ejemplo web async fue retirado del árbol y la línea de trabajo quedó reabierta como backlog granular y más testeable, lista para retomarse por entregables pequeños.

### Validación

- `git revert 379b5e7`
- `cargo check --workspace`

### Bloqueos

- No hubo bloqueos técnicos durante la reversión.
- La integración web async sigue pendiente; lo que cambió es su estrategia de ejecución, ahora orientada a subtareas pequeñas con más cobertura.

### Próximo paso recomendado

- Ejecutar la primera subtarea nueva: `Etapa 14: Definir shape mínima del ejemplo web async y cubrirla con pruebas unitarias de configuración/arranque sin depender todavía de servidor HTTP real`.

### Sesión: wiring público de `DbContext` sobre pool

- Se retomó como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md` y se ejecutó la subtarea siguiente de Etapa 14: `Exponer wiring público DbContext desde pool sin romper connect, from_connection ni SharedConnection`.
- Se movió en `docs/tasks.md` la subtarea a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm/src/context.rs` dejó de modelar `SharedConnection` como alias a `Arc<Mutex<MssqlConnection>>` y ahora lo representa como wrapper clonable capaz de envolver conexión directa o pool, manteniendo el mismo nombre público.
- La nueva implementación introduce un guard uniforme `SharedConnectionGuard<'_>` que expone `Deref/DerefMut` hacia `MssqlConnection`, permitiendo que `DbSet`, `DbSetQuery`, `health_check()` y `transaction(...)` sigan reutilizando la misma semántica operativa aunque el origen real sea una conexión directa o una adquisición desde pool.
- Bajo `pool-bb8`, `SharedConnection::from_pool(...)` y `connect_shared_from_pool(...)` ya permiten integrar un `MssqlPool` con la surface existente sin romper `connect_shared(...)`, `connect_shared_with_options(...)`, `connect_shared_with_config(...)` ni la ruta directa `from_connection(...)`.
- `crates/mssql-orm-macros/src/lib.rs` ahora genera `AppDbContext::from_pool(pool)` detrás de `pool-bb8`, preservando simultáneamente `from_shared_connection(...)`, `from_connection(...)`, `connect(...)`, `connect_with_options(...)` y `connect_with_config(...)`.
- La ruta directa se conserva intacta: `connect*` y `from_connection(...)` siguen construyendo un `SharedConnection` sobre una conexión única, mientras que la ruta de pool crea un `SharedConnection` respaldado por `MssqlPool`.
- `crates/mssql-orm/src/lib.rs` ahora reexporta también `connect_shared_from_pool` bajo `pool-bb8`, y la cobertura pública feature-gated ya verifica que existan tanto la surface del pool como el wiring `DerivedDbContext::from_pool`.

### Resultado

- La Etapa 14 ya expone wiring público de `DbContext` desde pool sin romper las rutas previas de conexión directa ni el contrato público `SharedConnection`; el mismo contexto derivado puede nacer ahora desde conexión única o desde pool según el feature y el constructor usados.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm --lib`
- `cargo check -p mssql-orm --features pool-bb8`
- `cargo test -p mssql-orm --features pool-bb8 --lib`

### Bloqueos

- No hubo bloqueos funcionales persistentes.
- Esta sesión no creó todavía el ejemplo web async real; esa capacidad permanece como siguiente subtarea de Etapa 14.

### Próximo paso recomendado

- Implementar `Etapa 14: Crear ejemplo de integración con framework web async usando pool, health check y configuración operativa real`.

### Sesión: pooling opcional con feature gate `pool-bb8`

- Se retomó como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md` y se ejecutó la subtarea siguiente de Etapa 14: `Implementar pooling opcional de conexiones con feature gate y límites explícitos de ownership`.
- Se movió en `docs/tasks.md` la subtarea a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm-tiberius/Cargo.toml` ahora declara el feature gate `pool-bb8` y añade `bb8` como dependencia opcional; `crates/mssql-orm/Cargo.toml` reexpone ese mismo feature hacia la crate pública.
- Se añadió `crates/mssql-orm-tiberius/src/pool.rs`, implementando `MssqlPool`, `MssqlPoolBuilder`, `MssqlPooledConnection` y `MssqlConnectionManager` sobre `bb8`.
- La API del pool quedó deliberadamente acotada para preservar ownership explícito en esta etapa: `MssqlPool::builder()` configura el pool y `MssqlPool::acquire()` entrega un `MssqlPooledConnection<'_>` que expone `Deref/DerefMut` hacia `MssqlConnection`, sin introducir todavía `DbContext::from_pool(...)` ni un nuevo tipo equivalente a `SharedConnection`.
- `MssqlConnectionManager` reutiliza `MssqlConnection::connect_with_config(...)` para crear conexiones y `connection.health_check().await` como validación de checkout, manteniendo la lógica de ejecución/health en el adaptador Tiberius en lugar de duplicarla en el manager del pool.
- La configuración efectiva del pool se sigue modelando con `MssqlPoolOptions`; el builder soporta `max_size`, `min_idle`, `acquire_timeout`, `idle_timeout`, `max_lifetime` y `with_pool_options(...)`, y al construir el pool preserva esas opciones también dentro de `MssqlConnectionConfig`.
- `crates/mssql-orm-tiberius/src/lib.rs` y `crates/mssql-orm/src/lib.rs` ahora reexportan la surface del pool de forma condicional bajo `pool-bb8`, sin alterar la surface por defecto cuando el feature no está activo.
- Se añadió cobertura unitaria feature-gated en ambas crates para la shape del builder, el reemplazo de opciones y la reexportación pública del pool.

### Resultado

- La Etapa 14 ya dispone de pooling opcional detrás de `pool-bb8`, con límites de ownership explícitos y sin romper el flujo actual basado en conexión directa o `SharedConnection`. El wiring de `DbContext` sobre pool sigue pendiente como subtarea separada.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo check -p mssql-orm-tiberius --features pool-bb8`
- `cargo check -p mssql-orm --features pool-bb8`
- `cargo test -p mssql-orm-tiberius --features pool-bb8 --lib`
- `cargo test -p mssql-orm --features pool-bb8 --lib`

### Bloqueos

- No hubo bloqueos funcionales persistentes.
- Esta sesión implementa solo el pool y su ownership explícito; todavía no existe `DbContext::from_pool(...)` ni integración de contexto sobre recursos del pool, porque esa capacidad permanece como subtarea separada en el backlog.

### Próximo paso recomendado

- Implementar `Etapa 14: Exponer wiring público DbContext desde pool sin romper connect, from_connection ni SharedConnection`.

### Sesión: retry policy mínima para operaciones idempotentes

- Se retomó como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md` y se ejecutó la subtarea siguiente de Etapa 14: `Implementar retry policy opcional y acotada para fallos transitorios en operaciones idempotentes`.
- Se movió en `docs/tasks.md` la subtarea a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm-tiberius/src/error.rs` ahora expone clasificación interna de errores transitorios reutilizable por el adaptador, limitada a deadlocks, timeouts/abortos/reset de I/O y a un conjunto acotado de códigos SQL Server/Azure SQL típicamente transitorios (`1222`, `40197`, `40501`, `40613`, `49918`, `49919`, `49920`).
- `crates/mssql-orm-tiberius/src/parameter.rs` ahora separa la ejecución nativa del driver (`execute_driver`, `query_driver`) del mapeo a `OrmError`, permitiendo decidir retries cuando todavía se conserva el `tiberius::error::Error` real.
- `crates/mssql-orm-tiberius/src/executor.rs` ahora aplica retry opcional solo a lecturas materializadas clasificadas como `select`, es decir, a las rutas internas que soportan `fetch_one` y `fetch_all`; `execute`, `query_raw` y las rutas transaccionales siguen sin retry automático por seguridad.
- La política queda deliberadamente acotada: solo reintenta si `MssqlRetryOptions.enabled` está activo, `max_retries > 0`, el SQL compilado se clasifica como `select` y el error detectado es transitorio.
- El delay entre intentos usa backoff exponencial simple basado en `base_delay`, con tope en `max_delay`, y emite un `warn` estructurado `orm.query.retry` con `server_addr`, `operation`, `attempt`, `max_retries`, `delay_ms` y `error_code`.
- `MssqlConnection::fetch_one`, `fetch_all` y, por transitividad, `health_check()` ya usan esta política cuando la configuración la habilita; `MssqlTransaction` fuerza `MssqlRetryOptions::disabled()` para no reintentar automáticamente dentro de una transacción activa.
- Se añadió cobertura unitaria para clasificación de errores transitorios, selección de queries retryables y cálculo/cap del backoff.

### Resultado

- La Etapa 14 ya soporta retry opcional y acotado para fallos transitorios en operaciones idempotentes de lectura, sin reintentar escrituras ni queries con stream abierto y sin mover ejecución fuera de `mssql-orm-tiberius`.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm-tiberius --lib`
- `cargo test -p mssql-orm --lib`

### Bloqueos

- No hubo bloqueos funcionales.
- No se ejecutó validación real dedicada contra SQL Server para esta subtarea porque la política se acopló solo a rutas de lectura ya cubiertas por tests unitarios y no se introdujo nueva surface de integración fuera del adaptador.

### Próximo paso recomendado

- Implementar `Etapa 14: Implementar pooling opcional de conexiones con feature gate y límites explícitos de ownership`.

### Sesión: health checks mínimos para SQL Server/Tiberius

- Se retomó como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md` y se ejecutó la siguiente subtarea prioritaria de Etapa 14: `Exponer health checks mínimos de conectividad y ejecución simple para SQL Server/Tiberius`.
- Se movió en `docs/tasks.md` la subtarea a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm-tiberius/src/config.rs` ahora fija el SQL estable del health check mínimo mediante `MssqlHealthCheckQuery::sql()`, quedando `SelectOne => SELECT 1 AS [health_check]`.
- `crates/mssql-orm-tiberius/src/connection.rs` ahora expone `MssqlConnection::health_check()`, que ejecuta el health check mínimo sobre la conexión real reutilizando el pipeline existente de ejecución, tracing y slow query logging.
- El health check usa `MssqlHealthCheckOptions::timeout` cuando existe y, si no, cae en `query_timeout`; si la ejecución vence el plazo retorna `SQL Server health check timed out`.
- La misma capa valida explícitamente que el query devuelva una fila con el valor esperado, de modo que el health check cubra tanto conectividad como ejecución simple y no solo apertura de socket.
- `crates/mssql-orm/src/context.rs` ahora añade `DbContext::health_check(&self)` como helper público por defecto sobre `SharedConnection`, y `crates/mssql-orm-macros/src/lib.rs` genera el wrapper inherente `AppDbContext::health_check(&self)` sin romper `connect`, `connect_with_config`, `transaction` ni `save_changes`.
- Se añadió cobertura unitaria para SQL estable y resolución efectiva de timeout, una aserción pública en `crates/mssql-orm/src/lib.rs` para el nuevo método del contexto y un integration test mínimo en `crates/mssql-orm-tiberius/tests/sqlserver_integration.rs`.

### Resultado

- La Etapa 14 ya dispone de health checks mínimos en la capa adaptadora y en la crate pública: `MssqlConnection::health_check()` y `DbContext::health_check()`/`AppDbContext::health_check()` ejecutan `SELECT 1` con timeout operativo coherente y reutilizan el runtime ya instrumentado.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm-tiberius --lib`
- `cargo test -p mssql-orm --lib`
- `cargo test -p mssql-orm --test trybuild`
- `cargo test -p mssql-orm-tiberius --test sqlserver_integration health_check -- --nocapture`

### Bloqueos

- No hubo bloqueos funcionales.
- La validación real del integration test de health check hizo skip limpio en este entorno porque `MSSQL_ORM_TEST_CONNECTION_STRING` no estaba definido; el wiring quedó validado hasta ese punto y la prueba queda lista para CI o entorno local con SQL Server disponible.

### Próximo paso recomendado

- Implementar `Etapa 14: Implementar retry policy opcional y acotada para fallos transitorios en operaciones idempotentes`.

### Sesión: corrección de fixtures `trybuild` para Active Record en CI

- Se revisó el fallo reportado por GitHub Actions en `cargo test -p mssql-orm --test active_record_trybuild` y se confirmó que no provenía de la lógica productiva de Active Record, sino de drift en fixtures UI frente al diagnóstico actual del compilador y del derive `DbContext`.
- Se creó y movió en `docs/tasks.md` la subtarea operativa `Sincronizar fixture trybuild de Active Record con el diagnóstico actual de DbContext` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm/tests/ui/active_record_delete_public_valid.rs` ahora implementa `FromRow` para `User`, alineando el fixture con el contrato actual de `#[derive(DbContext)]`, que exige entidades materializables para campos `DbSet<T>`.
- `crates/mssql-orm/tests/ui/active_record_missing_entity_set.rs` ahora implementa `FromRow` para `Order`, de modo que el caso compile-fail vuelva a aislar el error objetivo (`DbContextEntitySet<User>` ausente) y no falle por una precondición secundaria del contexto.
- `crates/mssql-orm/tests/ui/active_record_missing_entity_set.stderr` se sincronizó con la salida real actual del compilador, incluyendo los nuevos números de línea del fixture y el shape efectivo del diagnóstico emitido en esta toolchain.

### Resultado

- La batería `active_record_trybuild` vuelve a pasar localmente y el fallo observado en GitHub Actions queda corregido como desalineación de fixtures, no como regresión funcional del crate.

### Validación

- `cargo fmt --all --check`
- `cargo test -p mssql-orm --test active_record_trybuild`

### Bloqueos

- No hubo bloqueos persistentes.

### Próximo paso recomendado

- Reejecutar CI y, si queda limpia, continuar con `Etapa 14: Exponer health checks mínimos de conectividad y ejecución simple para SQL Server/Tiberius`.

### Sesión: slow query logs configurables sobre la instrumentación existente

- Se volvió a tomar como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md`; esta ruta difiere de la entrada original en raíz y queda registrada aquí para trazabilidad.
- Se movió en `docs/tasks.md` la subtarea `Etapa 14: Agregar slow query logs configurables reutilizando la instrumentación de tracing` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm-tiberius/src/telemetry.rs` ahora reutiliza `trace_query(...)` para emitir `orm.query.slow` con `tracing::warn!` cuando la duración observada alcanza `MssqlSlowQueryOptions::threshold`, sin crear un pipeline paralelo de observabilidad.
- El evento de slow query publica campos estables `server_addr`, `operation`, `timeout_ms`, `threshold_ms`, `duration_ms`, `param_count`, `sql`, `params_mode` y `params`, manteniendo por defecto la política de redacción sin exponer valores sensibles.
- La misma capa ahora soporta `slow_query.enabled` incluso cuando `tracing.enabled` está apagado: los eventos `orm.query.start/finish/error` siguen dependiendo de `MssqlTracingOptions`, pero `orm.query.slow` puede activarse independientemente sobre el mismo punto de medición.
- `crates/mssql-orm-tiberius/src/connection.rs`, `src/executor.rs` y `src/transaction.rs` ahora propagan `MssqlSlowQueryOptions` junto con el contexto de tracing existente, cubriendo queries ejecutadas tanto en conexión directa como dentro de transacciones.
- Se añadió cobertura unitaria específica en `telemetry.rs` para fijar el umbral efectivo del slow log y una aserción adicional en `executor.rs` para preservar el shape operativo de `MssqlSlowQueryOptions`.

### Resultado

- La Etapa 14 ya registra slow queries configurables en la capa correcta (`mssql-orm-tiberius`), reutilizando la instrumentación y la medición de duración existentes sin mover ejecución fuera del adaptador ni exponer parámetros por defecto.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm-tiberius --lib`

### Bloqueos

- No hubo bloqueos funcionales del cambio.
- Durante la validación hubo esperas breves por file locks de `cargo` en el entorno local, pero las corridas terminaron correctamente.

### Próximo paso recomendado

- Implementar `Etapa 14: Exponer health checks mínimos de conectividad y ejecución simple para SQL Server/Tiberius`.

### Sesión: instrumentación con `tracing` en conexión, queries y transacciones

- Se volvió a tomar como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md`; para esta subtarea se siguieron explícitamente las secciones `17.3. Logs sin datos sensibles` y `18. Observabilidad`, que exigen `tracing`, eventos `orm.query.start/finish/error`, eventos transaccionales y no exponer parámetros por defecto.
- Se movió en `docs/tasks.md` la subtarea `Etapa 14: Instrumentar conexión, ejecución y transacciones con tracing estructurado y campos estables` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se añadió `crates/mssql-orm-tiberius/src/telemetry.rs` como módulo interno de instrumentación, centralizando clasificación de operaciones SQL, formateo de timeouts y redacción estable de parámetros (`disabled` o `[REDACTED]`) según `MssqlTracingOptions`.
- `crates/mssql-orm-tiberius/src/connection.rs` ahora emite tracing estructurado para la conexión (`orm.connection.start`, `orm.connection.finish`, `orm.connection.error`) usando `server_addr` y `timeout_ms` como campos estables.
- `crates/mssql-orm-tiberius/src/executor.rs` ahora instrumenta ejecución de queries con span `mssql_orm.query` y eventos `orm.query.start`, `orm.query.finish` y `orm.query.error`, incluyendo `server_addr`, `operation`, `timeout_ms`, `param_count`, `sql`, `params_mode` y `params`.
- La redacción de parámetros quedó alineada con el plan: por defecto no se exponen valores; el tracing solo deja `disabled` o `[REDACTED]` según la configuración vigente.
- `crates/mssql-orm-tiberius/src/transaction.rs` ahora instrumenta `BEGIN`, `COMMIT` y `ROLLBACK` con span `mssql_orm.transaction` y eventos `orm.transaction.begin`, `orm.transaction.commit`, `orm.transaction.rollback`; los fallos transaccionales se reportan mediante `orm.transaction.error`.
- Se añadió la dependencia `tracing` a `mssql-orm-tiberius` y cobertura unitaria específica para clasificación de operaciones, estabilidad del formateo de timeouts y redacción de parámetros.

### Resultado

- La Etapa 14 ya dispone de instrumentación estructurada con `tracing` en conexión, queries y transacciones dentro del adaptador Tiberius, manteniendo SQL y ejecución en sus crates correctas y sin exponer parámetros sensibles por defecto.

### Validación

- `cargo fmt --all`
- `cargo check --workspace`
- `cargo test -p mssql-orm-tiberius --lib`
- `cargo test -p mssql-orm --lib`
- `cargo test -p mssql-orm --test trybuild`

### Bloqueos

- No hubo bloqueos persistentes.
- Esta sesión solo instrumenta tracing base; todavía no existe diferenciación específica para slow queries, métricas agregadas ni health checks. Esas capacidades siguen pendientes como subtareas separadas.

### Próximo paso recomendado

- Implementar `Etapa 14: Agregar slow query logs configurables reutilizando la instrumentación de tracing`.

### Sesión: timeouts configurables de conexión y ejecución

- Se volvió a tomar como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md` y se ejecutó la subtarea siguiente de Etapa 14 usando la surface de configuración definida en la sesión previa.
- Se movió en `docs/tasks.md` la subtarea `Etapa 14: Implementar timeouts configurables de conexión y ejecución sin mover SQL fuera de sqlserver ni ejecución fuera de tiberius` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm-tiberius/src/connection.rs` ahora aplica `connect_timeout` desde `MssqlOperationalOptions` al bootstrap completo del cliente Tiberius mediante un helper interno `run_with_timeout(...)`, retornando error explícito `SQL Server connection timed out` cuando vence el plazo.
- La misma capa ahora expone `query_timeout()` en `MssqlConnection` y propaga ese valor a la apertura y cierre de transacciones.
- `crates/mssql-orm-tiberius/src/executor.rs` ahora aplica `query_timeout` a `execute`, `query_raw`, `fetch_one` y `fetch_all`, de modo que el límite de ejecución viva exclusivamente en la crate adaptadora y no contamine el compilador SQL ni la API pública con un pipeline paralelo.
- `crates/mssql-orm-tiberius/src/transaction.rs` ahora preserva `query_timeout` dentro de `MssqlTransaction` y lo reaplica a ejecución de queries y a comandos `BEGIN`, `COMMIT` y `ROLLBACK`.
- Se añadieron pruebas unitarias del helper de timeout y se activó `tokio::time` en la crate adaptadora para soportar la implementación real.

### Resultado

- La Etapa 14 ya soporta timeouts configurables de conexión y ejecución en la capa correcta (`mssql-orm-tiberius`), reutilizando la surface pública ya definida y sin mover SQL fuera de `mssql-orm-sqlserver` ni ejecución fuera del adaptador Tiberius.

### Validación

- `cargo fmt --all`
- `cargo check --workspace`
- `cargo test -p mssql-orm-tiberius --lib`
- `cargo test -p mssql-orm --lib`
- `cargo test -p mssql-orm --test trybuild`

### Bloqueos

- No hubo bloqueos persistentes.
- La sesión cubre timeouts de conexión, ejecución y comandos transaccionales, pero todavía no instrumenta eventos, slow query logs ni health checks; esas capacidades siguen como subtareas separadas de Etapa 14.

### Próximo paso recomendado

- Implementar `Etapa 14: Instrumentar conexión, ejecución y transacciones con tracing estructurado y campos estables`.

### Sesión: surface y configuración operativa de producción para `mssql-orm-tiberius`

- Se volvió a tomar como fuente de verdad el plan maestro en su ruta real `docs/plan_orm_sqlserver_tiberius_code_first.md`; la Etapa 14 del plan exige `pool opcional`, `timeouts`, `retry policy`, `tracing`, `slow query logs` y `health checks`, así que esta sesión se limitó a definir el contrato y el wiring público sin activar todavía comportamiento nuevo.
- Se movió en `docs/tasks.md` la subtarea `Etapa 14: Definir surface y configuración operativa de producción para mssql-orm-tiberius y la crate pública` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- `crates/mssql-orm-tiberius/src/config.rs` ahora define `MssqlOperationalOptions` como contrato raíz y los shapes explícitos `MssqlTimeoutOptions`, `MssqlRetryOptions`, `MssqlTracingOptions`, `MssqlSlowQueryOptions`, `MssqlHealthCheckOptions` y `MssqlPoolOptions`, además de enums auxiliares para `MssqlParameterLogMode`, `MssqlHealthCheckQuery` y `MssqlPoolBackend`.
- `MssqlConnectionConfig` ahora preserva esas opciones operativas y expone `from_connection_string_with_options(...)`, `with_options(...)` y `options()`, dejando resuelto el ownership/config shape para las siguientes subtareas de implementación.
- La crate pública `mssql-orm` ahora reexporta ese surface y expone `connect_shared_with_options(...)` y `connect_shared_with_config(...)` como puntos de entrada explícitos, sin romper `connect_shared(...)`.
- `#[derive(DbContext)]` ahora genera también `connect_with_options(...)` y `connect_with_config(...)`, alineando la API derivada con la nueva configuración pública.
- Se amplió la cobertura con pruebas unitarias del adaptador, pruebas de la crate pública, `trybuild` para `DbContext` y ajustes mecánicos en fixtures de tests que construyen `EntityMetadata` manualmente para mantener coherencia con `renamed_from`.

### Resultado

- La surface de configuración de producción quedó definida y expuesta de forma estable en el adaptador Tiberius y en la crate pública, sin adelantar todavía timeouts, tracing, slow query logs, health checks, retries ni pooling reales.

### Validación

- `cargo fmt --all`
- `cargo fmt --all --check`
- `cargo check --workspace`
- `cargo test -p mssql-orm-tiberius --lib`
- `cargo test -p mssql-orm --lib`
- `cargo test -p mssql-orm --test trybuild`
- `cargo test -p mssql-orm-query --lib`
- `cargo test -p mssql-orm-sqlserver --test compiler_snapshots`

### Bloqueos

- No hubo bloqueos persistentes.
- La sesión definió solo contratos y wiring; ninguna opción nueva cambia todavía el comportamiento runtime del adaptador. Esa implementación queda para las subtareas siguientes de timeouts, `tracing`, slow query logs, health checks, retries y pooling.

### Próximo paso recomendado

- Implementar `Etapa 14: Implementar timeouts configurables de conexión y ejecución sin mover SQL fuera de sqlserver ni ejecución fuera de tiberius`.

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
- `MSSQL_ORM_TEST_CONNECTION_STRING='Server=localhost;Database=tempdb;User Id=<usuario>;Password=<password>;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;' cargo test -p mssql-orm --test stage5_public_crud -- --test-threads=1`

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
- `MSSQL_ORM_TEST_CONNECTION_STRING='Server=localhost;Database=tempdb;User Id=<usuario>;Password=<password>;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;' cargo test -p mssql-orm --test stage5_public_crud -- --test-threads=1`

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
- `MSSQL_ORM_TEST_CONNECTION_STRING='Server=localhost;Database=tempdb;User Id=<usuario>;Password=<password>;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;' cargo test -p mssql-orm --test stage5_public_crud -- --test-threads=1`

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
- `MSSQL_ORM_TEST_CONNECTION_STRING='Server=localhost;Database=tempdb;User Id=<usuario>;Password=<password>;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;' cargo test -p mssql-orm --test stage5_public_crud -- --test-threads=1`

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
- `sqlcmd -S localhost -U '<usuario>' -P '<password>' -d tempdb -C -b -i /tmp/mssql_orm_stage7_retry.sql`
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
- `sqlcmd -S localhost -U '<usuario>' -P '<password>' -d tempdb -C -b -i <script.sql>`
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
- Se validó de forma explícita la prueba real con `MSSQL_ORM_TEST_CONNECTION_STRING='Server=localhost;Database=master;User Id=<usuario>;Password=<password>;' cargo test -p mssql-orm-tiberius --test sqlserver_integration -- --nocapture`.
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

### Sesión: Evaluación de `soft_delete = SoftDelete`

- Se confirmó nuevamente que el plan maestro real del repositorio está en `docs/plan_orm_sqlserver_tiberius_code_first.md`, y se usó esa ruta como fuente de verdad junto con `docs/instructions.md`, `docs/tasks.md`, `docs/worklog.md` y `docs/context.md`.
- Se movió en `docs/tasks.md` la tarea `Etapa 16+: Evaluar soft_delete = SoftDelete como cambio semántico explícito de delete, entity.delete(&db), queries por defecto y migraciones, documentando sus riesgos antes de implementarlo` a `En Progreso` antes de revisar y luego a `Completadas` al cerrar la evaluación documental.
- Se revisaron las rutas reales de borrado y consulta en `crates/mssql-orm/src/context.rs`, `crates/mssql-orm/src/active_record.rs` y `crates/mssql-orm/src/dbset_query.rs` para confirmar el comportamiento actual.
- La revisión confirmó que `DbSet::delete(...)`, `delete_by_sql_value(...)`, `entity.delete(&db)` y la persistencia experimental de entidades `Deleted` siguen convergiendo en `DeleteQuery` + `SqlServerCompiler::compile_delete(...)`, es decir, borrado físico real.
- También se confirmó que `DbSet::query()` y `DbSetQuery` parten de `SelectQuery::from_entity::<E>()` sin filtros implícitos, por lo que hoy no existe ninguna exclusión automática para filas lógicamente borradas.
- Se dejó documentado en `docs/entity-policies.md` que `soft_delete = SoftDelete` sí encaja en el roadmap, pero no como una policy de metadata pura: cambia semántica observable de `delete`, Active Record, change tracking, consultas por defecto y preservación de `rowversion`/`ConcurrencyConflict`.
- La decisión vigente queda explícita: no implementar `soft_delete` en esta sesión y no tratarlo como alias o convención implícita por nombres mágicos; antes debe diseñarse su semántica pública y su integración sobre las rutas existentes.
- Se actualizó `docs/context.md` para reflejar que la evaluación ya quedó cerrada y que el siguiente paso correcto es diseñar la semántica pública de `soft_delete`, no reevaluarla.
- La validación ejecutada para cerrar esta sesión fue `cargo fmt --all --check` y `cargo check --workspace`.
- No se ejecutó `cargo test --workspace` ni `cargo clippy --workspace --all-targets --all-features` porque esta sesión sólo cerró documentación de diseño y trazabilidad; no se modificó código de crates ni superficies ejecutables.

### Resultado

- La evaluación de `soft_delete = SoftDelete` quedó cerrada y trazada: el repositorio ya documenta que es una extensión semántica de alto impacto y que debe diseñarse explícitamente antes de tocar macros, query builder, persistencia o migraciones.

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Implementar `Etapa 16+: Diseñar soft_delete = SoftDelete para que DbSet::delete(...), entity.delete(&db), remove_tracked(...) y save_changes() no emitan DELETE físico cuando la entidad tenga esa política; deben emitir UPDATE sobre columnas como deleted_at/deleted_by y respetar rowversion/ConcurrencyConflict`.

### Sesión: Diseño de `soft_delete = SoftDelete` sobre rutas de borrado existentes

- Se confirmó nuevamente que el plan maestro real del repositorio está en `docs/plan_orm_sqlserver_tiberius_code_first.md`, y se usó esa ruta como fuente de verdad junto con `docs/instructions.md`, `docs/tasks.md`, `docs/worklog.md` y `docs/context.md`.
- Se movió en `docs/tasks.md` la tarea `Etapa 16+: Diseñar soft_delete = SoftDelete para que DbSet::delete(...), entity.delete(&db), remove_tracked(...) y save_changes() no emitan DELETE físico cuando la entidad tenga esa política; deben emitir UPDATE sobre columnas como deleted_at/deleted_by y respetar rowversion/ConcurrencyConflict` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se revisaron las rutas reales de persistencia en `crates/mssql-orm/src/context.rs`, `crates/mssql-orm/src/active_record.rs`, `crates/mssql-orm/src/dbset_query.rs` y `crates/mssql-orm/src/tracking.rs` para diseñar `soft_delete` sobre el comportamiento existente, no sobre supuestos.
- La decisión principal quedó documentada en `docs/entity-policies.md`: `DeleteQuery` y `SqlServerCompiler::compile_delete(...)` conservan semántica de borrado físico, y el cambio a borrado lógico debe resolverse en `DbSet`, Active Record y tracking mediante `UpdateQuery`.
- Para entidades con `soft_delete`, `DbSet::delete(...)`, `delete_by_sql_value(...)`, `delete_tracked_by_sql_value(...)`, `entity.delete(&db)` y `save_tracked_deleted()` deben converger en una única ruta de `UPDATE` lógico con predicate por primary key y `rowversion` cuando exista.
- También quedó fijado que la detección de `OrmError::ConcurrencyConflict` no puede depender de futuras queries filtradas por `soft_delete`; las comprobaciones internas de existencia deben usar una ruta sin filtros implícitos para distinguir correctamente entre “no existe” y “token stale”.
- Se dejó explícito que `remove_tracked(...)` no resuelve por sí mismo el borrado lógico: solo marca `Deleted`, mientras la semántica real permanece centralizada en `save_changes()`.
- Se agregó una subtarea nueva en `docs/tasks.md` para definir cómo `soft_delete` obtiene valores runtime para columnas como `deleted_at`, `deleted_by` o `is_deleted`, porque ese contrato no puede resolverse solo con metadata.
- Se actualizó `docs/context.md` para reflejar el diseño ya decidido y mover el siguiente foco a la semántica de consultas (`with_deleted`/`only_deleted`) y al contrato runtime de valores de borrado lógico.
- La validación ejecutada para cerrar esta sesión fue `cargo fmt --all --check` y `cargo check --workspace`.
- No se ejecutó `cargo test --workspace` ni `cargo clippy --workspace --all-targets --all-features` porque esta sesión solo cerró decisiones de diseño y trazabilidad documental; no hubo cambios en código ejecutable.

### Resultado

- El repositorio ya documenta cómo debe encajar `soft_delete` en las rutas actuales de `DbSet`, Active Record y change tracking sin romper `DeleteQuery`, `rowversion` ni `ConcurrencyConflict`.

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Implementar `Etapa 16+: Definir cómo consultar entidades con soft_delete: por defecto las queries de entidades con la política deben excluir filas borradas lógicamente, y debe existir una API explícita para incluir o consultar solo eliminadas sin afectar entidades que no declaran soft_delete`.

### Sesión: Diseño de consultas para `soft_delete = SoftDelete`

- Se confirmó nuevamente que el plan maestro real del repositorio está en `docs/plan_orm_sqlserver_tiberius_code_first.md`, y se usó esa ruta como fuente de verdad junto con `docs/instructions.md`, `docs/tasks.md`, `docs/worklog.md` y `docs/context.md`.
- Se movió en `docs/tasks.md` la tarea `Etapa 16+: Definir cómo consultar entidades con soft_delete: por defecto las queries de entidades con la política deben excluir filas borradas lógicamente, y debe existir una API explícita para incluir o consultar solo eliminadas sin afectar entidades que no declaran soft_delete` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se revisó la surface real de `DbSet::query()`, `DbSet::query_with(...)`, `DbSet::find(...)` y `DbSetQuery` en `crates/mssql-orm/src/context.rs` y `crates/mssql-orm/src/dbset_query.rs` para fijar el diseño sobre la implementación existente.
- La decisión principal quedó documentada en `docs/entity-policies.md`: para entidades con `soft_delete`, `query()` y `find()` deben excluir por defecto filas borradas lógicamente de la entidad raíz, y `DbSetQuery<E>` debe exponer APIs explícitas `with_deleted()` y `only_deleted()`.
- También quedó fijado que el estado de visibilidad no debe vivir en `SelectQuery`; debe vivir en `DbSetQuery<E>` y materializarse al construir la consulta final, preservando que `mssql-orm-query` siga siendo un AST neutral.
- Se dejó explícito que `query_with(select_query)` tampoco debe saltarse la política: aun con un `SelectQuery` custom, la entidad raíz soft-deleted nace en modo `ActiveOnly` salvo que el usuario llame una API visible para cambiarlo.
- `find(...)` queda alineado con esa semántica pública: una fila soft-deleted se comporta como ausente y retorna `None`; las rutas internas que necesiten existencia física real para resolver `ConcurrencyConflict` o verificaciones post-write deben usar helpers sin filtro implícito.
- Para joins, se fijó el límite conservador de esta fase: el filtro automático de `soft_delete` aplica solo a la entidad raíz `E` de `DbSetQuery<E>`, no a todas las entidades unidas manualmente. Cualquier filtrado adicional sobre joins deberá ser explícito hasta que exista un diseño más fuerte para aliases y múltiples tablas.
- Se actualizó `docs/context.md` para reflejar esta decisión y mover el siguiente foco al contrato runtime que poblará `deleted_at`, `deleted_by` o `is_deleted`.
- La validación ejecutada para cerrar esta sesión fue `cargo fmt --all --check` y `cargo check --workspace`.
- No se ejecutó `cargo test --workspace` ni `cargo clippy --workspace --all-targets --all-features` porque esta sesión solo cerró decisiones de diseño y trazabilidad documental; no hubo cambios en código ejecutable.

### Resultado

- El repositorio ya documenta cómo debe funcionar la visibilidad de lectura para `soft_delete`, incluyendo filtros por defecto, escapes públicos y el límite actual sobre joins explícitos.

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Implementar `Etapa 16+: Definir cómo soft_delete obtiene valores runtime para columnas como deleted_at, deleted_by o is_deleted sin acoplar core a contexto por request ni duplicar la lógica actual de update`.

### Sesión: Diseño del contrato runtime para valores de `soft_delete`

- Se confirmó nuevamente que el plan maestro real del repositorio está en `docs/plan_orm_sqlserver_tiberius_code_first.md`, y se usó esa ruta como fuente de verdad junto con `docs/instructions.md`, `docs/tasks.md`, `docs/worklog.md` y `docs/context.md`.
- Se movió en `docs/tasks.md` la tarea `Etapa 16+: Definir cómo soft_delete obtiene valores runtime para columnas como deleted_at, deleted_by o is_deleted sin acoplar core a contexto por request ni duplicar la lógica actual de update` a `En Progreso` antes de editar y luego a `Completadas` tras validarla.
- Se revisaron las decisiones ya fijadas para `AuditProvider` y las rutas reales de persistencia en `crates/mssql-orm/src/context.rs` y `crates/mssql-orm/src/active_record.rs` para no introducir un segundo mecanismo incompatible de mutación de `Vec<ColumnValue>`.
- La decisión principal quedó documentada en `docs/entity-policies.md`: `soft_delete` necesita un contrato runtime separado de su metadata, responsable de mutar un `Vec<ColumnValue>` ya normalizado dentro de `mssql-orm`.
- También quedó explícito que ese provider/runtime contract no debe vivir en `mssql-orm-core`, no debe inferir columnas por nombres mágicos como `deleted_at` o `is_deleted`, y no debe generar SQL directo ni tocar el AST de `mssql-orm-query`.
- Se fijó además que `DbSet::delete(...)`, `ActiveRecord::delete(&db)` y `save_tracked_deleted()` deben converger en un único punto interno de mutación de cambios antes de compilar el `UpdateQuery`, exactamente para evitar duplicación de lógica runtime.
- La documentación deja alineado el principio de precedencia con `AuditProvider`: valores explícitos no se sobrescriben silenciosamente, duplicados fallan y la ausencia de un valor requerido debe detectarse antes de compilar SQL.
- Se actualizó `docs/context.md` para reflejar que `soft_delete` ya tiene definido su contrato runtime a nivel conceptual y que el siguiente paso razonable pasa a ser cobertura e implementación con pruebas.
- La validación ejecutada para cerrar esta sesión fue `cargo fmt --all --check` y `cargo check --workspace`.
- No se ejecutó `cargo test --workspace` ni `cargo clippy --workspace --all-targets --all-features` porque esta sesión solo cerró decisiones de diseño y trazabilidad documental; no hubo cambios en código ejecutable.

### Resultado

- El repositorio ya documenta cómo debe resolverse en runtime el llenado de `deleted_at`, `deleted_by` o `is_deleted` para `soft_delete`, sin romper la separación entre metadata, AST, compilación SQL y ejecución.

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Implementar `Etapa 16+: Cubrir soft_delete con pruebas de SQL compilado, CRUD público, Active Record, change tracking y migraciones para evitar que alguna ruta siga haciendo borrado físico por accidente`.

### Sesión: descomposición operativa de `soft_delete`

- Se detectó una inconsistencia en `docs/tasks.md`: el diseño conceptual de `soft_delete` ya estaba cerrado, pero el siguiente pendiente visible era una tarea amplia de cobertura, sin subtareas explícitas de implementación para macros, runtime, `DbSet`, `DbSetQuery` y migraciones.
- Para no forzar una sesión futura a saltar directamente a pruebas sin código intermedio, se reemplazó esa tarea amplia por subtareas ejecutables y ordenadas: soporte en `#[derive(Entity)]`, contrato runtime de `Vec<ColumnValue>`, rutas de borrado en `DbSet`/Active Record/tracking, visibilidad de lectura en `DbSetQuery`, helpers internos sin filtro implícito, integración con snapshots/diff/DDL y cobertura final.
- La tarea operativa `Descomponer la implementación de soft_delete en subtareas ejecutables antes de intentar cobertura integral` quedó registrada como completada en `docs/tasks.md`.
- Se actualizó `docs/context.md` para reflejar que el siguiente paso correcto ya es implementación concreta de `soft_delete` desde macros y runtime, no más diseño general.
- La validación ejecutada para cerrar esta sesión fue `cargo fmt --all --check` y `cargo check --workspace`.
- No se ejecutó `cargo test --workspace` ni `cargo clippy --workspace --all-targets --all-features` porque esta sesión solo reordenó backlog y trazabilidad documental; no hubo cambios en código ejecutable.

### Resultado

- El backlog de `soft_delete` quedó alineado con el estado real del proyecto y ahora permite avanzar por implementación incremental en vez de saltar desde diseño a una tarea de pruebas demasiado amplia.

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Implementar `Etapa 16+: Extender #[derive(Entity)] para aceptar #[orm(soft_delete = SoftDelete)], generar metadata/runtime contract auxiliar de la policy y rechazar configuraciones inválidas en compile-time`.

### Sesión: soporte base de macros para `soft_delete`

- Se ejecutó la subtarea `Etapa 16+: Extender #[derive(Entity)] para aceptar #[orm(soft_delete = SoftDelete)], generar metadata/runtime contract auxiliar de la policy y rechazar configuraciones inválidas en compile-time`.
- Se extendió `parse_entity_config(...)` en `crates/mssql-orm-macros/src/lib.rs` para aceptar `soft_delete = Tipo`, rechazar una segunda declaración `soft_delete` y seguir preservando errores explícitos en compile-time.
- `#[derive(Entity)]` ahora expande columnas de `soft_delete` dentro de `EntityMetadata.columns` igual que otras columnas ordinarias, preservando el pipeline único de metadata/snapshot/diff/DDL.
- El macro ahora valida colisiones entre columnas propias de la entidad y columnas de `soft_delete`, y también entre columnas de `audit` y `soft_delete`, fallando en compile-time con mensajes accionables.
- La crate pública `mssql-orm` ahora expone el trait `SoftDeleteEntity`, y el derive `Entity` lo implementa para toda entidad derivada: devuelve `Some(EntityPolicyMetadata)` cuando la entidad declara `soft_delete` y `None` cuando no.
- Se añadieron pruebas unitarias y `trybuild` nuevas en `crates/mssql-orm/tests/` para cubrir el camino válido, tipo inexistente, duplicación de policy y colisión de columna.
- Validaciones ejecutadas: `cargo fmt --all`, `cargo fmt --all --check`, `cargo check --workspace`, `cargo test -p mssql-orm soft_delete_policy_columns_are_expanded_into_entity_metadata -- --nocapture` y `cargo test -p mssql-orm --test trybuild entity_derive_ui -- --nocapture`.
- No se ejecutó `cargo test --workspace` completo ni `cargo clippy --workspace --all-targets --all-features` porque esta sesión quedó acotada al soporte de macro/metadata y la cobertura dirigida ya valida la superficie tocada.

### Resultado

- `soft_delete` ya tiene soporte parcial real en la etapa de macros: la sintaxis compila, las columnas entran a metadata y el runtime futuro ya dispone de un contrato auxiliar para detectar la policy de la entidad.

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Implementar `Etapa 16+: Implementar el contrato runtime de soft_delete en mssql-orm para producir y validar Vec<ColumnValue> de borrado lógico sin duplicar la lógica de update`.

### Sesión: contrato runtime de `soft_delete` en `mssql-orm`

- Se ejecutó la subtarea `Etapa 16+: Implementar el contrato runtime de soft_delete en mssql-orm para producir y validar Vec<ColumnValue> de borrado lógico sin duplicar la lógica de update`.
- Se agregó `crates/mssql-orm/src/soft_delete_runtime.rs` con la surface pública `SoftDeleteProvider`, `SoftDeleteContext`, `SoftDeleteOperation` y `SoftDeleteRequestValues`, además del helper interno `apply_soft_delete_values::<E>(...)`.
- El helper runtime valida duplicados de columna, respeta la ausencia de policy en entidades sin `soft_delete`, aplica el provider opcional y luego vuelve a validar el shape final antes de que una futura ruta de `delete` compile un `UpdateQuery`.
- También quedaron cubiertas las reglas mínimas del contrato: una columna de `soft_delete` no puede mutarse si no es `updatable`, no puede recibir `NULL` si no es nullable y debe existir valor runtime para toda columna de la policy que sea `updatable`, no nullable y sin `default_sql`.
- `mssql-orm::prelude::*` ahora reexporta también la surface runtime de `soft_delete`, para mantener la API pública consolidada en la crate raíz.
- Se añadieron pruebas unitarias en el nuevo módulo para cubrir pass-through en entidades sin policy, aplicación del provider, detección de duplicados y ausencia de columnas requeridas.
- Durante la primera corrida, el test dirigido falló porque las pruebas usaban `chrono` sin dependencia directa en la crate pública; se corrigió reemplazando esos valores por `SqlValue::String(...)`, suficiente para validar el contrato actual sin introducir dependencias nuevas.
- Validaciones ejecutadas: `cargo fmt --all`, `cargo fmt --all --check`, `cargo check --workspace` y `cargo test -p mssql-orm soft_delete_runtime -- --nocapture`.
- No se ejecutó `cargo test --workspace` completo ni `cargo clippy --workspace --all-targets --all-features` porque esta sesión quedó acotada a la surface runtime nueva de `mssql-orm`; la cobertura dirigida ya ejercita la superficie tocada.
- Se actualizó `docs/tasks.md` para mover la subtarea a `Completadas` y `docs/context.md` para reflejar que el contrato runtime ya existe, aunque todavía no está cableado a `DbSet`, Active Record ni change tracking.

### Resultado

- `soft_delete` ya tiene contrato runtime real dentro de `mssql-orm`, listo para que la siguiente subtarea conecte las rutas públicas de borrado con `UpdateQuery` sin duplicar la validación de valores lógicos.

### Bloqueos

- No hubo bloqueos técnicos después de corregir la dependencia accidental en pruebas.

### Próximo paso recomendado

- Implementar `Etapa 16+: Hacer que DbSet::delete(...), delete_by_sql_value(...), delete_tracked_by_sql_value(...), entity.delete(&db) y save_tracked_deleted() usen UpdateQuery cuando la entidad tenga soft_delete`.

### Sesión: rutas de borrado con `soft_delete`

- Se ejecutó la subtarea `Etapa 16+: Hacer que DbSet::delete(...), delete_by_sql_value(...), delete_tracked_by_sql_value(...), entity.delete(&db) y save_tracked_deleted() usen UpdateQuery cuando la entidad tenga soft_delete`.
- En `crates/mssql-orm/src/context.rs` se centralizó la decisión de borrado en un helper compartido que ahora compila `DELETE` físico para entidades normales y `UPDATE ... OUTPUT INSERTED.*` para entidades que declaran `soft_delete`.
- `DbSet::delete(...)`, `delete_by_sql_value(...)`, `delete_tracked_by_sql_value(...)` y `save_tracked_deleted()` ya convergen en esa misma ruta; `ActiveRecord::delete(&db)` quedó alineado sobre `DbSet` sin duplicar semántica.
- La ruta compartida preserva el predicate por primary key y añade `rowversion` cuando existe `concurrency_token`; si la operación no afecta filas y el registro sigue existiendo, retorna `OrmError::ConcurrencyConflict`.
- Para no introducir inferencia mágica, la compilación del `UPDATE` lógico usa el contrato runtime ya agregado (`SoftDeleteProvider`, `SoftDeleteContext`, `SoftDeleteRequestValues`) y falla explícitamente si la entidad soft-deleted necesita valores runtime que todavía no están disponibles.
- Se añadió cobertura unitaria en `crates/mssql-orm/src/context.rs` para fijar cuatro casos: `DELETE` físico en entidades normales, `UPDATE` lógico en entidades con `soft_delete`, preservación de `rowversion` en ese `UPDATE` y error explícito cuando falta un provider/valor runtime.
- También se ajustó `crates/mssql-orm/src/active_record.rs` para que la prueba unitaria local siga compilando bajo el nuevo bound de `SoftDeleteEntity`.
- Se actualizó `docs/tasks.md`, `docs/context.md` y `docs/entity-policies.md` para dejar explícito que el branching de borrado ya existe, pero que sigue faltando integrar `SoftDeleteProvider` al contexto público.
- Validaciones ejecutadas: `cargo fmt --all`, `cargo fmt --all --check`, `cargo check --workspace`, `cargo test -p mssql-orm dbset_delete_compiled_query --lib -- --nocapture`, `cargo test -p mssql-orm active_record_delete_reuses_dbset_error_contract --lib -- --nocapture` y `cargo test -p mssql-orm public_active_record_respects_rowversion_on_save_and_delete -- --nocapture`.
- La última validación no pudo ejecutar la integración real de SQL Server porque `MSSQL_ORM_TEST_CONNECTION_STRING` no estaba definido; el test quedó compilado y se auto-saltó, lo cual se considera esperado para esta sesión.

### Resultado

- El ORM ya no decide el tipo de borrado solo por el método llamado: para entidades con `soft_delete`, las rutas públicas de delete y change tracking ya entran al camino de `UpdateQuery` y preservan `rowversion`/`ConcurrencyConflict`.

### Bloqueos

- Falta wiring público de `SoftDeleteProvider`/valores por request en `DbContext`, así que una policy que necesite `deleted_at`, `deleted_by` o `is_deleted` todavía no puede completarse automáticamente desde esas rutas.

### Próximo paso recomendado

- Implementar `Etapa 16+: Integrar SoftDeleteProvider y/o SoftDeleteRequestValues al DbContext/DbSet para que delete, Active Record y change tracking puedan poblar columnas como deleted_at, deleted_by o is_deleted sin wiring manual interno`.

### Sesión: integración pública de `SoftDeleteProvider` en `DbContext`

- Se ejecutó la subtarea `Etapa 16+: Integrar SoftDeleteProvider y/o SoftDeleteRequestValues al DbContext/DbSet para que delete, Active Record y change tracking puedan poblar columnas como deleted_at, deleted_by o is_deleted sin wiring manual interno`.
- En `crates/mssql-orm/src/context.rs` se extendió `SharedConnection` con runtime config inmutable para `SoftDeleteProvider` y `SoftDeleteRequestValues`, más helpers públicos `with_soft_delete_provider(...)`, `with_soft_delete_request_values(...)` y `clear_soft_delete_request_values()`.
- `DbSet::delete_by_sql_value(...)` ahora lee esa configuración desde la conexión compartida y la pasa al helper `apply_soft_delete_values::<E>(...)`, de modo que las rutas de `delete`, Active Record y change tracking ya consumen el provider real sin wiring por `DbSet`.
- En `crates/mssql-orm-macros/src/lib.rs`, `#[derive(DbContext)]` ahora genera un constructor interno compartido para preservar el mismo `tracking_registry`, y expone métodos públicos `with_soft_delete_provider(...)`, `with_soft_delete_request_values(...)` y `clear_soft_delete_request_values()` sobre el contexto derivado.
- Se reforzó la surface pública en `crates/mssql-orm/src/lib.rs` y en `crates/mssql-orm/tests/ui/dbcontext_valid.rs` para fijar compile-time que esos helpers forman parte del contrato del derive.
- Se agregó `crates/mssql-orm/tests/stage16_soft_delete_runtime.rs` como smoke opcional contra SQL Server real: crea una entidad con `soft_delete`, configura el provider desde el contexto, ejecuta `delete`, verifica que la fila siga existiendo y confirma que `deleted_at` quedó poblado. El test se auto-salta cuando `MSSQL_ORM_TEST_CONNECTION_STRING` no está definido.
- Validaciones ejecutadas: `cargo fmt --all`, `cargo fmt --all --check`, `cargo check --workspace`, `cargo test -p mssql-orm --test trybuild entity_derive_ui -- --nocapture`, `cargo test -p mssql-orm exposes_dbcontext_soft_delete_runtime_helpers --lib -- --nocapture` y `cargo test -p mssql-orm public_dbcontext_soft_delete_provider_routes_delete_through_update -- --nocapture`.
- La última validación no pudo ejecutar SQL Server real en esta sesión porque `MSSQL_ORM_TEST_CONNECTION_STRING` no estaba definido; el smoke quedó compilado y se auto-saltó según diseño.

### Resultado

- `SoftDeleteProvider` ya no es solo un contrato aislado: quedó integrado al contexto público y puede viajar por `DbContext`, `DbSet`, Active Record, tracking y transacciones mediante `SharedConnection`.

### Bloqueos

- No hubo bloqueos técnicos después de ajustar el smoke de integración para usar `fetch_one(...)` en lugar de iterar `query_raw(...)`.

### Próximo paso recomendado

- Implementar `Etapa 16+: Implementar visibilidad de lectura para soft_delete en DbSetQuery con modo por defecto ActiveOnly y APIs públicas with_deleted() / only_deleted()`.

### Sesión: visibilidad de lectura para `soft_delete`

- Se ejecutó la subtarea `Etapa 16+: Implementar visibilidad de lectura para soft_delete en DbSetQuery con modo por defecto ActiveOnly y APIs públicas with_deleted() / only_deleted()`.
- En `crates/mssql-orm/src/dbset_query.rs` se agregó estado interno de visibilidad para `soft_delete` dentro de `DbSetQuery<E>`, manteniendo `SelectQuery` como AST neutral.
- `DbSetQuery<E>` ahora expone `with_deleted()` y `only_deleted()`. La visibilidad se materializa solo al ejecutar `all()`, `first()` y `count()`, no al construir el AST base.
- La convención implementada es explícita y mínima: la primera columna de la policy `soft_delete` controla visibilidad; si es nullable se usa `IS NULL` / `IS NOT NULL`, y si es `BIT` se usa `false` / `true`.
- `DbSet::find(...)` y `find_tracked(...)` ahora respetan esa visibilidad por defecto, mientras que `find_by_sql_value(...)` pasó a usar una ruta interna sin filtro implícito para preservar checks de existencia reales.
- También se ajustaron los bounds de `DbSet::update(...)`, `update_entity_by_sql_value(...)`, Active Record y `save_changes()` derivado para mantener consistencia con la nueva ruta interna que depende de `SoftDeleteEntity`.
- Se actualizó el smoke opcional `crates/mssql-orm/tests/stage16_soft_delete_runtime.rs`: después de `delete`, `find()` y `query().count()` ya ocultan la fila, mientras `with_deleted()` y `only_deleted()` siguen viéndola.
- Validaciones ejecutadas: `cargo fmt --all`, `cargo check --workspace`, `cargo test -p mssql-orm dbset_query_ --lib -- --nocapture`, `cargo test -p mssql-orm active_record_find_reuses_dbset_error_contract --lib -- --nocapture` y `cargo test -p mssql-orm public_dbcontext_soft_delete_provider_routes_delete_through_update -- --nocapture`.
- La última validación no pudo ejecutar SQL Server real en esta sesión porque `MSSQL_ORM_TEST_CONNECTION_STRING` no estaba definido; el smoke quedó compilado y se auto-saltó según diseño.

### Resultado

- `soft_delete` ya afecta tanto escritura como lectura pública de la entidad raíz: las queries normales esconden filas borradas lógicamente y el usuario puede recuperarlas de forma explícita con `with_deleted()` u `only_deleted()`.

### Bloqueos

- No hubo bloqueos técnicos; solo fue necesario propagar los bounds de `SoftDeleteEntity` a rutas que reutilizan búsquedas internas sin filtro.

### Próximo paso recomendado

- Implementar `Etapa 16+: Agregar rutas internas sin filtro implícito de soft_delete para comprobaciones de existencia y ConcurrencyConflict sin exponer bypass público accidental`.

### Sesión: rutas internas sin filtro implícito para `soft_delete`

- Se ejecutó la subtarea `Etapa 16+: Agregar rutas internas sin filtro implícito de soft_delete para comprobaciones de existencia y ConcurrencyConflict sin exponer bypass público accidental`.
- En `crates/mssql-orm/src/context.rs` se renombró y explicitó la intención de la ruta interna: `query_with_internal_visibility(...)`, `find_by_sql_value_internal(...)` y `exists_by_sql_value_internal(...)` quedan dedicados a lectura/existencia real de filas con `WithDeleted`, no a surface pública de consulta.
- Los checks de `ConcurrencyConflict` en `update(...)`, `delete(...)` y `update_entity_values_by_sql_value(...)` ya no dependen de nombres ambiguos; usan `exists_by_sql_value_internal(...)` como helper interno explícito.
- `ActiveRecord::save(...)` también dejó de usar una lectura interna con nombre ambiguo y ahora consulta existencia real mediante el helper interno de `DbSet`.
- Se añadió una prueba unitaria dedicada en `crates/mssql-orm/src/context.rs` para fijar que la ruta interna efectivamente nace sin el filtro implícito de `soft_delete`, pero sigue encapsulada dentro de `DbSet`.
- Validaciones ejecutadas: `cargo fmt --all`, `cargo fmt --all --check`, `cargo check --workspace`, `cargo test -p mssql-orm dbset_internal_query_visibility_bypasses_soft_delete_filter --lib -- --nocapture` y `cargo test -p mssql-orm active_record_find_reuses_dbset_error_contract --lib -- --nocapture`.
- No se ejecutó un smoke adicional contra SQL Server real porque esta sesión solo endureció helpers internos y nombres de intención; no cambió la semántica observable de lectura o borrado ya cubierta en la subtarea anterior.

### Resultado

- El bypass sin filtro implícito de `soft_delete` ya quedó encapsulado como helper interno de `DbSet` para existencia/lectura real, y dejó de parecer una ruta pública accidental dentro de la crate.

### Bloqueos

- No hubo bloqueos técnicos.

### Próximo paso recomendado

- Implementar `Etapa 16+: Integrar soft_delete en snapshots, diff y DDL como columnas ordinarias sin abrir un segundo pipeline de esquema`.

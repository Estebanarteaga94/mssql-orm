# Tasks

## Pendientes
- [ ] Etapa 1: Generar columnas estáticas para el futuro query builder
- [ ] Etapa 1: Agregar pruebas `trybuild` para casos válidos e inválidos de entidades
- [ ] Etapa 2: Implementar `FromRow`, `Insertable`, `Changeset` y `SqlValue`
- [ ] Etapa 2: Implementar derives `#[derive(Insertable)]` y `#[derive(Changeset)]`
- [ ] Etapa 2: Definir mapeo base Rust -> SQL Server para tipos soportados
- [ ] Etapa 2: Crear pruebas de mapping de filas y extracción de valores persistibles
- [ ] Etapa 3: Implementar AST de queries y `CompiledQuery`
- [ ] Etapa 3: Implementar quoting seguro de identificadores SQL Server
- [ ] Etapa 3: Compilar `select`, `insert`, `update`, `delete` y `count` a SQL parametrizado `@P1..@Pn`
- [ ] Etapa 3: Agregar snapshot tests para SQL y orden de parámetros
- [ ] Etapa 4: Implementar `MssqlConnection` y configuración desde connection string
- [ ] Etapa 4: Implementar `Executor` sobre Tiberius con binding de parámetros
- [ ] Etapa 4: Implementar wrapper `MssqlRow` y conversión de errores a `OrmError`
- [ ] Etapa 4: Agregar pruebas de integración contra SQL Server real
- [ ] Etapa 5: Implementar `DbContext` trait, `DbSet<T>` y `#[derive(DbContext)]`
- [ ] Etapa 5: Exponer API CRUD base `find`, `insert`, `update`, `delete`, `query`
- [ ] Etapa 5: Crear ejemplo funcional `basic-crud`
- [ ] Etapa 6: Implementar query builder público con filtros, composición lógica, ordenamiento, limit y paginación
- [ ] Etapa 6: Agregar pruebas snapshot y de seguridad de parámetros para el query builder público
- [ ] Etapa 7: Implementar `ModelSnapshot`, diff engine y operaciones básicas de migración
- [ ] Etapa 7: Implementar generación SQL y tabla `__mssql_orm_migrations`
- [ ] Etapa 7: Implementar CLI mínima con `migration add`, `database update` y `migration list`
- [ ] Etapa 7: Validar migraciones iniciales e incrementales contra SQL Server real
- [ ] Etapa 8: Implementar transacciones con commit en `Ok` y rollback en `Err`
- [ ] Etapa 8: Agregar pruebas de commit y rollback
- [ ] Etapa 9: Implementar metadata de relaciones, foreign keys, joins explícitos e índices asociados
- [ ] Etapa 9: Soportar `delete behavior` inicial (`no action`, `cascade`, `set null`)
- [ ] Etapa 10: Implementar capa opcional Active Record sobre `DbSet`
- [ ] Etapa 11: Implementar soporte de concurrencia optimista con `rowversion`
- [ ] Etapa 11: Retornar `OrmError::ConcurrencyConflict` en conflictos de actualización o borrado
- [ ] Etapa 12: Implementar change tracking experimental con `Tracked<T>` y `save_changes`
- [ ] Etapa 13: Soportar migraciones avanzadas: renombres, computed columns, FKs completas, índices compuestos y scripts idempotentes
- [ ] Etapa 14: Implementar pooling opcional, timeouts, `tracing`, slow query logs y health checks
- [ ] Etapa 14: Crear ejemplo de integración con framework web async
- [ ] Etapa 15: Preparar release con documentación pública, quickstart, ejemplos completos y changelog

## En Progreso
- [ ] (vacío)

## Completadas
- [x] Inicialización del sistema autónomo
- [x] Crear base documental operativa en `docs/`
- [x] Etapa 0: Crear workspace Rust con crates internas base (`mssql-orm`, `core`, `macros`, `query`, `sqlserver`, `tiberius`, `migrate`, `cli`)
- [x] Operativo: Consolidar repositorio Git único en la raíz y registrar política de commit al cierre
- [x] Etapa 0: Configurar CI base con `cargo check`, `cargo test`, `rustfmt` y `clippy`
- [x] Etapa 0: Crear README principal, ADRs iniciales y documentación arquitectónica mínima
- [x] Etapa 0: Crear documentación de colaboración con IA en `docs/ai/`
- [x] Etapa 1: Implementar `Entity` trait y metadata base (`EntityMetadata`, `ColumnMetadata`, índices y foreign keys)
- [x] Operativo: Corregir desalineaciones contra el plan maestro en metadata base de entidades
- [x] Etapa 1: Implementar `#[derive(Entity)]` con parser de atributos `#[orm(...)]`
- [x] Etapa 1: Soportar atributos base `table`, `schema`, `primary_key`, `identity`, `length`, `nullable`, `default_sql`, `index` y `unique`

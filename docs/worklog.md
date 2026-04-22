# Worklog

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

# Arquitectura del ORM

## Objetivo

El proyecto construye un ORM `code-first` para SQL Server. La arquitectura está separada por capas para evitar mezclar metadata, composición de consultas, compilación SQL y ejecución.

## Flujo esperado

1. El usuario define entidades y contexto en Rust.
2. `mssql-orm-macros` genera metadata estática y contratos auxiliares.
3. `mssql-orm-query` construye un AST tipado y libre de SQL.
4. `mssql-orm-sqlserver` compila ese AST a SQL parametrizado para SQL Server.
5. `mssql-orm-tiberius` ejecuta la consulta y adapta resultados y errores.
6. `mssql-orm-migrate` usa la metadata para snapshots y migraciones.
7. `mssql-orm` expone la superficie pública soportada por el sistema.

## Límites por crate

### `mssql-orm-core`

- Define contratos estables compartidos.
- Aloja metadata, tipos comunes y errores.
- No conoce Tiberius ni detalles de ejecución.

### `mssql-orm-macros`

- Implementa `derive` y parsing de `#[orm(...)]`.
- Genera metadata y código auxiliar en compilación.
- No debe asumir generación SQL ni acceso a red.

### `mssql-orm-query`

- Representa el AST y el query builder tipado.
- Modela selección, filtros, ordenamiento, paginación y composición.
- No emite SQL directo.

### `mssql-orm-sqlserver`

- Convierte el AST en SQL Server parametrizado.
- Centraliza quoting de identificadores, placeholders `@P1..@Pn` y decisiones de dialecto.
- No abre conexiones ni ejecuta consultas.

### `mssql-orm-tiberius`

- Encapsula conexión, ejecución, filas y transacciones.
- Traduce errores del driver a errores del ORM.
- No define metadata ni compila AST a SQL.

### `mssql-orm-migrate`

- Calcula snapshots y diffs del modelo.
- Produce operaciones y SQL de migración para SQL Server.
- Depende del modelo y del compilador SQL, no del query builder público.

### `mssql-orm-cli`

- Orquesta comandos de migración y tareas operativas.
- Debe apoyarse en crates internas, no duplicar lógica de dominio.

### `mssql-orm`

- Es la única superficie pública consolidada.
- Reexporta tipos, derives y módulos soportados para consumidores.

## Decisiones vigentes

- Se soporta solo SQL Server en esta fase.
- La separación por crates es estructural y no debe colapsarse por conveniencia.
- El MVP prioriza metadata, derives, CRUD base y migraciones iniciales antes de features avanzadas.

## Estado actual

- La arquitectura está implementada como workspace con crates separadas para la API pública, core, macros, query AST, compilación SQL Server, ejecución Tiberius, migraciones y CLI.
- `docs/repository-audit.md` mantiene el inventario verificado de APIs reales, features implementadas, límites y features diferidas.
- Pending verification: cualquier claim de estado funcional no cubierto por `docs/repository-audit.md`, pruebas versionadas o `docs/worklog.md` debe verificarse contra el código antes de repetirse en documentación pública.

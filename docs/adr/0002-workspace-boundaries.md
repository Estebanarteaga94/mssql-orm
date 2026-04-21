# ADR 0002: Separación estricta por crates

## Estado

Aprobado

## Contexto

El sistema necesita separar metadata, macros, AST, compilación SQL, ejecución y migraciones para evitar acoplamientos tempranos. El workspace ya fue creado con crates independientes y esa segmentación debe preservarse antes de introducir funcionalidad real.

## Decisión

Se mantiene una arquitectura modular con crates dedicadas:

- `mssql-orm-core`
- `mssql-orm-macros`
- `mssql-orm-query`
- `mssql-orm-sqlserver`
- `mssql-orm-tiberius`
- `mssql-orm-migrate`
- `mssql-orm-cli`
- `mssql-orm`

## Consecuencias

- `core` no depende de Tiberius.
- `query` no genera SQL directo.
- `mssql-orm-sqlserver` concentra la generación de SQL.
- `mssql-orm-tiberius` concentra la ejecución.
- `mssql-orm` centraliza la API pública.
- Los cambios futuros deben respetar estos límites salvo nueva decisión explícita.

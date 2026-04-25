# Entity Policies

Diseno publico inicial de `Entity Policies` para `mssql-orm`.

Este documento fija el concepto de producto y las fronteras arquitectonicas de la Etapa 16. No declara la feature como disponible todavia; las tareas siguientes deben convertir este diseno en contratos de `core`, derives, tests y documentacion publica de uso.

## Objetivo

Una `Entity Policy` es una pieza reutilizable de modelo `code-first` que una entidad puede declarar para incorporar columnas transversales y, en etapas futuras, comportamiento asociado.

El problema que resuelve es evitar duplicar manualmente los mismos campos estructurales en muchas entidades, por ejemplo columnas de auditoria, timestamps, borrado logico o tenant. La policy no reemplaza al modelo de entidad actual: lo extiende de forma declarativa.

Ejemplo objetivo de lectura publica:

```rust
use mssql_orm::prelude::*;

#[derive(AuditFields)]
struct Audit {
    #[orm(default_sql = "SYSUTCDATETIME()")]
    created_at: chrono::NaiveDateTime,

    #[orm(nullable)]
    updated_at: Option<chrono::NaiveDateTime>,
}

#[derive(Entity, Debug, Clone)]
#[orm(table = "todos", schema = "todo", audit = Audit)]
struct Todo {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(length = 200)]
    title: String,
}
```

## Principio Central

Las columnas aportadas por una policy deben terminar como `ColumnMetadata` normales dentro de `EntityMetadata.columns`.

Esta regla evita crear un segundo pipeline de esquema. El resto del sistema debe poder seguir trabajando con las mismas piezas:

- `ModelSnapshot::from_entities(...)` lee columnas desde `EntityMetadata`.
- El diff compara `ColumnSnapshot` sin saber si una columna vino de un campo propio o de una policy.
- `mssql-orm-sqlserver` compila DDL desde snapshots y operaciones normales.
- `DbContext` y la CLI de migraciones siguen consumiendo metadata de entidades.

La policy puede tener metadata interna para validacion y ergonomia, pero esa metadata no debe convertirse en una ruta paralela para snapshots, diff o DDL.

## Forma Publica Esperada

El concepto publico se expresa en atributos sobre la entidad:

```rust
#[derive(Entity)]
#[orm(table = "orders", schema = "sales", audit = Audit)]
struct Order {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,
}
```

La policy referenciada debe ser un tipo Rust visible desde el sitio donde se deriva la entidad. La intencion de Etapa 16 es empezar con un contrato tipo `#[derive(AuditFields)]` o equivalente, para que el usuario defina el shape reusable en su propio crate.

La declaracion vive en compile-time. No debe depender de configuracion runtime, reflection o descubrimiento dinamico.

## Relacion con el Enfoque Code-First

`Entity Policies` siguen el mismo modelo mental del resto del ORM:

- El codigo Rust es la fuente de verdad.
- Los `proc_macros` generan metadata estatica.
- SQL Server sigue siendo el unico backend objetivo.
- La API publica sigue concentrada en la crate raiz `mssql-orm`.
- La generacion SQL sigue perteneciendo a `mssql-orm-sqlserver`.
- La ejecucion sigue perteneciendo a `mssql-orm-tiberius`.

Una policy no debe introducir un DSL alternativo ni una capa de configuracion fluida en esta etapa.

## Politicas Candidatas

El concepto general cubre varias preocupaciones transversales, pero no todas pertenecen al MVP:

- `audit = Audit`: columnas como `created_at`, `created_by`, `updated_at`, `updated_by`.
- `timestamps = Timestamps`: variante reducida para `created_at` y `updated_at`.
- `concurrency = RowVersion`: forma declarativa futura sobre el soporte actual de `#[orm(rowversion)]`.
- `soft_delete = SoftDelete`: columnas y semantica de borrado logico.
- `tenant = TenantScope`: columna y filtros obligatorios de seguridad por tenant.

La primera policy que debe implementarse es auditoria como generacion de columnas. Las politicas que cambian comportamiento de lectura o escritura requieren diseno separado porque afectan `DbSet`, Active Record, transacciones y change tracking.

## Limites Arquitectonicos

Las tareas de implementacion deben respetar estos limites:

- `core` puede definir contratos neutrales, pero no puede depender de Tiberius ni de SQL ejecutable.
- `query` no debe conocer policies ni generar SQL directo.
- `sqlserver` no debe recibir una segunda representacion de esquema para policies.
- `tiberius` no debe interpretar metadata de policies.
- `migrate` debe seguir operando sobre snapshots derivados de columnas normales.
- La crate publica debe reexportar solo la surface necesaria para el consumidor.

## Comportamiento Diferido

`Entity Policies` pueden llegar a aportar comportamiento automatico, pero eso no forma parte del primer corte de implementacion.

Quedan diferidos hasta tener contratos especificos:

- autollenado de `created_at`, `created_by`, `updated_at` y `updated_by`
- provider runtime de auditoria por request o transaccion
- filtros automaticos de `soft_delete`
- reemplazo de `DELETE` fisico por `UPDATE` logico
- filtros obligatorios de tenant
- insercion automatica de `tenant_id`

El primer objetivo verificable es que las columnas declaradas por una policy aparezcan en metadata, snapshots, diff y DDL como columnas ordinarias.

## Criterio de Aceptacion Conceptual

El concepto queda listo para pasar a implementacion cuando se cumplan estas condiciones:

- Existe una sintaxis publica documentada para declarar una policy en una entidad.
- Existe un contrato de metadata reutilizable que produce `ColumnMetadata`.
- Las columnas generadas tienen orden estable y reglas de colision claras.
- Las rutas existentes de snapshots, diff y DDL no reciben un pipeline paralelo.
- Los comportamientos automaticos quedan explicitamente fuera del MVP o en tareas futuras.


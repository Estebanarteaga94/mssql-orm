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

## Contrato de Metadata

El contrato neutral vive en `mssql-orm-core` y no conoce Tiberius, SQL ejecutable ni el sistema de migraciones:

```rust
pub struct EntityPolicyMetadata {
    pub name: &'static str,
    pub columns: &'static [ColumnMetadata],
}

pub trait EntityPolicy: Sized + Send + Sync + 'static {
    const POLICY_NAME: &'static str;

    fn columns() -> &'static [ColumnMetadata];

    fn metadata() -> EntityPolicyMetadata {
        EntityPolicyMetadata::new(Self::POLICY_NAME, Self::columns())
    }
}
```

La responsabilidad del contrato es minima: una policy reusable expone un nombre estable y un slice estatico de `ColumnMetadata`. La expansion dentro de una entidad sigue siendo responsabilidad de `mssql-orm-macros`.

El contrato no agrega una coleccion de policies a `EntityMetadata` en esta etapa. Esa decision es intencional: el dato que debe circular por snapshots, diff y DDL es la columna resultante, no la policy que la produjo.

Las siguientes tareas deben definir como `AuditFields` implementa o reutiliza `EntityPolicy`, como se validan colisiones y como el derive `Entity` incorpora esas columnas al arreglo final de `EntityMetadata.columns`.

Estado actual: `#[derive(AuditFields)]` ya implementa `EntityPolicy` para el struct de auditoria y expone sus campos como `ColumnMetadata` reutilizable. La integracion con `#[derive(Entity)]` mediante `#[orm(audit = Audit)]` sigue pendiente.

## Forma Publica Esperada

El concepto publico se expresa en atributos sobre la entidad. Para el MVP de auditoria, la sintaxis canónica soportada sera:

```rust
#[derive(Entity)]
#[orm(table = "orders", schema = "sales", audit = Audit)]
struct Order {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,
}
```

La policy referenciada debe ser un tipo Rust visible desde el sitio donde se deriva la entidad. La intencion de Etapa 16 es empezar con `#[derive(AuditFields)]`, para que el usuario defina el shape reusable en su propio crate.

La declaracion vive en compile-time. No debe depender de configuracion runtime, reflection o descubrimiento dinamico.

## Sintaxis MVP de Auditoria

La sintaxis MVP queda fijada como atributo de entidad:

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

Reglas de sintaxis:

- `audit = Audit` se declara en `#[orm(...)]` a nivel del struct que deriva `Entity`
- el lado derecho debe ser un path Rust hacia un tipo que implemente el contrato de auditoria definido por el derive posterior
- `Audit` se resuelve en compile-time, desde el scope normal de Rust del consumidor
- una entidad puede declarar como maximo una policy `audit`
- entidades sin `audit` deben conservar exactamente el comportamiento actual

Formas permitidas:

```rust
#[orm(audit = Audit)]
#[orm(audit = crate::model::Audit)]
#[orm(table = "orders", schema = "sales", audit = common::Audit)]
```

Formas rechazadas para el MVP:

```rust
#[orm(audit)]
#[orm(audit = "Audit")]
#[orm(audit(Audit))]
#[orm(audit = Audit::default())]
#[orm(audit_provider = provider)]
```

Estas variantes se rechazan porque introducen inferencia implicita, strings sin chequeo de tipos, sintaxis paralela o configuracion runtime antes de tener contratos estables.

La sintaxis tambien excluye por ahora configuracion inline de columnas dentro de la entidad:

```rust
#[orm(audit(created_at, updated_at))]
#[orm(audit(created_at = "created_on"))]
```

El shape de columnas debe vivir en el struct de auditoria reusable, no en cada entidad consumidora. Esto mantiene la policy reutilizable y evita duplicar configuracion por entidad.

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

## Alcance Inicial

El alcance inicial de Etapa 16 es deliberadamente estrecho: probar que el modelo `code-first` puede reutilizar columnas transversales sin cambiar el pipeline de metadata, snapshots, diff ni DDL.

La unica policy que entra al MVP de implementacion es `audit = Audit`.

`timestamps = Timestamps` queda reconocida como policy de columnas generadas, pero no entra al primer corte de codigo. Debe evaluarse despues de `audit` para decidir si sera una policy separada, un alias reducido o una convencion encima del mismo contrato de metadata.

`soft_delete = SoftDelete`, `tenant = TenantScope` y cualquier autollenado runtime quedan fuera del MVP.

| Policy | Estado de Etapa 16 | Alcance permitido |
| --- | --- | --- |
| `audit = Audit` | MVP | Generar columnas normales de auditoria dentro de `EntityMetadata.columns`. |
| `timestamps = Timestamps` | Diferido dentro de Etapa 16 | Disenar despues de `audit`; solo podria aportar columnas si no duplica nombres ni semantica. |
| `soft_delete = SoftDelete` | Etapa 16+ | Requiere redisenar rutas de borrado, queries por defecto y Active Record. |
| `tenant = TenantScope` | Etapa 16+ | Requiere contrato de seguridad, tenant activo y filtros obligatorios en toda ruta publica. |
| `AuditProvider` o autollenado | Etapa 16+ | Requiere integracion runtime con inserts, updates, transacciones y change tracking. |

## Que Significa Columnas Generadas

Una policy de columnas generadas aporta metadata de columnas como si esas columnas hubieran sido declaradas manualmente en la entidad.

Para el MVP, eso significa:

- cada columna generada tiene `rust_field`, `column_name`, tipo SQL, nullability, defaults y flags `insertable`/`updatable`
- el orden de columnas en `EntityMetadata.columns` debe ser estable
- las columnas generadas participan en snapshots, diff y DDL sin rutas especiales
- las colisiones con campos propios o con otras policies deben fallar en compile-time
- las columnas generadas no implican autollenado de valores en operaciones de escritura

Esta definicion permite validar la feature en migraciones antes de introducir comportamiento runtime.

## Alcance de `audit = Audit`

`audit = Audit` es una policy definida por el usuario mediante un tipo Rust reusable. Su responsabilidad inicial es describir columnas auditables.

El usuario debe poder controlar el shape de esas columnas desde atributos de columna ya familiares en el proyecto, por ejemplo:

- `column`
- `length`
- `nullable`
- `default_sql`
- `sql_type`
- `precision`
- `scale`

El MVP no debe imponer nombres globales como unica forma valida. Nombres frecuentes como `created_at`, `created_by`, `updated_at` o `updated_by` deben surgir del struct reusable o de atributos explicitos.

Reglas iniciales esperadas:

- los campos de `Audit` se expanden despues de las columnas propias de la entidad, salvo que una tarea posterior documente otro orden estable
- los campos de `Audit` no forman parte de la primary key de la entidad
- los campos de `Audit` no crean foreign keys automaticamente
- los campos de `Audit` no generan filtros ni hooks runtime
- defaults SQL como `SYSUTCDATETIME()` son metadata de esquema, no valores calculados por Rust

## Shape de `AuditFields`

El struct de auditoria del usuario debe ser un struct Rust con campos nombrados y `#[derive(AuditFields)]`.

Ejemplo objetivo:

```rust
use mssql_orm::prelude::*;

#[derive(AuditFields)]
struct Audit {
    #[orm(default_sql = "SYSUTCDATETIME()")]
    #[orm(updatable = false)]
    created_at: chrono::NaiveDateTime,

    #[orm(nullable)]
    #[orm(length = 120)]
    created_by: Option<String>,

    #[orm(nullable)]
    updated_at: Option<chrono::NaiveDateTime>,

    #[orm(nullable)]
    #[orm(length = 120)]
    updated_by: Option<String>,
}
```

Cada campo de `Audit` se convierte en un `ColumnMetadata` normal. El nombre Rust del campo se usa como `rust_field` y, por defecto, tambien como `column_name`. El atributo `#[orm(column = "...")]` puede cambiar el nombre de columna sin cambiar el nombre Rust del campo.

Tipos soportados:

- cualquier tipo que implemente `SqlTypeMapping`
- `Option<T>` cuando `T: SqlTypeMapping`, marcando la columna como nullable
- los tipos ya soportados por `core`: `bool`, enteros soportados, `f64`, `String`, `Vec<u8>`, `uuid::Uuid`, `rust_decimal::Decimal`, `chrono::NaiveDate` y `chrono::NaiveDateTime`

Reglas de nullability:

- `Option<T>` implica `nullable = true`
- `#[orm(nullable)]` tambien marca la columna como nullable
- un campo no `Option<T>` sin `#[orm(nullable)]` queda como `nullable = false`
- `#[orm(nullable)]` sobre `Option<T>` es redundante pero aceptable si el derive base ya lo acepta de forma consistente

Atributos permitidos en campos de auditoria:

- `column`
- `length`
- `nullable`
- `default_sql`
- `sql_type`
- `precision`
- `scale`
- `renamed_from`
- `insertable`
- `updatable`

Atributos rechazados en campos de auditoria:

- `primary_key`
- `identity`
- `computed_sql`
- `rowversion`
- `index`
- `unique`
- `foreign_key`
- `on_delete`

La razon es que `AuditFields` solo debe aportar columnas reutilizables. Primary keys, identity, computed columns, rowversion, indices y relaciones siguen perteneciendo al entity o a tareas futuras con contrato propio.

Reglas de escritura:

- por defecto, una columna auditable es `insertable = true`
- por defecto, una columna auditable es `updatable = true`
- `#[orm(insertable = false)]` permite excluir una columna del contrato de insercion
- `#[orm(updatable = false)]` permite excluir una columna del contrato de update
- `created_at` y `created_by` deberian declararse normalmente con `updatable = false`
- `updated_at` y `updated_by` pueden quedar `updatable = true`

Estas flags son solo metadata de columna. En el MVP no hacen que `DbSet::insert`, `DbSet::update`, Active Record ni `save_changes` rellenen valores automaticamente.

## Alcance de `timestamps = Timestamps`

`timestamps = Timestamps` queda reservado como policy candidata para aportar solo columnas temporales, normalmente `created_at` y `updated_at`.

No se implementa junto con `audit` en el primer corte porque puede solaparse con nombres y semantica de auditoria. Antes de implementarla se debe decidir:

- si reutiliza el mismo contrato de metadata que `AuditFields`
- si es un alias predefinido o un struct definido por el usuario
- como se detectan colisiones con `audit = Audit`
- si tendra defaults SQL o autollenado futuro

Hasta resolver esas decisiones, `timestamps` no debe aparecer como API compilable.

## Fuera del MVP

Estas capacidades quedan explicitamente fuera del MVP aunque sean parte del concepto general de `Entity Policies`:

- rellenar `created_at` o `updated_at` desde Rust al insertar o actualizar
- leer usuario actual desde contexto, request o variable global
- modificar `Insertable`, `Changeset` o `EntityPersist` para inyectar valores auditables
- convertir `delete` en borrado logico
- agregar filtros implicitos a `query()`, `find`, `count`, joins o Active Record
- exigir tenant activo o validar `tenant_id` en inserts
- alterar el comportamiento de `save_changes()`

Cada una de esas capacidades necesita su propia tarea, pruebas y contrato publico antes de entrar al codigo.

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

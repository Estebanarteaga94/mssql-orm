# Entity Policies

Diseno publico inicial de `Entity Policies` para `mssql-orm`.

Este documento fija el concepto de producto, las fronteras arquitectonicas y el estado publico de `Entity Policies` en la Etapa 16. El MVP disponible cubre auditoria como metadata/schema mediante `#[derive(AuditFields)]` y `#[orm(audit = Audit)]`; las policies con comportamiento runtime siguen diferidas.

## Objetivo

Una `Entity Policy` es una pieza reutilizable de modelo `code-first` que una entidad puede declarar para incorporar columnas transversales y, en etapas futuras, comportamiento asociado.

El problema que resuelve es evitar duplicar manualmente los mismos campos estructurales en muchas entidades, por ejemplo columnas de auditoria, borrado logico o tenant. La policy no reemplaza al modelo de entidad actual: lo extiende de forma declarativa.

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
    const COLUMN_NAMES: &'static [&'static str] = &[];

    fn columns() -> &'static [ColumnMetadata];

    fn metadata() -> EntityPolicyMetadata {
        EntityPolicyMetadata::new(Self::POLICY_NAME, Self::columns())
    }
}
```

La responsabilidad del contrato es minima: una policy reusable expone un nombre estable, un slice estatico de nombres de columna para validaciones `const` y un slice estatico de `ColumnMetadata`. La expansion dentro de una entidad sigue siendo responsabilidad de `mssql-orm-macros`.

El contrato no agrega una coleccion de policies a `EntityMetadata` en esta etapa. Esa decision es intencional: el dato que debe circular por snapshots, diff y DDL es la columna resultante, no la policy que la produjo.

Las siguientes tareas deben definir como se validan colisiones entre columnas propias y columnas generadas, y como se cubre el pipeline de snapshots, diff y DDL con esas columnas ordinarias.

Estado actual: `#[derive(AuditFields)]` ya implementa `EntityPolicy` para el struct de auditoria y expone sus campos como `ColumnMetadata` reutilizable, ademas de `COLUMN_NAMES` para validacion compile-time. `#[derive(Entity)]` ya acepta `#[orm(audit = Audit)]`, rechaza una segunda declaracion `audit`, valida colisiones entre columnas propias y columnas auditables mediante aserciones constantes y expande esas columnas dentro de `EntityMetadata.columns`.

La surface publica necesaria para el caso valido esta reexportada desde `mssql_orm::prelude::*`. Existe cobertura `trybuild` desde la perspectiva de un consumidor para derivar `AuditFields`, declarar `#[orm(audit = Audit)]`, consultar metadata, acceder al contrato `EntityPolicy` y compilar `FromRow` sin importar rutas internas.

La cobertura negativa de `trybuild` ya fija estos errores de auditoria: tipo de policy inexistente en `#[orm(audit = ...)]`, `AuditFields` sobre struct sin campos nombrados, atributo no soportado, `column = ""`, columna duplicada y campo con tipo sin `SqlTypeMapping`.

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
- una entidad puede declarar como maximo una policy `audit`; repetir `audit` falla en compile-time en vez de sobrescribir silenciosamente la policy anterior
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
- `soft_delete = SoftDelete`: columnas y semantica de borrado logico.
- `tenant = TenantScope`: columna y filtros obligatorios de seguridad por tenant.

La primera policy que debe implementarse es auditoria como generacion de columnas. Las politicas que cambian comportamiento de lectura o escritura requieren diseno separado porque afectan `DbSet`, Active Record, transacciones y change tracking.

## Alcance Inicial

El alcance inicial de Etapa 16 es deliberadamente estrecho: probar que el modelo `code-first` puede reutilizar columnas transversales sin cambiar el pipeline de metadata, snapshots, diff ni DDL.

La unica policy que entra al MVP de implementacion es `audit = Audit`. Los casos que solo necesitan columnas temporales pueden modelarse con un struct `AuditFields` reducido que declare `created_at` y `updated_at`.

`soft_delete = SoftDelete`, `tenant = TenantScope` y cualquier autollenado runtime quedan fuera del MVP.

| Policy | Estado de Etapa 16 | Alcance permitido |
| --- | --- | --- |
| `audit = Audit` | MVP | Generar columnas normales de auditoria dentro de `EntityMetadata.columns`. |
| `soft_delete = SoftDelete` | Etapa 16+ | Requiere redisenar rutas de borrado, queries por defecto y Active Record. |
| `tenant = TenantScope` | Etapa 16+ | Requiere contrato de seguridad, tenant activo y filtros obligatorios en toda ruta publica. |
| `AuditProvider` o autollenado | Etapa 16+ | Requiere integracion runtime con inserts, updates, transacciones y change tracking. |

## Concurrencia y `rowversion`

La concurrencia optimista no debe modelarse como una `Entity Policy` separada.

El soporte vigente ya esta alineado con el plan maestro mediante un atributo de columna:

```rust
#[derive(Entity, Debug, Clone)]
struct Customer {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(rowversion)]
    version: Vec<u8>,
}
```

Ese shape debe seguir siendo la forma canonica porque la columna `rowversion` necesita existir como campo Rust visible en la entidad. La entidad materializada debe conservar el token devuelto por SQL Server para que `EntityPersist::concurrency_token()`, `Changeset::concurrency_token()`, Active Record y `save_changes()` puedan emitir predicates protegidos como:

```sql
WHERE [id] = @P1 AND [version] = @P2
```

Una policy oculta tipo `#[orm(concurrency = RowVersion)]` aportaria una columna sin campo Rust visible, igual que `audit = Audit`. Eso no sirve para la concurrencia actual: sin token visible en la entidad no hay valor que comparar en updates/deletes posteriores, y el resultado seria un segundo pipeline o una semantica incompleta.

Decision vigente:

- no implementar `concurrency = RowVersion` como policy declarativa
- mantener `#[orm(rowversion)]` como API publica canonica
- preservar `Vec<u8>` como tipo requerido para el token
- preservar `OrmError::ConcurrencyConflict` como error publico estable cuando el token no coincide y la primary key todavia existe
- no mover reglas de concurrencia a `AuditProvider`, `EntityPolicy`, `mssql-orm-query`, `mssql-orm-sqlserver` ni `mssql-orm-tiberius`

Si en el futuro se quiere mejorar ergonomia, debe hacerse sobre el atributo de campo existente, por ejemplo con documentacion o helpers de derives, no con una policy que genere columnas ocultas.

## Evaluacion de `soft_delete = SoftDelete`

`soft_delete = SoftDelete` si encaja en el concepto general de `Entity Policies`, pero no es una policy de metadata pura como `audit = Audit`.

Cambiar esa feature implica redefinir comportamiento observable en varias rutas publicas que hoy ya existen y hoy emiten borrado fisico real:

- `DbSet::delete(...)` compila `DeleteQuery` y termina en `DELETE FROM ...`
- `entity.delete(&db)` delega a `DbSet::delete_by_sql_value(...)`
- `DbSet::remove_tracked(...)` y `save_changes()` hoy convergen en rutas que terminan borrando fisicamente
- `DbSet::query()` y `DbSetQuery` parten de `SelectQuery::from_entity::<E>()` sin filtros implicitos
- migraciones y snapshots hoy solo conocen columnas, no semantica de borrado

Por eso, la decision vigente es tratar `soft_delete` como un cambio semantico de alto riesgo, no como una extension menor del derive.

Riesgos concretos que deben quedar resueltos antes de implementar:

- si alguna ruta publica sigue compilando `DELETE FROM` para una entidad con `soft_delete`, la feature queda rota de forma silenciosa
- si las queries por defecto no excluyen filas borradas logicamente, el modelo queda inconsistente con la expectativa del usuario
- si los joins o `count()` ignoran el filtro de borrado logico en unas rutas si y en otras no, la API publica se vuelve impredecible
- si `save_changes()` marca `Deleted` pero luego intenta hacer `DELETE` fisico, se rompe la unidad de trabajo experimental
- si `rowversion` no participa en el `UPDATE` de borrado logico, se pierde la proteccion de `ConcurrencyConflict`
- si migraciones/snapshots no tratan las columnas de `SoftDelete` como columnas ordinarias, se abre un segundo pipeline de esquema

Decision de evaluacion:

- `soft_delete = SoftDelete` sigue siendo candidata valida de roadmap
- no debe implementarse como alias ni como convencion por nombres magicos (`deleted_at`, `is_deleted`) sin declaracion explicita
- antes de tocar macros o runtime, debe existir un diseño separado para:
  `delete`, Active Record, tracking, queries por defecto, APIs explicitas `with_deleted`/`only_deleted` o equivalente, y migraciones
- la implementacion debe converger en las rutas actuales de `DbSet`, `ActiveRecord`, `save_changes()` y `DbSetQuery`; no debe abrir una segunda via de ejecucion

En otras palabras: `soft_delete` es razonable como feature futura, pero solo despues de redisenar explicitamente el contrato de borrado y lectura. No debe entrar como una policy “pequeña” solo porque comparta el mecanismo de metadata.

## Que Significa Columnas Generadas

Una policy de columnas generadas aporta metadata de columnas como si esas columnas hubieran sido declaradas manualmente en la entidad.

Para el MVP, eso significa:

- cada columna generada tiene `rust_field`, `column_name`, tipo SQL, nullability, defaults y flags `insertable`/`updatable`
- el orden de columnas en `EntityMetadata.columns` es estable: primero columnas propias de la entidad en orden de declaracion Rust, despues columnas aportadas por `AuditFields` en orden de declaracion Rust
- las columnas generadas participan en snapshots, diff y DDL sin rutas especiales
- las colisiones con campos propios fallan en compile-time con un mensaje que nombra la columna duplicada
- el MVP rechaza multiples declaraciones `audit`, evitando sobrescritura silenciosa de columnas generadas
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

Reglas iniciales:

- los campos de `Audit` se expanden despues de las columnas propias de la entidad, preservando el orden de declaracion del struct de auditoria
- los campos de `Audit` no forman parte de la primary key de la entidad
- los campos de `Audit` no crean foreign keys automaticamente
- los campos de `Audit` no generan filtros ni hooks runtime
- defaults SQL como `SYSUTCDATETIME()` son metadata de esquema, no valores calculados por Rust
- en este corte, las columnas auditables son columnas de metadata/schema; no generan campos Rust visibles dentro de la entidad ni simbolos asociados como `Todo::created_at`

### Simbolos de columna en el MVP

`#[derive(Entity)]` genera simbolos asociados `EntityColumn`, como `Todo::title`, solo para campos Rust declarados directamente en la entidad.

Las columnas aportadas por `#[orm(audit = Audit)]` no generan simbolos asociados en esta etapa. Aunque `created_at` aparezca en `EntityMetadata.columns`, `Todo::created_at` no existe si `created_at` viene del struct `AuditFields`.

Esta limitacion es intencional en el MVP: el macro de entidad solo recibe el path `audit = Audit` y no debe intentar duplicar o inferir los campos de otro derive para crear API de query. Exponer esos simbolos requiere un diseno posterior que preserve coherencia con `FromRow`, campos Rust visibles, autollenado y ergonomia del query builder.

Mientras tanto, las columnas auditables participan en snapshots, diff y DDL como metadata ordinaria, pero no forman parte del DSL tipado de columnas del entity.

### Materializacion con `FromRow`

`#[derive(Entity)]` sigue generando `FromRow` solo para los campos Rust declarados directamente en la entidad.

Si una entidad declara `#[orm(audit = Audit)]`, las columnas auditables expandidas en `EntityMetadata.columns` no se leen ni se asignan durante `FromRow` porque no existen campos Rust donde materializarlas. Una fila puede incluir columnas como `created_at` o `updated_by`, pero el entity resultante solo contiene sus campos propios.

Tambien es valido materializar una entidad auditada desde una fila que no traiga columnas auditables, siempre que incluya las columnas propias requeridas por el struct. Esto preserva el contrato MVP: auditoria como metadata/schema, no como estado Rust visible.

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

## Diseno futuro de `AuditProvider`

`AuditProvider` es el contrato runtime futuro para calcular valores auditables en operaciones de escritura. No forma parte del MVP implementado de Etapa 16 y no debe activar autollenado mientras no exista una tarea de implementacion dedicada.

La responsabilidad del provider debe ser producir valores para columnas auditables ya declaradas por `#[derive(AuditFields)]`. No debe crear columnas, modificar metadata ni participar en snapshots, diff o DDL. La metadata sigue viniendo de `audit = Audit`; el provider solo resuelve valores runtime para columnas existentes.

Shape conceptual esperado:

```rust
pub struct AuditContext<'a> {
    pub operation: AuditOperation,
    pub entity: &'static EntityMetadata,
    pub audit_columns: &'static [ColumnMetadata],
    pub request_values: &'a AuditRequestValues,
    pub transaction_id: Option<&'a str>,
}

pub enum AuditOperation {
    Insert,
    Update,
}

pub trait AuditProvider: Send + Sync + 'static {
    fn now(&self, ctx: &AuditContext<'_>) -> chrono::NaiveDateTime;
    fn current_user(&self, ctx: &AuditContext<'_>) -> Option<SqlValue>;
    fn value_for(&self, column: &ColumnMetadata, ctx: &AuditContext<'_>) -> Option<SqlValue>;
}
```

Este shape es deliberadamente conceptual. Antes de llevarlo a codigo se debe decidir si `now` debe usar `chrono::NaiveDateTime`, un tipo propio, o una abstraccion que permita preservar precision SQL Server `datetime2`. Tambien se debe decidir si `current_user` devuelve `SqlValue` directamente o un tipo dedicado que pueda mapearse con seguridad a columnas `String`, `i64`, `Uuid` u otros identificadores.

### Tiempo actual

`AuditProvider::now(...)` debe ser la unica fuente runtime de tiempo para autollenado desde Rust. Esto evita mezclar relojes de sistema, defaults SQL y valores ad hoc dentro de `Insertable`, `Changeset`, Active Record o change tracking.

Reglas esperadas:

- `created_at` se calcula en insert cuando la columna auditable existe y participa en insercion.
- `updated_at` se calcula en insert y update solo si la policy/columna lo permite.
- Si la columna usa `default_sql = "SYSUTCDATETIME()"` y no se autollena desde Rust, el valor puede quedar delegado a SQL Server.
- No se deben mezclar valores generados por Rust y defaults SQL para la misma columna sin una regla explicita de precedencia.

### Usuario actual y valores por request

El provider no debe leer usuario actual desde variables globales implicitas. El usuario, tenant, correlation id u otros valores por request deben llegar a traves de un contenedor explicito asociado al contexto:

```rust
pub struct AuditRequestValues {
    pub user_id: Option<SqlValue>,
    pub user_name: Option<String>,
    pub correlation_id: Option<String>,
}
```

La forma concreta puede cambiar, pero el principio no: un `DbContext` debe poder cargar valores por request de forma explicita y clonable, y esos valores deben viajar con contextos transaccionales derivados.

Reglas esperadas:

- Si una columna auditable requiere usuario y no hay usuario activo, la operacion debe fallar cerrado o dejar la columna sin autollenar segun una politica explicita.
- El provider debe poder mapear `created_by`/`updated_by` a columnas `String`, `i64`, `Uuid` u otros tipos soportados por `SqlTypeMapping`.
- Los valores por request no deben vivir en estado global compartido porque eso rompe concurrencia async y tests paralelos.

### Integracion con `DbContext`

La integracion publica esperada es configurar el provider y los valores de request en el contexto, no en cada `DbSet` por separado.

Forma conceptual:

```rust
let db = AppDb::connect(connection_string)
    .await?
    .with_audit_provider(provider)
    .with_audit_values(values);
```

El derive `DbContext` no debe duplicar reglas de auditoria por entidad. Debe propagar un handle compartido a los `DbSet<T>` y a las rutas que ya existen:

- `DbSet::insert(...)`
- `DbSet::update(...)`
- `entity.save(&db)`
- `save_changes()`

La integracion futura debe preservar que la API publica permanezca concentrada en `mssql-orm`; `core` solo deberia contener contratos neutrales si hacen falta, y `tiberius` no debe conocer `AuditProvider`.

### Acoplamiento con la implementacion actual

El diseno de `AuditProvider` debe engancharse en los puntos donde la crate publica ya centraliza persistencia, no en los derives ni en el adaptador Tiberius.

Puntos reales de integracion:

- `Insertable<E>::values()` sigue siendo responsable de extraer valores explicitos del payload de insercion.
- `Changeset<E>::changes()` sigue siendo responsable de extraer cambios explicitos del payload de update.
- `EntityPersist::insert_values()` y `EntityPersist::update_changes()` siguen siendo la fuente para Active Record y `save_changes()`.
- `DbSet::insert(...)` y `DbSet::update(...)` son los puntos publicos donde esos `Vec<ColumnValue>` se convierten en queries.
- Las rutas internas que ya existen para valores crudos (`insert_entity_values`, `update_entity_values_by_sql_value`, `insert_query_values`, `update_query_sql_value`, `RawInsertable` y `RawChangeset`) son el lugar natural para aplicar una transformacion comun antes de construir `InsertQuery` o `UpdateQuery`.

Por esa razon, una implementacion futura no deberia modificar los derives `Insertable`, `Changeset` ni `Entity` para inyectar valores auditables. Esos derives deben seguir siendo conversiones puras desde structs Rust hacia `ColumnValue`. El autollenado debe vivir en la capa publica de persistencia, donde ya convergen `DbSet`, Active Record y change tracking.

Tampoco debe agregarse logica a `mssql-orm-query`: esa crate solo recibe `InsertQuery` o `UpdateQuery` ya armados con valores. `mssql-orm-sqlserver` debe seguir compilando el AST sin saber si un valor vino del usuario o del provider. `mssql-orm-tiberius` debe seguir ejecutando queries parametrizadas sin interpretar metadata de auditoria.

### Mutacion de `Vec<ColumnValue>`

La implementacion futura debe introducir una unica transformacion interna en la crate publica, conceptualmente:

```rust
fn apply_audit_values<E: Entity>(
    operation: AuditOperation,
    values: Vec<ColumnValue>,
    audit_columns: &'static [ColumnMetadata],
    audit_provider: Option<&dyn AuditProvider>,
    request_values: &AuditRequestValues,
) -> Result<Vec<ColumnValue>, OrmError>;
```

El nombre puede cambiar, pero la responsabilidad no: recibe los valores explicitos ya producidos por `Insertable`, `Changeset` o `EntityPersist`, completa columnas auditables faltantes y devuelve otro `Vec<ColumnValue>` para el pipeline existente.

Reglas de integracion:

- `DbSet::insert(...)` debe obtener `insertable.values()`, aplicar la transformacion comun y delegar en `insert_entity_values(...)`.
- `DbSet::update(...)` debe obtener `changeset.changes()`, aplicar la transformacion comun y despues compilar mediante `update_query_sql_value(...)`.
- Active Record ya converge en `insert_entity(...)` y `update_entity_by_sql_value(...)`; esos metodos deben seguir delegando en `insert_entity_values(...)` y `update_entity_values_by_sql_value(...)`, donde se aplica la misma transformacion.
- `save_changes()` ya converge en `save_tracked_added(...)` y `save_tracked_modified(...)`, que llaman a `insert_entity(...)` y `update_entity_by_sql_value(...)`; no debe tener logica de auditoria propia.
- `RawInsertable` y `RawChangeset` deben seguir siendo adaptadores mecanicos para construir `InsertQuery`/`UpdateQuery` desde valores ya normalizados.

Reglas de precedencia:

- Los valores explicitos del usuario ganan por defecto. Si el payload ya contiene `created_at`, `created_by`, `updated_at` o `updated_by`, el provider no debe sobrescribirlos silenciosamente.
- El provider solo agrega una columna auditable cuando la columna existe en la policy, no esta ya presente en el `Vec<ColumnValue>` y la metadata permite escribirla para la operacion actual.
- En insert solo se consideran columnas con `insertable = true`.
- En update solo se consideran columnas con `updatable = true`.
- Una columna con `default_sql` puede omitirse si el provider no produce valor; entonces SQL Server conserva la responsabilidad del default.
- Si el provider declara que una columna es requerida y no puede producir valor, la operacion debe fallar antes de compilar SQL.
- Si el `Vec<ColumnValue>` de entrada contiene columnas duplicadas, la transformacion debe devolver `OrmError` en vez de elegir una de forma implicita.

Reglas por operacion:

- `AuditOperation::Insert` puede agregar columnas como `created_at`, `created_by`, `updated_at` y `updated_by` si son parte del struct `AuditFields` y sus flags de metadata permiten insercion.
- `AuditOperation::Update` no debe agregar `created_at` ni `created_by` salvo que el usuario haya definido explicitamente esas columnas como `updatable = true` y el provider haya decidido producirlas; la forma recomendada es limitar updates automaticos a `updated_at` y `updated_by`.
- La transformacion no debe crear predicados, filtros, `OUTPUT`, `rowversion` ni reglas de concurrencia. La concurrencia sigue viviendo en `Changeset::concurrency_token()`, `EntityPersist::concurrency_token()` y `update_query_sql_value(...)`.

La transformacion debe trabajar contra columnas auditables declaradas por la entidad, no contra nombres magicos globales. Como `EntityMetadata.columns` no conserva hoy el origen de cada columna, la implementacion runtime de `AuditProvider` necesitara exponer el slice de columnas auditables generado por `#[orm(audit = Audit)]` mediante un contrato auxiliar de runtime, por ejemplo un trait interno implementado por el derive de `Entity`. Ese contrato no debe cambiar el pipeline de snapshots, diff ni DDL: sigue siendo solo una forma de que `mssql-orm` sepa que subconjunto de `ColumnMetadata` puede autollenar.

No se debe inferir auditoria por convencion de nombres como `created_at` o `updated_by` sobre cualquier entidad. Una entidad solo participa en autollenado cuando declara `#[orm(audit = Audit)]` y el contexto tiene un `AuditProvider` configurado.

### Transacciones

Dentro de `db.transaction(...)`, el contexto transaccional debe heredar el mismo `AuditProvider` y los mismos valores por request que tenia el contexto padre al abrir la transaccion.

Reglas esperadas:

- Todas las operaciones dentro de una transaccion deben ver el mismo provider y los mismos valores por request.
- Si se desea un `now` estable por transaccion, debe modelarse de forma explicita, por ejemplo cacheando un instante en `AuditContext` o en un `TransactionAuditScope`.
- Si se desea un `now` por operacion, cada insert/update puede invocar `provider.now(...)` de forma independiente.
- La decision entre `now` estable por transaccion y `now` por operacion debe ser configurable o al menos documentada antes de implementar.
- `db.transaction(...)` sobre pool sigue bloqueado hasta resolver pinning de conexion; `AuditProvider` no debe relajar ese limite.

### Limites de esta tarea

Este diseno no implementa `AuditProvider`, no agrega autollenado runtime y no cambia los contratos publicos actuales. La tarea solo fija donde y como debe ocurrir la futura mutacion de `Vec<ColumnValue>` para que una implementacion posterior pueda ser pequena, verificable y sin duplicacion entre `DbSet`, Active Record y change tracking.

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

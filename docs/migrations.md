# Migraciones

Guia practica para trabajar bien con migraciones en el estado actual de `mssql-orm`.

Esta guia esta escrita contra la surface real disponible hoy:

- `migration add <Name>`
- `migration list`
- `database update`

Y deja explicito lo que la CLI hace y lo que todavia no hace.

## Resumen honesto

Hoy el flujo de migraciones del repo es `code-first` en intencion arquitectonica, pero la experiencia operativa publica todavia es mixta:

- el modelo y la metadata viven en Rust
- el workspace ya tiene snapshots, diff y DDL SQL Server en crates internas
- la CLI publica actual scaffolda migraciones y genera un script SQL acumulado
- el SQL de cada migracion sigue siendo editado manualmente en `up.sql`

Lo importante es no trabajar como si la CLI ya generara automaticamente cada migracion desde entidades. Hoy no es asi.

## 1. Flujo recomendado de trabajo

El flujo mas seguro hoy es este:

1. Cambia primero tus entidades y metadata Rust.
2. Crea una migracion con nombre pequeno y concreto.
3. Escribe y revisa `up.sql` manualmente.
4. Escribe `down.sql` aunque hoy no exista un comando publico que lo ejecute.
5. Genera el script acumulado de `database update`.
6. Revisalo antes de aplicarlo en SQL Server.
7. Aplícalo con una herramienta externa como `sqlcmd`.
8. Si ya se aplicó en una base compartida, no reescribas esa migracion: crea una nueva.

## 2. Crear una migracion

Mientras la CLI no este instalada globalmente, desde el repo puedes usar:

```bash
cargo run --manifest-path crates/mssql-orm-cli/Cargo.toml -- migration add CreateCustomers
```

Si ya tienes un snapshot actual del modelo generado por un consumidor o fixture, puedes pasarlo de forma explicita:

```bash
cargo run --manifest-path crates/mssql-orm-cli/Cargo.toml -- \
  migration add CreateCustomers --model-snapshot target/current_model_snapshot.json
```

Eso crea un directorio dentro de `./migrations/` con este shape:

```text
migrations/
  <timestamp>_create_customers/
    up.sql
    down.sql
    model_snapshot.json
```

Detalles operativos relevantes:

- el `id` incluye timestamp con resolucion de nanosegundos para reducir colisiones y preservar orden lexico
- el nombre visible se deriva del slug del directorio
- `up.sql` y `down.sql` nacen como plantillas vacias con comentario
- sin `--model-snapshot`, `model_snapshot.json` se scaffolda con un snapshot vacio valido
- con `--model-snapshot`, la CLI lee ese JSON y lo versiona como `model_snapshot.json` de la migracion
- la CLI actual todavia no carga automaticamente el tipo `DbContext` desde tu crate Rust; esa integracion queda como siguiente paso del pipeline code-first

## 3. Como nombrar bien una migracion

Usa nombres que describan un cambio pequeno y verificable:

- `CreateCustomers`
- `AddPhoneToCustomers`
- `RenameOrdersTotalToTotalCents`
- `AddTodoItemsCompletedByUserFk`

Evita nombres vagos como:

- `FixThings`
- `Changes`
- `UpdateSchema`

La regla practica es simple: una migracion deberia representar una intencion clara y revisable en `git diff`.

## 4. Que editar en `up.sql`

`up.sql` es la fuente de verdad operativa de lo que se aplicara en la base.

Ejemplo:

```sql
CREATE SCHEMA [sales];

CREATE TABLE [sales].[customers] (
    [id] BIGINT IDENTITY(1,1) NOT NULL,
    [email] NVARCHAR(180) NOT NULL,
    [phone] NVARCHAR(40) NULL,
    CONSTRAINT [pk_customers] PRIMARY KEY ([id])
);

CREATE UNIQUE INDEX [ix_customers_email]
ON [sales].[customers] ([email]);
```

Buenas practicas:

- escribe SQL Server explicito, no pseudo-SQL generico
- manten una migracion acotada; si el cambio crece demasiado, divide
- si el cambio afecta datos, deja la transformacion en la propia migracion de forma explicita
- revisa nombres de constraints e indices para que queden estables

## 5. Para que sirve `down.sql` hoy

Debes seguir escribiendolo, pero con una expectativa correcta:

- sirve como rollback documentado y material de revision
- hoy la CLI publica no expone un comando para ejecutar `down.sql`
- por eso no debes confiar en rollback automatico desde la herramienta

En otras palabras: `down.sql` sigue siendo valioso, pero hoy es soporte operativo manual, no flujo automatizado.

## 6. Listar migraciones locales

Para ver el orden efectivo de las migraciones locales:

```bash
cargo run --manifest-path crates/mssql-orm-cli/Cargo.toml -- migration list
```

La salida lista cada entrada con:

- `id`
- nombre derivado
- ruta del directorio

Usalo para confirmar orden antes de generar o aplicar el script acumulado.

## 7. Que hace realmente `database update`

Este punto es el mas importante para trabajar bien.

Hoy `database update` no se conecta a SQL Server ni ejecuta nada por si solo. Lo que hace es imprimir un script SQL acumulado a `stdout`.

Ejemplo:

```bash
cargo run --manifest-path crates/mssql-orm-cli/Cargo.toml -- database update > target/mssql-orm-database-update.sql
```

Luego tu aplicas ese script con una herramienta externa, por ejemplo:

```bash
sqlcmd -S localhost -U '<usuario>' -P '<password>' -d tempdb -C -b -i target/mssql-orm-database-update.sql
```

Eso obliga a una disciplina sana:

- generar script
- revisarlo
- aplicarlo conscientemente

## 8. Garantias actuales del script acumulado

El script generado hoy ya trae varias protecciones utiles:

- crea `dbo.__mssql_orm_migrations` si no existe
- emite `SET` de sesion requeridos por SQL Server para escenarios como indices sobre computed columns
- procesa cada migracion dentro de un bloque idempotente
- guarda `id`, nombre, checksum y version del ORM en la tabla de historial
- ejecuta cada migracion dentro de `BEGIN TRY / BEGIN TRANSACTION / COMMIT`
- hace `ROLLBACK` si falla una migracion
- falla con `THROW 50001` si encuentra el mismo `id` aplicado con contenido distinto

La implicacion practica es muy importante: no edites una migracion ya aplicada esperando que el sistema la "reaplique". El checksum lo tratara como drift y el script fallara a proposito.

## 9. Como trabajar bien en equipo

Reglas recomendadas para no romper historial:

- no reescribas `up.sql` de una migracion ya aplicada por otros
- crea una migracion nueva para el ajuste siguiente
- revisa en PR tanto el cambio en entidades Rust como el SQL de la migracion
- si una migracion cambia columnas computadas, foreign keys o renames, valida el SQL con mas cuidado
- prueba primero en una base desechable como `tempdb`

## 10. Limitaciones explicitas actuales

Conviene asumir estos limites hoy:

- la CLI actual no genera automaticamente `up.sql` desde tus entidades
- la CLI actual no aplica el script directamente a SQL Server; solo lo imprime
- la CLI actual no expone `database downgrade`
- `down.sql` no se consume automaticamente
- `model_snapshot.json` se scaffolda, pero no se mantiene solo
- la separacion por sentencias del `up.sql` es deliberadamente simple; conviene escribir migraciones SQL Server limpias y bien delimitadas

## 11. Flujo recomendado para cambios reales

Para cambios pequenos y seguros:

1. Cambia la entidad Rust.
2. Crea migracion: `migration add AddPhoneToCustomers`.
3. Escribe `up.sql` y `down.sql`.
4. Genera script con `database update`.
5. Revisa el SQL generado.
6. Aplícalo en `tempdb`.
7. Verifica tablas, columnas, indices o foreign keys con `sqlcmd`.
8. Solo despues lo promueves a un entorno compartido.

Para cambios delicados:

- si hay renombres, prefiere operaciones explicitas y revisables
- si hay cambios destructivos, no los escondas dentro de una migracion grande
- si el cambio mezcla schema y data migration, deja ambas partes claramente separadas dentro del SQL

## Referencias relacionadas

- Guia `code-first`: [docs/code-first.md](code-first.md)
- Quickstart: [docs/quickstart.md](quickstart.md)
- Plan maestro: [docs/plan_orm_sqlserver_tiberius_code_first.md](plan_orm_sqlserver_tiberius_code_first.md)

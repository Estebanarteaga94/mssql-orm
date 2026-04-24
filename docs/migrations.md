# Migraciones

Guia practica para trabajar bien con migraciones en el estado actual de `mssql-orm`.

Esta guia esta escrita contra la surface real disponible hoy:

- `migration add <Name>`
- `migration list`
- `database update`
- `database update --execute`

Y deja explicito lo que la CLI hace y lo que todavia no hace.

## Resumen honesto

Hoy el flujo de migraciones del repo es `code-first` en intencion arquitectonica, pero la experiencia operativa publica todavia es mixta:

- el modelo y la metadata viven en Rust
- el workspace ya tiene snapshots, diff y DDL SQL Server en crates internas
- la CLI publica actual scaffolda migraciones, genera un script SQL acumulado y puede aplicarlo explicitamente con `--execute`
- `up.sql` se autogenera cuando entregas un snapshot actual real; sigue siendo un artefacto revisable y editable antes de aplicar la migracion

Lo importante es no trabajar como si la CLI ya resolviera todo el ciclo de vida por ti: aun debes exportar el snapshot actual, revisar el SQL generado y aplicar el script conscientemente.

## 1. Flujo recomendado de trabajo

El flujo mas seguro hoy es este:

1. Cambia primero tus entidades y metadata Rust.
2. Crea una migracion con nombre pequeno y concreto.
3. Genera y revisa `up.sql`; editalo si el cambio requiere ajustes manuales.
4. Escribe `down.sql` aunque hoy no exista un comando publico que lo ejecute.
5. Genera el script acumulado de `database update`.
6. Revisalo antes de aplicarlo en SQL Server.
7. Aplicalo explicitamente con `database update --execute` o con una herramienta externa como `sqlcmd`.
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

Tambien puedes hacer que la CLI lo exporte invocando un binario del proyecto consumidor:

```bash
cargo run --manifest-path crates/mssql-orm-cli/Cargo.toml -- \
  migration add CreateCustomers \
  --snapshot-bin app-model-snapshot \
  --manifest-path examples/todo-app/Cargo.toml
```

Ese binario debe imprimir a `stdout` el JSON del snapshot actual. La crate publica ya expone helpers para eso:

```rust
use mssql_orm::prelude::*;

#[derive(DbContext, Debug, Clone)]
struct AppDbContext {
    users: DbSet<User>,
}

fn main() {
    print!(
        "{}",
        mssql_orm::model_snapshot_json_from_source::<AppDbContext>().unwrap()
    );
}
```

Eso crea un directorio dentro de `./migrations/` con este shape:

```text
migrations/
  <timestamp>_create_customers/
    up.sql
    down.sql
    model_snapshot.json
```

Este es el artefacto editable MVP de migracion. El plan maestro menciona tambien un `migration.rs`, pero queda diferido explicitamente hasta disenar una API Rust de migraciones que no duplique ni contradiga el pipeline actual de snapshots, diff y DDL SQL Server.

Detalles operativos relevantes:

- el `id` incluye timestamp con resolucion de nanosegundos para reducir colisiones y preservar orden lexico
- el nombre visible se deriva del slug del directorio
- la salida de `migration add` lista las rutas de `up.sql`, `down.sql` y `model_snapshot.json`, y marca `migration.rs` como diferido para el MVP
- `up.sql` se genera automaticamente cuando `migration add` dispone de snapshot actual real; `down.sql` nace como plantilla manual de rollback
- sin `--model-snapshot`, `model_snapshot.json` se scaffolda con un snapshot vacio valido
- con `--model-snapshot`, la CLI lee ese JSON y lo versiona como `model_snapshot.json` de la migracion
- con `--snapshot-bin`, la CLI ejecuta `cargo run --bin <bin>` sobre el manifest indicado, captura `stdout` y valida que sea un `ModelSnapshot` JSON valido
- el exportador del consumidor sigue siendo explicito: la CLI no resuelve sola el nombre del `DbContext`, sino que delega esa seleccion al binario que uses para exportar el snapshot
- cuando existe una migracion local previa y `migration add` recibe un snapshot actual real, la CLI carga el `model_snapshot.json` de la ultima migracion y reporta ambos lados (`Previous snapshot` y `Current snapshot`) como base del siguiente paso de diff
- en ese mismo caso, la CLI ejecuta internamente `snapshot -> diff -> MigrationOperation -> DDL SQL Server`, reporta `Planned operations` y `Compiled SQL statements`, y escribe ese SQL compilado en `up.sql`
- si el diff detecta un cambio destructivo, `migration add` falla por defecto antes de crear la nueva migracion; usa `--allow-destructive` solo cuando hayas revisado el impacto y quieras generar igualmente el artefacto editable

Actualmente se consideran destructivos estos cambios:

- `DropTable`
- `DropColumn`
- reduccion de longitud de columna
- cambio de tipo de columna
- conversion de nullable a non-nullable sin `default_sql`

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
Ahora puede nacer autogenerado por la CLI, pero sigue siendo un artefacto revisable y editable antes de aplicar la migracion.

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

Por defecto, `database update` no se conecta a SQL Server ni ejecuta nada. Imprime un script SQL acumulado a `stdout`.

Ejemplo:

```bash
cargo run --manifest-path crates/mssql-orm-cli/Cargo.toml -- database update > target/mssql-orm-database-update.sql
```

Ese modo sigue siendo el recomendado cuando quieres revisar o archivar el SQL antes de aplicarlo. Luego puedes aplicar ese script con una herramienta externa, por ejemplo:

```bash
sqlcmd -S localhost -U '<usuario>' -P '<password>' -d tempdb -C -b -i target/mssql-orm-database-update.sql
```

Eso obliga a una disciplina sana:

- generar script
- revisarlo
- aplicarlo conscientemente

Si quieres aplicar desde la CLI del proyecto, usa `--execute`:

```bash
DATABASE_URL='Server=localhost;Database=tempdb;User Id=<usuario>;Password=<password>;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;' \
  cargo run --manifest-path crates/mssql-orm-cli/Cargo.toml -- database update --execute
```

Tambien puedes pasar la conexion de forma explicita:

```bash
cargo run --manifest-path crates/mssql-orm-cli/Cargo.toml -- \
  database update --execute --connection-string '<connection-string>'
```

`--execute` reutiliza el mismo script acumulado de `database update` y lo ejecuta mediante `mssql-orm-tiberius`. La cadena de conexion se resuelve en este orden: `--connection-string`, `DATABASE_URL`, `MSSQL_ORM_TEST_CONNECTION_STRING`.

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

## 9. Flujo rapido con `examples/todo-app`

Si borraste `examples/todo-app/migrations/` y quieres recrear la migracion inicial desde el modelo actual del ejemplo, usa este flujo desde la raiz del repositorio:

```bash
cargo build -p mssql-orm-cli
```

```bash
cd examples/todo-app
```

```bash
../../target/debug/mssql-orm-cli migration add CreateTodoSchema \
  --snapshot-bin model_snapshot \
  --manifest-path Cargo.toml
```

Para revisar el orden local:

```bash
../../target/debug/mssql-orm-cli migration list
```

Para inspeccionar el SQL acumulado sin ejecutar:

```bash
../../target/debug/mssql-orm-cli database update
```

Para aplicar contra SQL Server desde la CLI:

```bash
DATABASE_URL='Server=localhost;Database=tempdb;User Id=<usuario>;Password=<password>;TrustServerCertificate=True;Encrypt=False;Connection Timeout=30;MultipleActiveResultSets=true;' \
  ../../target/debug/mssql-orm-cli database update --execute
```

Puedes repetir el ultimo comando para validar que el historial `dbo.__mssql_orm_migrations` evita reaplicar migraciones ya registradas.

Importante: la migracion inicial espera una base limpia para los objetos que va a crear, o un historial ya alineado. Si existen tablas como `todo.todo_items` pero no existe la fila correspondiente en `dbo.__mssql_orm_migrations`, SQL Server rechazara el `CREATE TABLE` por objeto existente.

## 10. Como trabajar bien en equipo

Reglas recomendadas para no romper historial:

- no reescribas `up.sql` de una migracion ya aplicada por otros
- crea una migracion nueva para el ajuste siguiente
- revisa en PR tanto el cambio en entidades Rust como el SQL de la migracion
- si una migracion cambia columnas computadas, foreign keys o renames, valida el SQL con mas cuidado
- prueba primero en una base desechable como `tempdb`

## 11. Limitaciones explicitas actuales

Conviene asumir estos limites hoy:

- la CLI actual ya genera `up.sql` desde el diff del snapshot actual contra la ultima migracion local cuando le proporcionas un snapshot actual real
- la CLI bloquea por defecto cambios destructivos detectados en `migration add`; el bypass explicito es `--allow-destructive`
- por defecto, `database update` solo imprime el script; la ejecucion directa requiere `database update --execute`
- `database update --execute` no toma la conexion desde el `DbContext`; usa `--connection-string`, `DATABASE_URL` o `MSSQL_ORM_TEST_CONNECTION_STRING`
- no hay baselining automatico para objetos existentes sin historial de migracion
- la CLI actual no expone `database downgrade`
- `down.sql` no se consume automaticamente
- `down.sql` no se genera como inversion completa del diff; queda como rollback manual revisable hasta que las operaciones conserven payload suficiente para invertir cambios de forma segura
- `migration.rs` queda fuera del MVP actual y no se genera
- `model_snapshot.json` se scaffolda, pero no se mantiene solo
- la separacion por sentencias del `up.sql` es deliberadamente simple; conviene escribir migraciones SQL Server limpias y bien delimitadas

## 12. Flujo recomendado para cambios reales

Para cambios pequenos y seguros:

1. Cambia la entidad Rust.
2. Crea migracion: `migration add AddPhoneToCustomers`.
3. Revisa `up.sql` generado o escribelo manualmente si no usaste snapshot actual real; completa `down.sql`.
4. Genera script con `database update`.
5. Revisa el SQL generado.
6. Aplícalo en `tempdb`.
7. Verifica tablas, columnas, indices o foreign keys con `sqlcmd`.
8. Solo despues lo promueves a un entorno compartido.

Para cambios delicados:

- si hay renombres, prefiere operaciones explicitas y revisables
- si hay cambios destructivos, no los escondas dentro de una migracion grande; usa `--allow-destructive` solo para generar el artefacto tras revisar el riesgo
- si el cambio mezcla schema y data migration, deja ambas partes claramente separadas dentro del SQL

## Referencias relacionadas

- Guia `code-first`: [docs/code-first.md](code-first.md)
- Quickstart: [docs/quickstart.md](quickstart.md)
- Plan maestro: [docs/plan_orm_sqlserver_tiberius_code_first.md](plan_orm_sqlserver_tiberius_code_first.md)

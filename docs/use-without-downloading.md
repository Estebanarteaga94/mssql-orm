# Use Without Manually Downloading the Repository

This guide explains how to try `mssql-orm` from another Rust project without
manually cloning this repository into your workspace.

Current status: the repository is usable as a Git dependency. A crates.io
release is not documented in this repository yet.

## What You Still Need

You do not need to download this repository manually, but you still need:

- a Rust project;
- access to SQL Server;
- a connection string for your database;
- network access to the Git repository if using a Git dependency.

## Add the ORM as a Git Dependency

In your application's `Cargo.toml`, depend on the public crate package from the
repository:

```toml
[dependencies]
mssql-orm = { git = "https://github.com/<owner>/mssql-orm", package = "mssql-orm" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

Replace `<owner>` with the actual repository owner or use the Git URL provided
by the maintainer.

For reproducible builds, pin a revision:

```toml
[dependencies]
mssql-orm = { git = "https://github.com/<owner>/mssql-orm", package = "mssql-orm", rev = "<commit-sha>" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

If you need the optional pool integration:

```toml
[dependencies]
mssql-orm = { git = "https://github.com/<owner>/mssql-orm", package = "mssql-orm", features = ["pool-bb8"] }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Import the Public API

The Rust package name is `mssql-orm`, but the crate is imported as
`mssql_orm`:

```rust
use mssql_orm::prelude::*;
```

## Minimal Example

Create a table in SQL Server:

```sql
CREATE TABLE dbo.demo_users (
    id BIGINT IDENTITY(1,1) NOT NULL PRIMARY KEY,
    name NVARCHAR(120) NOT NULL,
    active BIT NOT NULL
);
```

Then use the ORM from your application:

```rust
use mssql_orm::prelude::*;

#[derive(Entity, Debug, Clone, PartialEq)]
#[orm(table = "demo_users", schema = "dbo")]
struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(length = 120)]
    name: String,

    active: bool,
}

#[derive(Insertable, Debug, Clone)]
#[orm(entity = User)]
struct NewUser {
    name: String,
    active: bool,
}

#[derive(DbContext, Debug, Clone)]
struct AppDb {
    pub users: DbSet<User>,
}

#[tokio::main]
async fn main() -> Result<(), OrmError> {
    let connection_string = std::env::var("DATABASE_URL")
        .map_err(|_| OrmError::new("DATABASE_URL is required"))?;

    let db = AppDb::connect(&connection_string).await?;

    let user = db
        .users
        .insert(NewUser {
            name: "Ana".to_string(),
            active: true,
        })
        .await?;

    let active_users = db
        .users
        .query()
        .filter(User::active.eq(true))
        .order_by(User::id.asc())
        .all()
        .await?;

    println!("inserted user id: {}", user.id);
    println!("active users: {}", active_users.len());

    Ok(())
}
```

Run it with your own SQL Server connection string:

```bash
DATABASE_URL='Server=localhost;Database=tempdb;User Id=SA;Password=<password>;TrustServerCertificate=True;Encrypt=False;' cargo run
```

Do not commit real credentials.

## Using the CLI Without Cloning

The CLI package can also be installed from Git when needed:

```bash
cargo install --git https://github.com/<owner>/mssql-orm mssql-orm-cli
```

Pin a revision for reproducibility:

```bash
cargo install --git https://github.com/<owner>/mssql-orm --rev <commit-sha> mssql-orm-cli
```

CLI workflows may require additional project setup, especially for migration
snapshot export. See `docs/migrations.md` before using migration commands in a
real application.

## When to Use a Local Checkout Instead

Use a local clone when you want to:

- contribute to `mssql-orm`;
- run the full workspace test suite;
- inspect internal crates;
- change macros, SQL compilation, migrations or the Tiberius adapter;
- run examples from this repository.

For normal application usage, a pinned Git dependency is enough.

## Known Limits

- SQL Server is the only supported database.
- Some APIs are experimental, especially change tracking.
- Raw SQL does not automatically apply ORM-level `tenant` or `soft_delete`
  filters.
- Advanced query features such as table aliases and high-level aggregation APIs
  are still limited.
- If a behavior is unclear in the current code, treat it as `Pending
  verification` rather than relying on it in production.

## Related Documents

- Core concepts: [core-concepts.md](core-concepts.md)
- Quickstart: [quickstart.md](quickstart.md)
- Public API guide: [api.md](api.md)
- Migrations guide: [migrations.md](migrations.md)
- Repository audit: [repository-audit.md](repository-audit.md)

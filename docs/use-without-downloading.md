# Use Without Manually Downloading the Repository

You can consume `mssql-orm` from another Rust project through a Git dependency. You do not need to manually clone this repository into the consuming project.

## Git Dependency

```toml
[dependencies]
mssql-orm = { git = "https://github.com/<owner>/<repo>.git", package = "mssql-orm" }
```

To pin a branch, tag, or revision:

```toml
[dependencies]
mssql-orm = { git = "https://github.com/<owner>/<repo>.git", package = "mssql-orm", branch = "main" }
```

```toml
[dependencies]
mssql-orm = { git = "https://github.com/<owner>/<repo>.git", package = "mssql-orm", rev = "<commit-sha>" }
```

## Optional Pooling

```toml
[dependencies]
mssql-orm = { git = "https://github.com/<owner>/<repo>.git", package = "mssql-orm", features = ["pool-bb8"] }
```

## Basic Consumer Code

```rust
use mssql_orm::prelude::*;

#[derive(Entity, Debug, Clone)]
#[orm(table = "users", schema = "dbo")]
struct User {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,

    #[orm(length = 180)]
    email: String,
}

#[derive(DbContext)]
struct AppDb {
    users: DbSet<User>,
}
```

## Migration Snapshot Export

If your project wants to use `migration add --snapshot-bin`, add a small binary that prints the model snapshot:

```rust
use mssql_orm::prelude::*;

fn main() {
    print!(
        "{}",
        mssql_orm::model_snapshot_json_from_source::<AppDb>()
            .expect("snapshot should serialize")
    );
}
```

Then call the CLI with the consumer manifest:

```bash
mssql-orm-cli migration add CreateSchema \
  --manifest-path path/to/consumer/Cargo.toml \
  --snapshot-bin model_snapshot
```

## Notes

- Cargo downloads Git dependencies into its own cache.
- Prefer pinning a revision for reproducible builds.
- Do not commit connection strings or credentials.
- The root public API is `mssql_orm::prelude::*`.

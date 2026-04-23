use mssql_orm_core::OrmError;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const MIGRATIONS_DIR: &str = "migrations";
const ORM_VERSION: &str = env!("CARGO_PKG_VERSION");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationScaffold {
    pub id: String,
    pub name: String,
    pub directory: PathBuf,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MigrationEntry {
    pub id: String,
    pub name: String,
    pub directory: PathBuf,
    pub up_sql_path: PathBuf,
    pub down_sql_path: PathBuf,
    pub snapshot_path: PathBuf,
}

pub fn create_migration_scaffold(root: &Path, name: &str) -> Result<MigrationScaffold, OrmError> {
    if name.trim().is_empty() {
        return Err(OrmError::new("migration name cannot be empty"));
    }

    let slug = slugify(name);
    let timestamp = migration_timestamp()?;
    let id = format!("{timestamp}_{slug}");
    let migrations_dir = root.join(MIGRATIONS_DIR);
    let directory = migrations_dir.join(&id);

    fs::create_dir_all(&directory)
        .map_err(|_| OrmError::new("failed to create migration directory"))?;
    fs::write(
        directory.join("up.sql"),
        format!("-- Migration: {id}\n-- Write SQL Server DDL here.\n"),
    )
    .map_err(|_| OrmError::new("failed to write migration up.sql"))?;
    fs::write(
        directory.join("down.sql"),
        format!("-- Migration: {id}\n-- Write rollback SQL here.\n"),
    )
    .map_err(|_| OrmError::new("failed to write migration down.sql"))?;
    fs::write(
        directory.join("model_snapshot.json"),
        "{\n  \"schemas\": []\n}\n",
    )
    .map_err(|_| OrmError::new("failed to write migration model snapshot"))?;

    Ok(MigrationScaffold {
        id,
        name: name.to_string(),
        directory,
    })
}

pub fn list_migrations(root: &Path) -> Result<Vec<MigrationEntry>, OrmError> {
    let migrations_dir = root.join(MIGRATIONS_DIR);
    if !migrations_dir.exists() {
        return Ok(Vec::new());
    }

    let mut entries = fs::read_dir(&migrations_dir)
        .map_err(|_| OrmError::new("failed to read migrations directory"))?
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false))
        .filter_map(|entry| parse_migration_entry(entry.path()))
        .collect::<Vec<_>>();

    entries.sort_by(|left, right| left.id.cmp(&right.id));
    Ok(entries)
}

pub fn build_database_update_script(
    root: &Path,
    history_table_sql: &str,
) -> Result<String, OrmError> {
    let migrations = list_migrations(root)?;
    let mut script = vec![
        "-- mssql-orm database update".to_string(),
        history_table_sql.to_string(),
    ];

    for migration in migrations {
        let up_sql = fs::read_to_string(&migration.up_sql_path)
            .map_err(|_| OrmError::new("failed to read migration up.sql"))?;
        let checksum = checksum_hex(up_sql.as_bytes());
        script.push(format!(
            "IF NOT EXISTS (SELECT 1 FROM [dbo].[__mssql_orm_migrations] WHERE [id] = N'{id}')\nBEGIN\n{body}\n    INSERT INTO [dbo].[__mssql_orm_migrations] ([id], [name], [checksum], [orm_version]) VALUES (N'{id}', N'{name}', N'{checksum}', N'{version}');\nEND",
            id = migration.id,
            name = migration.name,
            checksum = checksum,
            version = ORM_VERSION,
            body = indent_sql_block(&up_sql),
        ));
    }

    Ok(script.join("\n\n"))
}

fn parse_migration_entry(path: PathBuf) -> Option<MigrationEntry> {
    let file_name = path.file_name()?.to_str()?;
    let (timestamp, slug) = file_name.split_once('_')?;
    if timestamp.is_empty() || slug.is_empty() {
        return None;
    }

    Some(MigrationEntry {
        id: file_name.to_string(),
        name: slug.replace('_', " "),
        up_sql_path: path.join("up.sql"),
        down_sql_path: path.join("down.sql"),
        snapshot_path: path.join("model_snapshot.json"),
        directory: path,
    })
}

fn migration_timestamp() -> Result<String, OrmError> {
    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|_| OrmError::new("system clock is before UNIX_EPOCH"))?;
    Ok(duration.as_secs().to_string())
}

fn slugify(name: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = false;

    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            slug.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if !previous_was_separator {
            slug.push('_');
            previous_was_separator = true;
        }
    }

    slug.trim_matches('_').to_string()
}

fn checksum_hex(bytes: &[u8]) -> String {
    let mut hash = 0xcbf29ce484222325u64;
    for byte in bytes {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x100000001b3);
    }

    format!("{hash:016x}")
}

fn indent_sql_block(sql: &str) -> String {
    sql.lines()
        .map(|line| {
            if line.trim().is_empty() {
                "    ".to_string()
            } else {
                format!("    {line}")
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[cfg(test)]
mod tests {
    use super::{build_database_update_script, create_migration_scaffold, list_migrations};
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_project_root() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mssql_orm_migrate_{unique}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn creates_scaffolded_migration_files() {
        let root = temp_project_root();

        let scaffold = create_migration_scaffold(&root, "Create Customers").unwrap();

        assert!(scaffold.id.contains("create_customers"));
        assert!(scaffold.directory.join("up.sql").exists());
        assert!(scaffold.directory.join("down.sql").exists());
        assert!(scaffold.directory.join("model_snapshot.json").exists());
    }

    #[test]
    fn lists_migrations_in_sorted_order() {
        let root = temp_project_root();
        let migrations_dir = root.join("migrations");
        fs::create_dir_all(migrations_dir.join("200_create_orders")).unwrap();
        fs::create_dir_all(migrations_dir.join("100_create_customers")).unwrap();

        let migrations = list_migrations(&root).unwrap();

        assert_eq!(migrations.len(), 2);
        assert_eq!(migrations[0].id, "100_create_customers");
        assert_eq!(migrations[1].id, "200_create_orders");
    }

    #[test]
    fn builds_database_update_script_with_history_inserts() {
        let root = temp_project_root();
        let scaffold = create_migration_scaffold(&root, "Create Customers").unwrap();
        fs::write(
            scaffold.directory.join("up.sql"),
            "CREATE TABLE [sales].[customers] ([id] bigint NOT NULL);",
        )
        .unwrap();

        let script = build_database_update_script(
            &root,
            "CREATE TABLE [dbo].[__mssql_orm_migrations] (...);",
        )
        .unwrap();

        assert!(script.contains("CREATE TABLE [dbo].[__mssql_orm_migrations]"));
        assert!(script.contains("IF NOT EXISTS (SELECT 1 FROM [dbo].[__mssql_orm_migrations]"));
        assert!(script.contains("CREATE TABLE [sales].[customers]"));
        assert!(script.contains("INSERT INTO [dbo].[__mssql_orm_migrations]"));
    }
}

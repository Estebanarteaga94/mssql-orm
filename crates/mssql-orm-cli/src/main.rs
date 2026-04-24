use mssql_orm_migrate::{
    ModelSnapshot, build_database_update_script, create_migration_scaffold,
    create_migration_scaffold_with_snapshot, list_migrations, read_latest_model_snapshot,
    read_model_snapshot,
};
use mssql_orm_sqlserver::SqlServerCompiler;
use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    match run(env::args().collect(), Path::new(".")) {
        Ok(output) => println!("{output}"),
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(1);
        }
    }
}

fn run(args: Vec<String>, root: &Path) -> Result<String, String> {
    match parse_command(&args)? {
        CliCommand::MigrationAdd { name, options } => {
            let current_snapshot = load_current_model_snapshot(root, &options)?;
            let previous_snapshot = match current_snapshot {
                Some(_) => load_previous_model_snapshot(root)?,
                None => None,
            };
            let scaffold = match current_snapshot.as_ref() {
                Some(snapshot) => create_migration_scaffold_with_snapshot(root, &name, snapshot),
                None => create_migration_scaffold(root, &name),
            }
            .map_err(|error| error.to_string())?;

            let mut output = format!(
                "Created migration {}\nPath: {}",
                scaffold.id,
                scaffold.directory.display()
            );

            if let Some((migration, previous_snapshot)) = previous_snapshot {
                output.push_str(&format!(
                    "\nPrevious snapshot: {} (schemas: {})",
                    migration.id,
                    previous_snapshot.schemas.len()
                ));
            } else if current_snapshot.is_some() {
                output.push_str("\nPrevious snapshot: none");
            }

            if let Some(current_snapshot) = current_snapshot {
                output.push_str(&format!(
                    "\nCurrent snapshot: schemas={} tables={}",
                    current_snapshot.schemas.len(),
                    current_snapshot
                        .schemas
                        .iter()
                        .map(|schema| schema.tables.len())
                        .sum::<usize>()
                ));
            }

            Ok(output)
        }
        CliCommand::MigrationList => {
            let migrations = list_migrations(root).map_err(|error| error.to_string())?;
            if migrations.is_empty() {
                return Ok("No migrations found.".to_string());
            }

            Ok(migrations
                .iter()
                .map(|migration| {
                    format!(
                        "{} | {} | {}",
                        migration.id,
                        migration.name,
                        migration.directory.display()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n"))
        }
        CliCommand::DatabaseUpdate => {
            let history_table_sql = SqlServerCompiler::compile_migrations_history_table()
                .map_err(|error| error.to_string())?;
            build_database_update_script(root, &history_table_sql)
                .map_err(|error| error.to_string())
        }
    }
}

fn load_current_model_snapshot(
    root: &Path,
    options: &MigrationAddOptions,
) -> Result<Option<ModelSnapshot>, String> {
    if let Some(path) = &options.model_snapshot {
        let snapshot_path = resolve_project_path(root, path);
        let snapshot = read_model_snapshot(&snapshot_path).map_err(|error| {
            format!(
                "failed to load current model snapshot from {}: {error}",
                snapshot_path.display()
            )
        })?;
        return Ok(Some(snapshot));
    }

    if let Some(snapshot_bin) = &options.snapshot_bin {
        let manifest_path = options
            .manifest_path
            .as_ref()
            .map(|path| resolve_project_path(root, path));
        let output = run_snapshot_exporter(snapshot_bin, manifest_path.as_deref())?;
        let snapshot = ModelSnapshot::from_json(&output)
            .map_err(|error| format!("failed to deserialize snapshot exporter output: {error}"))?;
        return Ok(Some(snapshot));
    }

    Ok(None)
}

fn load_previous_model_snapshot(
    root: &Path,
) -> Result<Option<(mssql_orm_migrate::MigrationEntry, ModelSnapshot)>, String> {
    read_latest_model_snapshot(root)
        .map_err(|error| format!("failed to load previous model snapshot: {error}"))
}

fn run_snapshot_exporter(
    snapshot_bin: &str,
    manifest_path: Option<&Path>,
) -> Result<String, String> {
    let mut command = Command::new("cargo");
    command.arg("run").arg("--quiet");

    if let Some(manifest_path) = manifest_path {
        command.arg("--manifest-path").arg(manifest_path);
    }

    command.arg("--bin").arg(snapshot_bin);

    let output = command
        .output()
        .map_err(|error| format!("failed to execute snapshot exporter binary: {error}"))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stderr = stderr.trim();
        return Err(if stderr.is_empty() {
            format!("snapshot exporter binary `{snapshot_bin}` failed")
        } else {
            format!("snapshot exporter binary `{snapshot_bin}` failed: {stderr}")
        });
    }

    String::from_utf8(output.stdout)
        .map_err(|_| "snapshot exporter emitted non-utf8 output".to_string())
}

fn resolve_project_path(root: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        root.join(path)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CliCommand {
    MigrationAdd {
        name: String,
        options: MigrationAddOptions,
    },
    MigrationList,
    DatabaseUpdate,
}

fn parse_command(args: &[String]) -> Result<CliCommand, String> {
    match args {
        [_bin, group, action, name, rest @ ..] if group == "migration" && action == "add" => {
            Ok(CliCommand::MigrationAdd {
                name: name.clone(),
                options: parse_migration_add_options(rest)?,
            })
        }
        [_bin, group, action] if group == "migration" && action == "list" => {
            Ok(CliCommand::MigrationList)
        }
        [_bin, group, action] if group == "database" && action == "update" => {
            Ok(CliCommand::DatabaseUpdate)
        }
        _ => Err(
            "Usage:\n  mssql-orm-cli migration add <Name> [--model-snapshot <Path>] [--snapshot-bin <BinName> [--manifest-path <Path>]]\n  mssql-orm-cli migration list\n  mssql-orm-cli database update".to_string(),
        ),
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct MigrationAddOptions {
    model_snapshot: Option<PathBuf>,
    snapshot_bin: Option<String>,
    manifest_path: Option<PathBuf>,
}

fn parse_migration_add_options(args: &[String]) -> Result<MigrationAddOptions, String> {
    let mut options = MigrationAddOptions::default();
    let mut index = 0;

    while index < args.len() {
        match args[index].as_str() {
            "--model-snapshot" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--model-snapshot requires a path".to_string())?;
                options.model_snapshot = Some(PathBuf::from(value));
                index += 2;
            }
            "--snapshot-bin" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--snapshot-bin requires a binary name".to_string())?;
                options.snapshot_bin = Some(value.clone());
                index += 2;
            }
            "--manifest-path" => {
                let value = args
                    .get(index + 1)
                    .ok_or_else(|| "--manifest-path requires a path".to_string())?;
                options.manifest_path = Some(PathBuf::from(value));
                index += 2;
            }
            unknown => {
                return Err(format!("unknown migration add option: {unknown}"));
            }
        }
    }

    if options.model_snapshot.is_some() && options.snapshot_bin.is_some() {
        return Err("--model-snapshot and --snapshot-bin cannot be used together".to_string());
    }

    if options.snapshot_bin.is_none() && options.manifest_path.is_some() {
        return Err("--manifest-path requires --snapshot-bin".to_string());
    }

    Ok(options)
}

#[cfg(test)]
mod tests {
    use super::{CliCommand, MigrationAddOptions, parse_command, run};
    use mssql_orm_migrate::read_model_snapshot;
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_project_root() -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("mssql_orm_cli_{unique}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    #[test]
    fn parses_minimal_cli_commands() {
        assert_eq!(
            parse_command(&[
                "mssql-orm-cli".to_string(),
                "migration".to_string(),
                "add".to_string(),
                "CreateCustomers".to_string(),
            ])
            .unwrap(),
            CliCommand::MigrationAdd {
                name: "CreateCustomers".to_string(),
                options: MigrationAddOptions::default()
            }
        );
        assert_eq!(
            parse_command(&[
                "mssql-orm-cli".to_string(),
                "migration".to_string(),
                "add".to_string(),
                "CreateCustomers".to_string(),
                "--model-snapshot".to_string(),
                "target/current_model_snapshot.json".to_string(),
            ])
            .unwrap(),
            CliCommand::MigrationAdd {
                name: "CreateCustomers".to_string(),
                options: MigrationAddOptions {
                    model_snapshot: Some(PathBuf::from("target/current_model_snapshot.json")),
                    snapshot_bin: None,
                    manifest_path: None,
                }
            }
        );
        assert_eq!(
            parse_command(&[
                "mssql-orm-cli".to_string(),
                "migration".to_string(),
                "add".to_string(),
                "CreateCustomers".to_string(),
                "--snapshot-bin".to_string(),
                "app-model-snapshot".to_string(),
                "--manifest-path".to_string(),
                "examples/todo-app/Cargo.toml".to_string(),
            ])
            .unwrap(),
            CliCommand::MigrationAdd {
                name: "CreateCustomers".to_string(),
                options: MigrationAddOptions {
                    model_snapshot: None,
                    snapshot_bin: Some("app-model-snapshot".to_string()),
                    manifest_path: Some(PathBuf::from("examples/todo-app/Cargo.toml")),
                }
            }
        );
        assert_eq!(
            parse_command(&[
                "mssql-orm-cli".to_string(),
                "migration".to_string(),
                "list".to_string(),
            ])
            .unwrap(),
            CliCommand::MigrationList
        );
        assert_eq!(
            parse_command(&[
                "mssql-orm-cli".to_string(),
                "database".to_string(),
                "update".to_string(),
            ])
            .unwrap(),
            CliCommand::DatabaseUpdate
        );
    }

    #[test]
    fn run_migration_add_creates_scaffold() {
        let root = temp_project_root();

        let output = run(
            vec![
                "mssql-orm-cli".to_string(),
                "migration".to_string(),
                "add".to_string(),
                "CreateCustomers".to_string(),
            ],
            &root,
        )
        .unwrap();

        assert!(output.contains("Created migration"));
        assert!(root.join("migrations").exists());
    }

    #[test]
    fn run_migration_add_uses_current_model_snapshot_when_provided() {
        let root = temp_project_root();
        let snapshot_path = root.join("current_model_snapshot.json");
        fs::write(
            &snapshot_path,
            "{\n  \"schemas\": [\n    {\n      \"name\": \"sales\",\n      \"tables\": []\n    }\n  ]\n}\n",
        )
        .unwrap();

        let output = run(
            vec![
                "mssql-orm-cli".to_string(),
                "migration".to_string(),
                "add".to_string(),
                "CreateCustomers".to_string(),
                "--model-snapshot".to_string(),
                "current_model_snapshot.json".to_string(),
            ],
            &root,
        )
        .unwrap();

        assert!(output.contains("Previous snapshot: none"));
        assert!(output.contains("Current snapshot: schemas=1 tables=0"));

        let migration_path = output
            .lines()
            .find_map(|line| line.strip_prefix("Path: "))
            .map(PathBuf::from)
            .unwrap();
        let snapshot = read_model_snapshot(&migration_path.join("model_snapshot.json")).unwrap();

        assert!(snapshot.schema("sales").is_some());
    }

    #[test]
    fn run_migration_add_uses_snapshot_exporter_binary_when_provided() {
        let root = temp_project_root();
        let fixture = root.join("fixture_app");
        let fixture_src = fixture.join("src");
        let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .parent()
            .unwrap()
            .to_path_buf();
        let orm_crate_root = repo_root.join("crates/mssql-orm");
        let escaped_repo_root = orm_crate_root.display().to_string().replace('\\', "\\\\");

        fs::create_dir_all(&fixture_src).unwrap();
        fs::write(
            fixture.join("Cargo.toml"),
            format!(
                "[package]\nname = \"fixture-app\"\nversion = \"0.1.0\"\nedition = \"2024\"\n\n[dependencies]\nmssql-orm = {{ path = \"{}\" }}\n",
                escaped_repo_root
            ),
        )
        .unwrap();
        fs::write(
            fixture_src.join("main.rs"),
            "use mssql_orm::prelude::*;\n\n#[derive(Entity, Debug, Clone)]\n#[orm(schema = \"sales\", table = \"customers\")]\nstruct Customer {\n    #[orm(primary_key)]\n    id: i64,\n}\n\n#[derive(DbContext, Debug, Clone)]\nstruct AppDbContext {\n    customers: DbSet<Customer>,\n}\n\nfn main() {\n    print!(\"{}\", mssql_orm::model_snapshot_json_from_source::<AppDbContext>().unwrap());\n}\n",
        )
        .unwrap();

        let output = run(
            vec![
                "mssql-orm-cli".to_string(),
                "migration".to_string(),
                "add".to_string(),
                "CreateCustomers".to_string(),
                "--snapshot-bin".to_string(),
                "fixture-app".to_string(),
                "--manifest-path".to_string(),
                fixture.join("Cargo.toml").display().to_string(),
            ],
            &root,
        );

        let output = output.unwrap();
        assert!(output.contains("Previous snapshot: none"));
        assert!(output.contains("Current snapshot: schemas=1 tables=1"));
        let migration_path = output
            .lines()
            .find_map(|line| line.strip_prefix("Path: "))
            .map(PathBuf::from)
            .unwrap();
        let snapshot = read_model_snapshot(&migration_path.join("model_snapshot.json")).unwrap();

        assert_eq!(snapshot.schemas.len(), 1);
        assert!(
            snapshot
                .schema("sales")
                .unwrap()
                .table("customers")
                .is_some()
        );
    }

    #[test]
    fn run_migration_add_loads_previous_snapshot_from_latest_local_migration() {
        let root = temp_project_root();
        let previous_dir = root.join("migrations/100_create_customers");
        let current_snapshot_path = root.join("current_model_snapshot.json");

        fs::create_dir_all(&previous_dir).unwrap();
        fs::write(previous_dir.join("up.sql"), "-- noop").unwrap();
        fs::write(previous_dir.join("down.sql"), "-- noop").unwrap();
        fs::write(
            previous_dir.join("model_snapshot.json"),
            "{\n  \"schemas\": [\n    {\n      \"name\": \"dbo\",\n      \"tables\": []\n    }\n  ]\n}\n",
        )
        .unwrap();
        fs::write(
            &current_snapshot_path,
            "{\n  \"schemas\": [\n    {\n      \"name\": \"sales\",\n      \"tables\": []\n    }\n  ]\n}\n",
        )
        .unwrap();

        let output = run(
            vec![
                "mssql-orm-cli".to_string(),
                "migration".to_string(),
                "add".to_string(),
                "CreateOrders".to_string(),
                "--model-snapshot".to_string(),
                "current_model_snapshot.json".to_string(),
            ],
            &root,
        )
        .unwrap();

        assert!(output.contains("Previous snapshot: 100_create_customers (schemas: 1)"));
        assert!(output.contains("Current snapshot: schemas=1 tables=0"));
    }

    #[test]
    fn run_migration_list_prints_existing_migrations() {
        let root = temp_project_root();
        fs::create_dir_all(root.join("migrations/100_create_customers")).unwrap();

        let output = run(
            vec![
                "mssql-orm-cli".to_string(),
                "migration".to_string(),
                "list".to_string(),
            ],
            &root,
        )
        .unwrap();

        assert!(output.contains("100_create_customers"));
    }

    #[test]
    fn run_database_update_outputs_sql_script() {
        let root = temp_project_root();
        let migration_dir = root.join("migrations/100_create_customers");
        fs::create_dir_all(&migration_dir).unwrap();
        fs::write(
            migration_dir.join("up.sql"),
            "CREATE TABLE [sales].[customers] ([id] bigint NOT NULL);",
        )
        .unwrap();
        fs::write(migration_dir.join("down.sql"), "-- noop").unwrap();
        fs::write(
            migration_dir.join("model_snapshot.json"),
            "{ \"schemas\": [] }",
        )
        .unwrap();

        let output = run(
            vec![
                "mssql-orm-cli".to_string(),
                "database".to_string(),
                "update".to_string(),
            ],
            &root,
        )
        .unwrap();

        assert!(output.contains("CREATE TABLE [dbo].[__mssql_orm_migrations]"));
        assert!(output.contains("SET QUOTED_IDENTIFIER ON;"));
        assert!(output.contains("CREATE TABLE [sales].[customers]"));
        assert!(output.contains("INSERT INTO [dbo].[__mssql_orm_migrations]"));
        assert!(output.contains("THROW 50001, N'mssql-orm migration checksum mismatch"));
        assert!(output.contains("BEGIN TRANSACTION;"));
        assert!(output.contains("ROLLBACK TRANSACTION;"));
    }
}

use mssql_orm_migrate::{build_database_update_script, create_migration_scaffold, list_migrations};
use mssql_orm_sqlserver::SqlServerCompiler;
use std::env;
use std::path::Path;

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
        CliCommand::MigrationAdd { name } => {
            let scaffold =
                create_migration_scaffold(root, &name).map_err(|error| error.to_string())?;
            Ok(format!(
                "Created migration {}\nPath: {}",
                scaffold.id,
                scaffold.directory.display()
            ))
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

#[derive(Debug, Clone, PartialEq, Eq)]
enum CliCommand {
    MigrationAdd { name: String },
    MigrationList,
    DatabaseUpdate,
}

fn parse_command(args: &[String]) -> Result<CliCommand, String> {
    match args {
        [_bin, group, action, name] if group == "migration" && action == "add" => {
            Ok(CliCommand::MigrationAdd { name: name.clone() })
        }
        [_bin, group, action] if group == "migration" && action == "list" => {
            Ok(CliCommand::MigrationList)
        }
        [_bin, group, action] if group == "database" && action == "update" => {
            Ok(CliCommand::DatabaseUpdate)
        }
        _ => Err(
            "Usage:\n  mssql-orm-cli migration add <Name>\n  mssql-orm-cli migration list\n  mssql-orm-cli database update".to_string(),
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::{CliCommand, parse_command, run};
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
                name: "CreateCustomers".to_string()
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
        assert!(output.contains("CREATE TABLE [sales].[customers]"));
        assert!(output.contains("INSERT INTO [dbo].[__mssql_orm_migrations]"));
        assert!(output.contains("THROW 50001, N'mssql-orm migration checksum mismatch"));
        assert!(output.contains("BEGIN TRANSACTION;"));
        assert!(output.contains("ROLLBACK TRANSACTION;"));
    }
}

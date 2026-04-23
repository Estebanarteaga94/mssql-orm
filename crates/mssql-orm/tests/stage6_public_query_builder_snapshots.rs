use insta::assert_snapshot;
use mssql_orm::prelude::*;
use mssql_orm::query::CompiledQuery;
use mssql_orm::sqlserver::SqlServerCompiler;

#[allow(dead_code)]
#[derive(Entity, Debug, Clone)]
#[orm(table = "snapshot_users", schema = "dbo")]
struct SnapshotUser {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,
    #[orm(length = 180)]
    email: String,
    active: bool,
}

#[test]
fn public_query_builder_snapshot_preserves_sql_and_parameter_order() {
    let compiled = SqlServerCompiler::compile_select(
        &mssql_orm::query::SelectQuery::from_entity::<SnapshotUser>()
            .filter(
                SnapshotUser::active
                    .eq(true)
                    .and(SnapshotUser::email.contains("@example.com")),
            )
            .order_by(SnapshotUser::email.desc())
            .paginate(PageRequest::new(2, 20).to_pagination()),
    )
    .unwrap();

    assert_snapshot!(
        "public_query_builder_compiled_select",
        render_snapshot(&compiled)
    );
}

#[test]
fn public_query_builder_keeps_untrusted_values_out_of_sql_text() {
    let malicious = "'; DROP TABLE dbo.snapshot_users; --";

    let compiled = SqlServerCompiler::compile_select(
        &mssql_orm::query::SelectQuery::from_entity::<SnapshotUser>()
            .filter(SnapshotUser::email.contains(malicious))
            .order_by(SnapshotUser::id.asc())
            .paginate(PageRequest::new(1, 5).to_pagination()),
    )
    .unwrap();

    assert!(!compiled.sql.contains(malicious));
    assert_eq!(compiled.params.len(), 3);
    assert_eq!(
        compiled.params[0],
        SqlValue::String(format!("%{malicious}%"))
    );
    assert_eq!(compiled.params[1], SqlValue::I64(0));
    assert_eq!(compiled.params[2], SqlValue::I64(5));
}

fn render_snapshot(compiled: &CompiledQuery) -> String {
    let params = compiled
        .params
        .iter()
        .enumerate()
        .map(|(index, value)| format!("{}: {}", index + 1, render_sql_value(value)))
        .collect::<Vec<_>>();

    if params.is_empty() {
        format!("SQL: {}\nParams:\n<none>", compiled.sql)
    } else {
        format!("SQL: {}\nParams:\n{}", compiled.sql, params.join("\n"))
    }
}

fn render_sql_value(value: &SqlValue) -> String {
    match value {
        SqlValue::Null => "Null".to_string(),
        SqlValue::Bool(value) => format!("Bool({value})"),
        SqlValue::I32(value) => format!("I32({value})"),
        SqlValue::I64(value) => format!("I64({value})"),
        SqlValue::F64(value) => format!("F64({value})"),
        SqlValue::String(value) => format!("String({value:?})"),
        SqlValue::Bytes(value) => format!("Bytes({value:?})"),
        SqlValue::Uuid(value) => format!("Uuid({value})"),
        SqlValue::Decimal(value) => format!("Decimal({value})"),
        SqlValue::Date(value) => format!("Date({value})"),
        SqlValue::DateTime(value) => format!("DateTime({value})"),
    }
}

use mssql_orm::prelude::*;
use mssql_orm::query::{
    Expr, OrderBy, Pagination, Predicate, SelectQuery, SortDirection, TableRef,
};

#[allow(dead_code)]
#[derive(Entity, Debug, Clone)]
#[orm(table = "query_builder_users", schema = "dbo")]
struct QueryUser {
    #[orm(primary_key)]
    #[orm(identity)]
    id: i64,
    #[orm(length = 180)]
    email: String,
    active: bool,
}

#[test]
fn public_query_builder_extensions_produce_expected_ast() {
    let query = SelectQuery::from_entity::<QueryUser>()
        .filter(
            QueryUser::active
                .eq(true)
                .and(QueryUser::email.contains("@example.com")),
        )
        .order_by(QueryUser::email.desc())
        .paginate(PageRequest::new(2, 20).to_pagination());

    assert_eq!(
        query,
        SelectQuery::from_entity::<QueryUser>()
            .filter(Predicate::and(vec![
                Predicate::eq(
                    Expr::from(QueryUser::active),
                    Expr::value(SqlValue::Bool(true)),
                ),
                Predicate::like(
                    Expr::from(QueryUser::email),
                    Expr::value(SqlValue::String("%@example.com%".to_string())),
                ),
            ]))
            .order_by(OrderBy::new(
                TableRef::new("dbo", "query_builder_users"),
                "email",
                SortDirection::Desc,
            ))
            .paginate(Pagination::new(20, 20))
    );
}

#[test]
fn public_predicate_composition_flattens_logical_groups() {
    let predicate = QueryUser::active
        .eq(true)
        .and(QueryUser::email.contains("@example.com"))
        .and(QueryUser::email.is_not_null())
        .or(QueryUser::id.gt(10_i64));

    assert_eq!(
        predicate,
        Predicate::or(vec![
            Predicate::and(vec![
                Predicate::eq(
                    Expr::from(QueryUser::active),
                    Expr::value(SqlValue::Bool(true)),
                ),
                Predicate::like(
                    Expr::from(QueryUser::email),
                    Expr::value(SqlValue::String("%@example.com%".to_string())),
                ),
                Predicate::is_not_null(Expr::from(QueryUser::email)),
            ]),
            Predicate::gt(Expr::from(QueryUser::id), Expr::value(SqlValue::I64(10))),
        ])
    );
}

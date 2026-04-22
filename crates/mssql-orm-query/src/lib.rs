//! Query AST foundations for the ORM.

mod delete;
mod expr;
mod insert;
mod order;
mod pagination;
mod predicate;
mod select;
mod update;

use mssql_orm_core::{CrateIdentity, SqlValue};

pub use delete::DeleteQuery;
pub use expr::{BinaryOp, ColumnRef, Expr, TableRef, UnaryOp};
pub use insert::InsertQuery;
pub use order::{OrderBy, SortDirection};
pub use pagination::Pagination;
pub use predicate::Predicate;
pub use select::{CountQuery, SelectQuery};
pub use update::UpdateQuery;

#[derive(Debug, Clone, PartialEq)]
pub struct CompiledQuery {
    pub sql: String,
    pub params: Vec<SqlValue>,
}

impl CompiledQuery {
    pub fn new(sql: impl Into<String>, params: Vec<SqlValue>) -> Self {
        Self {
            sql: sql.into(),
            params,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Query {
    Select(SelectQuery),
    Insert(InsertQuery),
    Update(UpdateQuery),
    Delete(DeleteQuery),
    Count(CountQuery),
}

pub const CRATE_IDENTITY: CrateIdentity = CrateIdentity {
    name: "mssql-orm-query",
    responsibility: "typed AST and query builder primitives without SQL generation",
};

#[cfg(test)]
mod tests {
    use super::{
        BinaryOp, CRATE_IDENTITY, ColumnRef, CompiledQuery, CountQuery, DeleteQuery, Expr,
        InsertQuery, OrderBy, Pagination, Predicate, Query, SelectQuery, SortDirection, TableRef,
        UpdateQuery,
    };
    use mssql_orm_core::{
        Changeset, ColumnMetadata, ColumnValue, Entity, EntityColumn, EntityMetadata,
        IdentityMetadata, Insertable, PrimaryKeyMetadata, SqlServerType, SqlValue,
    };

    #[allow(dead_code)]
    struct Customer;

    static CUSTOMER_COLUMNS: [ColumnMetadata; 4] = [
        ColumnMetadata {
            rust_field: "id",
            column_name: "id",
            sql_type: SqlServerType::BigInt,
            nullable: false,
            primary_key: true,
            identity: Some(IdentityMetadata::new(1, 1)),
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: false,
            updatable: false,
            max_length: None,
            precision: None,
            scale: None,
        },
        ColumnMetadata {
            rust_field: "email",
            column_name: "email",
            sql_type: SqlServerType::NVarChar,
            nullable: false,
            primary_key: false,
            identity: None,
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: true,
            updatable: true,
            max_length: Some(160),
            precision: None,
            scale: None,
        },
        ColumnMetadata {
            rust_field: "active",
            column_name: "active",
            sql_type: SqlServerType::Bit,
            nullable: false,
            primary_key: false,
            identity: None,
            default_sql: Some("1"),
            computed_sql: None,
            rowversion: false,
            insertable: true,
            updatable: true,
            max_length: None,
            precision: None,
            scale: None,
        },
        ColumnMetadata {
            rust_field: "created_at",
            column_name: "created_at",
            sql_type: SqlServerType::DateTime2,
            nullable: false,
            primary_key: false,
            identity: None,
            default_sql: Some("SYSUTCDATETIME()"),
            computed_sql: None,
            rowversion: false,
            insertable: true,
            updatable: true,
            max_length: None,
            precision: None,
            scale: None,
        },
    ];

    static CUSTOMER_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "Customer",
        schema: "sales",
        table: "customers",
        columns: &CUSTOMER_COLUMNS,
        primary_key: PrimaryKeyMetadata::new(Some("pk_customers"), &["id"]),
        indexes: &[],
        foreign_keys: &[],
    };

    impl Entity for Customer {
        fn metadata() -> &'static EntityMetadata {
            &CUSTOMER_METADATA
        }
    }

    #[allow(non_upper_case_globals)]
    impl Customer {
        const id: EntityColumn<Customer> = EntityColumn::new("id", "id");
        const email: EntityColumn<Customer> = EntityColumn::new("email", "email");
        const active: EntityColumn<Customer> = EntityColumn::new("active", "active");
        const created_at: EntityColumn<Customer> = EntityColumn::new("created_at", "created_at");
    }

    struct NewCustomer {
        email: String,
        active: bool,
    }

    impl Insertable<Customer> for NewCustomer {
        fn values(&self) -> Vec<ColumnValue> {
            vec![
                ColumnValue::new("email", SqlValue::String(self.email.clone())),
                ColumnValue::new("active", SqlValue::Bool(self.active)),
            ]
        }
    }

    struct UpdateCustomer {
        email: Option<String>,
    }

    impl Changeset<Customer> for UpdateCustomer {
        fn changes(&self) -> Vec<ColumnValue> {
            self.email
                .clone()
                .map(|email| vec![ColumnValue::new("email", SqlValue::String(email))])
                .unwrap_or_default()
        }
    }

    #[test]
    fn keeps_query_layer_sql_free() {
        assert!(
            CRATE_IDENTITY
                .responsibility
                .contains("without SQL generation")
        );
    }

    #[test]
    fn entity_columns_become_table_aware_column_refs() {
        let column = ColumnRef::for_entity_column(Customer::email);

        assert_eq!(column.table, TableRef::new("sales", "customers"));
        assert_eq!(column.rust_field, "email");
        assert_eq!(column.column_name, "email");
    }

    #[test]
    fn expr_supports_columns_values_functions_and_operations() {
        let expr = Expr::binary(
            Expr::function("LOWER", vec![Expr::from(Customer::email)]),
            BinaryOp::Add,
            Expr::value(SqlValue::String("@example.com".to_string())),
        );

        match expr {
            Expr::Binary { left, op, right } => {
                assert_eq!(op, BinaryOp::Add);
                assert!(matches!(*left, Expr::Function { .. }));
                assert_eq!(
                    *right,
                    Expr::Value(SqlValue::String("@example.com".to_string()))
                );
            }
            other => panic!("unexpected expr shape: {other:?}"),
        }
    }

    #[test]
    fn predicates_can_be_composed_without_sql_rendering() {
        let predicate = Predicate::and(vec![
            Predicate::eq(
                Expr::from(Customer::active),
                Expr::value(SqlValue::Bool(true)),
            ),
            Predicate::like(
                Expr::from(Customer::email),
                Expr::value(SqlValue::String("%@example.com".to_string())),
            ),
        ]);

        match predicate {
            Predicate::And(parts) => assert_eq!(parts.len(), 2),
            other => panic!("unexpected predicate shape: {other:?}"),
        }
    }

    #[test]
    fn select_query_captures_projection_filters_order_and_pagination() {
        let query = SelectQuery::from_entity::<Customer>()
            .select(vec![Expr::from(Customer::id), Expr::from(Customer::email)])
            .filter(Predicate::eq(
                Expr::from(Customer::active),
                Expr::value(SqlValue::Bool(true)),
            ))
            .filter(Predicate::like(
                Expr::from(Customer::email),
                Expr::value(SqlValue::String("%@example.com".to_string())),
            ))
            .order_by(OrderBy::desc(Customer::created_at))
            .paginate(Pagination::page(2, 20));

        assert_eq!(query.from, TableRef::new("sales", "customers"));
        assert_eq!(query.projection.len(), 2);
        assert_eq!(
            query.order_by,
            vec![OrderBy::new(
                TableRef::new("sales", "customers"),
                "created_at",
                SortDirection::Desc,
            )]
        );
        assert_eq!(query.pagination, Some(Pagination::new(20, 20)));
        assert!(matches!(query.predicate, Some(Predicate::And(_))));
    }

    #[test]
    fn insert_update_delete_and_count_queries_capture_operation_data() {
        let insert = InsertQuery::for_entity::<Customer, _>(&NewCustomer {
            email: "ana@example.com".to_string(),
            active: true,
        });
        let update = UpdateQuery::for_entity::<Customer, _>(&UpdateCustomer {
            email: Some("ana.maria@example.com".to_string()),
        })
        .filter(Predicate::eq(
            Expr::from(Customer::id),
            Expr::value(SqlValue::I64(7)),
        ));
        let delete = DeleteQuery::from_entity::<Customer>().filter(Predicate::eq(
            Expr::from(Customer::id),
            Expr::value(SqlValue::I64(7)),
        ));
        let count = CountQuery::from_entity::<Customer>().filter(Predicate::eq(
            Expr::from(Customer::active),
            Expr::value(SqlValue::Bool(true)),
        ));

        assert_eq!(insert.into, TableRef::new("sales", "customers"));
        assert_eq!(insert.values.len(), 2);
        assert_eq!(update.table, TableRef::new("sales", "customers"));
        assert_eq!(update.changes.len(), 1);
        assert!(update.predicate.is_some());
        assert_eq!(delete.from, TableRef::new("sales", "customers"));
        assert!(delete.predicate.is_some());
        assert_eq!(count.from, TableRef::new("sales", "customers"));
        assert!(count.predicate.is_some());

        assert!(matches!(Query::Insert(insert.clone()), Query::Insert(_)));
        assert!(matches!(Query::Update(update.clone()), Query::Update(_)));
        assert!(matches!(Query::Delete(delete.clone()), Query::Delete(_)));
        assert!(matches!(Query::Count(count.clone()), Query::Count(_)));
    }

    #[test]
    fn compiled_query_keeps_sql_and_parameter_order() {
        let compiled = CompiledQuery::new(
            "SELECT [id] FROM [sales].[customers] WHERE [active] = @P1 AND [email] LIKE @P2",
            vec![
                SqlValue::Bool(true),
                SqlValue::String("%@example.com".to_string()),
            ],
        );

        assert_eq!(
            compiled.sql,
            "SELECT [id] FROM [sales].[customers] WHERE [active] = @P1 AND [email] LIKE @P2"
        );
        assert_eq!(
            compiled.params,
            vec![
                SqlValue::Bool(true),
                SqlValue::String("%@example.com".to_string()),
            ]
        );
    }
}

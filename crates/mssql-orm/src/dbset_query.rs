use crate::context::SharedConnection;
use crate::page_request::PageRequest;
use mssql_orm_core::{Entity, FromRow, OrmError, Row, SqlValue};
use mssql_orm_query::{CountQuery, Join, OrderBy, Pagination, Predicate, SelectQuery};
use mssql_orm_sqlserver::SqlServerCompiler;

#[derive(Clone)]
pub struct DbSetQuery<E: Entity> {
    connection: Option<SharedConnection>,
    select_query: SelectQuery,
    _entity: core::marker::PhantomData<fn() -> E>,
}

impl<E: Entity> DbSetQuery<E> {
    pub(crate) fn new(connection: Option<SharedConnection>, select_query: SelectQuery) -> Self {
        Self {
            connection,
            select_query,
            _entity: core::marker::PhantomData,
        }
    }

    pub fn with_select_query(mut self, select_query: SelectQuery) -> Self {
        self.select_query = select_query;
        self
    }

    pub fn filter(mut self, predicate: Predicate) -> Self {
        self.select_query = self.select_query.filter(predicate);
        self
    }

    pub fn join(mut self, join: Join) -> Self {
        self.select_query = self.select_query.join(join);
        self
    }

    pub fn inner_join<J: Entity>(mut self, on: Predicate) -> Self {
        self.select_query = self.select_query.inner_join::<J>(on);
        self
    }

    pub fn left_join<J: Entity>(mut self, on: Predicate) -> Self {
        self.select_query = self.select_query.left_join::<J>(on);
        self
    }

    pub fn order_by(mut self, order: OrderBy) -> Self {
        self.select_query = self.select_query.order_by(order);
        self
    }

    pub fn limit(mut self, limit: u64) -> Self {
        self.select_query = self.select_query.paginate(Pagination::new(0, limit));
        self
    }

    pub fn take(self, limit: u64) -> Self {
        self.limit(limit)
    }

    pub fn paginate(mut self, request: PageRequest) -> Self {
        self.select_query = self.select_query.paginate(request.to_pagination());
        self
    }

    pub fn select_query(&self) -> &SelectQuery {
        &self.select_query
    }

    pub fn into_select_query(self) -> SelectQuery {
        self.select_query
    }

    pub async fn all(self) -> Result<Vec<E>, OrmError>
    where
        E: FromRow + Send,
    {
        let compiled = SqlServerCompiler::compile_select(&self.select_query)?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await;
        connection.fetch_all(compiled).await
    }

    pub async fn first(self) -> Result<Option<E>, OrmError>
    where
        E: FromRow + Send,
    {
        let compiled = SqlServerCompiler::compile_select(&self.select_query)?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await;
        connection.fetch_one(compiled).await
    }

    pub async fn count(self) -> Result<i64, OrmError> {
        let compiled = SqlServerCompiler::compile_count(&self.count_query())?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await;
        let row = connection.fetch_one::<CountRow>(compiled).await?;

        row.map(|row| row.value)
            .ok_or_else(|| OrmError::new("count query did not return a row"))
    }

    fn count_query(&self) -> CountQuery {
        CountQuery {
            from: self.select_query.from,
            predicate: self.select_query.predicate.clone(),
        }
    }

    fn require_connection(&self) -> Result<SharedConnection, OrmError> {
        self.connection
            .as_ref()
            .cloned()
            .ok_or_else(|| OrmError::new("DbSetQuery requires an initialized shared connection"))
    }
}

impl<E: Entity> core::fmt::Debug for DbSetQuery<E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DbSetQuery")
            .field("entity", &E::metadata().rust_name)
            .field("table", &E::metadata().table)
            .field("select_query", &self.select_query)
            .finish()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct CountRow {
    value: i64,
}

impl FromRow for CountRow {
    fn from_row<R: Row>(row: &R) -> Result<Self, OrmError> {
        match row.get_required("count")? {
            SqlValue::I32(value) => Ok(Self {
                value: i64::from(value),
            }),
            SqlValue::I64(value) => Ok(Self { value }),
            _ => Err(OrmError::new(
                "expected SQL Server COUNT result as i32 or i64",
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::DbSetQuery;
    use crate::context::DbSet;
    use crate::page_request::PageRequest;
    use mssql_orm_core::{
        Entity, EntityMetadata, FromRow, OrmError, PrimaryKeyMetadata, Row, SqlValue,
    };
    use mssql_orm_query::{
        Expr, Join, JoinType, OrderBy, Pagination, Predicate, SelectQuery, SortDirection, TableRef,
    };

    struct TestEntity;
    struct JoinedEntity;

    static TEST_ENTITY_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "TestEntity",
        schema: "dbo",
        table: "test_entities",
        renamed_from: None,
        columns: &[],
        primary_key: PrimaryKeyMetadata {
            name: None,
            columns: &[],
        },
        indexes: &[],
        foreign_keys: &[],
    };

    impl Entity for TestEntity {
        fn metadata() -> &'static EntityMetadata {
            &TEST_ENTITY_METADATA
        }
    }

    static JOINED_ENTITY_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "JoinedEntity",
        schema: "dbo",
        table: "joined_entities",
        renamed_from: None,
        columns: &[],
        primary_key: PrimaryKeyMetadata {
            name: None,
            columns: &[],
        },
        indexes: &[],
        foreign_keys: &[],
    };

    impl Entity for JoinedEntity {
        fn metadata() -> &'static EntityMetadata {
            &JOINED_ENTITY_METADATA
        }
    }

    #[test]
    fn dbset_query_starts_from_entity_select_query() {
        let dbset = DbSet::<TestEntity>::disconnected();
        let query = dbset.query();

        assert_eq!(
            query.select_query(),
            &SelectQuery::from_entity::<TestEntity>()
        );
    }

    #[test]
    fn dbset_query_accepts_replacement_select_query() {
        let dbset = DbSet::<TestEntity>::disconnected();
        let custom = SelectQuery::from_entity::<TestEntity>().filter(Predicate::eq(
            Expr::value(SqlValue::Bool(true)),
            Expr::value(SqlValue::Bool(true)),
        ));

        let query = dbset.query().with_select_query(custom.clone());

        assert_eq!(query.select_query(), &custom);
        assert_eq!(query.into_select_query(), custom);
    }

    #[test]
    fn dbset_query_filter_builds_on_internal_select_query() {
        let dbset = DbSet::<TestEntity>::disconnected();

        let query = dbset.query().filter(Predicate::eq(
            Expr::value(SqlValue::Bool(true)),
            Expr::value(SqlValue::Bool(true)),
        ));

        assert_eq!(
            query.into_select_query(),
            SelectQuery::from_entity::<TestEntity>().filter(Predicate::eq(
                Expr::value(SqlValue::Bool(true)),
                Expr::value(SqlValue::Bool(true)),
            ))
        );
    }

    #[test]
    fn dbset_query_order_by_builds_on_internal_select_query() {
        let dbset = DbSet::<TestEntity>::disconnected();

        let query = dbset.query().order_by(OrderBy::new(
            TableRef::new("dbo", "test_entities"),
            "created_at",
            SortDirection::Desc,
        ));

        assert_eq!(
            query.into_select_query(),
            SelectQuery::from_entity::<TestEntity>().order_by(OrderBy::new(
                TableRef::new("dbo", "test_entities"),
                "created_at",
                SortDirection::Desc,
            ))
        );
    }

    #[test]
    fn dbset_query_join_builds_on_internal_select_query() {
        let dbset = DbSet::<TestEntity>::disconnected();
        let join = Join::left(
            TableRef::new("dbo", "joined_entities"),
            Predicate::eq(
                Expr::value(SqlValue::Bool(true)),
                Expr::value(SqlValue::Bool(true)),
            ),
        );

        let query = dbset.query().join(join.clone());

        assert_eq!(
            query.into_select_query(),
            SelectQuery::from_entity::<TestEntity>().join(join)
        );
    }

    #[test]
    fn dbset_query_exposes_entity_targeted_join_helpers() {
        let dbset = DbSet::<TestEntity>::disconnected();

        let query = dbset
            .query()
            .inner_join::<JoinedEntity>(Predicate::eq(
                Expr::value(SqlValue::Bool(true)),
                Expr::value(SqlValue::Bool(true)),
            ))
            .left_join::<JoinedEntity>(Predicate::eq(
                Expr::value(SqlValue::Bool(false)),
                Expr::value(SqlValue::Bool(false)),
            ));

        let select = query.into_select_query();

        assert_eq!(select.joins.len(), 2);
        assert_eq!(select.joins[0].join_type, JoinType::Inner);
        assert_eq!(
            select.joins[0].table,
            TableRef::new("dbo", "joined_entities")
        );
        assert_eq!(select.joins[1].join_type, JoinType::Left);
        assert_eq!(
            select.joins[1].table,
            TableRef::new("dbo", "joined_entities")
        );
    }

    #[test]
    fn dbset_query_supports_chaining_filter_and_order_by() {
        let dbset = DbSet::<TestEntity>::disconnected();

        let query = dbset
            .query()
            .filter(Predicate::eq(
                Expr::value(SqlValue::Bool(true)),
                Expr::value(SqlValue::Bool(true)),
            ))
            .order_by(OrderBy::new(
                TableRef::new("dbo", "test_entities"),
                "created_at",
                SortDirection::Asc,
            ));

        assert_eq!(
            query.into_select_query(),
            SelectQuery::from_entity::<TestEntity>()
                .filter(Predicate::eq(
                    Expr::value(SqlValue::Bool(true)),
                    Expr::value(SqlValue::Bool(true)),
                ))
                .order_by(OrderBy::new(
                    TableRef::new("dbo", "test_entities"),
                    "created_at",
                    SortDirection::Asc,
                ))
        );
    }

    #[test]
    fn dbset_query_limit_builds_zero_offset_pagination() {
        let dbset = DbSet::<TestEntity>::disconnected();

        let query = dbset.query().limit(25);

        assert_eq!(
            query.into_select_query(),
            SelectQuery::from_entity::<TestEntity>().paginate(Pagination::new(0, 25))
        );
    }

    #[test]
    fn dbset_query_take_is_alias_for_limit() {
        let dbset = DbSet::<TestEntity>::disconnected();

        let limited = dbset.query().limit(10).into_select_query();
        let taken = dbset.query().take(10).into_select_query();

        assert_eq!(limited, taken);
    }

    #[test]
    fn dbset_query_paginate_uses_page_request_contract() {
        let dbset = DbSet::<TestEntity>::disconnected();

        let query = dbset.query().paginate(PageRequest::new(3, 25));

        assert_eq!(
            query.into_select_query(),
            SelectQuery::from_entity::<TestEntity>().paginate(Pagination::new(50, 25))
        );
    }

    #[test]
    fn count_row_accepts_i32_and_i64_results() {
        struct CountTestRow {
            value: SqlValue,
        }

        impl Row for CountTestRow {
            fn try_get(&self, column: &str) -> Result<Option<SqlValue>, OrmError> {
                Ok((column == "count").then(|| self.value.clone()))
            }
        }

        let from_i32 = super::CountRow::from_row(&CountTestRow {
            value: SqlValue::I32(7),
        })
        .unwrap();
        let from_i64 = super::CountRow::from_row(&CountTestRow {
            value: SqlValue::I64(9),
        })
        .unwrap();

        assert_eq!(from_i32.value, 7);
        assert_eq!(from_i64.value, 9);
    }

    #[test]
    fn count_row_rejects_non_integer_results() {
        struct CountTestRow;

        impl Row for CountTestRow {
            fn try_get(&self, column: &str) -> Result<Option<SqlValue>, OrmError> {
                Ok((column == "count").then(|| SqlValue::String("7".to_string())))
            }
        }

        let error = super::CountRow::from_row(&CountTestRow).unwrap_err();

        assert_eq!(
            error.message(),
            "expected SQL Server COUNT result as i32 or i64"
        );
    }

    #[test]
    fn debug_mentions_entity_type() {
        let query = DbSetQuery::<TestEntity>::new(None, SelectQuery::from_entity::<TestEntity>());

        let rendered = format!("{query:?}");

        assert!(rendered.contains("DbSetQuery"));
        assert!(rendered.contains("test_entities"));
    }
}

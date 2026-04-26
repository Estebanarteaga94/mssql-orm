use crate::SoftDeleteEntity;
use crate::context::SharedConnection;
use crate::page_request::PageRequest;
use mssql_orm_core::{Entity, FromRow, OrmError, Row, SqlServerType, SqlValue};
use mssql_orm_query::{
    ColumnRef, CountQuery, Expr, Join, OrderBy, Pagination, Predicate, SelectQuery, TableRef,
};
use mssql_orm_sqlserver::SqlServerCompiler;

#[derive(Clone)]
pub struct DbSetQuery<E: Entity> {
    connection: Option<SharedConnection>,
    select_query: SelectQuery,
    visibility: SoftDeleteVisibility,
    _entity: core::marker::PhantomData<fn() -> E>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SoftDeleteVisibility {
    Default,
    WithDeleted,
    OnlyDeleted,
}

impl<E: Entity> DbSetQuery<E> {
    pub(crate) fn new(connection: Option<SharedConnection>, select_query: SelectQuery) -> Self {
        Self {
            connection,
            select_query,
            visibility: SoftDeleteVisibility::Default,
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

    #[cfg(test)]
    pub(crate) fn select_query(&self) -> &SelectQuery {
        &self.select_query
    }

    pub fn with_deleted(mut self) -> Self {
        self.visibility = SoftDeleteVisibility::WithDeleted;
        self
    }

    pub fn only_deleted(mut self) -> Self {
        self.visibility = SoftDeleteVisibility::OnlyDeleted;
        self
    }

    #[cfg(test)]
    pub(crate) fn into_select_query(self) -> SelectQuery {
        self.select_query
    }

    pub async fn all(self) -> Result<Vec<E>, OrmError>
    where
        E: FromRow + Send + SoftDeleteEntity,
    {
        let compiled = SqlServerCompiler::compile_select(&self.effective_select_query()?)?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await?;
        connection.fetch_all(compiled).await
    }

    pub async fn first(self) -> Result<Option<E>, OrmError>
    where
        E: FromRow + Send + SoftDeleteEntity,
    {
        let compiled = SqlServerCompiler::compile_select(&self.effective_select_query()?)?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await?;
        connection.fetch_one(compiled).await
    }

    pub async fn count(self) -> Result<i64, OrmError>
    where
        E: SoftDeleteEntity,
    {
        let compiled = SqlServerCompiler::compile_count(&self.count_query())?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await?;
        let row = connection.fetch_one::<CountRow>(compiled).await?;

        row.map(|row| row.value)
            .ok_or_else(|| OrmError::new("count query did not return a row"))
    }

    fn count_query(&self) -> CountQuery
    where
        E: SoftDeleteEntity,
    {
        let effective = self
            .effective_select_query()
            .expect("count_query should materialize soft_delete visibility");
        CountQuery {
            from: effective.from,
            predicate: effective.predicate.clone(),
        }
    }

    fn effective_select_query(&self) -> Result<SelectQuery, OrmError>
    where
        E: SoftDeleteEntity,
    {
        let Some(predicate) = self.soft_delete_visibility_predicate()? else {
            return Ok(self.select_query.clone());
        };

        Ok(self.select_query.clone().filter(predicate))
    }

    fn soft_delete_visibility_predicate(&self) -> Result<Option<Predicate>, OrmError>
    where
        E: SoftDeleteEntity,
    {
        let Some(policy) = E::soft_delete_policy() else {
            return Ok(None);
        };

        let visibility = match self.visibility {
            SoftDeleteVisibility::Default => SoftDeleteVisibility::Default,
            SoftDeleteVisibility::WithDeleted => return Ok(None),
            SoftDeleteVisibility::OnlyDeleted => SoftDeleteVisibility::OnlyDeleted,
        };

        let indicator = policy.columns.first().ok_or_else(|| {
            OrmError::new("soft_delete query visibility requires at least one policy column")
        })?;
        let column = Expr::Column(ColumnRef::new(
            TableRef::for_entity::<E>(),
            indicator.rust_field,
            indicator.column_name,
        ));

        if indicator.sql_type == SqlServerType::Bit {
            return Ok(Some(match visibility {
                SoftDeleteVisibility::Default => {
                    Predicate::eq(column, Expr::Value(SqlValue::Bool(false)))
                }
                SoftDeleteVisibility::OnlyDeleted => {
                    Predicate::eq(column, Expr::Value(SqlValue::Bool(true)))
                }
                SoftDeleteVisibility::WithDeleted => unreachable!(),
            }));
        }

        if indicator.nullable {
            return Ok(Some(match visibility {
                SoftDeleteVisibility::Default => Predicate::is_null(column),
                SoftDeleteVisibility::OnlyDeleted => Predicate::is_not_null(column),
                SoftDeleteVisibility::WithDeleted => unreachable!(),
            }));
        }

        Err(OrmError::new(
            "soft_delete query visibility requires the first policy column to be nullable or bit",
        ))
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
    use crate::SoftDeleteEntity;
    use crate::context::DbSet;
    use crate::page_request::PageRequest;
    use mssql_orm_core::{
        ColumnMetadata, Entity, EntityMetadata, EntityPolicyMetadata, FromRow, OrmError,
        PrimaryKeyMetadata, Row, SqlServerType, SqlValue,
    };
    use mssql_orm_query::{
        Expr, Join, JoinType, OrderBy, Pagination, Predicate, SelectQuery, SortDirection, TableRef,
    };

    struct TestEntity;
    struct JoinedEntity;
    struct SoftDeleteEntityUnderTest;
    struct BoolSoftDeleteEntity;

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

    static SOFT_DELETE_POLICY_COLUMNS: [ColumnMetadata; 2] = [
        ColumnMetadata {
            rust_field: "deleted_at",
            column_name: "deleted_at",
            renamed_from: None,
            sql_type: SqlServerType::DateTime2,
            nullable: true,
            primary_key: false,
            identity: None,
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: false,
            updatable: true,
            max_length: None,
            precision: None,
            scale: None,
        },
        ColumnMetadata {
            rust_field: "deleted_by",
            column_name: "deleted_by",
            renamed_from: None,
            sql_type: SqlServerType::NVarChar,
            nullable: true,
            primary_key: false,
            identity: None,
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: false,
            updatable: true,
            max_length: Some(120),
            precision: None,
            scale: None,
        },
    ];

    static BOOL_SOFT_DELETE_POLICY_COLUMNS: [ColumnMetadata; 1] = [ColumnMetadata {
        rust_field: "is_deleted",
        column_name: "is_deleted",
        renamed_from: None,
        sql_type: SqlServerType::Bit,
        nullable: false,
        primary_key: false,
        identity: None,
        default_sql: Some("0"),
        computed_sql: None,
        rowversion: false,
        insertable: false,
        updatable: true,
        max_length: None,
        precision: None,
        scale: None,
    }];

    static SOFT_DELETE_ENTITY_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "SoftDeleteEntityUnderTest",
        schema: "dbo",
        table: "soft_delete_entities",
        renamed_from: None,
        columns: &[],
        primary_key: PrimaryKeyMetadata {
            name: None,
            columns: &[],
        },
        indexes: &[],
        foreign_keys: &[],
    };

    static BOOL_SOFT_DELETE_ENTITY_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "BoolSoftDeleteEntity",
        schema: "dbo",
        table: "bool_soft_delete_entities",
        renamed_from: None,
        columns: &[],
        primary_key: PrimaryKeyMetadata {
            name: None,
            columns: &[],
        },
        indexes: &[],
        foreign_keys: &[],
    };

    impl Entity for SoftDeleteEntityUnderTest {
        fn metadata() -> &'static EntityMetadata {
            &SOFT_DELETE_ENTITY_METADATA
        }
    }

    impl Entity for BoolSoftDeleteEntity {
        fn metadata() -> &'static EntityMetadata {
            &BOOL_SOFT_DELETE_ENTITY_METADATA
        }
    }

    impl SoftDeleteEntity for TestEntity {
        fn soft_delete_policy() -> Option<EntityPolicyMetadata> {
            None
        }
    }

    impl SoftDeleteEntity for JoinedEntity {
        fn soft_delete_policy() -> Option<EntityPolicyMetadata> {
            None
        }
    }

    impl SoftDeleteEntity for SoftDeleteEntityUnderTest {
        fn soft_delete_policy() -> Option<EntityPolicyMetadata> {
            Some(EntityPolicyMetadata::new(
                "soft_delete",
                &SOFT_DELETE_POLICY_COLUMNS,
            ))
        }
    }

    impl SoftDeleteEntity for BoolSoftDeleteEntity {
        fn soft_delete_policy() -> Option<EntityPolicyMetadata> {
            Some(EntityPolicyMetadata::new(
                "soft_delete",
                &BOOL_SOFT_DELETE_POLICY_COLUMNS,
            ))
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
    fn dbset_query_applies_active_only_visibility_for_nullable_indicator() {
        let dbset = DbSet::<SoftDeleteEntityUnderTest>::disconnected();

        let query = dbset.query().effective_select_query().unwrap();

        assert_eq!(
            query,
            SelectQuery::from_entity::<SoftDeleteEntityUnderTest>().filter(Predicate::is_null(
                Expr::Column(mssql_orm_query::ColumnRef::new(
                    TableRef::new("dbo", "soft_delete_entities"),
                    "deleted_at",
                    "deleted_at",
                )),
            ))
        );
    }

    #[test]
    fn dbset_query_with_deleted_removes_soft_delete_filter() {
        let dbset = DbSet::<SoftDeleteEntityUnderTest>::disconnected();

        let query = dbset
            .query()
            .with_deleted()
            .effective_select_query()
            .unwrap();

        assert_eq!(
            query,
            SelectQuery::from_entity::<SoftDeleteEntityUnderTest>()
        );
    }

    #[test]
    fn dbset_query_only_deleted_filters_nullable_indicator() {
        let dbset = DbSet::<SoftDeleteEntityUnderTest>::disconnected();

        let query = dbset
            .query()
            .only_deleted()
            .effective_select_query()
            .unwrap();

        assert_eq!(
            query,
            SelectQuery::from_entity::<SoftDeleteEntityUnderTest>().filter(Predicate::is_not_null(
                Expr::Column(mssql_orm_query::ColumnRef::new(
                    TableRef::new("dbo", "soft_delete_entities"),
                    "deleted_at",
                    "deleted_at",
                ))
            ))
        );
    }

    #[test]
    fn dbset_query_uses_bool_indicator_when_soft_delete_column_is_bit() {
        let dbset = DbSet::<BoolSoftDeleteEntity>::disconnected();

        let active = dbset.query().effective_select_query().unwrap();
        let deleted = dbset
            .query()
            .only_deleted()
            .effective_select_query()
            .unwrap();

        assert_eq!(
            active,
            SelectQuery::from_entity::<BoolSoftDeleteEntity>().filter(Predicate::eq(
                Expr::Column(mssql_orm_query::ColumnRef::new(
                    TableRef::new("dbo", "bool_soft_delete_entities"),
                    "is_deleted",
                    "is_deleted",
                )),
                Expr::Value(SqlValue::Bool(false)),
            ))
        );
        assert_eq!(
            deleted,
            SelectQuery::from_entity::<BoolSoftDeleteEntity>().filter(Predicate::eq(
                Expr::Column(mssql_orm_query::ColumnRef::new(
                    TableRef::new("dbo", "bool_soft_delete_entities"),
                    "is_deleted",
                    "is_deleted",
                )),
                Expr::Value(SqlValue::Bool(true)),
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

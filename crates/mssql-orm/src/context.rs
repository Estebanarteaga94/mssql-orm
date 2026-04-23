use crate::dbset_query::DbSetQuery;
use core::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::EntityPersist;
use mssql_orm_core::{
    Changeset, Entity, EntityMetadata, FromRow, Insertable, OrmError, SqlTypeMapping, SqlValue,
};
use mssql_orm_query::{
    ColumnRef, DeleteQuery, Expr, InsertQuery, Predicate, SelectQuery, TableRef, UpdateQuery,
};
use mssql_orm_sqlserver::SqlServerCompiler;
use mssql_orm_tiberius::{MssqlConnection, TokioConnectionStream};

pub type SharedConnection = Arc<tokio::sync::Mutex<MssqlConnection<TokioConnectionStream>>>;

pub trait DbContext: Sized {
    fn from_shared_connection(connection: SharedConnection) -> Self;
    fn shared_connection(&self) -> SharedConnection;

    fn transaction<F, Fut, T>(
        &self,
        operation: F,
    ) -> impl Future<Output = Result<T, OrmError>> + Send
    where
        F: FnOnce(Self) -> Fut + Send,
        Fut: Future<Output = Result<T, OrmError>> + Send,
        T: Send,
    {
        let shared_connection = self.shared_connection();
        async move {
            {
                let mut connection = shared_connection.lock().await;
                connection.begin_transaction_scope().await?;
            }

            let transaction_context = Self::from_shared_connection(Arc::clone(&shared_connection));
            let result = operation(transaction_context).await;

            match result {
                Ok(value) => {
                    let mut connection = shared_connection.lock().await;
                    connection.commit_transaction().await?;
                    Ok(value)
                }
                Err(error) => {
                    let mut connection = shared_connection.lock().await;
                    connection.rollback_transaction().await?;
                    Err(error)
                }
            }
        }
    }
}

pub trait DbContextEntitySet<E: Entity>: DbContext {
    fn db_set(&self) -> &DbSet<E>;
}

#[derive(Clone)]
pub struct DbSet<E: Entity> {
    connection: Option<SharedConnection>,
    _entity: PhantomData<fn() -> E>,
}

impl<E: Entity> DbSet<E> {
    pub fn new(connection: SharedConnection) -> Self {
        Self {
            connection: Some(connection),
            _entity: PhantomData,
        }
    }

    #[cfg(test)]
    pub(crate) fn disconnected() -> Self {
        Self {
            connection: None,
            _entity: PhantomData,
        }
    }

    pub fn entity_metadata(&self) -> &'static EntityMetadata {
        E::metadata()
    }

    pub fn query(&self) -> DbSetQuery<E> {
        DbSetQuery::new(
            self.connection.as_ref().cloned(),
            SelectQuery::from_entity::<E>(),
        )
    }

    pub fn query_with(&self, select_query: SelectQuery) -> DbSetQuery<E> {
        DbSetQuery::new(self.connection.as_ref().cloned(), select_query)
    }

    pub async fn find<K>(&self, key: K) -> Result<Option<E>, OrmError>
    where
        E: FromRow + Send,
        K: SqlTypeMapping,
    {
        self.query_with(self.find_select_query(key)?).first().await
    }

    pub async fn insert<I>(&self, insertable: I) -> Result<E, OrmError>
    where
        E: FromRow + Send,
        I: Insertable<E>,
    {
        let compiled = SqlServerCompiler::compile_insert(&self.insert_query(&insertable))?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await;
        let inserted = connection.fetch_one(compiled).await?;

        inserted.ok_or_else(|| OrmError::new("insert query did not return a row"))
    }

    pub async fn update<K, C>(&self, key: K, changeset: C) -> Result<Option<E>, OrmError>
    where
        E: FromRow + Send,
        K: SqlTypeMapping,
        C: Changeset<E>,
    {
        let compiled = SqlServerCompiler::compile_update(&self.update_query(key, &changeset)?)?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await;
        connection.fetch_one(compiled).await
    }

    pub async fn delete<K>(&self, key: K) -> Result<bool, OrmError>
    where
        K: SqlTypeMapping,
    {
        let compiled = SqlServerCompiler::compile_delete(&self.delete_query(key)?)?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await;
        let result = connection.execute(compiled).await?;

        Ok(result.total() > 0)
    }

    pub(crate) async fn delete_by_sql_value(&self, key: SqlValue) -> Result<bool, OrmError> {
        let compiled = SqlServerCompiler::compile_delete(&self.delete_query_sql_value(key)?)?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await;
        let result = connection.execute(compiled).await?;

        Ok(result.total() > 0)
    }

    pub(crate) async fn find_by_sql_value(&self, key: SqlValue) -> Result<Option<E>, OrmError>
    where
        E: FromRow + Send,
    {
        self.query_with(self.find_select_query_sql_value(key)?)
            .first()
            .await
    }

    pub(crate) async fn insert_entity_values(
        &self,
        values: Vec<mssql_orm_core::ColumnValue>,
    ) -> Result<E, OrmError>
    where
        E: FromRow + Send,
    {
        let compiled = SqlServerCompiler::compile_insert(&self.insert_query_values(values))?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await;
        let inserted = connection.fetch_one(compiled).await?;

        inserted.ok_or_else(|| OrmError::new("insert query did not return a row"))
    }

    pub(crate) async fn insert_entity(&self, entity: &E) -> Result<E, OrmError>
    where
        E: EntityPersist + FromRow + Send,
    {
        self.insert_entity_values(entity.insert_values()).await
    }

    pub(crate) async fn update_entity_values_by_sql_value(
        &self,
        key: SqlValue,
        changes: Vec<mssql_orm_core::ColumnValue>,
    ) -> Result<Option<E>, OrmError>
    where
        E: FromRow + Send,
    {
        let compiled =
            SqlServerCompiler::compile_update(&self.update_query_sql_value(key, changes)?)?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await;
        connection.fetch_one(compiled).await
    }

    pub(crate) async fn update_entity_by_sql_value(
        &self,
        key: SqlValue,
        entity: &E,
    ) -> Result<Option<E>, OrmError>
    where
        E: EntityPersist + FromRow + Send,
    {
        self.update_entity_values_by_sql_value(key, entity.update_changes())
            .await
    }

    pub fn shared_connection(&self) -> SharedConnection {
        Arc::clone(
            self.connection
                .as_ref()
                .expect("DbSet requires an initialized shared connection"),
        )
    }

    fn require_connection(&self) -> Result<SharedConnection, OrmError> {
        self.connection
            .as_ref()
            .cloned()
            .ok_or_else(|| OrmError::new("DbSet requires an initialized shared connection"))
    }
}

impl<E: Entity> std::fmt::Debug for DbSet<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DbSet")
            .field("entity", &E::metadata().rust_name)
            .field("table", &E::metadata().table)
            .finish()
    }
}

impl<E: Entity> DbSet<E> {
    fn find_select_query<K>(&self, key: K) -> Result<SelectQuery, OrmError>
    where
        K: SqlTypeMapping,
    {
        Ok(SelectQuery::from_entity::<E>().filter(self.primary_key_predicate(key)?))
    }

    fn find_select_query_sql_value(&self, key: SqlValue) -> Result<SelectQuery, OrmError> {
        Ok(SelectQuery::from_entity::<E>().filter(self.primary_key_predicate_value(key)?))
    }

    fn insert_query<I>(&self, insertable: &I) -> InsertQuery
    where
        I: Insertable<E>,
    {
        InsertQuery::for_entity::<E, I>(insertable)
    }

    fn insert_query_values(&self, values: Vec<mssql_orm_core::ColumnValue>) -> InsertQuery {
        InsertQuery::for_entity::<E, _>(&RawInsertable(values))
    }

    fn update_query<K, C>(&self, key: K, changeset: &C) -> Result<UpdateQuery, OrmError>
    where
        K: SqlTypeMapping,
        C: Changeset<E>,
    {
        Ok(UpdateQuery::for_entity::<E, C>(changeset).filter(self.primary_key_predicate(key)?))
    }

    fn update_query_sql_value(
        &self,
        key: SqlValue,
        changes: Vec<mssql_orm_core::ColumnValue>,
    ) -> Result<UpdateQuery, OrmError> {
        Ok(UpdateQuery::for_entity::<E, _>(&RawChangeset(changes))
            .filter(self.primary_key_predicate_value(key)?))
    }

    fn delete_query<K>(&self, key: K) -> Result<DeleteQuery, OrmError>
    where
        K: SqlTypeMapping,
    {
        Ok(DeleteQuery::from_entity::<E>().filter(self.primary_key_predicate(key)?))
    }

    fn delete_query_sql_value(&self, key: SqlValue) -> Result<DeleteQuery, OrmError> {
        Ok(DeleteQuery::from_entity::<E>().filter(self.primary_key_predicate_value(key)?))
    }

    fn primary_key_predicate<K>(&self, key: K) -> Result<Predicate, OrmError>
    where
        K: SqlTypeMapping,
    {
        self.primary_key_predicate_value(key.to_sql_value())
    }

    fn primary_key_predicate_value(&self, key: SqlValue) -> Result<Predicate, OrmError> {
        let metadata = E::metadata();
        let primary_key = metadata.primary_key_columns();

        if primary_key.len() != 1 {
            return Err(OrmError::new(
                "DbSet currently supports this operation only for entities with a single primary key column",
            ));
        }

        let column = primary_key[0];

        Ok(Predicate::eq(
            Expr::Column(ColumnRef::new(
                TableRef::for_entity::<E>(),
                column.rust_field,
                column.column_name,
            )),
            Expr::Value(key),
        ))
    }
}

struct RawInsertable(Vec<mssql_orm_core::ColumnValue>);

impl<E: Entity> Insertable<E> for RawInsertable {
    fn values(&self) -> Vec<mssql_orm_core::ColumnValue> {
        self.0.clone()
    }
}

struct RawChangeset(Vec<mssql_orm_core::ColumnValue>);

impl<E: Entity> Changeset<E> for RawChangeset {
    fn changes(&self) -> Vec<mssql_orm_core::ColumnValue> {
        self.0.clone()
    }
}

pub async fn connect_shared(connection_string: &str) -> Result<SharedConnection, OrmError> {
    let connection = MssqlConnection::connect(connection_string).await?;
    Ok(Arc::new(tokio::sync::Mutex::new(connection)))
}

#[cfg(test)]
mod tests {
    use super::{DbContext, DbContextEntitySet, DbSet};
    use mssql_orm_core::{
        ColumnMetadata, ColumnValue, Entity, EntityMetadata, PrimaryKeyMetadata, SqlServerType,
        SqlValue,
    };
    use mssql_orm_query::{
        ColumnRef, DeleteQuery, Expr, InsertQuery, Predicate, SelectQuery, TableRef, UpdateQuery,
    };

    struct TestEntity;
    struct CompositeKeyEntity;
    struct DummyContext {
        entities: DbSet<TestEntity>,
    }
    struct NewTestEntity {
        name: String,
        active: bool,
    }
    struct UpdateTestEntity {
        name: Option<String>,
        active: Option<bool>,
    }

    static TEST_ENTITY_COLUMNS: [ColumnMetadata; 3] = [
        ColumnMetadata {
            rust_field: "id",
            column_name: "id",
            sql_type: SqlServerType::BigInt,
            nullable: false,
            primary_key: true,
            identity: None,
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: true,
            updatable: false,
            max_length: None,
            precision: None,
            scale: None,
        },
        ColumnMetadata {
            rust_field: "name",
            column_name: "name",
            sql_type: SqlServerType::NVarChar,
            nullable: false,
            primary_key: false,
            identity: None,
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: true,
            updatable: true,
            max_length: Some(120),
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
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: true,
            updatable: true,
            max_length: None,
            precision: None,
            scale: None,
        },
    ];

    static TEST_ENTITY_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "TestEntity",
        schema: "dbo",
        table: "test_entities",
        columns: &TEST_ENTITY_COLUMNS,
        primary_key: PrimaryKeyMetadata {
            name: None,
            columns: &["id"],
        },
        indexes: &[],
        foreign_keys: &[],
    };

    static COMPOSITE_KEY_ENTITY_COLUMNS: [ColumnMetadata; 2] = [
        ColumnMetadata {
            rust_field: "tenant_id",
            column_name: "tenant_id",
            sql_type: SqlServerType::BigInt,
            nullable: false,
            primary_key: true,
            identity: None,
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: true,
            updatable: false,
            max_length: None,
            precision: None,
            scale: None,
        },
        ColumnMetadata {
            rust_field: "id",
            column_name: "id",
            sql_type: SqlServerType::BigInt,
            nullable: false,
            primary_key: true,
            identity: None,
            default_sql: None,
            computed_sql: None,
            rowversion: false,
            insertable: true,
            updatable: false,
            max_length: None,
            precision: None,
            scale: None,
        },
    ];

    static COMPOSITE_KEY_ENTITY_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "CompositeKeyEntity",
        schema: "dbo",
        table: "composite_entities",
        columns: &COMPOSITE_KEY_ENTITY_COLUMNS,
        primary_key: PrimaryKeyMetadata {
            name: None,
            columns: &["tenant_id", "id"],
        },
        indexes: &[],
        foreign_keys: &[],
    };

    impl Entity for TestEntity {
        fn metadata() -> &'static EntityMetadata {
            &TEST_ENTITY_METADATA
        }
    }

    impl Entity for CompositeKeyEntity {
        fn metadata() -> &'static EntityMetadata {
            &COMPOSITE_KEY_ENTITY_METADATA
        }
    }

    impl DbContext for DummyContext {
        fn from_shared_connection(_connection: super::SharedConnection) -> Self {
            unreachable!("DummyContext is only used in disconnected unit tests")
        }

        fn shared_connection(&self) -> super::SharedConnection {
            panic!("DummyContext is only used in disconnected unit tests")
        }
    }

    impl DbContextEntitySet<TestEntity> for DummyContext {
        fn db_set(&self) -> &DbSet<TestEntity> {
            &self.entities
        }
    }

    impl mssql_orm_core::Insertable<TestEntity> for NewTestEntity {
        fn values(&self) -> Vec<ColumnValue> {
            vec![
                ColumnValue::new("name", SqlValue::String(self.name.clone())),
                ColumnValue::new("active", SqlValue::Bool(self.active)),
            ]
        }
    }

    impl mssql_orm_core::Changeset<TestEntity> for UpdateTestEntity {
        fn changes(&self) -> Vec<ColumnValue> {
            let mut values = Vec::new();

            if let Some(name) = &self.name {
                values.push(ColumnValue::new("name", SqlValue::String(name.clone())));
            }

            if let Some(active) = self.active {
                values.push(ColumnValue::new("active", SqlValue::Bool(active)));
            }

            values
        }
    }

    impl mssql_orm_core::Changeset<CompositeKeyEntity> for UpdateTestEntity {
        fn changes(&self) -> Vec<ColumnValue> {
            <Self as mssql_orm_core::Changeset<TestEntity>>::changes(self)
        }
    }

    #[test]
    fn dbset_exposes_entity_metadata() {
        let dbset = DbSet::<TestEntity>::disconnected();

        assert_eq!(dbset.entity_metadata().table, "test_entities");
    }

    #[test]
    fn dbcontext_entity_set_trait_returns_typed_dbset() {
        let context = DummyContext {
            entities: DbSet::<TestEntity>::disconnected(),
        };

        let dbset = <DummyContext as DbContextEntitySet<TestEntity>>::db_set(&context);

        assert_eq!(dbset.entity_metadata().rust_name, "TestEntity");
        assert_eq!(dbset.entity_metadata().table, "test_entities");
    }

    #[test]
    fn dbset_debug_includes_entity_name() {
        let dbset = DbSet::<TestEntity>::disconnected();

        let rendered = format!("{dbset:?}");

        assert!(rendered.contains("TestEntity"));
        assert!(rendered.contains("test_entities"));
    }

    #[test]
    fn dbset_query_uses_entity_select_query_by_default() {
        let dbset = DbSet::<TestEntity>::disconnected();

        assert_eq!(
            dbset.query().into_select_query(),
            SelectQuery::from_entity::<TestEntity>()
        );
    }

    #[test]
    fn dbset_query_with_accepts_custom_select_query() {
        let dbset = DbSet::<TestEntity>::disconnected();
        let custom = SelectQuery::from_entity::<TestEntity>();

        assert_eq!(dbset.query_with(custom.clone()).into_select_query(), custom);
    }

    #[test]
    fn dbset_find_builds_select_query_for_single_primary_key() {
        let dbset = DbSet::<TestEntity>::disconnected();

        let query = dbset.find_select_query(7_i64).unwrap();

        assert_eq!(
            query,
            SelectQuery::from_entity::<TestEntity>().filter(Predicate::eq(
                Expr::Column(ColumnRef::new(
                    TableRef::new("dbo", "test_entities"),
                    "id",
                    "id",
                )),
                Expr::Value(mssql_orm_core::SqlValue::I64(7)),
            ))
        );
    }

    #[test]
    fn dbset_find_rejects_composite_primary_keys() {
        let dbset = DbSet::<CompositeKeyEntity>::disconnected();

        let error = dbset.find_select_query(7_i64).unwrap_err();

        assert_eq!(
            error.message(),
            "DbSet currently supports this operation only for entities with a single primary key column"
        );
    }

    #[test]
    fn dbset_insert_builds_insert_query_for_entity() {
        let dbset = DbSet::<TestEntity>::disconnected();
        let insertable = NewTestEntity {
            name: "ana".to_string(),
            active: true,
        };

        let query = dbset.insert_query(&insertable);

        assert_eq!(
            query,
            InsertQuery {
                into: TableRef::new("dbo", "test_entities"),
                values: vec![
                    ColumnValue::new("name", SqlValue::String("ana".to_string())),
                    ColumnValue::new("active", SqlValue::Bool(true)),
                ],
            }
        );
    }

    #[test]
    fn dbset_update_builds_update_query_for_entity_and_primary_key() {
        let dbset = DbSet::<TestEntity>::disconnected();
        let changeset = UpdateTestEntity {
            name: Some("ana maria".to_string()),
            active: Some(false),
        };

        let query = dbset.update_query(7_i64, &changeset).unwrap();

        assert_eq!(
            query,
            UpdateQuery::for_entity::<TestEntity, _>(&changeset).filter(Predicate::eq(
                Expr::Column(ColumnRef::new(
                    TableRef::new("dbo", "test_entities"),
                    "id",
                    "id",
                )),
                Expr::Value(SqlValue::I64(7)),
            ))
        );
    }

    #[test]
    fn dbset_update_rejects_composite_primary_keys() {
        let dbset = DbSet::<CompositeKeyEntity>::disconnected();
        let changeset = UpdateTestEntity {
            name: Some("ana".to_string()),
            active: None,
        };

        let error = dbset.update_query(7_i64, &changeset).unwrap_err();

        assert_eq!(
            error.message(),
            "DbSet currently supports this operation only for entities with a single primary key column"
        );
    }

    #[test]
    fn dbset_delete_builds_delete_query_for_entity_and_primary_key() {
        let dbset = DbSet::<TestEntity>::disconnected();

        let query = dbset.delete_query(7_i64).unwrap();

        assert_eq!(
            query,
            DeleteQuery::from_entity::<TestEntity>().filter(Predicate::eq(
                Expr::Column(ColumnRef::new(
                    TableRef::new("dbo", "test_entities"),
                    "id",
                    "id",
                )),
                Expr::Value(SqlValue::I64(7)),
            ))
        );
    }

    #[test]
    fn dbset_delete_query_sql_value_builds_delete_query_for_entity_and_primary_key() {
        let dbset = DbSet::<TestEntity>::disconnected();

        let query = dbset.delete_query_sql_value(SqlValue::I64(7)).unwrap();

        assert_eq!(
            query,
            DeleteQuery::from_entity::<TestEntity>().filter(Predicate::eq(
                Expr::Column(ColumnRef::new(
                    TableRef::new("dbo", "test_entities"),
                    "id",
                    "id",
                )),
                Expr::Value(SqlValue::I64(7)),
            ))
        );
    }

    #[test]
    fn dbset_delete_rejects_composite_primary_keys() {
        let dbset = DbSet::<CompositeKeyEntity>::disconnected();

        let error = dbset.delete_query(7_i64).unwrap_err();

        assert_eq!(
            error.message(),
            "DbSet currently supports this operation only for entities with a single primary key column"
        );
    }
}

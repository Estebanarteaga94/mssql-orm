use crate::dbset_query::DbSetQuery;
use crate::{Tracked, TrackingRegistry, TrackingRegistryHandle};
use core::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

use crate::{EntityPersist, EntityPrimaryKey};
use mssql_orm_core::{
    Changeset, Entity, EntityMetadata, FromRow, Insertable, OrmError, SqlTypeMapping, SqlValue,
};
use mssql_orm_query::{
    ColumnRef, DeleteQuery, Expr, InsertQuery, Predicate, SelectQuery, TableRef, UpdateQuery,
};
use mssql_orm_sqlserver::SqlServerCompiler;
use mssql_orm_tiberius::{
    MssqlConnection, MssqlConnectionConfig, MssqlOperationalOptions, TokioConnectionStream,
};
#[cfg(feature = "pool-bb8")]
use mssql_orm_tiberius::{MssqlPool, MssqlPooledConnection};

#[derive(Clone)]
pub struct SharedConnection {
    inner: Arc<SharedConnectionInner>,
}

enum SharedConnectionInner {
    Direct(tokio::sync::Mutex<MssqlConnection<TokioConnectionStream>>),
    #[cfg(feature = "pool-bb8")]
    Pool(MssqlPool),
}

pub enum SharedConnectionGuard<'a> {
    Direct(tokio::sync::MutexGuard<'a, MssqlConnection<TokioConnectionStream>>),
    #[cfg(feature = "pool-bb8")]
    Pool(MssqlPooledConnection<'a>),
}

impl SharedConnection {
    pub fn from_connection(connection: MssqlConnection<TokioConnectionStream>) -> Self {
        Self {
            inner: Arc::new(SharedConnectionInner::Direct(tokio::sync::Mutex::new(
                connection,
            ))),
        }
    }

    #[cfg(feature = "pool-bb8")]
    pub fn from_pool(pool: MssqlPool) -> Self {
        Self {
            inner: Arc::new(SharedConnectionInner::Pool(pool)),
        }
    }

    pub async fn lock(&self) -> Result<SharedConnectionGuard<'_>, OrmError> {
        match self.inner.as_ref() {
            SharedConnectionInner::Direct(connection) => {
                Ok(SharedConnectionGuard::Direct(connection.lock().await))
            }
            #[cfg(feature = "pool-bb8")]
            SharedConnectionInner::Pool(pool) => {
                Ok(SharedConnectionGuard::Pool(pool.acquire().await?))
            }
        }
    }
}

impl core::ops::Deref for SharedConnectionGuard<'_> {
    type Target = MssqlConnection<TokioConnectionStream>;

    fn deref(&self) -> &Self::Target {
        match self {
            SharedConnectionGuard::Direct(connection) => connection,
            #[cfg(feature = "pool-bb8")]
            SharedConnectionGuard::Pool(connection) => connection,
        }
    }
}

impl core::ops::DerefMut for SharedConnectionGuard<'_> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            SharedConnectionGuard::Direct(connection) => connection,
            #[cfg(feature = "pool-bb8")]
            SharedConnectionGuard::Pool(connection) => connection,
        }
    }
}

pub trait DbContext: Sized {
    fn from_shared_connection(connection: SharedConnection) -> Self;
    fn shared_connection(&self) -> SharedConnection;
    #[doc(hidden)]
    fn tracking_registry(&self) -> TrackingRegistryHandle;

    fn health_check(&self) -> impl Future<Output = Result<(), OrmError>> + Send {
        let shared_connection = self.shared_connection();

        async move {
            let mut connection = shared_connection.lock().await?;
            connection.health_check().await
        }
    }

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
                let mut connection = shared_connection.lock().await?;
                connection.begin_transaction_scope().await?;
            }

            let transaction_context = Self::from_shared_connection(shared_connection.clone());
            let result = operation(transaction_context).await;

            match result {
                Ok(value) => {
                    let mut connection = shared_connection.lock().await?;
                    connection.commit_transaction().await?;
                    Ok(value)
                }
                Err(error) => {
                    let mut connection = shared_connection.lock().await?;
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
    tracking_registry: TrackingRegistryHandle,
    _entity: PhantomData<fn() -> E>,
}

impl<E: Entity> DbSet<E> {
    pub fn new(connection: SharedConnection) -> Self {
        Self::with_tracking_registry(connection, Arc::new(TrackingRegistry::default()))
    }

    #[doc(hidden)]
    pub fn with_tracking_registry(
        connection: SharedConnection,
        tracking_registry: TrackingRegistryHandle,
    ) -> Self {
        Self {
            connection: Some(connection),
            tracking_registry,
            _entity: PhantomData,
        }
    }

    #[cfg(test)]
    pub(crate) fn disconnected() -> Self {
        Self {
            connection: None,
            tracking_registry: Arc::new(TrackingRegistry::default()),
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

    /// Loads an entity by its single-column primary key and wraps it in the
    /// experimental snapshot-based tracking container.
    pub async fn find_tracked<K>(&self, key: K) -> Result<Option<Tracked<E>>, OrmError>
    where
        E: Clone + FromRow + Send,
        K: SqlTypeMapping,
    {
        let mut tracked = self
            .find(key)
            .await
            .map(|entity| entity.map(Tracked::from_loaded))?;

        if let Some(entity) = tracked.as_mut() {
            entity.attach_registry(Arc::clone(&self.tracking_registry));
        }

        Ok(tracked)
    }

    /// Registers a new in-memory entity as experimentally tracked in `Added`
    /// state so a later `save_changes()` can persist it via `insert`.
    pub fn add_tracked(&self, entity: E) -> Tracked<E>
    where
        E: Clone,
    {
        let mut tracked = Tracked::from_added(entity);
        tracked.attach_registry(Arc::clone(&self.tracking_registry));
        tracked
    }

    /// Marks a tracked entity for deletion so a later `save_changes()` can
    /// persist it through the regular delete pipeline.
    pub fn remove_tracked(&self, tracked: &mut Tracked<E>) {
        let was_added = tracked.state() == crate::EntityState::Added;
        tracked.mark_deleted();

        // Deleting an entity that was never inserted should simply cancel the
        // pending tracked insert instead of issuing a database delete.
        if was_added {
            tracked.detach_registry();
        }
    }

    #[doc(hidden)]
    pub async fn save_tracked_added(&self) -> Result<usize, OrmError>
    where
        E: Clone + EntityPersist + FromRow + Send,
    {
        let tracked_entities = self.tracking_registry.tracked_for::<E>();
        let mut saved = 0;

        for tracked in tracked_entities {
            if tracked.state() != crate::EntityState::Added {
                continue;
            }

            let current: E = tracked.current_clone();
            let persisted = self.insert_entity(&current).await?;

            tracked.sync_persisted(persisted);
            saved += 1;
        }

        Ok(saved)
    }

    #[doc(hidden)]
    pub async fn save_tracked_deleted(&self) -> Result<usize, OrmError>
    where
        E: Clone + EntityPersist + EntityPrimaryKey + FromRow + Send,
    {
        let tracked_entities = self.tracking_registry.tracked_for::<E>();
        let mut saved = 0;

        for tracked in tracked_entities {
            if tracked.state() != crate::EntityState::Deleted {
                continue;
            }

            let current: E = tracked.current_clone();
            let key = current.primary_key_value()?;
            let deleted = self
                .delete_tracked_by_sql_value(key, current.concurrency_token()?)
                .await?;

            if !deleted {
                return Err(OrmError::new(
                    "save_changes could not delete a tracked entity for the current primary key",
                ));
            }

            self.tracking_registry.unregister(tracked.registration_id());
            saved += 1;
        }

        Ok(saved)
    }

    #[doc(hidden)]
    pub async fn save_tracked_modified(&self) -> Result<usize, OrmError>
    where
        E: Clone + EntityPersist + EntityPrimaryKey + FromRow + Send,
    {
        let tracked_entities = self.tracking_registry.tracked_for::<E>();
        let mut saved = 0;

        for tracked in tracked_entities {
            if tracked.state() != crate::EntityState::Modified {
                continue;
            }

            let current: E = tracked.current_clone();
            let key = current.primary_key_value()?;
            let persisted = self
                .update_entity_by_sql_value(key, &current, current.concurrency_token()?)
                .await?
                .ok_or_else(|| {
                    OrmError::new(
                        "save_changes could not update a tracked entity for the current primary key",
                    )
                })?;

            tracked.sync_persisted(persisted);
            saved += 1;
        }

        Ok(saved)
    }

    pub async fn insert<I>(&self, insertable: I) -> Result<E, OrmError>
    where
        E: FromRow + Send,
        I: Insertable<E>,
    {
        let compiled = SqlServerCompiler::compile_insert(&self.insert_query(&insertable))?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await?;
        let inserted = connection.fetch_one(compiled).await?;

        inserted.ok_or_else(|| OrmError::new("insert query did not return a row"))
    }

    pub async fn update<K, C>(&self, key: K, changeset: C) -> Result<Option<E>, OrmError>
    where
        E: FromRow + Send,
        K: SqlTypeMapping,
        C: Changeset<E>,
    {
        let key = key.to_sql_value();
        let concurrency_token = changeset.concurrency_token()?;
        let compiled = SqlServerCompiler::compile_update(&self.update_query_sql_value(
            key.clone(),
            changeset.changes(),
            concurrency_token.clone(),
        )?)?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await?;
        let updated = connection.fetch_one(compiled).await?;
        drop(connection);

        if updated.is_none()
            && concurrency_token.is_some()
            && self.find_by_sql_value(key).await?.is_some()
        {
            return Err(OrmError::concurrency_conflict());
        }

        Ok(updated)
    }

    pub async fn delete<K>(&self, key: K) -> Result<bool, OrmError>
    where
        K: SqlTypeMapping,
    {
        let compiled = SqlServerCompiler::compile_delete(&self.delete_query(key)?)?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await?;
        let result = connection.execute(compiled).await?;

        Ok(result.total() > 0)
    }

    pub(crate) async fn delete_by_sql_value(
        &self,
        key: SqlValue,
        concurrency_token: Option<SqlValue>,
    ) -> Result<bool, OrmError> {
        let compiled = SqlServerCompiler::compile_delete(
            &self.delete_query_sql_value(key, concurrency_token)?,
        )?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await?;
        let result = connection.execute(compiled).await?;

        Ok(result.total() > 0)
    }

    pub(crate) async fn delete_tracked_by_sql_value(
        &self,
        key: SqlValue,
        concurrency_token: Option<SqlValue>,
    ) -> Result<bool, OrmError>
    where
        E: FromRow + Send,
    {
        let deleted = self
            .delete_by_sql_value(key.clone(), concurrency_token.clone())
            .await?;

        if !deleted && concurrency_token.is_some() && self.find_by_sql_value(key).await?.is_some() {
            return Err(OrmError::concurrency_conflict());
        }

        Ok(deleted)
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
        let mut connection = shared_connection.lock().await?;
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
        concurrency_token: Option<SqlValue>,
    ) -> Result<Option<E>, OrmError>
    where
        E: FromRow + Send,
    {
        let compiled = SqlServerCompiler::compile_update(&self.update_query_sql_value(
            key.clone(),
            changes,
            concurrency_token.clone(),
        )?)?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await?;
        let updated = connection.fetch_one(compiled).await?;
        drop(connection);

        if updated.is_none()
            && concurrency_token.is_some()
            && self.find_by_sql_value(key).await?.is_some()
        {
            return Err(OrmError::concurrency_conflict());
        }

        Ok(updated)
    }

    pub(crate) async fn update_entity_by_sql_value(
        &self,
        key: SqlValue,
        entity: &E,
        concurrency_token: Option<SqlValue>,
    ) -> Result<Option<E>, OrmError>
    where
        E: EntityPersist + FromRow + Send,
    {
        self.update_entity_values_by_sql_value(key, entity.update_changes(), concurrency_token)
            .await
    }

    pub fn shared_connection(&self) -> SharedConnection {
        self.connection
            .as_ref()
            .expect("DbSet requires an initialized shared connection")
            .clone()
    }

    #[doc(hidden)]
    pub fn tracking_registry(&self) -> TrackingRegistryHandle {
        Arc::clone(&self.tracking_registry)
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

    #[cfg(test)]
    fn update_query<K, C>(&self, key: K, changeset: &C) -> Result<UpdateQuery, OrmError>
    where
        K: SqlTypeMapping,
        C: Changeset<E>,
    {
        let mut query =
            UpdateQuery::for_entity::<E, C>(changeset).filter(self.primary_key_predicate(key)?);

        if let Some(token) = changeset.concurrency_token()? {
            query = query.filter(self.rowversion_predicate_value(token)?);
        }

        Ok(query)
    }

    fn update_query_sql_value(
        &self,
        key: SqlValue,
        changes: Vec<mssql_orm_core::ColumnValue>,
        concurrency_token: Option<SqlValue>,
    ) -> Result<UpdateQuery, OrmError> {
        let mut query = UpdateQuery::for_entity::<E, _>(&RawChangeset(changes))
            .filter(self.primary_key_predicate_value(key)?);

        if let Some(token) = concurrency_token {
            query = query.filter(self.rowversion_predicate_value(token)?);
        }

        Ok(query)
    }

    fn delete_query<K>(&self, key: K) -> Result<DeleteQuery, OrmError>
    where
        K: SqlTypeMapping,
    {
        Ok(DeleteQuery::from_entity::<E>().filter(self.primary_key_predicate(key)?))
    }

    fn delete_query_sql_value(
        &self,
        key: SqlValue,
        concurrency_token: Option<SqlValue>,
    ) -> Result<DeleteQuery, OrmError> {
        let mut query =
            DeleteQuery::from_entity::<E>().filter(self.primary_key_predicate_value(key)?);

        if let Some(token) = concurrency_token {
            query = query.filter(self.rowversion_predicate_value(token)?);
        }

        Ok(query)
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

    fn rowversion_predicate_value(&self, token: SqlValue) -> Result<Predicate, OrmError> {
        let metadata = E::metadata();
        let column = metadata.rowversion_column().ok_or_else(|| {
            OrmError::new("DbSet concurrency checks require an entity rowversion column")
        })?;

        Ok(Predicate::eq(
            Expr::Column(ColumnRef::new(
                TableRef::for_entity::<E>(),
                column.rust_field,
                column.column_name,
            )),
            Expr::Value(token),
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
    Ok(SharedConnection::from_connection(connection))
}

pub async fn connect_shared_with_options(
    connection_string: &str,
    options: MssqlOperationalOptions,
) -> Result<SharedConnection, OrmError> {
    let config =
        MssqlConnectionConfig::from_connection_string_with_options(connection_string, options)?;
    connect_shared_with_config(config).await
}

pub async fn connect_shared_with_config(
    config: MssqlConnectionConfig,
) -> Result<SharedConnection, OrmError> {
    let connection = MssqlConnection::connect_with_config(config).await?;
    Ok(SharedConnection::from_connection(connection))
}

#[cfg(feature = "pool-bb8")]
pub fn connect_shared_from_pool(pool: MssqlPool) -> SharedConnection {
    SharedConnection::from_pool(pool)
}

#[cfg(test)]
mod tests {
    use super::{DbContext, DbContextEntitySet, DbSet};
    use crate::Tracked;
    use mssql_orm_core::{
        ColumnMetadata, ColumnValue, Entity, EntityMetadata, FromRow, OrmError, PrimaryKeyMetadata,
        Row, SqlServerType, SqlValue,
    };
    use mssql_orm_query::{
        ColumnRef, DeleteQuery, Expr, InsertQuery, Predicate, SelectQuery, TableRef, UpdateQuery,
    };

    #[derive(Debug, Clone)]
    struct TestEntity;
    struct VersionedEntity;
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
    struct UpdateVersionedEntity {
        name: Option<String>,
        version: Option<Vec<u8>>,
    }

    static TEST_ENTITY_COLUMNS: [ColumnMetadata; 3] = [
        ColumnMetadata {
            rust_field: "id",
            column_name: "id",
            renamed_from: None,
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
            renamed_from: None,
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
            renamed_from: None,
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
        renamed_from: None,
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
            renamed_from: None,
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
            renamed_from: None,
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
        renamed_from: None,
        columns: &COMPOSITE_KEY_ENTITY_COLUMNS,
        primary_key: PrimaryKeyMetadata {
            name: None,
            columns: &["tenant_id", "id"],
        },
        indexes: &[],
        foreign_keys: &[],
    };

    static VERSIONED_ENTITY_COLUMNS: [ColumnMetadata; 3] = [
        ColumnMetadata {
            rust_field: "id",
            column_name: "id",
            renamed_from: None,
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
            renamed_from: None,
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
            rust_field: "version",
            column_name: "version",
            renamed_from: None,
            sql_type: SqlServerType::RowVersion,
            nullable: false,
            primary_key: false,
            identity: None,
            default_sql: None,
            computed_sql: None,
            rowversion: true,
            insertable: false,
            updatable: false,
            max_length: None,
            precision: None,
            scale: None,
        },
    ];

    static VERSIONED_ENTITY_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "VersionedEntity",
        schema: "dbo",
        table: "versioned_entities",
        renamed_from: None,
        columns: &VERSIONED_ENTITY_COLUMNS,
        primary_key: PrimaryKeyMetadata {
            name: None,
            columns: &["id"],
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

    impl Entity for VersionedEntity {
        fn metadata() -> &'static EntityMetadata {
            &VERSIONED_ENTITY_METADATA
        }
    }

    impl FromRow for TestEntity {
        fn from_row<R: Row>(_row: &R) -> Result<Self, OrmError> {
            Ok(Self)
        }
    }

    impl DbContext for DummyContext {
        fn from_shared_connection(_connection: super::SharedConnection) -> Self {
            unreachable!("DummyContext is only used in disconnected unit tests")
        }

        fn shared_connection(&self) -> super::SharedConnection {
            panic!("DummyContext is only used in disconnected unit tests")
        }

        fn tracking_registry(&self) -> crate::TrackingRegistryHandle {
            self.entities.tracking_registry()
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

    impl mssql_orm_core::Changeset<VersionedEntity> for UpdateVersionedEntity {
        fn changes(&self) -> Vec<ColumnValue> {
            let mut values = Vec::new();

            if let Some(name) = &self.name {
                values.push(ColumnValue::new("name", SqlValue::String(name.clone())));
            }

            values
        }

        fn concurrency_token(&self) -> Result<Option<SqlValue>, mssql_orm_core::OrmError> {
            Ok(self.version.clone().map(SqlValue::Bytes))
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

    #[tokio::test]
    async fn dbset_find_tracked_reuses_find_connection_path() {
        let dbset = DbSet::<TestEntity>::disconnected();

        let error = dbset.find_tracked(7_i64).await.unwrap_err();

        assert_eq!(
            error.message(),
            "DbSetQuery requires an initialized shared connection"
        );
    }

    #[test]
    fn dbset_add_tracked_registers_added_entity_in_registry() {
        let dbset = DbSet::<TestEntity>::disconnected();
        let registry = dbset.tracking_registry();

        let tracked = dbset.add_tracked(TestEntity);

        assert_eq!(tracked.state(), crate::EntityState::Added);
        assert_eq!(registry.entry_count(), 1);
        assert_eq!(registry.registrations()[0].state, crate::EntityState::Added);
    }

    #[test]
    fn dbset_remove_tracked_marks_loaded_entity_as_deleted() {
        let dbset = DbSet::<TestEntity>::disconnected();
        let registry = dbset.tracking_registry();
        let mut tracked = Tracked::from_loaded(TestEntity);
        tracked.attach_registry(registry.clone());

        dbset.remove_tracked(&mut tracked);

        assert_eq!(tracked.state(), crate::EntityState::Deleted);
        assert_eq!(registry.entry_count(), 1);
        assert_eq!(
            registry.registrations()[0].state,
            crate::EntityState::Deleted
        );
    }

    #[test]
    fn dbset_remove_tracked_cancels_pending_added_entity() {
        let dbset = DbSet::<TestEntity>::disconnected();
        let registry = dbset.tracking_registry();
        let mut tracked = dbset.add_tracked(TestEntity);

        dbset.remove_tracked(&mut tracked);

        assert_eq!(tracked.state(), crate::EntityState::Deleted);
        assert_eq!(registry.entry_count(), 0);
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
    fn dbset_update_appends_rowversion_predicate_when_changeset_has_token() {
        let dbset = DbSet::<VersionedEntity>::disconnected();
        let changeset = UpdateVersionedEntity {
            name: Some("ana maria".to_string()),
            version: Some(vec![1, 2, 3, 4]),
        };

        let query = dbset.update_query(7_i64, &changeset).unwrap();

        assert_eq!(
            query,
            UpdateQuery::for_entity::<VersionedEntity, _>(&changeset).filter(Predicate::and(vec![
                Predicate::eq(
                    Expr::Column(ColumnRef::new(
                        TableRef::new("dbo", "versioned_entities"),
                        "id",
                        "id",
                    )),
                    Expr::Value(SqlValue::I64(7)),
                ),
                Predicate::eq(
                    Expr::Column(ColumnRef::new(
                        TableRef::new("dbo", "versioned_entities"),
                        "version",
                        "version",
                    )),
                    Expr::Value(SqlValue::Bytes(vec![1, 2, 3, 4])),
                ),
            ]))
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

        let query = dbset
            .delete_query_sql_value(SqlValue::I64(7), None)
            .unwrap();

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
    fn dbset_delete_query_sql_value_appends_rowversion_predicate_when_present() {
        let dbset = DbSet::<VersionedEntity>::disconnected();

        let query = dbset
            .delete_query_sql_value(SqlValue::I64(7), Some(SqlValue::Bytes(vec![9, 8, 7])))
            .unwrap();

        assert_eq!(
            query,
            DeleteQuery::from_entity::<VersionedEntity>().filter(Predicate::and(vec![
                Predicate::eq(
                    Expr::Column(ColumnRef::new(
                        TableRef::new("dbo", "versioned_entities"),
                        "id",
                        "id",
                    )),
                    Expr::Value(SqlValue::I64(7)),
                ),
                Predicate::eq(
                    Expr::Column(ColumnRef::new(
                        TableRef::new("dbo", "versioned_entities"),
                        "version",
                        "version",
                    )),
                    Expr::Value(SqlValue::Bytes(vec![9, 8, 7])),
                ),
            ]))
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

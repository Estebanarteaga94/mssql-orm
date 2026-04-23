use crate::{DbContextEntitySet, DbSetQuery};
use core::future::Future;
use mssql_orm_core::{ColumnValue, Entity, FromRow, OrmError, SqlTypeMapping, SqlValue};

#[doc(hidden)]
pub trait EntityPrimaryKey: Entity {
    fn primary_key_value(&self) -> Result<SqlValue, OrmError>;
}

#[doc(hidden)]
pub enum EntityPersistMode {
    Insert,
    InsertOrUpdate(SqlValue),
    Update(SqlValue),
}

#[doc(hidden)]
pub trait EntityPersist: Entity {
    fn persist_mode(&self) -> Result<EntityPersistMode, OrmError>;
    fn insert_values(&self) -> Vec<ColumnValue>;
    fn update_changes(&self) -> Vec<ColumnValue>;
    fn concurrency_token(&self) -> Result<Option<SqlValue>, OrmError>;
    fn sync_persisted(&mut self, persisted: Self);
}

pub trait ActiveRecord: Entity + Sized {
    fn query<C>(db: &C) -> DbSetQuery<Self>
    where
        C: DbContextEntitySet<Self>,
    {
        db.db_set().query()
    }

    fn find<C, K>(db: &C, key: K) -> impl Future<Output = Result<Option<Self>, OrmError>> + Send
    where
        C: DbContextEntitySet<Self>,
        Self: FromRow + Send,
        K: SqlTypeMapping + Send,
    {
        db.db_set().find(key)
    }

    fn delete<C>(&self, db: &C) -> impl Future<Output = Result<bool, OrmError>> + Send
    where
        C: DbContextEntitySet<Self> + Sync,
        Self: EntityPrimaryKey + EntityPersist,
    {
        let key = <Self as EntityPrimaryKey>::primary_key_value(self);
        let concurrency_token = <Self as EntityPersist>::concurrency_token(self);

        async move {
            db.db_set()
                .delete_by_sql_value(key?, concurrency_token?)
                .await
        }
    }

    fn save<C>(&mut self, db: &C) -> impl Future<Output = Result<(), OrmError>> + Send
    where
        C: DbContextEntitySet<Self> + Sync,
        Self: EntityPersist + FromRow + Send,
    {
        async move {
            match <Self as EntityPersist>::persist_mode(self)? {
                EntityPersistMode::Insert => {
                    let persisted = db.db_set().insert_entity(self).await?;
                    <Self as EntityPersist>::sync_persisted(self, persisted);
                    Ok(())
                }
                EntityPersistMode::InsertOrUpdate(key) => {
                    if db.db_set().find_by_sql_value(key.clone()).await?.is_some() {
                        if let Some(persisted) = db
                            .db_set()
                            .update_entity_by_sql_value(
                                key,
                                self,
                                <Self as EntityPersist>::concurrency_token(self)?,
                            )
                            .await?
                        {
                            <Self as EntityPersist>::sync_persisted(self, persisted);
                        } else {
                            return Err(OrmError::new(
                                "ActiveRecord save detected a rowversion mismatch while updating the current entity",
                            ));
                        }
                    } else {
                        let persisted = db.db_set().insert_entity(self).await?;
                        <Self as EntityPersist>::sync_persisted(self, persisted);
                    }

                    Ok(())
                }
                EntityPersistMode::Update(key) => {
                    let persisted = db
                        .db_set()
                        .update_entity_by_sql_value(
                            key,
                            self,
                            <Self as EntityPersist>::concurrency_token(self)?,
                        )
                        .await?
                        .ok_or_else(|| {
                            OrmError::new(
                                "ActiveRecord save could not update a row for the current primary key",
                            )
                        })?;
                    <Self as EntityPersist>::sync_persisted(self, persisted);
                    Ok(())
                }
            }
        }
    }
}

impl<E: Entity> ActiveRecord for E {}

#[cfg(test)]
mod tests {
    use super::{ActiveRecord, EntityPersist, EntityPersistMode, EntityPrimaryKey};
    use crate::{DbContext, DbContextEntitySet, DbSet};
    use mssql_orm_core::{
        ColumnMetadata, ColumnValue, Entity, EntityMetadata, FromRow, OrmError, PrimaryKeyMetadata,
        Row, SqlServerType,
    };
    use mssql_orm_query::SelectQuery;

    #[derive(Debug, Clone, PartialEq)]
    struct TestEntity {
        id: i64,
        name: String,
    }

    static TEST_ENTITY_COLUMNS: [ColumnMetadata; 2] = [
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

    impl Entity for TestEntity {
        fn metadata() -> &'static EntityMetadata {
            &TEST_ENTITY_METADATA
        }
    }

    impl FromRow for TestEntity {
        fn from_row<R: Row>(_row: &R) -> Result<Self, OrmError> {
            Ok(Self {
                id: 7,
                name: "Persisted".to_string(),
            })
        }
    }

    impl EntityPrimaryKey for TestEntity {
        fn primary_key_value(&self) -> Result<mssql_orm_core::SqlValue, OrmError> {
            Ok(mssql_orm_core::SqlValue::I64(self.id))
        }
    }

    impl EntityPersist for TestEntity {
        fn persist_mode(&self) -> Result<EntityPersistMode, OrmError> {
            Ok(EntityPersistMode::Update(mssql_orm_core::SqlValue::I64(
                self.id,
            )))
        }

        fn insert_values(&self) -> Vec<ColumnValue> {
            vec![ColumnValue::new(
                "name",
                mssql_orm_core::SqlValue::String(self.name.clone()),
            )]
        }

        fn update_changes(&self) -> Vec<ColumnValue> {
            vec![ColumnValue::new(
                "name",
                mssql_orm_core::SqlValue::String(self.name.clone()),
            )]
        }

        fn concurrency_token(&self) -> Result<Option<mssql_orm_core::SqlValue>, OrmError> {
            Ok(None)
        }

        fn sync_persisted(&mut self, persisted: Self) {
            *self = persisted;
        }
    }

    struct DummyContext {
        entities: DbSet<TestEntity>,
    }

    impl DbContext for DummyContext {
        fn from_shared_connection(_connection: crate::SharedConnection) -> Self {
            unreachable!("DummyContext is only used in disconnected unit tests")
        }

        fn shared_connection(&self) -> crate::SharedConnection {
            panic!("DummyContext is only used in disconnected unit tests")
        }
    }

    impl DbContextEntitySet<TestEntity> for DummyContext {
        fn db_set(&self) -> &DbSet<TestEntity> {
            &self.entities
        }
    }

    #[test]
    fn active_record_query_delegates_to_typed_dbset() {
        let context = DummyContext {
            entities: DbSet::<TestEntity>::disconnected(),
        };

        let query = TestEntity::query(&context);

        assert_eq!(
            query.into_select_query(),
            SelectQuery::from_entity::<TestEntity>()
        );
    }

    #[test]
    fn active_record_trait_is_available_for_entities() {
        fn require_active_record<E: ActiveRecord>() {}

        require_active_record::<TestEntity>();
    }

    #[test]
    fn active_record_find_reuses_dbset_error_contract() {
        let context = DummyContext {
            entities: DbSet::<TestEntity>::disconnected(),
        };

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let error = match runtime.block_on(TestEntity::find(&context, 1_i64)) {
            Ok(value) => panic!("expected disconnected ActiveRecord::find to fail, got {value:?}"),
            Err(error) => error,
        };

        assert_eq!(
            error,
            OrmError::new("DbSetQuery requires an initialized shared connection")
        );
    }

    #[test]
    fn active_record_delete_reuses_dbset_error_contract() {
        let context = DummyContext {
            entities: DbSet::<TestEntity>::disconnected(),
        };
        let entity = TestEntity {
            id: 7,
            name: "Ana".to_string(),
        };

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let error = match runtime.block_on(entity.delete(&context)) {
            Ok(value) => {
                panic!("expected disconnected ActiveRecord::delete to fail, got {value:?}")
            }
            Err(error) => error,
        };

        assert_eq!(
            error,
            OrmError::new("DbSet requires an initialized shared connection")
        );
    }

    #[test]
    fn active_record_save_reuses_dbset_error_contract() {
        let context = DummyContext {
            entities: DbSet::<TestEntity>::disconnected(),
        };
        let mut entity = TestEntity {
            id: 7,
            name: "Ana".to_string(),
        };

        let runtime = tokio::runtime::Runtime::new().expect("tokio runtime");
        let error = match runtime.block_on(entity.save(&context)) {
            Ok(()) => panic!("expected disconnected ActiveRecord::save to fail"),
            Err(error) => error,
        };

        assert_eq!(
            error,
            OrmError::new("DbSet requires an initialized shared connection")
        );
    }
}

use crate::{DbContextEntitySet, DbSetQuery};
use core::future::Future;
use mssql_orm_core::{Entity, FromRow, OrmError, SqlTypeMapping};

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
}

impl<E: Entity> ActiveRecord for E {}

#[cfg(test)]
mod tests {
    use super::ActiveRecord;
    use crate::{DbContext, DbContextEntitySet, DbSet};
    use mssql_orm_core::{
        ColumnMetadata, Entity, EntityMetadata, FromRow, OrmError, PrimaryKeyMetadata, Row,
        SqlServerType,
    };
    use mssql_orm_query::SelectQuery;

    #[derive(Debug)]
    struct TestEntity;

    static TEST_ENTITY_COLUMNS: [ColumnMetadata; 1] = [ColumnMetadata {
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
    }];

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
            Ok(Self)
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

        assert_eq!(query.into_select_query(), SelectQuery::from_entity::<TestEntity>());
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
}

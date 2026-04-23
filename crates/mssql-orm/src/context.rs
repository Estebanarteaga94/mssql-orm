use crate::dbset_query::DbSetQuery;
use std::marker::PhantomData;
use std::sync::Arc;

use mssql_orm_core::{Entity, EntityMetadata, OrmError};
use mssql_orm_query::SelectQuery;
use mssql_orm_tiberius::{MssqlConnection, TokioConnectionStream};

pub type SharedConnection = Arc<tokio::sync::Mutex<MssqlConnection<TokioConnectionStream>>>;

pub trait DbContext: Sized {
    fn from_shared_connection(connection: SharedConnection) -> Self;
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

    pub fn shared_connection(&self) -> SharedConnection {
        Arc::clone(
            self.connection
                .as_ref()
                .expect("DbSet requires an initialized shared connection"),
        )
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

pub async fn connect_shared(connection_string: &str) -> Result<SharedConnection, OrmError> {
    let connection = MssqlConnection::connect(connection_string).await?;
    Ok(Arc::new(tokio::sync::Mutex::new(connection)))
}

#[cfg(test)]
mod tests {
    use super::DbSet;
    use mssql_orm_core::{Entity, EntityMetadata, PrimaryKeyMetadata};
    use mssql_orm_query::SelectQuery;

    struct TestEntity;

    static TEST_ENTITY_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "TestEntity",
        schema: "dbo",
        table: "test_entities",
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

    #[test]
    fn dbset_exposes_entity_metadata() {
        let dbset = DbSet::<TestEntity>::disconnected();

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
}

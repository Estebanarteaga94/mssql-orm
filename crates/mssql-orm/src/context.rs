use crate::dbset_query::DbSetQuery;
use std::marker::PhantomData;
use std::sync::Arc;

use mssql_orm_core::{Entity, EntityMetadata, FromRow, Insertable, OrmError, SqlTypeMapping};
use mssql_orm_query::{ColumnRef, Expr, InsertQuery, Predicate, SelectQuery, TableRef};
use mssql_orm_sqlserver::SqlServerCompiler;
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
        let shared_connection = self.shared_connection();
        let mut connection = shared_connection.lock().await;
        let inserted = connection.fetch_one(compiled).await?;

        inserted.ok_or_else(|| OrmError::new("insert query did not return a row"))
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

impl<E: Entity> DbSet<E> {
    fn find_select_query<K>(&self, key: K) -> Result<SelectQuery, OrmError>
    where
        K: SqlTypeMapping,
    {
        let metadata = E::metadata();
        let primary_key = metadata.primary_key_columns();

        if primary_key.len() != 1 {
            return Err(OrmError::new(
                "DbSet::find currently supports only entities with a single primary key column",
            ));
        }

        let column = primary_key[0];
        let predicate = Predicate::eq(
            Expr::Column(ColumnRef::new(
                TableRef::for_entity::<E>(),
                column.rust_field,
                column.column_name,
            )),
            Expr::Value(key.to_sql_value()),
        );

        Ok(SelectQuery::from_entity::<E>().filter(predicate))
    }

    fn insert_query<I>(&self, insertable: &I) -> InsertQuery
    where
        I: Insertable<E>,
    {
        InsertQuery::for_entity::<E, I>(insertable)
    }
}

pub async fn connect_shared(connection_string: &str) -> Result<SharedConnection, OrmError> {
    let connection = MssqlConnection::connect(connection_string).await?;
    Ok(Arc::new(tokio::sync::Mutex::new(connection)))
}

#[cfg(test)]
mod tests {
    use super::DbSet;
    use mssql_orm_core::{
        ColumnMetadata, ColumnValue, Entity, EntityMetadata, PrimaryKeyMetadata, SqlServerType,
        SqlValue,
    };
    use mssql_orm_query::{ColumnRef, Expr, InsertQuery, Predicate, SelectQuery, TableRef};

    struct TestEntity;
    struct CompositeKeyEntity;
    struct NewTestEntity {
        name: String,
        active: bool,
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

    impl mssql_orm_core::Insertable<TestEntity> for NewTestEntity {
        fn values(&self) -> Vec<ColumnValue> {
            vec![
                ColumnValue::new("name", SqlValue::String(self.name.clone())),
                ColumnValue::new("active", SqlValue::Bool(self.active)),
            ]
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
            "DbSet::find currently supports only entities with a single primary key column"
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
}

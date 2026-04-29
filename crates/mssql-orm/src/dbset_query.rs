use crate::context::{ActiveTenant, SharedConnection};
use crate::page_request::PageRequest;
use crate::query_projection::SelectProjections;
use crate::{IncludeNavigation, SoftDeleteEntity, TenantScopedEntity};
use mssql_orm_core::{
    ColumnMetadata, Entity, EntityMetadata, FromRow, NavigationKind, OrmError, Row, SqlServerType,
    SqlValue,
};
use mssql_orm_query::{
    ColumnRef, CountQuery, Expr, Join, JoinType, OrderBy, Pagination, Predicate, SelectProjection,
    SelectQuery, TableRef,
};
use mssql_orm_sqlserver::SqlServerCompiler;

#[derive(Clone)]
/// Fluent query builder bound to one `DbSet<E>`.
///
/// `DbSetQuery` stores query intent as AST until execution. SQL text is
/// generated only by the SQL Server compiler. Mandatory runtime policies such
/// as tenant filtering and root-entity soft-delete visibility are applied when
/// the query is compiled or executed.
pub struct DbSetQuery<E: Entity> {
    connection: Option<SharedConnection>,
    active_tenant: Option<ActiveTenant>,
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
        let active_tenant = connection
            .as_ref()
            .and_then(SharedConnection::active_tenant);
        Self {
            connection,
            active_tenant,
            select_query,
            visibility: SoftDeleteVisibility::Default,
            _entity: core::marker::PhantomData,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_active_tenant_for_test(mut self, active_tenant: ActiveTenant) -> Self {
        self.active_tenant = Some(active_tenant);
        self
    }

    /// Replaces the underlying `SelectQuery` AST while keeping this query bound
    /// to the same connection and runtime policies.
    pub fn with_select_query(mut self, select_query: SelectQuery) -> Self {
        self.select_query = select_query;
        self
    }

    /// Adds a predicate to the query.
    pub fn filter(mut self, predicate: Predicate) -> Self {
        self.select_query = self.select_query.filter(predicate);
        self
    }

    /// Adds an explicit join described by the query AST.
    pub fn join(mut self, join: Join) -> Self {
        self.select_query = self.select_query.join(join);
        self
    }

    /// Adds an explicit `INNER JOIN` to another entity.
    pub fn inner_join<J: Entity>(mut self, on: Predicate) -> Self {
        self.select_query = self.select_query.inner_join::<J>(on);
        self
    }

    /// Adds an explicit `LEFT JOIN` to another entity.
    pub fn left_join<J: Entity>(mut self, on: Predicate) -> Self {
        self.select_query = self.select_query.left_join::<J>(on);
        self
    }

    /// Adds an `INNER JOIN` inferred from navigation metadata.
    ///
    /// The navigation must be declared on the root entity `E`, and its target
    /// table must match `J`. This only builds the SQL join; it does not load or
    /// materialize the related entity.
    pub fn try_inner_join_navigation<J: Entity>(
        self,
        navigation: &'static str,
    ) -> Result<Self, OrmError> {
        self.try_join_navigation::<J>(navigation, JoinType::Inner, None)
    }

    /// Adds a `LEFT JOIN` inferred from navigation metadata.
    ///
    /// The navigation must be declared on the root entity `E`, and its target
    /// table must match `J`. This only builds the SQL join; it does not load or
    /// materialize the related entity.
    pub fn try_left_join_navigation<J: Entity>(
        self,
        navigation: &'static str,
    ) -> Result<Self, OrmError> {
        self.try_join_navigation::<J>(navigation, JoinType::Left, None)
    }

    /// Adds an aliased `INNER JOIN` inferred from navigation metadata.
    pub fn try_inner_join_navigation_as<J: Entity>(
        self,
        navigation: &'static str,
        alias: &'static str,
    ) -> Result<Self, OrmError> {
        self.try_join_navigation::<J>(navigation, JoinType::Inner, Some(alias))
    }

    /// Adds an aliased `LEFT JOIN` inferred from navigation metadata.
    pub fn try_left_join_navigation_as<J: Entity>(
        self,
        navigation: &'static str,
        alias: &'static str,
    ) -> Result<Self, OrmError> {
        self.try_join_navigation::<J>(navigation, JoinType::Left, Some(alias))
    }

    /// Includes a single related entity through a `belongs_to` or `has_one`
    /// navigation.
    ///
    /// This first eager-loading cut uses a left join and materializes the
    /// related row into `Navigation<J>`. Collection navigations (`has_many`)
    /// are intentionally rejected because they need grouping or split-query
    /// semantics.
    pub fn include<J: Entity>(
        self,
        navigation: &'static str,
    ) -> Result<DbSetQueryIncludeOne<E, J>, OrmError> {
        self.include_as::<J>(navigation, navigation)
    }

    /// Includes a single related entity using an explicit table alias.
    pub fn include_as<J: Entity>(
        self,
        navigation: &'static str,
        alias: &'static str,
    ) -> Result<DbSetQueryIncludeOne<E, J>, OrmError> {
        let metadata = E::metadata();
        let navigation_metadata = metadata.navigation(navigation).ok_or_else(|| {
            OrmError::new(format!(
                "entity `{}` does not declare navigation `{}`",
                metadata.rust_name, navigation
            ))
        })?;

        if !matches!(
            navigation_metadata.kind,
            NavigationKind::BelongsTo | NavigationKind::HasOne
        ) {
            return Err(OrmError::new(format!(
                "include only supports belongs_to and has_one navigations; `{}` is {:?}",
                navigation_metadata.rust_field, navigation_metadata.kind
            )));
        }

        Ok(DbSetQueryIncludeOne {
            query: self.try_join_navigation::<J>(navigation, JoinType::Left, Some(alias))?,
            navigation,
            alias,
            _target: core::marker::PhantomData,
        })
    }

    /// Adds an ordering expression.
    pub fn order_by(mut self, order: OrderBy) -> Self {
        self.select_query = self.select_query.order_by(order);
        self
    }

    /// Limits the number of returned rows with zero offset.
    pub fn limit(mut self, limit: u64) -> Self {
        self.select_query = self.select_query.paginate(Pagination::new(0, limit));
        self
    }

    /// Alias for `limit(...)`.
    pub fn take(self, limit: u64) -> Self {
        self.limit(limit)
    }

    /// Applies page-based pagination.
    pub fn paginate(mut self, request: PageRequest) -> Self {
        self.select_query = self.select_query.paginate(request.to_pagination());
        self
    }

    /// Selects an explicit projection instead of materializing full entities.
    ///
    /// Use `all_as::<T>()` or `first_as::<T>()` to materialize the projection
    /// into a DTO implementing `FromRow`.
    pub fn select<P>(mut self, projection: P) -> Self
    where
        P: SelectProjections,
    {
        self.select_query = self
            .select_query
            .select(projection.into_select_projections());
        self
    }

    #[cfg(test)]
    pub(crate) fn select_query(&self) -> &SelectQuery {
        &self.select_query
    }

    /// Includes logically deleted rows for entities with `soft_delete`.
    ///
    /// This affects only the root entity `E`, not every manually joined entity.
    pub fn with_deleted(mut self) -> Self {
        self.visibility = SoftDeleteVisibility::WithDeleted;
        self
    }

    /// Returns only logically deleted rows for entities with `soft_delete`.
    ///
    /// This affects only the root entity `E`, not every manually joined entity.
    pub fn only_deleted(mut self) -> Self {
        self.visibility = SoftDeleteVisibility::OnlyDeleted;
        self
    }

    #[cfg(test)]
    pub(crate) fn into_select_query(self) -> SelectQuery {
        self.select_query
    }

    /// Executes the query and materializes full entities.
    pub async fn all(self) -> Result<Vec<E>, OrmError>
    where
        E: FromRow + Send + SoftDeleteEntity + TenantScopedEntity,
    {
        let compiled = SqlServerCompiler::compile_select(&self.effective_select_query()?)?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await?;
        connection.fetch_all(compiled).await
    }

    /// Executes the query and materializes the first full entity, if any.
    pub async fn first(self) -> Result<Option<E>, OrmError>
    where
        E: FromRow + Send + SoftDeleteEntity + TenantScopedEntity,
    {
        let compiled = SqlServerCompiler::compile_select(&self.effective_select_query()?)?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await?;
        connection.fetch_one(compiled).await
    }

    /// Executes the query and materializes projected rows as DTOs.
    pub async fn all_as<T>(self) -> Result<Vec<T>, OrmError>
    where
        E: SoftDeleteEntity + TenantScopedEntity,
        T: FromRow + Send,
    {
        let compiled = SqlServerCompiler::compile_select(&self.effective_select_query()?)?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await?;
        connection.fetch_all(compiled).await
    }

    /// Executes the query and materializes the first projected DTO, if any.
    pub async fn first_as<T>(self) -> Result<Option<T>, OrmError>
    where
        E: SoftDeleteEntity + TenantScopedEntity,
        T: FromRow + Send,
    {
        let compiled = SqlServerCompiler::compile_select(&self.effective_select_query()?)?;
        let shared_connection = self.require_connection()?;
        let mut connection = shared_connection.lock().await?;
        connection.fetch_one(compiled).await
    }

    /// Executes the query as a `COUNT(*)` over the effective filters.
    pub async fn count(self) -> Result<i64, OrmError>
    where
        E: SoftDeleteEntity + TenantScopedEntity,
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
        E: SoftDeleteEntity + TenantScopedEntity,
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
        E: SoftDeleteEntity + TenantScopedEntity,
    {
        let mut query = self.select_query.clone();

        if let Some(predicate) = self.tenant_predicate()? {
            query = query.filter(predicate);
        }

        if let Some(predicate) = self.soft_delete_visibility_predicate()? {
            query = query.filter(predicate);
        }

        Ok(query)
    }

    fn tenant_predicate(&self) -> Result<Option<Predicate>, OrmError>
    where
        E: TenantScopedEntity,
    {
        let Some(policy) = E::tenant_policy() else {
            return Ok(None);
        };

        if policy.columns.len() != 1 {
            return Err(OrmError::new(
                "tenant query filter requires exactly one tenant policy column",
            ));
        }

        let tenant_column = &policy.columns[0];
        let active_tenant = self.active_tenant.as_ref().ok_or_else(|| {
            OrmError::new("tenant-scoped query requires an active tenant in the DbContext")
        })?;

        if active_tenant.column_name != tenant_column.column_name {
            return Err(OrmError::new(format!(
                "active tenant column `{}` does not match entity tenant column `{}`",
                active_tenant.column_name, tenant_column.column_name
            )));
        }

        if !tenant_value_matches_column_type(&active_tenant.value, tenant_column) {
            return Err(OrmError::new(format!(
                "active tenant value is not compatible with entity tenant column `{}`",
                tenant_column.column_name
            )));
        }

        Ok(Some(Predicate::eq(
            Expr::Column(ColumnRef::new(
                TableRef::for_entity::<E>(),
                tenant_column.rust_field,
                tenant_column.column_name,
            )),
            Expr::Value(active_tenant.value.clone()),
        )))
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

    fn try_join_navigation<J: Entity>(
        mut self,
        navigation: &'static str,
        join_type: JoinType,
        alias: Option<&'static str>,
    ) -> Result<Self, OrmError> {
        let join = self.navigation_join::<J>(navigation, join_type, alias)?;
        self.select_query = self.select_query.join(join);
        Ok(self)
    }

    fn navigation_join<J: Entity>(
        &self,
        navigation: &'static str,
        join_type: JoinType,
        alias: Option<&'static str>,
    ) -> Result<Join, OrmError> {
        let root_metadata = E::metadata();
        let target_metadata = J::metadata();
        let navigation = root_metadata.navigation(navigation).ok_or_else(|| {
            OrmError::new(format!(
                "entity `{}` does not declare navigation `{}`",
                root_metadata.rust_name, navigation
            ))
        })?;

        if navigation.target_schema != target_metadata.schema
            || navigation.target_table != target_metadata.table
        {
            return Err(OrmError::new(format!(
                "navigation `{}` on `{}` targets `{}.{}`, not entity `{}` (`{}.{}`)",
                navigation.rust_field,
                root_metadata.rust_name,
                navigation.target_schema,
                navigation.target_table,
                target_metadata.rust_name,
                target_metadata.schema,
                target_metadata.table
            )));
        }

        if navigation.local_columns.is_empty()
            || navigation.local_columns.len() != navigation.target_columns.len()
        {
            return Err(OrmError::new(format!(
                "navigation `{}` on `{}` has invalid join column metadata",
                navigation.rust_field, root_metadata.rust_name
            )));
        }

        let target_table = match alias {
            Some(alias) => TableRef::for_entity_as::<J>(alias),
            None => TableRef::for_entity::<J>(),
        };

        let predicates = navigation
            .local_columns
            .iter()
            .zip(navigation.target_columns.iter())
            .map(|(local_column, target_column)| {
                Ok(Predicate::eq(
                    metadata_column_expr(root_metadata, self.select_query.from, local_column)?,
                    metadata_column_expr(target_metadata, target_table, target_column)?,
                ))
            })
            .collect::<Result<Vec<_>, OrmError>>()?;

        let on = if predicates.len() == 1 {
            predicates[0].clone()
        } else {
            Predicate::and(predicates)
        };

        Ok(Join::new(join_type, target_table, on))
    }
}

/// Query builder returned by `DbSetQuery::include::<T>(...)` for a single
/// included navigation.
pub struct DbSetQueryIncludeOne<E: Entity, J: Entity> {
    query: DbSetQuery<E>,
    navigation: &'static str,
    alias: &'static str,
    _target: core::marker::PhantomData<fn() -> J>,
}

impl<E: Entity, J: Entity> DbSetQueryIncludeOne<E, J> {
    /// Executes the query and materializes root entities with one included
    /// navigation attached.
    pub async fn all(self) -> Result<Vec<E>, OrmError>
    where
        E: FromRow + IncludeNavigation<J> + Send + SoftDeleteEntity + TenantScopedEntity,
        J: FromRow + Send,
    {
        let navigation = self.navigation;
        let alias = self.alias;
        let compiled = SqlServerCompiler::compile_select(&self.effective_select_query()?)?;
        let shared_connection = self.query.require_connection()?;
        let mut connection = shared_connection.lock().await?;
        connection
            .fetch_all_with(compiled, move |row| {
                materialize_include_one::<E, J>(&row, navigation, alias)
            })
            .await
    }

    /// Executes the query and materializes the first root entity with one
    /// included navigation attached, if any.
    pub async fn first(self) -> Result<Option<E>, OrmError>
    where
        E: FromRow + IncludeNavigation<J> + Send + SoftDeleteEntity + TenantScopedEntity,
        J: FromRow + Send,
    {
        let navigation = self.navigation;
        let alias = self.alias;
        let compiled = SqlServerCompiler::compile_select(&self.effective_select_query()?)?;
        let shared_connection = self.query.require_connection()?;
        let mut connection = shared_connection.lock().await?;
        connection
            .fetch_one_with(compiled, move |row| {
                materialize_include_one::<E, J>(&row, navigation, alias)
            })
            .await
    }

    #[cfg(test)]
    pub(crate) fn select_query(&self) -> Result<SelectQuery, OrmError>
    where
        E: SoftDeleteEntity + TenantScopedEntity,
    {
        self.effective_select_query()
    }

    fn effective_select_query(&self) -> Result<SelectQuery, OrmError>
    where
        E: SoftDeleteEntity + TenantScopedEntity,
    {
        let query = self.query.effective_select_query()?;
        apply_include_projection::<E, J>(query, self.alias)
    }
}

fn apply_include_projection<E: Entity, J: Entity>(
    mut query: SelectQuery,
    alias: &'static str,
) -> Result<SelectQuery, OrmError> {
    let mut projection = Vec::new();

    projection.extend(E::metadata().columns.iter().map(|column| {
        SelectProjection::expr_as(
            Expr::Column(ColumnRef::new(
                query.from,
                column.rust_field,
                column.column_name,
            )),
            column.column_name,
        )
    }));

    let target_table = TableRef::for_entity_as::<J>(alias);
    for column in J::metadata().columns {
        projection.push(SelectProjection::expr_as(
            Expr::Column(ColumnRef::new(
                target_table,
                column.rust_field,
                column.column_name,
            )),
            include_column_alias(alias, column.column_name),
        ));
    }

    query.projection = projection;
    Ok(query)
}

fn materialize_include_one<E, J>(
    row: &impl Row,
    navigation: &'static str,
    alias: &'static str,
) -> Result<E, OrmError>
where
    E: FromRow + IncludeNavigation<J>,
    J: Entity + FromRow,
{
    let mut entity = E::from_row(row)?;
    let related = materialize_prefixed_entity::<J>(row, alias)?;
    entity.set_included_navigation(navigation, related)?;
    Ok(entity)
}

fn materialize_prefixed_entity<J: Entity + FromRow>(
    row: &impl Row,
    alias: &'static str,
) -> Result<Option<J>, OrmError> {
    let prefix = include_prefix(alias);
    let mut saw_value = false;

    for column in J::metadata().columns {
        let projected = prefixed_column_name(&prefix, column.column_name);
        if let Some(value) = row.try_get(&projected)? {
            if !value.is_null() {
                saw_value = true;
                break;
            }
        }
    }

    if !saw_value {
        return Ok(None);
    }

    Ok(Some(J::from_row(&PrefixedRow { row, prefix })?))
}

struct PrefixedRow<'a, R: Row + ?Sized> {
    row: &'a R,
    prefix: String,
}

impl<R: Row + ?Sized> Row for PrefixedRow<'_, R> {
    fn try_get(&self, column: &str) -> Result<Option<SqlValue>, OrmError> {
        self.row
            .try_get(&prefixed_column_name(&self.prefix, column))
    }
}

fn include_prefix(alias: &'static str) -> String {
    format!("{alias}__")
}

fn include_column_alias(alias: &'static str, column_name: &'static str) -> &'static str {
    Box::leak(format!("{alias}__{column_name}").into_boxed_str())
}

fn prefixed_column_name(prefix: &str, column_name: &str) -> String {
    format!("{prefix}{column_name}")
}

fn metadata_column_expr(
    metadata: &'static EntityMetadata,
    table: TableRef,
    column_name: &str,
) -> Result<Expr, OrmError> {
    let column = metadata.column(column_name).ok_or_else(|| {
        OrmError::new(format!(
            "entity `{}` metadata does not contain column `{}` required by navigation join",
            metadata.rust_name, column_name
        ))
    })?;

    Ok(Expr::Column(ColumnRef::new(
        table,
        column.rust_field,
        column.column_name,
    )))
}

pub(crate) fn tenant_value_matches_column_type(value: &SqlValue, column: &ColumnMetadata) -> bool {
    if value.is_null() {
        return false;
    }

    match column.sql_type {
        SqlServerType::BigInt => matches!(value, SqlValue::I64(_)),
        SqlServerType::Int | SqlServerType::SmallInt | SqlServerType::TinyInt => {
            matches!(value, SqlValue::I32(_))
        }
        SqlServerType::Bit => matches!(value, SqlValue::Bool(_)),
        SqlServerType::UniqueIdentifier => matches!(value, SqlValue::Uuid(_)),
        SqlServerType::Date => matches!(value, SqlValue::Date(_)),
        SqlServerType::DateTime2 => matches!(value, SqlValue::DateTime(_)),
        SqlServerType::Decimal | SqlServerType::Money => matches!(value, SqlValue::Decimal(_)),
        SqlServerType::Float => matches!(value, SqlValue::F64(_)),
        SqlServerType::NVarChar | SqlServerType::Custom(_) => {
            matches!(value, SqlValue::String(_))
        }
        SqlServerType::VarBinary | SqlServerType::RowVersion => matches!(value, SqlValue::Bytes(_)),
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
    use super::{DbSetQuery, tenant_value_matches_column_type};
    use crate::context::{ActiveTenant, DbSet};
    use crate::page_request::PageRequest;
    use crate::{SoftDeleteEntity, TenantScopedEntity};
    use mssql_orm_core::{
        ColumnMetadata, Entity, EntityColumn, EntityMetadata, EntityPolicyMetadata, FromRow,
        NavigationKind, NavigationMetadata, OrmError, PrimaryKeyMetadata, Row, SqlServerType,
        SqlValue,
    };
    use mssql_orm_query::{
        ColumnRef, Expr, Join, JoinType, OrderBy, Pagination, Predicate, SelectProjection,
        SelectQuery, SortDirection, TableRef,
    };
    use mssql_orm_sqlserver::SqlServerCompiler;

    struct TestEntity;
    struct JoinedEntity;
    struct NavigationRoot;
    struct NavigationTarget;
    struct SoftDeleteEntityUnderTest;
    struct BoolSoftDeleteEntity;
    struct TenantEntity;

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
        navigations: &[],
    };

    impl Entity for TestEntity {
        fn metadata() -> &'static EntityMetadata {
            &TEST_ENTITY_METADATA
        }
    }

    #[allow(non_upper_case_globals)]
    impl TestEntity {
        const id: EntityColumn<TestEntity> = EntityColumn::new("id", "id");
        const name: EntityColumn<TestEntity> = EntityColumn::new("name", "name");
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
        navigations: &[],
    };

    impl Entity for JoinedEntity {
        fn metadata() -> &'static EntityMetadata {
            &JOINED_ENTITY_METADATA
        }
    }

    static NAVIGATION_ROOT_COLUMNS: [ColumnMetadata; 1] = [ColumnMetadata {
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
        insertable: false,
        updatable: false,
        max_length: None,
        precision: None,
        scale: None,
    }];

    static NAVIGATION_TARGET_COLUMNS: [ColumnMetadata; 2] = [
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
            insertable: false,
            updatable: false,
            max_length: None,
            precision: None,
            scale: None,
        },
        ColumnMetadata {
            rust_field: "owner_id",
            column_name: "owner_id",
            renamed_from: None,
            sql_type: SqlServerType::BigInt,
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

    static NAVIGATION_ROOT_NAVIGATIONS: [NavigationMetadata; 1] = [NavigationMetadata::new(
        "orders",
        NavigationKind::HasMany,
        "NavigationTarget",
        "sales",
        "navigation_targets",
        &["id"],
        &["owner_id"],
        Some("fk_navigation_targets_owner"),
    )];

    static NAVIGATION_ROOT_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "NavigationRoot",
        schema: "dbo",
        table: "navigation_roots",
        renamed_from: None,
        columns: &NAVIGATION_ROOT_COLUMNS,
        primary_key: PrimaryKeyMetadata {
            name: None,
            columns: &["id"],
        },
        indexes: &[],
        foreign_keys: &[],
        navigations: &NAVIGATION_ROOT_NAVIGATIONS,
    };

    static NAVIGATION_TARGET_NAVIGATIONS: [NavigationMetadata; 1] = [NavigationMetadata::new(
        "owner",
        NavigationKind::BelongsTo,
        "NavigationRoot",
        "dbo",
        "navigation_roots",
        &["owner_id"],
        &["id"],
        Some("fk_navigation_targets_owner"),
    )];

    static NAVIGATION_TARGET_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "NavigationTarget",
        schema: "sales",
        table: "navigation_targets",
        renamed_from: None,
        columns: &NAVIGATION_TARGET_COLUMNS,
        primary_key: PrimaryKeyMetadata {
            name: None,
            columns: &["id"],
        },
        indexes: &[],
        foreign_keys: &[],
        navigations: &NAVIGATION_TARGET_NAVIGATIONS,
    };

    impl Entity for NavigationRoot {
        fn metadata() -> &'static EntityMetadata {
            &NAVIGATION_ROOT_METADATA
        }
    }

    impl Entity for NavigationTarget {
        fn metadata() -> &'static EntityMetadata {
            &NAVIGATION_TARGET_METADATA
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
        navigations: &[],
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
        navigations: &[],
    };

    static TENANT_POLICY_COLUMNS: [ColumnMetadata; 1] = [ColumnMetadata {
        rust_field: "tenant_id",
        column_name: "tenant_id",
        renamed_from: None,
        sql_type: SqlServerType::BigInt,
        nullable: false,
        primary_key: false,
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

    static TENANT_ENTITY_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "TenantEntity",
        schema: "sales",
        table: "tenant_entities",
        renamed_from: None,
        columns: &TENANT_POLICY_COLUMNS,
        primary_key: PrimaryKeyMetadata {
            name: None,
            columns: &[],
        },
        indexes: &[],
        foreign_keys: &[],
        navigations: &[],
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

    impl Entity for TenantEntity {
        fn metadata() -> &'static EntityMetadata {
            &TENANT_ENTITY_METADATA
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

    impl SoftDeleteEntity for TenantEntity {
        fn soft_delete_policy() -> Option<EntityPolicyMetadata> {
            None
        }
    }

    impl TenantScopedEntity for TestEntity {
        fn tenant_policy() -> Option<EntityPolicyMetadata> {
            None
        }
    }

    impl TenantScopedEntity for JoinedEntity {
        fn tenant_policy() -> Option<EntityPolicyMetadata> {
            None
        }
    }

    impl TenantScopedEntity for SoftDeleteEntityUnderTest {
        fn tenant_policy() -> Option<EntityPolicyMetadata> {
            None
        }
    }

    impl TenantScopedEntity for BoolSoftDeleteEntity {
        fn tenant_policy() -> Option<EntityPolicyMetadata> {
            None
        }
    }

    impl TenantScopedEntity for TenantEntity {
        fn tenant_policy() -> Option<EntityPolicyMetadata> {
            Some(EntityPolicyMetadata::new("tenant", &TENANT_POLICY_COLUMNS))
        }
    }

    impl SoftDeleteEntity for NavigationRoot {
        fn soft_delete_policy() -> Option<EntityPolicyMetadata> {
            None
        }
    }

    impl SoftDeleteEntity for NavigationTarget {
        fn soft_delete_policy() -> Option<EntityPolicyMetadata> {
            None
        }
    }

    impl TenantScopedEntity for NavigationRoot {
        fn tenant_policy() -> Option<EntityPolicyMetadata> {
            None
        }
    }

    impl TenantScopedEntity for NavigationTarget {
        fn tenant_policy() -> Option<EntityPolicyMetadata> {
            None
        }
    }

    #[derive(Debug)]
    struct TestProjectionRow;

    impl FromRow for TestProjectionRow {
        fn from_row<R: Row>(_row: &R) -> Result<Self, OrmError> {
            Ok(Self)
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
    fn dbset_query_select_builds_projection_with_aliases() {
        let dbset = DbSet::<TestEntity>::disconnected();

        let query = dbset
            .query()
            .select((TestEntity::id, TestEntity::name))
            .into_select_query();

        assert_eq!(
            query.projection,
            vec![
                SelectProjection::column(TestEntity::id),
                SelectProjection::column(TestEntity::name),
            ]
        );
    }

    #[tokio::test]
    async fn dbset_query_all_as_reuses_projection_compilation_before_connection() {
        let dbset = DbSet::<TestEntity>::disconnected();

        let error = dbset
            .query()
            .select(TestEntity::id)
            .all_as::<TestProjectionRow>()
            .await
            .unwrap_err();

        assert_eq!(
            error.message(),
            "DbSetQuery requires an initialized shared connection"
        );
    }

    #[tokio::test]
    async fn dbset_query_first_as_rejects_unaliased_expression_projection() {
        let dbset = DbSet::<TestEntity>::disconnected();

        let error = dbset
            .query()
            .select(Expr::function("LOWER", vec![Expr::from(TestEntity::name)]))
            .first_as::<TestProjectionRow>()
            .await
            .unwrap_err();

        assert_eq!(
            error.message(),
            "SQL Server projection expressions require an explicit alias"
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
    fn dbset_query_applies_active_tenant_filter_for_tenant_scoped_entities() {
        let query = DbSetQuery::<TenantEntity>::new(
            None,
            SelectQuery::from_entity::<TenantEntity>().filter(Predicate::eq(
                Expr::value(SqlValue::Bool(true)),
                Expr::value(SqlValue::Bool(true)),
            )),
        )
        .with_active_tenant_for_test(ActiveTenant {
            column_name: "tenant_id",
            value: SqlValue::I64(42),
        })
        .effective_select_query()
        .unwrap();

        assert_eq!(
            query,
            SelectQuery::from_entity::<TenantEntity>()
                .filter(Predicate::eq(
                    Expr::value(SqlValue::Bool(true)),
                    Expr::value(SqlValue::Bool(true)),
                ))
                .filter(Predicate::eq(
                    Expr::Column(mssql_orm_query::ColumnRef::new(
                        TableRef::new("sales", "tenant_entities"),
                        "tenant_id",
                        "tenant_id",
                    )),
                    Expr::Value(SqlValue::I64(42)),
                ))
        );
    }

    #[test]
    fn tenant_security_guardrail_keeps_joined_read_sql_tenant_scoped() {
        let query = DbSetQuery::<TenantEntity>::new(
            None,
            SelectQuery::from_entity::<TenantEntity>().inner_join::<JoinedEntity>(Predicate::eq(
                Expr::value(SqlValue::Bool(true)),
                Expr::value(SqlValue::Bool(true)),
            )),
        )
        .with_active_tenant_for_test(ActiveTenant {
            column_name: "tenant_id",
            value: SqlValue::I64(42),
        })
        .effective_select_query()
        .unwrap();

        let compiled = SqlServerCompiler::compile_select(&query).unwrap();

        assert!(
            compiled.sql.contains("INNER JOIN [dbo].[joined_entities]"),
            "joined tenant read should preserve explicit joins: {}",
            compiled.sql
        );
        assert!(
            compiled
                .sql
                .contains("[sales].[tenant_entities].[tenant_id] = @P"),
            "joined tenant read must include tenant predicate on the root entity: {}",
            compiled.sql
        );
        assert!(
            compiled.params.contains(&SqlValue::I64(42)),
            "joined tenant read params must include active tenant value: {:?}",
            compiled.params
        );
    }

    #[test]
    fn dbset_query_fails_closed_without_active_tenant_for_tenant_scoped_entities() {
        let error =
            DbSetQuery::<TenantEntity>::new(None, SelectQuery::from_entity::<TenantEntity>())
                .effective_select_query()
                .unwrap_err();

        assert!(
            error
                .message()
                .contains("requires an active tenant in the DbContext")
        );
    }

    #[test]
    fn dbset_query_rejects_mismatched_active_tenant_column() {
        let error =
            DbSetQuery::<TenantEntity>::new(None, SelectQuery::from_entity::<TenantEntity>())
                .with_active_tenant_for_test(ActiveTenant {
                    column_name: "company_id",
                    value: SqlValue::I64(42),
                })
                .effective_select_query()
                .unwrap_err();

        assert!(error.message().contains("does not match"));
    }

    #[test]
    fn dbset_query_rejects_incompatible_active_tenant_value() {
        let error =
            DbSetQuery::<TenantEntity>::new(None, SelectQuery::from_entity::<TenantEntity>())
                .with_active_tenant_for_test(ActiveTenant {
                    column_name: "tenant_id",
                    value: SqlValue::String("not-a-bigint".to_string()),
                })
                .effective_select_query()
                .unwrap_err();

        assert!(error.message().contains("not compatible"));
    }

    #[test]
    fn tenant_value_type_matching_rejects_null_even_for_nullable_columns() {
        assert!(!tenant_value_matches_column_type(
            &SqlValue::Null,
            &TENANT_POLICY_COLUMNS[0],
        ));
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
    fn dbset_query_infers_navigation_join_from_metadata() {
        let dbset = DbSet::<NavigationRoot>::disconnected();

        let select = dbset
            .query()
            .try_inner_join_navigation::<NavigationTarget>("orders")
            .unwrap()
            .into_select_query();

        assert_eq!(select.joins.len(), 1);
        assert_eq!(select.joins[0].join_type, JoinType::Inner);
        assert_eq!(
            select.joins[0].table,
            TableRef::new("sales", "navigation_targets")
        );
        assert_eq!(
            select.joins[0].on,
            Predicate::eq(
                Expr::Column(ColumnRef::new(
                    TableRef::new("dbo", "navigation_roots"),
                    "id",
                    "id",
                )),
                Expr::Column(ColumnRef::new(
                    TableRef::new("sales", "navigation_targets"),
                    "owner_id",
                    "owner_id",
                )),
            )
        );
    }

    #[test]
    fn dbset_query_infers_aliased_navigation_join_from_metadata() {
        let dbset = DbSet::<NavigationRoot>::disconnected();

        let select = dbset
            .query()
            .try_left_join_navigation_as::<NavigationTarget>("orders", "orders")
            .unwrap()
            .into_select_query();

        assert_eq!(select.joins.len(), 1);
        assert_eq!(select.joins[0].join_type, JoinType::Left);
        assert_eq!(
            select.joins[0].table,
            TableRef::with_alias("sales", "navigation_targets", "orders")
        );
        assert_eq!(
            select.joins[0].on,
            Predicate::eq(
                Expr::Column(ColumnRef::new(
                    TableRef::new("dbo", "navigation_roots"),
                    "id",
                    "id",
                )),
                Expr::Column(ColumnRef::new(
                    TableRef::with_alias("sales", "navigation_targets", "orders"),
                    "owner_id",
                    "owner_id",
                )),
            )
        );
    }

    #[test]
    fn dbset_query_rejects_unknown_navigation_join() {
        let error = DbSet::<NavigationRoot>::disconnected()
            .query()
            .try_inner_join_navigation::<NavigationTarget>("missing")
            .unwrap_err();

        assert!(
            error
                .message()
                .contains("does not declare navigation `missing`")
        );
    }

    #[test]
    fn dbset_query_rejects_navigation_join_target_mismatch() {
        let error = DbSet::<NavigationRoot>::disconnected()
            .query()
            .try_inner_join_navigation::<JoinedEntity>("orders")
            .unwrap_err();

        assert!(
            error
                .message()
                .contains("targets `sales.navigation_targets`")
        );
    }

    #[test]
    fn dbset_query_include_projects_root_and_prefixed_related_columns() {
        let include = DbSet::<NavigationTarget>::disconnected()
            .query()
            .include_as::<NavigationRoot>("owner", "owner")
            .unwrap();

        let select = include.select_query().unwrap();

        assert_eq!(select.joins.len(), 1);
        assert_eq!(select.joins[0].join_type, JoinType::Left);
        assert_eq!(
            select.joins[0].table,
            TableRef::with_alias("dbo", "navigation_roots", "owner")
        );
        assert_eq!(select.projection.len(), 3);
        assert_eq!(select.projection[0].alias, Some("id"));
        assert_eq!(select.projection[1].alias, Some("owner_id"));
        assert_eq!(select.projection[2].alias, Some("owner__id"));
    }

    #[test]
    fn dbset_query_include_rejects_collection_navigation() {
        let result = DbSet::<NavigationRoot>::disconnected()
            .query()
            .include::<NavigationTarget>("orders");
        let error = match result {
            Ok(_) => panic!("expected collection include to be rejected"),
            Err(error) => error,
        };

        assert!(error.message().contains("belongs_to and has_one"));
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

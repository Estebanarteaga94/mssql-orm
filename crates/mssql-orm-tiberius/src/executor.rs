use crate::connection::{MssqlConnection, run_with_timeout};
use crate::error::{TiberiusErrorContext, map_tiberius_error};
use crate::parameter::PreparedQuery;
use crate::row::MssqlRow;
use crate::transaction::MssqlTransaction;
use async_trait::async_trait;
use futures_io::{AsyncRead, AsyncWrite};
use mssql_orm_core::{FromRow, OrmError};
use mssql_orm_query::CompiledQuery;
use tiberius::Client;
use tiberius::QueryStream;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecuteResult {
    rows_affected: Vec<u64>,
}

impl ExecuteResult {
    pub fn new(rows_affected: Vec<u64>) -> Self {
        Self { rows_affected }
    }

    pub fn rows_affected(&self) -> &[u64] {
        &self.rows_affected
    }

    pub fn total(&self) -> u64 {
        self.rows_affected.iter().sum()
    }
}

#[async_trait]
pub trait Executor {
    async fn execute(&mut self, query: CompiledQuery) -> Result<ExecuteResult, OrmError>;
    async fn fetch_one<T>(&mut self, query: CompiledQuery) -> Result<Option<T>, OrmError>
    where
        T: FromRow + Send;
    async fn fetch_all<T>(&mut self, query: CompiledQuery) -> Result<Vec<T>, OrmError>
    where
        T: FromRow + Send;
}

#[async_trait]
impl<S> Executor for MssqlConnection<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    async fn execute(&mut self, query: CompiledQuery) -> Result<ExecuteResult, OrmError> {
        MssqlConnection::execute(self, query).await
    }

    async fn fetch_one<T>(&mut self, query: CompiledQuery) -> Result<Option<T>, OrmError>
    where
        T: FromRow + Send,
    {
        MssqlConnection::fetch_one(self, query).await
    }

    async fn fetch_all<T>(&mut self, query: CompiledQuery) -> Result<Vec<T>, OrmError>
    where
        T: FromRow + Send,
    {
        MssqlConnection::fetch_all(self, query).await
    }
}

#[async_trait]
impl<S> Executor for MssqlTransaction<'_, S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    async fn execute(&mut self, query: CompiledQuery) -> Result<ExecuteResult, OrmError> {
        MssqlTransaction::execute(self, query).await
    }

    async fn fetch_one<T>(&mut self, query: CompiledQuery) -> Result<Option<T>, OrmError>
    where
        T: FromRow + Send,
    {
        MssqlTransaction::fetch_one(self, query).await
    }

    async fn fetch_all<T>(&mut self, query: CompiledQuery) -> Result<Vec<T>, OrmError>
    where
        T: FromRow + Send,
    {
        MssqlTransaction::fetch_all(self, query).await
    }
}

impl<S> MssqlConnection<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    pub async fn execute(&mut self, query: CompiledQuery) -> Result<ExecuteResult, OrmError> {
        run_with_timeout(self.query_timeout(), "SQL Server query timed out", async {
            execute_compiled(self.client_mut(), query).await
        })
        .await
    }

    pub async fn query_raw<'a>(
        &'a mut self,
        query: CompiledQuery,
    ) -> Result<QueryStream<'a>, OrmError> {
        let query_timeout = self.query_timeout();
        run_with_timeout(query_timeout, "SQL Server query timed out", async {
            query_raw_compiled(self.client_mut(), query).await
        })
        .await
    }

    pub async fn fetch_one<T>(&mut self, query: CompiledQuery) -> Result<Option<T>, OrmError>
    where
        T: FromRow + Send,
    {
        run_with_timeout(self.query_timeout(), "SQL Server query timed out", async {
            fetch_one_compiled(self.client_mut(), query).await
        })
        .await
    }

    pub async fn fetch_all<T>(&mut self, query: CompiledQuery) -> Result<Vec<T>, OrmError>
    where
        T: FromRow + Send,
    {
        run_with_timeout(self.query_timeout(), "SQL Server query timed out", async {
            fetch_all_compiled(self.client_mut(), query).await
        })
        .await
    }
}

pub(crate) async fn execute_compiled<S>(
    client: &mut Client<S>,
    query: CompiledQuery,
) -> Result<ExecuteResult, OrmError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    let prepared = PreparedQuery::from_compiled(query);
    prepared.validate_parameter_count()?;

    let result = prepared.execute(client).await?;

    Ok(ExecuteResult::new(result.rows_affected().to_vec()))
}

pub(crate) async fn query_raw_compiled<'a, S>(
    client: &'a mut Client<S>,
    query: CompiledQuery,
) -> Result<QueryStream<'a>, OrmError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    let prepared = PreparedQuery::from_compiled(query);
    prepared.validate_parameter_count()?;

    prepared.query(client).await
}

pub(crate) async fn fetch_one_compiled<S, T>(
    client: &mut Client<S>,
    query: CompiledQuery,
) -> Result<Option<T>, OrmError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
    T: FromRow + Send,
{
    let row = query_raw_compiled(client, query)
        .await?
        .into_row()
        .await
        .map_err(|error| map_tiberius_error(&error, TiberiusErrorContext::ExecuteQuery))?;

    row.as_ref()
        .map(|row| T::from_row(&MssqlRow::new(row)))
        .transpose()
}

pub(crate) async fn fetch_all_compiled<S, T>(
    client: &mut Client<S>,
    query: CompiledQuery,
) -> Result<Vec<T>, OrmError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
    T: FromRow + Send,
{
    let rows = query_raw_compiled(client, query)
        .await?
        .into_first_result()
        .await
        .map_err(|error| map_tiberius_error(&error, TiberiusErrorContext::ExecuteQuery))?;

    rows.iter()
        .map(|row| T::from_row(&MssqlRow::new(row)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{ExecuteResult, fetch_all_compiled, fetch_one_compiled, query_raw_compiled};
    use mssql_orm_core::{FromRow, OrmError, Row};

    struct TestRowModel;

    impl FromRow for TestRowModel {
        fn from_row<R: Row>(_row: &R) -> Result<Self, OrmError> {
            Ok(Self)
        }
    }

    #[test]
    fn execute_result_exposes_rows_affected_and_total() {
        let result = ExecuteResult::new(vec![1, 2, 3]);

        assert_eq!(result.rows_affected(), &[1, 2, 3]);
        assert_eq!(result.total(), 6);
    }

    #[test]
    fn reuses_shared_execution_helpers_from_transaction_boundary() {
        let query_raw = query_raw_compiled::<tokio_util::compat::Compat<tokio::net::TcpStream>>;
        let fetch_one =
            fetch_one_compiled::<tokio_util::compat::Compat<tokio::net::TcpStream>, TestRowModel>;
        let fetch_all =
            fetch_all_compiled::<tokio_util::compat::Compat<tokio::net::TcpStream>, TestRowModel>;

        let _ = (query_raw, fetch_one, fetch_all);
    }
}

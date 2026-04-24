use crate::config::{MssqlSlowQueryOptions, MssqlTracingOptions};
use crate::connection::{MssqlConnection, run_with_timeout};
use crate::error::{TiberiusErrorContext, map_tiberius_error};
use crate::parameter::PreparedQuery;
use crate::row::MssqlRow;
use crate::telemetry::{QueryTrace, trace_query};
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
        let tracing_options = self.tracing_options();
        let slow_query_options = self.slow_query_options();
        let server_addr = self.server_addr();
        let query_timeout = self.query_timeout();
        run_with_timeout(self.query_timeout(), "SQL Server query timed out", async {
            execute_compiled(
                self.client_mut(),
                query,
                tracing_options,
                slow_query_options,
                &server_addr,
                query_timeout,
            )
            .await
        })
        .await
    }

    pub async fn query_raw<'a>(
        &'a mut self,
        query: CompiledQuery,
    ) -> Result<QueryStream<'a>, OrmError> {
        let tracing_options = self.tracing_options();
        let slow_query_options = self.slow_query_options();
        let server_addr = self.server_addr();
        let query_timeout = self.query_timeout();
        run_with_timeout(query_timeout, "SQL Server query timed out", async {
            query_raw_compiled(
                self.client_mut(),
                query,
                tracing_options,
                slow_query_options,
                &server_addr,
                query_timeout,
            )
            .await
        })
        .await
    }

    pub async fn fetch_one<T>(&mut self, query: CompiledQuery) -> Result<Option<T>, OrmError>
    where
        T: FromRow + Send,
    {
        let tracing_options = self.tracing_options();
        let slow_query_options = self.slow_query_options();
        let server_addr = self.server_addr();
        let query_timeout = self.query_timeout();
        run_with_timeout(self.query_timeout(), "SQL Server query timed out", async {
            fetch_one_compiled(
                self.client_mut(),
                query,
                tracing_options,
                slow_query_options,
                &server_addr,
                query_timeout,
            )
            .await
        })
        .await
    }

    pub async fn fetch_all<T>(&mut self, query: CompiledQuery) -> Result<Vec<T>, OrmError>
    where
        T: FromRow + Send,
    {
        let tracing_options = self.tracing_options();
        let slow_query_options = self.slow_query_options();
        let server_addr = self.server_addr();
        let query_timeout = self.query_timeout();
        run_with_timeout(self.query_timeout(), "SQL Server query timed out", async {
            fetch_all_compiled(
                self.client_mut(),
                query,
                tracing_options,
                slow_query_options,
                &server_addr,
                query_timeout,
            )
            .await
        })
        .await
    }
}

pub(crate) async fn execute_compiled<S>(
    client: &mut Client<S>,
    query: CompiledQuery,
    tracing_options: MssqlTracingOptions,
    slow_query_options: MssqlSlowQueryOptions,
    server_addr: &str,
    query_timeout: Option<std::time::Duration>,
) -> Result<ExecuteResult, OrmError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    let prepared = PreparedQuery::from_compiled(query);
    let trace = QueryTrace::new(server_addr, query_timeout, tracing_options, &prepared);
    let result = trace_query(tracing_options, slow_query_options, trace, async {
        prepared.validate_parameter_count()?;
        prepared.execute(client).await
    })
    .await?;

    Ok(ExecuteResult::new(result.rows_affected().to_vec()))
}

pub(crate) async fn query_raw_compiled<'a, S>(
    client: &'a mut Client<S>,
    query: CompiledQuery,
    tracing_options: MssqlTracingOptions,
    slow_query_options: MssqlSlowQueryOptions,
    server_addr: &str,
    query_timeout: Option<std::time::Duration>,
) -> Result<QueryStream<'a>, OrmError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    let prepared = PreparedQuery::from_compiled(query);
    let trace = QueryTrace::new(server_addr, query_timeout, tracing_options, &prepared);
    trace_query(tracing_options, slow_query_options, trace, async {
        prepared.validate_parameter_count()?;
        prepared.query(client).await
    })
    .await
}

pub(crate) async fn fetch_one_compiled<S, T>(
    client: &mut Client<S>,
    query: CompiledQuery,
    tracing_options: MssqlTracingOptions,
    slow_query_options: MssqlSlowQueryOptions,
    server_addr: &str,
    query_timeout: Option<std::time::Duration>,
) -> Result<Option<T>, OrmError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
    T: FromRow + Send,
{
    let row = query_raw_compiled(
        client,
        query,
        tracing_options,
        slow_query_options,
        server_addr,
        query_timeout,
    )
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
    tracing_options: MssqlTracingOptions,
    slow_query_options: MssqlSlowQueryOptions,
    server_addr: &str,
    query_timeout: Option<std::time::Duration>,
) -> Result<Vec<T>, OrmError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
    T: FromRow + Send,
{
    let rows = query_raw_compiled(
        client,
        query,
        tracing_options,
        slow_query_options,
        server_addr,
        query_timeout,
    )
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
    use crate::config::{MssqlSlowQueryOptions, MssqlTracingOptions};
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

    #[test]
    fn compiled_query_helpers_accept_tracing_context_shape() {
        let tracing = MssqlTracingOptions::enabled();
        let slow_query = MssqlSlowQueryOptions::enabled(std::time::Duration::from_millis(250));

        assert!(tracing.enabled);
        assert!(slow_query.enabled);
    }
}

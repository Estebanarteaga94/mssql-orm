use crate::connection::MssqlConnection;
use crate::error::{TiberiusErrorContext, map_tiberius_error};
use crate::parameter::PreparedQuery;
use crate::row::MssqlRow;
use async_trait::async_trait;
use futures_io::{AsyncRead, AsyncWrite};
use mssql_orm_core::{FromRow, OrmError};
use mssql_orm_query::CompiledQuery;
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

impl<S> MssqlConnection<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    pub async fn execute(&mut self, query: CompiledQuery) -> Result<ExecuteResult, OrmError> {
        let prepared = PreparedQuery::from_compiled(query);
        prepared.validate_parameter_count()?;

        let result = prepared.execute(self.client_mut()).await?;

        Ok(ExecuteResult::new(result.rows_affected().to_vec()))
    }

    pub async fn query_raw<'a>(
        &'a mut self,
        query: CompiledQuery,
    ) -> Result<QueryStream<'a>, OrmError> {
        let prepared = PreparedQuery::from_compiled(query);
        prepared.validate_parameter_count()?;

        prepared.query(self.client_mut()).await
    }

    pub async fn fetch_one<T>(&mut self, query: CompiledQuery) -> Result<Option<T>, OrmError>
    where
        T: FromRow + Send,
    {
        let row = self
            .query_raw(query)
            .await?
            .into_row()
            .await
            .map_err(|error| map_tiberius_error(&error, TiberiusErrorContext::ExecuteQuery))?;

        row.as_ref()
            .map(|row| T::from_row(&MssqlRow::new(row)))
            .transpose()
    }

    pub async fn fetch_all<T>(&mut self, query: CompiledQuery) -> Result<Vec<T>, OrmError>
    where
        T: FromRow + Send,
    {
        let rows = self
            .query_raw(query)
            .await?
            .into_first_result()
            .await
            .map_err(|error| map_tiberius_error(&error, TiberiusErrorContext::ExecuteQuery))?;

        rows.iter()
            .map(|row| T::from_row(&MssqlRow::new(row)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::ExecuteResult;

    #[test]
    fn execute_result_exposes_rows_affected_and_total() {
        let result = ExecuteResult::new(vec![1, 2, 3]);

        assert_eq!(result.rows_affected(), &[1, 2, 3]);
        assert_eq!(result.total(), 6);
    }
}

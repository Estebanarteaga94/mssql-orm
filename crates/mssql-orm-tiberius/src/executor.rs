use crate::connection::MssqlConnection;
use crate::parameter::PreparedQuery;
use async_trait::async_trait;
use futures_io::{AsyncRead, AsyncWrite};
use mssql_orm_core::OrmError;
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
}

#[async_trait]
impl<S> Executor for MssqlConnection<S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    async fn execute(&mut self, query: CompiledQuery) -> Result<ExecuteResult, OrmError> {
        MssqlConnection::execute(self, query).await
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

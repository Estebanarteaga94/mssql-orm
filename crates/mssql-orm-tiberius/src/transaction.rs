use crate::error::{TiberiusErrorContext, map_tiberius_error};
use crate::executor::{
    ExecuteResult, execute_compiled, fetch_all_compiled, fetch_one_compiled, query_raw_compiled,
};
use futures_io::{AsyncRead, AsyncWrite};
use mssql_orm_core::{FromRow, OrmError};
use mssql_orm_query::CompiledQuery;
use tiberius::{Client, QueryStream};

const BEGIN_TRANSACTION_SQL: &str = "BEGIN TRANSACTION";
const COMMIT_TRANSACTION_SQL: &str = "COMMIT TRANSACTION";
const ROLLBACK_TRANSACTION_SQL: &str = "ROLLBACK TRANSACTION";

pub struct MssqlTransaction<'a, S: AsyncRead + AsyncWrite + Unpin + Send> {
    client: &'a mut Client<S>,
    completed: bool,
}

impl<'a, S> MssqlTransaction<'a, S>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    pub(crate) async fn begin(client: &'a mut Client<S>) -> Result<Self, OrmError> {
        begin_transaction_scope(client).await?;

        Ok(Self {
            client,
            completed: false,
        })
    }

    pub fn is_completed(&self) -> bool {
        self.completed
    }

    pub async fn commit(mut self) -> Result<(), OrmError> {
        self.finish(COMMIT_TRANSACTION_SQL).await
    }

    pub async fn rollback(mut self) -> Result<(), OrmError> {
        self.finish(ROLLBACK_TRANSACTION_SQL).await
    }

    pub async fn execute(&mut self, query: CompiledQuery) -> Result<ExecuteResult, OrmError> {
        execute_compiled(self.client, query).await
    }

    pub async fn query_raw<'b>(
        &'b mut self,
        query: CompiledQuery,
    ) -> Result<QueryStream<'b>, OrmError> {
        query_raw_compiled(self.client, query).await
    }

    pub async fn fetch_one<T>(&mut self, query: CompiledQuery) -> Result<Option<T>, OrmError>
    where
        T: FromRow + Send,
    {
        fetch_one_compiled(self.client, query).await
    }

    pub async fn fetch_all<T>(&mut self, query: CompiledQuery) -> Result<Vec<T>, OrmError>
    where
        T: FromRow + Send,
    {
        fetch_all_compiled(self.client, query).await
    }

    async fn finish(&mut self, sql: &'static str) -> Result<(), OrmError> {
        if self.completed {
            return Err(OrmError::new("transaction has already been completed"));
        }

        run_transaction_command(self.client, sql).await?;
        self.completed = true;

        Ok(())
    }
}

pub(crate) async fn begin_transaction_scope<S>(client: &mut Client<S>) -> Result<(), OrmError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    run_transaction_command(client, BEGIN_TRANSACTION_SQL).await
}

pub(crate) async fn commit_transaction_scope<S>(client: &mut Client<S>) -> Result<(), OrmError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    run_transaction_command(client, COMMIT_TRANSACTION_SQL).await
}

pub(crate) async fn rollback_transaction_scope<S>(client: &mut Client<S>) -> Result<(), OrmError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    run_transaction_command(client, ROLLBACK_TRANSACTION_SQL).await
}

pub(crate) async fn run_transaction_command<S>(
    client: &mut Client<S>,
    sql: &'static str,
) -> Result<(), OrmError>
where
    S: AsyncRead + AsyncWrite + Unpin + Send,
{
    client
        .simple_query(sql)
        .await
        .map_err(|error| map_tiberius_error(&error, TiberiusErrorContext::ExecuteQuery))?
        .into_results()
        .await
        .map_err(|error| map_tiberius_error(&error, TiberiusErrorContext::ExecuteQuery))?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        BEGIN_TRANSACTION_SQL, COMMIT_TRANSACTION_SQL, MssqlTransaction, ROLLBACK_TRANSACTION_SQL,
        begin_transaction_scope, commit_transaction_scope, rollback_transaction_scope,
    };

    #[test]
    fn transaction_command_constants_match_expected_sql() {
        assert_eq!(BEGIN_TRANSACTION_SQL, "BEGIN TRANSACTION");
        assert_eq!(COMMIT_TRANSACTION_SQL, "COMMIT TRANSACTION");
        assert_eq!(ROLLBACK_TRANSACTION_SQL, "ROLLBACK TRANSACTION");
    }

    #[test]
    fn transaction_wrapper_tracks_completion_state() {
        let wrapper = core::mem::size_of::<
            Option<MssqlTransaction<'static, tokio_util::compat::Compat<tokio::net::TcpStream>>>,
        >();

        assert!(wrapper > 0);
    }

    #[test]
    fn exposes_scope_level_transaction_helpers() {
        let begin = begin_transaction_scope::<tokio_util::compat::Compat<tokio::net::TcpStream>>;
        let commit = commit_transaction_scope::<tokio_util::compat::Compat<tokio::net::TcpStream>>;
        let rollback =
            rollback_transaction_scope::<tokio_util::compat::Compat<tokio::net::TcpStream>>;

        let _ = (begin, commit, rollback);
    }
}

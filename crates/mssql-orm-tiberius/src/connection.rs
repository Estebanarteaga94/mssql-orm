use crate::config::MssqlConnectionConfig;
use crate::error::{TiberiusErrorContext, map_tiberius_error};
use crate::telemetry::trace_connection;
use crate::transaction::{
    MssqlTransaction, begin_transaction_scope, commit_transaction_scope, rollback_transaction_scope,
};
use futures_io::{AsyncRead, AsyncWrite};
use mssql_orm_core::OrmError;
use std::time::Duration;
use tiberius::Client;
use tokio::net::TcpStream;
use tokio::time::timeout;
use tokio_util::compat::{Compat, TokioAsyncWriteCompatExt};

pub type TokioConnectionStream = Compat<TcpStream>;

pub struct MssqlConnection<S: AsyncRead + AsyncWrite + Unpin + Send = TokioConnectionStream> {
    client: Client<S>,
    config: MssqlConnectionConfig,
}

impl MssqlConnection<TokioConnectionStream> {
    pub async fn connect(connection_string: &str) -> Result<Self, OrmError> {
        let config = MssqlConnectionConfig::from_connection_string(connection_string)?;
        Self::connect_with_config(config).await
    }

    pub async fn connect_with_config(config: MssqlConnectionConfig) -> Result<Self, OrmError> {
        let tracing_options = config.options().tracing;
        let connect_timeout = config.options().timeouts.connect_timeout;
        let addr = config.addr();
        let trace_addr = addr.clone();
        let tiberius_config = config.tiberius_config().clone();

        let client = trace_connection(tracing_options, &trace_addr, connect_timeout, async {
            run_with_timeout(connect_timeout, "SQL Server connection timed out", async {
                let tcp = TcpStream::connect(addr).await.map_err(|error| {
                    map_tiberius_error(&error.into(), TiberiusErrorContext::ConnectTcp)
                })?;
                tcp.set_nodelay(true).map_err(|error| {
                    map_tiberius_error(&error.into(), TiberiusErrorContext::ConfigureTcp)
                })?;

                Client::connect(tiberius_config, tcp.compat_write())
                    .await
                    .map_err(|error| {
                        map_tiberius_error(&error, TiberiusErrorContext::InitializeClient)
                    })
            })
            .await
        })
        .await?;

        Ok(Self { client, config })
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> MssqlConnection<S> {
    pub fn new(client: Client<S>, config: MssqlConnectionConfig) -> Self {
        Self { client, config }
    }

    pub fn config(&self) -> &MssqlConnectionConfig {
        &self.config
    }

    pub fn client(&self) -> &Client<S> {
        &self.client
    }

    pub fn client_mut(&mut self) -> &mut Client<S> {
        &mut self.client
    }

    pub(crate) fn query_timeout(&self) -> Option<Duration> {
        self.config.options().timeouts.query_timeout
    }

    pub(crate) fn tracing_options(&self) -> crate::config::MssqlTracingOptions {
        self.config.options().tracing
    }

    pub(crate) fn server_addr(&self) -> String {
        self.config.addr()
    }

    pub async fn begin_transaction<'a>(&'a mut self) -> Result<MssqlTransaction<'a, S>, OrmError> {
        let query_timeout = self.query_timeout();
        let tracing_options = self.tracing_options();
        let server_addr = self.server_addr();
        MssqlTransaction::begin(
            self.client_mut(),
            query_timeout,
            tracing_options,
            server_addr,
        )
        .await
    }

    pub async fn begin_transaction_scope(&mut self) -> Result<(), OrmError> {
        let query_timeout = self.query_timeout();
        let tracing_options = self.tracing_options();
        let server_addr = self.server_addr();
        begin_transaction_scope(
            self.client_mut(),
            query_timeout,
            tracing_options,
            &server_addr,
        )
        .await
    }

    pub async fn commit_transaction(&mut self) -> Result<(), OrmError> {
        let query_timeout = self.query_timeout();
        let tracing_options = self.tracing_options();
        let server_addr = self.server_addr();
        commit_transaction_scope(
            self.client_mut(),
            query_timeout,
            tracing_options,
            &server_addr,
        )
        .await
    }

    pub async fn rollback_transaction(&mut self) -> Result<(), OrmError> {
        let query_timeout = self.query_timeout();
        let tracing_options = self.tracing_options();
        let server_addr = self.server_addr();
        rollback_transaction_scope(
            self.client_mut(),
            query_timeout,
            tracing_options,
            &server_addr,
        )
        .await
    }

    pub fn into_inner(self) -> Client<S> {
        self.client
    }
}

pub(crate) async fn run_with_timeout<F, T>(
    duration: Option<Duration>,
    timeout_message: &'static str,
    future: F,
) -> Result<T, OrmError>
where
    F: core::future::Future<Output = Result<T, OrmError>>,
{
    match duration {
        Some(duration) => timeout(duration, future)
            .await
            .map_err(|_| OrmError::new(timeout_message))?,
        None => future.await,
    }
}

#[cfg(test)]
mod tests {
    use super::run_with_timeout;
    use std::time::Duration;

    #[tokio::test]
    async fn run_with_timeout_returns_future_result_without_timeout() {
        let value = run_with_timeout(None, "timeout", async {
            Ok::<_, mssql_orm_core::OrmError>(7)
        })
        .await
        .unwrap();

        assert_eq!(value, 7);
    }

    #[tokio::test]
    async fn run_with_timeout_fails_when_future_exceeds_deadline() {
        let error = run_with_timeout(
            Some(Duration::from_millis(5)),
            "SQL Server connection timed out",
            async {
                tokio::time::sleep(Duration::from_millis(25)).await;
                Ok::<_, mssql_orm_core::OrmError>(())
            },
        )
        .await
        .unwrap_err();

        assert_eq!(error.message(), "SQL Server connection timed out");
    }
}

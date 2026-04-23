use crate::config::MssqlConnectionConfig;
use crate::error::{TiberiusErrorContext, map_tiberius_error};
use futures_io::{AsyncRead, AsyncWrite};
use mssql_orm_core::OrmError;
use tiberius::Client;
use tokio::net::TcpStream;
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
        let tcp = TcpStream::connect(config.addr())
            .await
            .map_err(|error| map_tiberius_error(&error.into(), TiberiusErrorContext::ConnectTcp))?;
        tcp.set_nodelay(true).map_err(|error| {
            map_tiberius_error(&error.into(), TiberiusErrorContext::ConfigureTcp)
        })?;

        let client = Client::connect(config.tiberius_config().clone(), tcp.compat_write())
            .await
            .map_err(|error| map_tiberius_error(&error, TiberiusErrorContext::InitializeClient))?;

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

    pub fn into_inner(self) -> Client<S> {
        self.client
    }
}

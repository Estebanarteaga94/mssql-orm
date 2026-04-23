use mssql_orm_core::OrmError;
use tiberius::Config;

#[derive(Debug, Clone)]
pub struct MssqlConnectionConfig {
    connection_string: String,
    inner: Config,
}

impl MssqlConnectionConfig {
    pub fn from_connection_string(connection_string: &str) -> Result<Self, OrmError> {
        if connection_string.trim().is_empty() {
            return Err(OrmError::new("invalid SQL Server connection string"));
        }

        let inner = Config::from_ado_string(connection_string)
            .map_err(|_| OrmError::new("invalid SQL Server connection string"))?;
        validate_config(&inner)?;

        Ok(Self {
            connection_string: connection_string.to_string(),
            inner,
        })
    }

    pub fn connection_string(&self) -> &str {
        &self.connection_string
    }

    pub fn addr(&self) -> String {
        self.inner.get_addr()
    }

    pub(crate) fn tiberius_config(&self) -> &Config {
        &self.inner
    }
}

fn validate_config(config: &Config) -> Result<(), OrmError> {
    let addr = config.get_addr();

    if addr.is_empty() || addr.starts_with(':') {
        return Err(OrmError::new("invalid SQL Server connection string"));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::MssqlConnectionConfig;

    #[test]
    fn parses_valid_ado_connection_string() {
        let config = MssqlConnectionConfig::from_connection_string(
            "server=tcp:localhost,1433;database=AppDb;user=sa;password=Password123;TrustServerCertificate=true;Application Name=mssql-orm-tests",
        )
        .unwrap();

        assert_eq!(
            config.connection_string(),
            "server=tcp:localhost,1433;database=AppDb;user=sa;password=Password123;TrustServerCertificate=true;Application Name=mssql-orm-tests"
        );
        assert_eq!(config.addr(), "localhost:1433");
    }

    #[test]
    fn rejects_invalid_connection_string() {
        let error = MssqlConnectionConfig::from_connection_string("server=").unwrap_err();

        assert_eq!(error.message(), "invalid SQL Server connection string");
    }
}

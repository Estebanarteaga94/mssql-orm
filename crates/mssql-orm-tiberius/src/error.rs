use mssql_orm_core::OrmError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TiberiusErrorContext {
    ConnectTcp,
    ConfigureTcp,
    InitializeClient,
    ExecuteQuery,
    ReadRowValue,
}

pub(crate) fn map_tiberius_error(
    error: &tiberius::error::Error,
    context: TiberiusErrorContext,
) -> OrmError {
    if error.is_deadlock() {
        return OrmError::new("SQL Server deadlock detected");
    }

    match context {
        TiberiusErrorContext::ConnectTcp => {
            OrmError::new("failed to connect to SQL Server over TCP")
        }
        TiberiusErrorContext::ConfigureTcp => {
            OrmError::new("failed to configure SQL Server TCP stream")
        }
        TiberiusErrorContext::InitializeClient => {
            OrmError::new("failed to initialize Tiberius client")
        }
        TiberiusErrorContext::ExecuteQuery => OrmError::new("failed to execute SQL Server query"),
        TiberiusErrorContext::ReadRowValue => OrmError::new("failed to read SQL Server row value"),
    }
}

#[cfg(test)]
mod tests {
    use super::{TiberiusErrorContext, map_tiberius_error};
    use tiberius::error::Error;

    #[test]
    fn maps_contextual_driver_error_to_orm_error() {
        let error = Error::Conversion("boom".into());

        assert_eq!(
            map_tiberius_error(&error, TiberiusErrorContext::ExecuteQuery).message(),
            "failed to execute SQL Server query"
        );
    }
}

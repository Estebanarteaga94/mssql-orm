use crate::error::{TiberiusErrorContext, map_tiberius_error};
use mssql_orm_core::{OrmError, SqlValue};
use mssql_orm_query::CompiledQuery;
use tiberius::numeric::Numeric;
use tiberius::{Client, Query, QueryStream};

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum BoundSqlValue {
    Null,
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(f64),
    String(String),
    Bytes(Vec<u8>),
    Uuid(uuid::Uuid),
    Decimal(rust_decimal::Decimal),
    Date(chrono::NaiveDate),
    DateTime(chrono::NaiveDateTime),
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PreparedQuery {
    pub sql: String,
    pub params: Vec<BoundSqlValue>,
}

impl PreparedQuery {
    pub fn from_compiled(query: CompiledQuery) -> Self {
        Self {
            sql: query.sql,
            params: query.params.into_iter().map(BoundSqlValue::from).collect(),
        }
    }

    pub fn validate_parameter_count(&self) -> Result<(), OrmError> {
        let expected = count_sql_parameters(&self.sql);

        if expected != self.params.len() {
            return Err(OrmError::new(
                "compiled query parameter count does not match SQL placeholders",
            ));
        }

        Ok(())
    }

    pub async fn execute<S>(
        self,
        client: &mut Client<S>,
    ) -> Result<tiberius::ExecuteResult, OrmError>
    where
        S: futures_io::AsyncRead + futures_io::AsyncWrite + Unpin + Send,
    {
        let mut query = Query::new(self.sql.as_str());

        for param in &self.params {
            bind_sql_value(&mut query, param);
        }

        query
            .execute(client)
            .await
            .map_err(|error| map_tiberius_error(&error, TiberiusErrorContext::ExecuteQuery))
    }

    pub async fn query<'a, S>(self, client: &'a mut Client<S>) -> Result<QueryStream<'a>, OrmError>
    where
        S: futures_io::AsyncRead + futures_io::AsyncWrite + Unpin + Send,
    {
        let mut query = Query::new(self.sql.as_str());

        for param in &self.params {
            bind_sql_value(&mut query, param);
        }

        query
            .query(client)
            .await
            .map_err(|error| map_tiberius_error(&error, TiberiusErrorContext::ExecuteQuery))
    }
}

impl From<SqlValue> for BoundSqlValue {
    fn from(value: SqlValue) -> Self {
        match value {
            SqlValue::Null => Self::Null,
            SqlValue::Bool(value) => Self::Bool(value),
            SqlValue::I32(value) => Self::I32(value),
            SqlValue::I64(value) => Self::I64(value),
            SqlValue::F64(value) => Self::F64(value),
            SqlValue::String(value) => Self::String(value),
            SqlValue::Bytes(value) => Self::Bytes(value),
            SqlValue::Uuid(value) => Self::Uuid(value),
            SqlValue::Decimal(value) => Self::Decimal(value),
            SqlValue::Date(value) => Self::Date(value),
            SqlValue::DateTime(value) => Self::DateTime(value),
        }
    }
}

fn bind_sql_value<'a>(query: &mut Query<'a>, value: &'a BoundSqlValue) {
    match value {
        BoundSqlValue::Null => query.bind(Option::<String>::None),
        BoundSqlValue::Bool(value) => query.bind(*value),
        BoundSqlValue::I32(value) => query.bind(*value),
        BoundSqlValue::I64(value) => query.bind(*value),
        BoundSqlValue::F64(value) => query.bind(*value),
        BoundSqlValue::String(value) => query.bind(value),
        BoundSqlValue::Bytes(value) => query.bind(value),
        BoundSqlValue::Uuid(value) => query.bind(value),
        BoundSqlValue::Decimal(value) => query.bind(Numeric::new_with_scale(
            value.mantissa(),
            value.scale() as u8,
        )),
        BoundSqlValue::Date(value) => query.bind(*value),
        BoundSqlValue::DateTime(value) => query.bind(*value),
    }
}

fn count_sql_parameters(sql: &str) -> usize {
    let bytes = sql.as_bytes();
    let mut count = 0;
    let mut index = 0;

    while index + 2 < bytes.len() {
        if bytes[index] == b'@' && bytes[index + 1] == b'P' && bytes[index + 2].is_ascii_digit() {
            count += 1;
            index += 3;

            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }

            continue;
        }

        index += 1;
    }

    count
}

#[cfg(test)]
mod tests {
    use super::{BoundSqlValue, PreparedQuery};
    use chrono::NaiveDate;
    use mssql_orm_core::SqlValue;
    use mssql_orm_query::CompiledQuery;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    #[test]
    fn prepares_query_preserving_sql_and_parameter_order() {
        let compiled = CompiledQuery::new(
            "SELECT @P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10",
            vec![
                SqlValue::Null,
                SqlValue::Bool(true),
                SqlValue::I32(1),
                SqlValue::I64(2),
                SqlValue::F64(3.5),
                SqlValue::String("ana@example.com".to_string()),
                SqlValue::Bytes(vec![1, 2, 3]),
                SqlValue::Uuid(Uuid::nil()),
                SqlValue::Decimal(Decimal::new(1234, 2)),
                SqlValue::DateTime(
                    NaiveDate::from_ymd_opt(2026, 4, 23)
                        .unwrap()
                        .and_hms_opt(10, 20, 30)
                        .unwrap(),
                ),
            ],
        );

        let prepared = PreparedQuery::from_compiled(compiled);

        assert_eq!(
            prepared.sql,
            "SELECT @P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10"
        );
        assert_eq!(
            prepared.params,
            vec![
                BoundSqlValue::Null,
                BoundSqlValue::Bool(true),
                BoundSqlValue::I32(1),
                BoundSqlValue::I64(2),
                BoundSqlValue::F64(3.5),
                BoundSqlValue::String("ana@example.com".to_string()),
                BoundSqlValue::Bytes(vec![1, 2, 3]),
                BoundSqlValue::Uuid(Uuid::nil()),
                BoundSqlValue::Decimal(Decimal::new(1234, 2)),
                BoundSqlValue::DateTime(
                    NaiveDate::from_ymd_opt(2026, 4, 23)
                        .unwrap()
                        .and_hms_opt(10, 20, 30)
                        .unwrap(),
                ),
            ]
        );
    }

    #[test]
    fn validates_parameter_count_against_sql_placeholders() {
        let prepared = PreparedQuery::from_compiled(CompiledQuery::new(
            "SELECT @P1, @P2",
            vec![SqlValue::Bool(true), SqlValue::Bool(false)],
        ));

        assert!(prepared.validate_parameter_count().is_ok());
    }

    #[test]
    fn rejects_mismatched_parameter_count() {
        let prepared = PreparedQuery::from_compiled(CompiledQuery::new(
            "SELECT @P1, @P2",
            vec![SqlValue::Bool(true)],
        ));

        let error = prepared.validate_parameter_count().unwrap_err();

        assert_eq!(
            error.message(),
            "compiled query parameter count does not match SQL placeholders"
        );
    }

    #[test]
    fn supports_date_values_in_prepared_query() {
        let prepared = PreparedQuery::from_compiled(CompiledQuery::new(
            "SELECT @P1",
            vec![SqlValue::Date(
                NaiveDate::from_ymd_opt(2026, 4, 23).unwrap(),
            )],
        ));

        assert_eq!(
            prepared.params,
            vec![BoundSqlValue::Date(
                NaiveDate::from_ymd_opt(2026, 4, 23).unwrap()
            )]
        );
    }
}

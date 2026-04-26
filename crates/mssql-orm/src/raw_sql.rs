use crate::context::SharedConnection;
use mssql_orm_core::{FromRow, OrmError, SqlTypeMapping, SqlValue};
use mssql_orm_query::CompiledQuery;
use mssql_orm_tiberius::ExecuteResult;
use std::collections::BTreeSet;
use std::marker::PhantomData;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct RawPlaceholderPlan {
    max_index: usize,
}

impl RawPlaceholderPlan {
    const fn expected_param_count(&self) -> usize {
        self.max_index
    }
}

pub trait RawParam {
    fn into_sql_value(self) -> SqlValue;
}

macro_rules! impl_raw_param_via_sql_type_mapping {
    ($($ty:ty),+ $(,)?) => {
        $(
            impl RawParam for $ty {
                fn into_sql_value(self) -> SqlValue {
                    <Self as SqlTypeMapping>::to_sql_value(self)
                }
            }
        )+
    };
}

impl_raw_param_via_sql_type_mapping!(
    bool,
    i32,
    i64,
    f64,
    String,
    Vec<u8>,
    uuid::Uuid,
    rust_decimal::Decimal,
    chrono::NaiveDate,
    chrono::NaiveDateTime,
);

impl RawParam for SqlValue {
    fn into_sql_value(self) -> SqlValue {
        self
    }
}

impl RawParam for &str {
    fn into_sql_value(self) -> SqlValue {
        SqlValue::String(self.to_string())
    }
}

impl<T> RawParam for Option<T>
where
    T: RawParam,
{
    fn into_sql_value(self) -> SqlValue {
        self.map(RawParam::into_sql_value).unwrap_or(SqlValue::Null)
    }
}

pub trait RawParams {
    fn into_sql_values(self) -> Vec<SqlValue>;
}

impl RawParams for () {
    fn into_sql_values(self) -> Vec<SqlValue> {
        Vec::new()
    }
}

impl<T> RawParams for Vec<T>
where
    T: RawParam,
{
    fn into_sql_values(self) -> Vec<SqlValue> {
        self.into_iter().map(RawParam::into_sql_value).collect()
    }
}

macro_rules! impl_raw_params_tuple {
    ($($name:ident),+ $(,)?) => {
        impl<$($name),+> RawParams for ($($name,)+)
        where
            $($name: RawParam),+
        {
            #[allow(non_snake_case)]
            fn into_sql_values(self) -> Vec<SqlValue> {
                let ($($name,)+) = self;
                vec![$($name.into_sql_value()),+]
            }
        }
    };
}

impl_raw_params_tuple!(A);
impl_raw_params_tuple!(A, B);
impl_raw_params_tuple!(A, B, C);
impl_raw_params_tuple!(A, B, C, D);
impl_raw_params_tuple!(A, B, C, D, E);
impl_raw_params_tuple!(A, B, C, D, E, F);
impl_raw_params_tuple!(A, B, C, D, E, F, G);
impl_raw_params_tuple!(A, B, C, D, E, F, G, H);
impl_raw_params_tuple!(A, B, C, D, E, F, G, H, I);
impl_raw_params_tuple!(A, B, C, D, E, F, G, H, I, J);
impl_raw_params_tuple!(A, B, C, D, E, F, G, H, I, J, K);
impl_raw_params_tuple!(A, B, C, D, E, F, G, H, I, J, K, L);

#[derive(Clone)]
pub struct RawQuery<T> {
    connection: SharedConnection,
    sql: String,
    params: Vec<SqlValue>,
    _row: PhantomData<fn() -> T>,
}

impl<T> RawQuery<T>
where
    T: FromRow + Send,
{
    pub(crate) fn new(connection: SharedConnection, sql: impl Into<String>) -> Self {
        Self {
            connection,
            sql: sql.into(),
            params: Vec::new(),
            _row: PhantomData,
        }
    }

    pub fn param<P>(mut self, value: P) -> Self
    where
        P: RawParam,
    {
        self.params.push(value.into_sql_value());
        self
    }

    pub fn params<P>(mut self, values: P) -> Self
    where
        P: RawParams,
    {
        self.params.extend(values.into_sql_values());
        self
    }

    pub async fn all(self) -> Result<Vec<T>, OrmError> {
        let compiled = self.compiled_query()?;
        let mut connection = self.connection.lock().await?;
        connection.fetch_all(compiled).await
    }

    pub async fn first(self) -> Result<Option<T>, OrmError> {
        let compiled = self.compiled_query()?;
        let mut connection = self.connection.lock().await?;
        connection.fetch_one(compiled).await
    }

    fn compiled_query(&self) -> Result<CompiledQuery, OrmError> {
        compiled_raw_query(&self.sql, self.params.clone())
    }
}

#[derive(Clone)]
pub struct RawCommand {
    connection: SharedConnection,
    sql: String,
    params: Vec<SqlValue>,
}

impl RawCommand {
    pub(crate) fn new(connection: SharedConnection, sql: impl Into<String>) -> Self {
        Self {
            connection,
            sql: sql.into(),
            params: Vec::new(),
        }
    }

    pub fn param<P>(mut self, value: P) -> Self
    where
        P: RawParam,
    {
        self.params.push(value.into_sql_value());
        self
    }

    pub fn params<P>(mut self, values: P) -> Self
    where
        P: RawParams,
    {
        self.params.extend(values.into_sql_values());
        self
    }

    pub async fn execute(self) -> Result<ExecuteResult, OrmError> {
        let compiled = self.compiled_query()?;
        let mut connection = self.connection.lock().await?;
        connection.execute(compiled).await
    }

    fn compiled_query(&self) -> Result<CompiledQuery, OrmError> {
        compiled_raw_query(&self.sql, self.params.clone())
    }
}

fn compiled_raw_query(sql: &str, params: Vec<SqlValue>) -> Result<CompiledQuery, OrmError> {
    validate_raw_sql_parameters(sql, params.len())?;
    Ok(CompiledQuery::new(sql, params))
}

pub(crate) fn validate_raw_sql_parameters(sql: &str, param_count: usize) -> Result<(), OrmError> {
    let plan = analyze_placeholders(sql)?;

    if plan.expected_param_count() != param_count {
        return Err(OrmError::new(format!(
            "raw SQL parameter count mismatch: SQL expects {} parameter(s), received {}",
            plan.expected_param_count(),
            param_count
        )));
    }

    Ok(())
}

fn analyze_placeholders(sql: &str) -> Result<RawPlaceholderPlan, OrmError> {
    let bytes = sql.as_bytes();
    let mut index = 0;
    let mut placeholders = BTreeSet::new();

    while index + 2 < bytes.len() {
        if bytes[index] == b'@' && bytes[index + 1] == b'P' && bytes[index + 2].is_ascii_digit() {
            index += 2;
            let start = index;

            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }

            let raw_index = sql[start..index]
                .parse::<usize>()
                .map_err(|_| OrmError::new("raw SQL placeholder index is larger than supported"))?;

            if raw_index == 0 {
                return Err(OrmError::new("raw SQL placeholders must start at @P1"));
            }

            placeholders.insert(raw_index);
            continue;
        }

        index += 1;
    }

    let max_index = placeholders.iter().next_back().copied().unwrap_or(0);
    for expected in 1..=max_index {
        if !placeholders.contains(&expected) {
            return Err(OrmError::new(format!(
                "raw SQL placeholders must be continuous from @P1 to @P{}",
                max_index
            )));
        }
    }

    Ok(RawPlaceholderPlan { max_index })
}

#[cfg(test)]
mod tests {
    use super::{RawParam, RawParams, compiled_raw_query, validate_raw_sql_parameters};
    use chrono::NaiveDate;
    use mssql_orm_core::SqlValue;
    use rust_decimal::Decimal;
    use uuid::Uuid;

    #[test]
    fn validates_continuous_placeholders_by_max_index() {
        validate_raw_sql_parameters("SELECT @P1, @P2, @P3", 3).unwrap();
    }

    #[test]
    fn validates_continuous_placeholders_through_highest_index() {
        validate_raw_sql_parameters(
            "SELECT @P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10, @P11, @P12",
            12,
        )
        .unwrap();
    }

    #[test]
    fn allows_repeated_placeholder_to_reuse_one_param() {
        validate_raw_sql_parameters("SELECT @P1 WHERE owner_id = @P1", 1).unwrap();
    }

    #[test]
    fn rejects_extra_params_without_placeholders() {
        let error = validate_raw_sql_parameters("SELECT 1", 1).unwrap_err();

        assert!(error.message().contains("expects 0 parameter"));
    }

    #[test]
    fn rejects_missing_params() {
        let error = validate_raw_sql_parameters("SELECT @P1, @P2", 1).unwrap_err();

        assert!(error.message().contains("expects 2 parameter"));
    }

    #[test]
    fn rejects_non_continuous_placeholders() {
        let error = validate_raw_sql_parameters("SELECT @P1, @P3", 2).unwrap_err();

        assert!(error.message().contains("continuous from @P1 to @P3"));
    }

    #[test]
    fn rejects_zero_index_placeholder() {
        let error = validate_raw_sql_parameters("SELECT @P0", 0).unwrap_err();

        assert!(error.message().contains("start at @P1"));
    }

    #[test]
    fn raw_params_tuple_preserves_order_and_values() {
        let values = (
            true,
            7_i32,
            9_i64,
            3.5_f64,
            "draft",
            String::from("owned"),
            vec![1_u8, 2],
            Uuid::nil(),
            Decimal::new(1234, 2),
            NaiveDate::from_ymd_opt(2026, 4, 26).unwrap(),
            NaiveDate::from_ymd_opt(2026, 4, 26)
                .unwrap()
                .and_hms_opt(10, 20, 30)
                .unwrap(),
            SqlValue::Null,
        )
            .into_sql_values();

        assert_eq!(
            values,
            vec![
                SqlValue::Bool(true),
                SqlValue::I32(7),
                SqlValue::I64(9),
                SqlValue::F64(3.5),
                SqlValue::String("draft".to_string()),
                SqlValue::String("owned".to_string()),
                SqlValue::Bytes(vec![1, 2]),
                SqlValue::Uuid(Uuid::nil()),
                SqlValue::Decimal(Decimal::new(1234, 2)),
                SqlValue::Date(NaiveDate::from_ymd_opt(2026, 4, 26).unwrap()),
                SqlValue::DateTime(
                    NaiveDate::from_ymd_opt(2026, 4, 26)
                        .unwrap()
                        .and_hms_opt(10, 20, 30)
                        .unwrap()
                ),
                SqlValue::Null,
            ]
        );
    }

    #[test]
    fn raw_param_option_none_maps_to_null() {
        assert_eq!(Option::<i64>::None.into_sql_value(), SqlValue::Null);
    }

    #[test]
    fn raw_param_option_some_maps_inner_value() {
        assert_eq!(Some(42_i64).into_sql_value(), SqlValue::I64(42));
    }

    #[test]
    fn raw_params_vec_preserves_order() {
        let values = vec![1_i64, 2_i64, 3_i64].into_sql_values();

        assert_eq!(
            values,
            vec![SqlValue::I64(1), SqlValue::I64(2), SqlValue::I64(3)]
        );
    }

    #[test]
    fn raw_params_unit_maps_to_empty_params() {
        assert_eq!(().into_sql_values(), Vec::<SqlValue>::new());
    }

    #[test]
    fn compiled_raw_query_preserves_sql_and_parameter_order() {
        let params = (
            SqlValue::Null,
            true,
            7_i32,
            9_i64,
            3.5_f64,
            "draft",
            vec![1_u8, 2],
            Uuid::nil(),
            Decimal::new(1234, 2),
            NaiveDate::from_ymd_opt(2026, 4, 26).unwrap(),
            NaiveDate::from_ymd_opt(2026, 4, 26)
                .unwrap()
                .and_hms_opt(10, 20, 30)
                .unwrap(),
        )
            .into_sql_values();

        let compiled = compiled_raw_query(
            "SELECT @P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10, @P11",
            params,
        )
        .unwrap();

        assert_eq!(
            compiled.sql,
            "SELECT @P1, @P2, @P3, @P4, @P5, @P6, @P7, @P8, @P9, @P10, @P11"
        );
        assert_eq!(
            compiled.params,
            vec![
                SqlValue::Null,
                SqlValue::Bool(true),
                SqlValue::I32(7),
                SqlValue::I64(9),
                SqlValue::F64(3.5),
                SqlValue::String("draft".to_string()),
                SqlValue::Bytes(vec![1, 2]),
                SqlValue::Uuid(Uuid::nil()),
                SqlValue::Decimal(Decimal::new(1234, 2)),
                SqlValue::Date(NaiveDate::from_ymd_opt(2026, 4, 26).unwrap()),
                SqlValue::DateTime(
                    NaiveDate::from_ymd_opt(2026, 4, 26)
                        .unwrap()
                        .and_hms_opt(10, 20, 30)
                        .unwrap()
                ),
            ]
        );
    }

    #[test]
    fn compiled_raw_query_allows_repeated_placeholder_with_single_param() {
        let compiled = compiled_raw_query(
            "SELECT * FROM users WHERE owner_id = @P1 OR reviewer_id = @P1",
            vec![SqlValue::I64(42)],
        )
        .unwrap();

        assert_eq!(compiled.params, vec![SqlValue::I64(42)]);
    }

    #[test]
    fn compiled_raw_query_rejects_non_continuous_placeholders() {
        let error = compiled_raw_query(
            "SELECT * FROM users WHERE owner_id = @P1 OR reviewer_id = @P3",
            vec![SqlValue::I64(42), SqlValue::I64(7)],
        )
        .unwrap_err();

        assert!(error.message().contains("continuous from @P1 to @P3"));
    }
}

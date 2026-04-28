use mssql_orm_core::{ColumnValue, EntityMetadata, OrmError};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuditOperation {
    Insert,
    Update,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct AuditRequestValues {
    values: Vec<ColumnValue>,
}

impl AuditRequestValues {
    pub fn new(values: Vec<ColumnValue>) -> Self {
        Self { values }
    }

    pub fn values(&self) -> &[ColumnValue] {
        &self.values
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AuditContext<'a> {
    pub entity: &'static EntityMetadata,
    pub operation: AuditOperation,
    pub request_values: Option<&'a AuditRequestValues>,
}

pub trait AuditProvider: Send + Sync {
    fn values(&self, context: AuditContext<'_>) -> Result<Vec<ColumnValue>, OrmError>;
}

#[doc(hidden)]
pub fn resolve_audit_values(
    values: Vec<ColumnValue>,
    context: AuditContext<'_>,
    audit_provider: Option<&dyn AuditProvider>,
) -> Result<Vec<ColumnValue>, OrmError> {
    validate_no_duplicate_columns("audit values", &values)?;

    let mut resolved = values;
    let mut seen = resolved
        .iter()
        .map(|value| value.column_name)
        .collect::<BTreeSet<_>>();

    if let Some(request_values) = context.request_values {
        validate_no_duplicate_columns("audit request values", request_values.values())?;
        append_missing_values(&mut resolved, &mut seen, request_values.values());
    }

    if let Some(provider) = audit_provider {
        let provider_values = provider.values(context)?;
        validate_no_duplicate_columns("audit provider values", &provider_values)?;
        append_missing_values(&mut resolved, &mut seen, &provider_values);
    }

    Ok(resolved)
}

fn validate_no_duplicate_columns(label: &str, values: &[ColumnValue]) -> Result<(), OrmError> {
    let mut seen = BTreeSet::new();

    for value in values {
        if !seen.insert(value.column_name) {
            return Err(OrmError::new(format!(
                "duplicate column `{}` in {label}",
                value.column_name
            )));
        }
    }

    Ok(())
}

fn append_missing_values(
    resolved: &mut Vec<ColumnValue>,
    seen: &mut BTreeSet<&'static str>,
    values: &[ColumnValue],
) {
    for value in values {
        if seen.insert(value.column_name) {
            resolved.push(value.clone());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        AuditContext, AuditOperation, AuditProvider, AuditRequestValues, resolve_audit_values,
    };
    use mssql_orm_core::{
        ColumnMetadata, ColumnValue, EntityMetadata, OrmError, PrimaryKeyMetadata, SqlServerType,
        SqlValue,
    };

    static TEST_ENTITY_COLUMNS: [ColumnMetadata; 1] = [ColumnMetadata {
        rust_field: "id",
        column_name: "id",
        renamed_from: None,
        sql_type: SqlServerType::BigInt,
        nullable: false,
        primary_key: true,
        identity: None,
        default_sql: None,
        computed_sql: None,
        rowversion: false,
        insertable: false,
        updatable: false,
        max_length: None,
        precision: None,
        scale: None,
    }];

    static TEST_ENTITY_METADATA: EntityMetadata = EntityMetadata {
        rust_name: "AuditedEntity",
        schema: "dbo",
        table: "audited_entities",
        renamed_from: None,
        columns: &TEST_ENTITY_COLUMNS,
        primary_key: PrimaryKeyMetadata::new(None, &["id"]),
        indexes: &[],
        foreign_keys: &[],
    };

    struct FixedAuditProvider;

    impl AuditProvider for FixedAuditProvider {
        fn values(&self, context: AuditContext<'_>) -> Result<Vec<ColumnValue>, OrmError> {
            assert_eq!(context.entity.rust_name, "AuditedEntity");
            assert_eq!(context.operation, AuditOperation::Insert);
            assert!(context.request_values.is_some());

            Ok(vec![
                ColumnValue::new(
                    "created_at",
                    SqlValue::String("provider-created-at".to_string()),
                ),
                ColumnValue::new(
                    "updated_by",
                    SqlValue::String("provider-updated-by".to_string()),
                ),
            ])
        }
    }

    fn context<'a>(request_values: Option<&'a AuditRequestValues>) -> AuditContext<'a> {
        AuditContext {
            entity: &TEST_ENTITY_METADATA,
            operation: AuditOperation::Insert,
            request_values,
        }
    }

    #[test]
    fn resolve_audit_values_preserves_user_values_before_request_and_provider_values() {
        let request_values = AuditRequestValues::new(vec![
            ColumnValue::new(
                "created_at",
                SqlValue::String("request-created-at".to_string()),
            ),
            ColumnValue::new(
                "created_by",
                SqlValue::String("request-created-by".to_string()),
            ),
        ]);

        let resolved = resolve_audit_values(
            vec![ColumnValue::new(
                "created_at",
                SqlValue::String("user-created-at".to_string()),
            )],
            context(Some(&request_values)),
            Some(&FixedAuditProvider),
        )
        .expect("audit values should resolve");

        assert_eq!(
            resolved,
            vec![
                ColumnValue::new(
                    "created_at",
                    SqlValue::String("user-created-at".to_string())
                ),
                ColumnValue::new(
                    "created_by",
                    SqlValue::String("request-created-by".to_string())
                ),
                ColumnValue::new(
                    "updated_by",
                    SqlValue::String("provider-updated-by".to_string())
                ),
            ]
        );
    }

    #[test]
    fn resolve_audit_values_uses_request_values_without_provider() {
        let request_values = AuditRequestValues::new(vec![ColumnValue::new(
            "updated_by",
            SqlValue::String("request-updated-by".to_string()),
        )]);

        let resolved = resolve_audit_values(vec![], context(Some(&request_values)), None)
            .expect("request audit values should resolve");

        assert_eq!(
            resolved,
            vec![ColumnValue::new(
                "updated_by",
                SqlValue::String("request-updated-by".to_string())
            )]
        );
    }

    #[test]
    fn resolve_audit_values_rejects_duplicate_user_columns() {
        let error = resolve_audit_values(
            vec![
                ColumnValue::new("created_at", SqlValue::String("first".to_string())),
                ColumnValue::new("created_at", SqlValue::String("second".to_string())),
            ],
            context(None),
            None,
        )
        .unwrap_err();

        assert_eq!(
            error,
            OrmError::new("duplicate column `created_at` in audit values")
        );
    }

    #[test]
    fn resolve_audit_values_rejects_duplicate_request_columns() {
        let request_values = AuditRequestValues::new(vec![
            ColumnValue::new("created_by", SqlValue::String("first".to_string())),
            ColumnValue::new("created_by", SqlValue::String("second".to_string())),
        ]);

        let error = resolve_audit_values(vec![], context(Some(&request_values)), None).unwrap_err();

        assert_eq!(
            error,
            OrmError::new("duplicate column `created_by` in audit request values")
        );
    }

    #[test]
    fn resolve_audit_values_rejects_duplicate_provider_columns() {
        struct DuplicateProvider;

        impl AuditProvider for DuplicateProvider {
            fn values(&self, _context: AuditContext<'_>) -> Result<Vec<ColumnValue>, OrmError> {
                Ok(vec![
                    ColumnValue::new("updated_at", SqlValue::String("first".to_string())),
                    ColumnValue::new("updated_at", SqlValue::String("second".to_string())),
                ])
            }
        }

        let error =
            resolve_audit_values(vec![], context(None), Some(&DuplicateProvider)).unwrap_err();

        assert_eq!(
            error,
            OrmError::new("duplicate column `updated_at` in audit provider values")
        );
    }
}

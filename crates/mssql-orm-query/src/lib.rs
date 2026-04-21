//! Query AST foundations for the ORM.

use mssql_orm_core::CrateIdentity;

/// Placeholder root node for future query expressions.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryAst {
    pub dialect: &'static str,
}

pub const CRATE_IDENTITY: CrateIdentity = CrateIdentity {
    name: "mssql-orm-query",
    responsibility: "typed AST and query builder primitives without SQL generation",
};

#[cfg(test)]
mod tests {
    use super::{CRATE_IDENTITY, QueryAst};

    #[test]
    fn keeps_query_layer_sql_free() {
        let ast = QueryAst {
            dialect: "sqlserver-ast",
        };

        assert_eq!(ast.dialect, "sqlserver-ast");
        assert!(
            CRATE_IDENTITY
                .responsibility
                .contains("without SQL generation")
        );
    }
}

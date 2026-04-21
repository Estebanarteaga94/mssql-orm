//! Core contracts and shared types for the ORM.

/// Common error type placeholder for the workspace foundations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrmError {
    message: &'static str,
}

impl OrmError {
    pub const fn new(message: &'static str) -> Self {
        Self { message }
    }

    pub const fn message(&self) -> &'static str {
        self.message
    }
}

/// Minimal crate identity metadata used while the rest of the model is defined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrateIdentity {
    pub name: &'static str,
    pub responsibility: &'static str,
}

pub const CRATE_IDENTITY: CrateIdentity = CrateIdentity {
    name: "mssql-orm-core",
    responsibility: "contracts, metadata, shared types and errors",
};

#[cfg(test)]
mod tests {
    use super::{CRATE_IDENTITY, OrmError};

    #[test]
    fn exposes_foundation_identity() {
        assert_eq!(CRATE_IDENTITY.name, "mssql-orm-core");
    }

    #[test]
    fn preserves_error_message() {
        let error = OrmError::new("foundation");
        assert_eq!(error.message(), "foundation");
    }
}

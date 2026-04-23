#[test]
fn entity_derive_ui() {
    let tests = trybuild::TestCases::new();

    tests.pass("tests/ui/entity_valid.rs");
    tests.pass("tests/ui/entity_foreign_key_default_schema_valid.rs");
    tests.pass("tests/ui/insertable_changeset_valid.rs");
    tests.pass("tests/ui/dbcontext_valid.rs");
    tests.pass("tests/ui/query_builder_public_valid.rs");
    tests.compile_fail("tests/ui/entity_missing_primary_key.rs");
    tests.compile_fail("tests/ui/entity_identity_invalid_type.rs");
    tests.compile_fail("tests/ui/entity_foreign_key_empty_segment.rs");
    tests.compile_fail("tests/ui/entity_foreign_key_invalid_format.rs");
    tests.compile_fail("tests/ui/entity_rowversion_invalid_type.rs");
    tests.compile_fail("tests/ui/insertable_missing_entity.rs");
    tests.compile_fail("tests/ui/changeset_field_not_option.rs");
    tests.compile_fail("tests/ui/dbcontext_invalid_field_type.rs");
}

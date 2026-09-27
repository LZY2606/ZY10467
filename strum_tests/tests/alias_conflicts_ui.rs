// Compile-fail/pass fixtures for the `EnumString` alias-conflict diagnostics.
// The `.stderr` snapshots lock the diagnostic spans (both attribute
// locations) and the stable reason code `strum::alias_conflict`.
#[test]
fn alias_conflict_ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/alias_conflicts/*.rs");
    t.pass("tests/ui/alias_conflicts/pass/*.rs");
}

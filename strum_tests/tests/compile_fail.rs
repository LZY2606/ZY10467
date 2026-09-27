//! Compile-fail/pass fixtures for alias-conflict diagnostics.
//!
//! The `.stderr` snapshots lock the diagnostic spans (both attribute
//! locations) and the stable reason codes (`strum::alias_conflict::*`).
//! trybuild reports each fixture individually, so a regression points at
//! the exact enum that changed behavior.

#[test]
fn alias_conflict_diagnostics() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/alias_conflict/*.rs");
    t.pass("tests/ui/alias_conflict_pass/*.rs");
}

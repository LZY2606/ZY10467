# Alias-conflict compile diagnostics for `EnumString`

## Problem

The same enum can produce identical *input aliases* from `serialize`,
`to_string`, `serialize_all` case conversion, and `ascii_case_insensitive`.
Before this change, such conflicts only surfaced inside the generated matching
code: silently as first-arm-wins in the plain `match` backend, or as an
opaque `duplicate key` error from inside `phf` in the `use_phf` backend.

## Design

`strum_macros::helpers::alias_conflicts` computes the normalized alias set of
an enum before any match code is generated, from the same data flow the
generator uses (`get_variant_properties` + `get_serializations` + the
effective per-variant `ascii_case_insensitive` flag). Two aliases of
different variants conflict when some input string matches both:

* two case-sensitive aliases conflict iff they are exactly equal;
* if either side is case-insensitive, they conflict iff their
  ASCII-lowercased forms are equal.

A conflict is a compile error carrying the stable reason code
`strum::alias_conflict` and spans pointing at both attribute locations (the
`LitStr` spans of the two serializations, or the variant ident for
`serialize_all`-derived names). Generation is only allowed with an explicit
priority strategy, `#[strum(on_alias_conflict = "first_wins")]`, documented
in the `EnumString` section of `strum_macros/src/lib.rs`.

`disabled` variants and `default` variants contribute no match arms, so their
attributes are not input aliases and are skipped. `prefix`/`suffix` only
affect the display side in this codebase and are therefore not part of the
parse-alias set; the same conflict rules apply uniformly to enums using them.

## Alternatives considered

### Conflict normalization: ASCII-only vs. Unicode case folding

* **Unicode case folding (e.g. `str::to_lowercase`)** would flag pairs like
  `"Äpfel"`/`"äpfel"` as conflicts. But the generated matcher uses
  `str::eq_ignore_ascii_case`, which never equates those strings — the
  diagnostic would disagree with the runtime behavior and reject enums that
  parse unambiguously.
* **ASCII-only folding (chosen)** mirrors `eq_ignore_ascii_case` exactly, so
  the compile-time analysis can never disagree with the generated matcher.
  Responsibility boundary: aliases that differ only in non-ASCII case are
  *not* conflicts; they parse to distinct variants. This is documented in the
  `EnumString` docs and verified by
  `unicode_aliases_use_ascii_only_normalization` (runtime) and
  `unicode_case_difference_is_not_a_conflict` (unit).

### Same-variant duplicates: merge vs. reject

* **Reject** would be simpler, but duplicate aliases on one variant are
  unambiguous (every occurrence maps to the same variant) and exist in the
  wild (`serialize = "a", serialize = "a"`, or `serialize` + `to_string`
  with the same value). Rejecting them would break previously compiling,
  behaviorally-correct enums.
* **Merge (chosen)**: the generator deduplicates emitted aliases, keeping the
  first occurrence. This is required for `use_phf` anyway — a duplicate key
  is a hard error inside `phf` — and it removes `unreachable_patterns`
  warnings in the plain `match` backend. Verified by
  `same_variant_duplicates_are_merged`,
  `same_variant_case_insensitive_duplicates_are_merged`,
  `phf_same_variant_duplicates_are_merged`, and the
  `pass/same_variant_duplicates.rs` UI fixture.

### `phf` vs. plain `match` consistency

* **Per-backend rules** (let `phf` keep erroring on duplicates, only diagnose
  in the `match` backend) would make the feature flag change whether an enum
  compiles — an unacceptable action-at-a-distance.
* **One shared rule (chosen)**: the conflict check runs before code
  generation regardless of backend, and the generator deduplicates exact
  `phf` keys (including the lowercase/UPPERCASE copies emitted for
  case-insensitive aliases) keeping the first occurrence. Since phf keys are
  emitted in declaration order, first-occurrence dedup gives `phf` the same
  first-declared-variant-wins semantics the `match` arms already have. This
  also fixes a pre-existing breakage where a case-insensitive alias that is
  already lowercase (e.g. `serialize = "red"`) made `phf_map!` fail with a
  bare `duplicate key` error. Verified by
  `phf_lowercase_alias_matches_match_backend` and
  `phf_first_wins_matches_match_backend`, which parse identical input tables
  through both backends and assert identical results; both tests fail to
  compile (`error: duplicate key` from `phf`) when the dedup is reverted.

## Invariants kept

* Conflict-free enums generate byte-identical match arms/phf keys in the same
  order as before (dedup only ever removes exact duplicates), and error
  classification is unchanged: unknown input still yields
  `ParseError::VariantNotFound` (or the configured `parse_err_ty`), asserted
  directly in the runtime tests.
* The check is a single O(n) pre-pass over the variant attributes; the main
  generation path is untouched when no duplicates exist.
* Diagnostics lock only spans and the stable reason code
  `strum::alias_conflict`; the UI snapshots in
  `strum_tests/tests/ui/alias_conflicts/` pin both attribute locations for
  multi-source conflicts (`serialize` vs `to_string`, `serialize_all` vs
  `serialize`, ident case conversion, `ascii_case_insensitive`, Unicode,
  `prefix`/`suffix` enums, `default` variants, and multiple conflicts in one
  enum).

## Verification evidence

* `cargo test --locked --workspace --all-features` — full suite green,
  including the pre-existing tests (no output change for conflict-free
  enums).
* `strum_macros` unit tests (`helpers::alias_conflicts::tests`) — the
  normalization rules, same-variant merge, disabled/default skipping, and
  strategy parsing.
* `strum_tests/tests/alias_conflicts.rs` — runtime `from_str`/`display`
  behavior, including direct error-state assertions
  (`Err(ParseError::VariantNotFound)`) and phf/match equivalence tables.
* `strum_tests/tests/alias_conflicts_ui.rs` + `tests/ui/alias_conflicts/` —
  trybuild compile-fail fixtures for multi-source conflicts and pass
  fixtures for allowed same-variant duplicates and `first_wins`.

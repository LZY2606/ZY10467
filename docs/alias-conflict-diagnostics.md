# Alias-conflict compile diagnostics in `EnumString`

## Problem

The same enum can produce identical input aliases from `serialize`,
`to_string`, `serialize_all` case conversion and `ascii_case_insensitive`.
Before this change, such conflicts only surfaced inside the generated matcher:
an unreachable match arm (silently shadowing a variant) or a `phf_map!`
duplicate-key error with no reference to the responsible attributes.

## Design

`EnumString` now computes the normalized alias set before generating any match
code (`strum_macros/src/helpers/alias_conflicts.rs`). A cross-variant conflict
is a compile error pointing at *both* attribute locations (the `LitStr` spans;
the variant identifier when the alias comes from the variant name) and carries
a stable reason code:

- `strum::alias_conflict::exact` — byte-identical aliases.
- `strum::alias_conflict::ascii_case` — aliases equal after ASCII case folding
  with case-insensitive matching enabled on at least one side.

Generation is only allowed when the enum opts in to an explicit priority
strategy: `#[strum(alias_conflict = "first_wins")]` (first declared variant
wins; shadowed arms/keys are dropped so the output stays warning-free).

The check follows the existing data flow: `disabled` variants and the
`default` variant produce no match arms, so they register no aliases;
`prefix`/`suffix` are only consumed by display-oriented derives and do not
participate; generics, `default_with`, `parse_err_*` and `use_phf` go through
the same registry.

## Alternatives considered

### Conflict normalization: ASCII-only vs. Unicode case folding

- **ASCII-only (chosen).** The generated matcher uses
  `str::eq_ignore_ascii_case`, so the conflict check mirrors the runtime
  semantics exactly: every reported conflict is a real ambiguity, and no real
  ambiguity goes unreported. It also keeps the macro free of Unicode tables
  (compile-time cost, `no_std` story).
- **Unicode case folding (rejected).** Would report "conflicts" (e.g. `café`
  vs `CAFÉ`) that the generated matcher does not actually consider equal —
  false positives the user cannot act on. Folding rules also change with the
  Unicode version, which would make diagnostics unstable across toolchains.

Responsibility boundary: normalization is limited to ASCII case folding;
non-ASCII aliases are compared byte-for-byte, exactly like the runtime
matcher. Verified by `unicode_aliases_match_byte_exactly` in
`strum_tests/tests/alias_conflict.rs` and the
`unicode_exact_duplicate.rs` compile-fail fixture.

### Same-variant duplicates: merge vs. error

- **Merge (chosen).** `serialize = "a", serialize = "a"` on one variant is
  unambiguous — every alias routes to the same variant. Erroring would break
  previously compiling (if warning-prone, and on `use_phf` previously
  *broken*) enums for no benefit. Duplicates are deduplicated before arm/key
  emission, which also removes the `unreachable_pattern` warnings and the
  `phf_map!` duplicate-key failure such enums produced before.
- **Error (rejected).** No ambiguity exists to protect the user from; it would
  only force noisy attribute cleanups.

Verified by `same_variant_duplicates_are_merged` and the
`same_variant_duplicates.rs` pass fixture (compile-fail suite asserts the
*absence* of an error in the right place, not a timeout).

### `phf` vs. plain `match` consistency

- **Shared shadow registry (chosen).** One registry records aliases in
  declaration order; both backends query it before emitting an arm/key. An
  exact key is dropped when an earlier alias already covers it (identical
  exact alias, or an earlier case-insensitive alias whose ASCII-lowercase
  form matches); a case-insensitive guard is dropped when an earlier
  case-insensitive alias covers the same lowercase form. Because the `phf`
  map is consulted before the fallback guards and both are built in
  declaration order, `first_wins` resolution is identical in both modes.
- **Backend-specific dedup (rejected).** Deduplicating only the `phf` keys
  (the backend that hard-fails on duplicates) would let `match` and `phf`
  resolve the same input to different variants in `first_wins` mode.

Verified by `phf_consistency::phf_and_match_agree_on_conflicting_aliases`,
which compiles the same conflicting enum twice (with/without `use_phf`) and
asserts identical resolution across a probe matrix including mixed-case
inputs; it fails on the exact input where the backends diverge, not on a
global timeout.

## Verification evidence

- `cargo fetch --locked` — environment preparation (lockfile is committed).
- `cargo test --locked --workspace --all-features` — full suite green,
  including the trybuild fixtures in `strum_tests/tests/ui/alias_conflict*`
  (multi-source conflicts: `serialize`/`serialize`,
  `serialize`/`to_string`, `serialize_all` case conversion, enum- and
  variant-level `ascii_case_insensitive`, `use_phf`, Unicode) whose `.stderr`
  snapshots lock both diagnostic spans and the stable reason codes.
- Output invariance: `STRUM_DEBUG=1` dumps of the generated code for every
  pre-existing conflict-free test enum (from_str, phf, prefix, suffix,
  to_string, display, serialize_all, as_ref_str) are byte-identical before
  and after this change (compared order-independently to factor out parallel
  rustc output interleaving).

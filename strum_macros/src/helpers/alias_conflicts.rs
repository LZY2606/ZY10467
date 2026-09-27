//! Compile-time detection of conflicting input aliases for `EnumString`.
//!
//! Before any match code is generated, the normalized alias set of an enum is
//! computed from the same data flow the generator uses (`serialize`,
//! `to_string`, the variant ident converted through `serialize_all`, and the
//! effective `ascii_case_insensitive` flag of each variant). Two aliases of
//! *different* variants conflict when some input string would match both:
//!
//! * two case-sensitive aliases conflict iff they are exactly equal;
//! * an alias of a case-insensitive variant conflicts with any alias whose
//!   ASCII-lowercased form is equal, mirroring `str::eq_ignore_ascii_case`.
//!
//! Normalization is deliberately limited to ASCII case folding, because the
//! generated matcher (`eq_ignore_ascii_case`) is ASCII-only as well. Unicode
//! case folding is out of scope: two aliases that differ only in non-ASCII
//! case (e.g. `"Ä"` vs `"ä"`) are *not* considered a conflict, matching the
//! runtime behavior.
//!
//! Aliases of `disabled` variants and `default` variants are not input aliases
//! (they never appear in the generated matcher) and are skipped. Duplicate
//! aliases on the *same* variant are allowed and merged by the generator.
//!
//! A conflict is a compile error unless the enum opts into an explicit
//! priority strategy via `#[strum(on_alias_conflict = "first_wins")]`, in
//! which case the first declared variant wins - the same semantics the
//! generated `match` already has - and `use_phf` maps are deduplicated
//! accordingly so both backends behave identically.

use std::collections::HashMap;

use proc_macro2::Span;
use syn::{punctuated::Punctuated, Ident, LitStr, Token, Variant};

use super::type_props::StrumTypeProperties;
use super::variant_props::HasStrumVariantProperties;

/// Stable reason code embedded in every alias-conflict diagnostic.
pub const ALIAS_CONFLICT_CODE: &str = "strum::alias_conflict";

/// Explicit priority strategies that permit code generation despite
/// conflicting aliases.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AliasConflictStrategy {
    /// The first declared variant wins; later conflicting aliases are
    /// unreachable (and deduplicated out of `phf` maps).
    FirstWins,
}

/// Parse and validate the `on_alias_conflict` strategy, if any.
pub fn alias_conflict_strategy(
    type_properties: &StrumTypeProperties,
) -> syn::Result<Option<AliasConflictStrategy>> {
    match &type_properties.on_alias_conflict {
        None => Ok(None),
        Some(lit) => match lit.value().as_str() {
            "first_wins" => Ok(Some(AliasConflictStrategy::FirstWins)),
            other => Err(syn::Error::new_spanned(
                lit,
                format!(
                    "unknown `on_alias_conflict` strategy `{}`; expected \"first_wins\"",
                    other
                ),
            )),
        },
    }
}

/// Normalize an alias for comparison. ASCII-only by design: this mirrors the
/// `eq_ignore_ascii_case` matcher used by the generated code, so the
/// compile-time analysis can never disagree with the runtime matcher.
fn normalize(alias: &str) -> String {
    alias.to_ascii_lowercase()
}

struct AliasRegistration {
    serialization: LitStr,
    variant_ident: Ident,
    case_insensitive: bool,
}

/// Check the parse aliases of every non-disabled, non-default variant for
/// cross-variant conflicts. Returns a combined error pointing at both
/// attribute locations of every conflicting pair unless an explicit priority
/// strategy allows generation.
pub fn check_enum_string_aliases(
    variants: &Punctuated<Variant, Token![,]>,
    type_properties: &StrumTypeProperties,
) -> syn::Result<()> {
    let strategy = alias_conflict_strategy(type_properties)?;

    // Bucket aliases by their ASCII-normalized form so that case-sensitive
    // and case-insensitive aliases that could collide land together.
    let mut by_normalized: HashMap<String, Vec<AliasRegistration>> = HashMap::new();
    let mut combined_error: Option<syn::Error> = None;

    for variant in variants {
        let variant_properties = variant.get_variant_properties()?;

        // Disabled and default variants never contribute match arms, so their
        // attributes are not input aliases.
        if variant_properties.disabled.is_some() || variant_properties.default.is_some() {
            continue;
        }

        let case_insensitive = variant_properties
            .ascii_case_insensitive
            .unwrap_or(type_properties.ascii_case_insensitive);

        for serialization in variant_properties.get_serializations(type_properties.case_style) {
            let normalized = normalize(&serialization.value());

            // Duplicates on the same variant are allowed and merged by the
            // generator; only cross-variant collisions are conflicts.
            let conflict = by_normalized.get(&normalized).and_then(|registrations| {
                registrations.iter().find(|registration| {
                    registration.variant_ident != variant.ident
                        && (case_insensitive
                            || registration.case_insensitive
                            || registration.serialization.value() == serialization.value())
                })
            });

            if let Some(registration) = conflict {
                let mut error = syn::Error::new_spanned(
                    &serialization,
                    format!(
                        "{code}: alias `{new}` of variant `{new_variant}` conflicts with alias \
                         `{old}` of variant `{old_variant}`",
                        code = ALIAS_CONFLICT_CODE,
                        new = serialization.value(),
                        new_variant = variant.ident,
                        old = registration.serialization.value(),
                        old_variant = registration.variant_ident,
                    ),
                );
                error.combine(syn::Error::new_spanned(
                    &registration.serialization,
                    "conflicting alias is defined here",
                ));
                error.combine(syn::Error::new(
                    Span::call_site(),
                    "note: duplicate aliases on a single variant are allowed and merged; for \
                     cross-variant conflicts, add `#[strum(on_alias_conflict = \"first_wins\")]` \
                     to the enum to keep the first declared variant",
                ));
                match &mut combined_error {
                    Some(combined) => combined.combine(error),
                    None => combined_error = Some(error),
                }
            }

            by_normalized
                .entry(normalized)
                .or_default()
                .push(AliasRegistration {
                    serialization,
                    variant_ident: variant.ident.clone(),
                    case_insensitive,
                });
        }
    }

    match (combined_error, strategy) {
        (Some(error), None) => Err(error),
        // With an explicit priority strategy the generator deduplicates
        // aliases (first declared variant wins), so generation may proceed.
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::helpers::type_props::HasTypeProperties;

    fn check(input: &str) -> syn::Result<()> {
        let ast: syn::DeriveInput = syn::parse_str(input).unwrap();
        let type_properties = ast.get_type_properties().unwrap();
        let variants = match &ast.data {
            syn::Data::Enum(data) => &data.variants,
            _ => unreachable!(),
        };
        check_enum_string_aliases(variants, &type_properties)
    }

    #[test]
    fn no_conflict_is_ok() {
        check("enum E { A, #[strum(serialize = \"b\")] B }").unwrap();
    }

    #[test]
    fn same_variant_duplicates_are_allowed() {
        check("enum E { #[strum(serialize = \"a\", serialize = \"a\", to_string = \"a\")] A, B }")
            .unwrap();
    }

    #[test]
    fn exact_cross_variant_conflict_is_reported() {
        let err = check("enum E { #[strum(serialize = \"a\")] A, #[strum(to_string = \"a\")] B }")
            .unwrap_err();
        let message = err.to_string();
        assert!(message.contains(ALIAS_CONFLICT_CODE), "{}", message);
        assert!(message.contains('A'), "{}", message);
        assert!(message.contains('B'), "{}", message);
    }

    #[test]
    fn serialize_all_conflict_is_reported() {
        let err = check(
            "#[strum(serialize_all = \"snake_case\")] enum E { FooBar, #[strum(serialize = \"foo_bar\")] Other }",
        )
        .unwrap_err();
        assert!(err.to_string().contains(ALIAS_CONFLICT_CODE));
    }

    #[test]
    fn ascii_case_insensitive_conflict_is_reported() {
        let err =
            check("#[strum(ascii_case_insensitive)] enum E { #[strum(serialize = \"Red\")] A, #[strum(serialize = \"RED\")] B }")
                .unwrap_err();
        assert!(err.to_string().contains(ALIAS_CONFLICT_CODE));
    }

    #[test]
    fn mixed_sensitivity_conflict_is_reported() {
        // One case-insensitive side is enough: some input matches both.
        let err = check(
            "enum E { #[strum(ascii_case_insensitive, serialize = \"Red\")] A, #[strum(serialize = \"RED\")] B }",
        )
        .unwrap_err();
        assert!(err.to_string().contains(ALIAS_CONFLICT_CODE));
    }

    #[test]
    fn unicode_case_difference_is_not_a_conflict() {
        // Normalization is ASCII-only, mirroring `eq_ignore_ascii_case`:
        // "Äpfel" and "äpfel" never collide in the generated matcher.
        check("#[strum(ascii_case_insensitive)] enum E { #[strum(serialize = \"Äpfel\")] A, #[strum(serialize = \"äpfel\")] B }")
            .unwrap();
    }

    #[test]
    fn unicode_exact_duplicate_is_a_conflict() {
        let err = check(
            "enum E { #[strum(serialize = \"日本語\")] A, #[strum(serialize = \"日本語\")] B }",
        )
        .unwrap_err();
        assert!(err.to_string().contains(ALIAS_CONFLICT_CODE));
    }

    #[test]
    fn disabled_and_default_variants_are_skipped() {
        check(
            "enum E { #[strum(serialize = \"a\")] A, #[strum(disabled, serialize = \"a\")] B, #[strum(default)] C(String) }",
        )
        .unwrap();
    }

    #[test]
    fn first_wins_strategy_allows_generation() {
        check(
            "#[strum(on_alias_conflict = \"first_wins\")] enum E { #[strum(serialize = \"a\")] A, #[strum(serialize = \"a\")] B }",
        )
        .unwrap();
    }

    #[test]
    fn unknown_strategy_is_rejected() {
        let ast: syn::DeriveInput =
            syn::parse_str("#[strum(on_alias_conflict = \"last_wins\")] enum E { A }").unwrap();
        let type_properties = ast.get_type_properties().unwrap();
        let err = alias_conflict_strategy(&type_properties).unwrap_err();
        assert!(err.to_string().contains("first_wins"));
    }
}

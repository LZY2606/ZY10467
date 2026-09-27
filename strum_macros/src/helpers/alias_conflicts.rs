use syn::{punctuated::Punctuated, Ident, LitStr, Token, Variant};

use super::type_props::{AliasConflictStrategy, StrumTypeProperties};
use super::variant_props::HasStrumVariantProperties;

/// Stable reason codes surfaced in alias-conflict diagnostics. The
/// compile-fail snapshots lock these strings; only change them intentionally.
pub const REASON_EXACT: &str = "strum::alias_conflict::exact";
pub const REASON_ASCII_CASE: &str = "strum::alias_conflict::ascii_case";

struct AliasEntry {
    variant: Ident,
    lit: LitStr,
    raw: String,
    ascii_lower: String,
    case_insensitive: bool,
}

/// The normalized set of parse aliases of an enum, computed before any match
/// code is generated.
///
/// Normalization is limited to ASCII case folding (`to_ascii_lowercase`),
/// mirroring the generated matcher which relies on `eq_ignore_ascii_case`.
/// Unicode case folding is intentionally out of scope: non-ASCII aliases are
/// compared byte-for-byte, exactly like the runtime matcher does.
pub struct AliasRegistry {
    entries: Vec<AliasEntry>,
}

impl AliasRegistry {
    fn new() -> Self {
        AliasRegistry {
            entries: Vec::new(),
        }
    }

    /// Register one serialization alias of `variant`.
    ///
    /// Duplicate aliases on the *same* variant are merged silently. Aliases
    /// colliding across *different* variants are an error unless an explicit
    /// `alias_conflict` strategy was provided on the enum.
    fn register(
        &mut self,
        variant: &Ident,
        lit: LitStr,
        case_insensitive: bool,
        strategy: Option<AliasConflictStrategy>,
    ) -> syn::Result<()> {
        let raw = lit.value();
        let ascii_lower = raw.to_ascii_lowercase();

        if strategy.is_none() {
            if let Some(other) = self.entries.iter().find(|other| {
                other.variant != *variant
                    && other.ascii_lower == ascii_lower
                    && (case_insensitive || other.case_insensitive || other.raw == raw)
            }) {
                return Err(alias_conflict_error(variant, &lit, &raw, other));
            }
        }

        self.entries.push(AliasEntry {
            variant: variant.clone(),
            lit,
            raw,
            ascii_lower,
            case_insensitive,
        });
        Ok(())
    }

    /// Whether an exact-match key `value` emitted for the alias at `index`
    /// is already covered by an alias registered earlier (in declaration
    /// order). Used to keep `first_wins` semantics and to merge duplicate
    /// keys, so `phf` maps and plain `match` statements stay consistent.
    pub fn exact_shadowed_before(&self, index: usize, value: &str) -> bool {
        let lower = value.to_ascii_lowercase();
        self.entries[..index].iter().any(|entry| {
            if entry.case_insensitive {
                entry.ascii_lower == lower
            } else {
                entry.raw == value
            }
        })
    }

    /// Whether a case-insensitive guard for an alias whose ASCII-lowercased
    /// form is `ascii_lower` is already fully covered by an earlier
    /// case-insensitive alias.
    pub fn ci_shadowed_before(&self, index: usize, ascii_lower: &str) -> bool {
        self.entries[..index]
            .iter()
            .any(|entry| entry.case_insensitive && entry.ascii_lower == ascii_lower)
    }
}

fn alias_conflict_error(
    variant: &Ident,
    lit: &LitStr,
    raw: &str,
    other: &AliasEntry,
) -> syn::Error {
    let reason = if raw == &other.raw {
        REASON_EXACT
    } else {
        REASON_ASCII_CASE
    };

    let mut err = syn::Error::new_spanned(
        lit,
        format!(
            "parse alias `{}` of variant `{}` conflicts with an alias of variant `{}` [{}]",
            raw, variant, other.variant, reason
        ),
    );
    err.combine(syn::Error::new_spanned(
        &other.lit,
        format!(
            "alias `{}` of variant `{}` defined here; to allow conflicting aliases, add \
             `#[strum(alias_conflict = \"first_wins\")]` to the enum (the first declared \
             variant wins) [{}]",
            other.raw, other.variant, reason
        ),
    ));
    err
}

/// Compute the normalized alias set of an enum before macro expansion.
///
/// The walk mirrors the code generator exactly: `disabled` variants and the
/// `default` variant produce no match arms, so they register no aliases;
/// `prefix`/`suffix` only apply to display-oriented derives and do not
/// participate either. Every other variant registers all of its
/// `serialize`/`to_string` aliases (or its case-converted identifier).
pub fn build_alias_registry(
    variants: &Punctuated<Variant, Token![,]>,
    type_properties: &StrumTypeProperties,
) -> syn::Result<AliasRegistry> {
    let mut registry = AliasRegistry::new();
    for variant in variants {
        let variant_properties = variant.get_variant_properties()?;

        if variant_properties.disabled.is_some() || variant_properties.default.is_some() {
            continue;
        }

        let is_ascii_case_insensitive = variant_properties
            .ascii_case_insensitive
            .unwrap_or(type_properties.ascii_case_insensitive);

        for serialization in variant_properties.get_serializations(type_properties.case_style) {
            registry.register(
                &variant.ident,
                serialization,
                is_ascii_case_insensitive,
                type_properties.alias_conflict,
            )?;
        }
    }
    Ok(registry)
}

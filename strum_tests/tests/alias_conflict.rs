//! Regression tests for explicit alias-conflict diagnostics and the
//! `alias_conflict = "first_wins"` opt-in strategy.

use std::str::FromStr;
use strum::{Display, EnumString};

mod core {} // ensure macros call `::core`

// Same-variant duplicates are allowed and merged: they must compile and
// parse, on both the plain `match` path and (below) the `phf` path.
#[derive(Debug, Eq, PartialEq, EnumString)]
enum SameVariantDuplicates {
    #[strum(serialize = "a", serialize = "a")]
    Alpha,
    #[strum(serialize = "b", serialize = "B", ascii_case_insensitive)]
    Beta,
}

#[test]
fn same_variant_duplicates_are_merged() {
    assert_eq!(SameVariantDuplicates::Alpha, "a".parse().unwrap());
    assert_eq!(SameVariantDuplicates::Beta, "b".parse().unwrap());
    assert_eq!(SameVariantDuplicates::Beta, "B".parse().unwrap());
}

// Cross-variant conflicts require the explicit priority strategy.
#[derive(Debug, Eq, PartialEq, EnumString, Display)]
#[strum(alias_conflict = "first_wins")]
enum FirstWins {
    #[strum(serialize = "same")]
    First,
    #[strum(serialize = "same", to_string = "first-display")]
    Second,
}

#[test]
fn first_wins_resolves_to_first_declared_variant() {
    assert_eq!(FirstWins::First, "same".parse().unwrap());
    // Display output is untouched by the parse-side conflict resolution.
    assert_eq!("first-display", FirstWins::Second.to_string());
    assert_eq!("same", FirstWins::First.to_string());
}

#[derive(Debug, Eq, PartialEq, EnumString)]
#[strum(ascii_case_insensitive, alias_conflict = "first_wins")]
#[allow(dead_code)] // `Lower` is intentionally shadowed by `Upper`.
enum FirstWinsCaseInsensitive {
    #[strum(serialize = "Foo")]
    Upper,
    #[strum(serialize = "foo")]
    Lower,
}

#[test]
fn first_wins_case_insensitive() {
    assert_eq!(FirstWinsCaseInsensitive::Upper, "FOO".parse().unwrap());
    assert_eq!(FirstWinsCaseInsensitive::Upper, "foo".parse().unwrap());
    assert_eq!(FirstWinsCaseInsensitive::Upper, "Foo".parse().unwrap());
}

// Aliases of `disabled` variants are not parse aliases: they never
// participate in conflicts and are never matched.
#[derive(Debug, Eq, PartialEq, EnumString)]
#[allow(dead_code)] // `AlsoRed` is disabled; its alias must not conflict.
enum DisabledOverlap {
    Red,
    #[strum(disabled, serialize = "Red")]
    AlsoRed,
}

#[test]
fn disabled_variant_aliases_do_not_conflict() {
    assert_eq!(DisabledOverlap::Red, "Red".parse().unwrap());
}

// The `default` variant catches everything and registers no aliases, so its
// `serialize` attributes cannot conflict either.
#[derive(Debug, Eq, PartialEq, EnumString)]
enum DefaultOverlap {
    Red,
    #[strum(default, serialize = "Red")]
    Fallback(String),
}

#[test]
fn default_variant_aliases_do_not_conflict() {
    assert_eq!(DefaultOverlap::Red, "Red".parse().unwrap());
    assert_eq!(
        DefaultOverlap::Fallback("green".to_owned()),
        "green".parse().unwrap()
    );
}

// Unicode aliases are compared byte-for-byte; ASCII case folding does not
// fold non-ASCII characters. This is the documented responsibility boundary
// of conflict normalization and of the generated matcher alike.
#[derive(Debug, Eq, PartialEq, EnumString)]
enum UnicodeAliases {
    #[strum(serialize = "café", ascii_case_insensitive)]
    Cafe,
    #[strum(serialize = "日本語")]
    Japanese,
}

#[test]
fn unicode_aliases_match_byte_exactly() {
    assert_eq!(UnicodeAliases::Cafe, "café".parse().unwrap());
    assert_eq!(UnicodeAliases::Cafe, "CAFé".parse().unwrap());
    // `é` is not ASCII-folded to `É`: only exact bytes match here.
    assert!(UnicodeAliases::from_str("CAFÉ").is_err());
    assert_eq!(UnicodeAliases::Japanese, "日本語".parse().unwrap());
}

// `prefix`/`suffix` apply to display-oriented output only; the parse-side
// conflict rule is unaffected by them.
#[derive(Debug, Eq, PartialEq, EnumString, Display)]
#[strum(prefix = "color/")]
enum PrefixDoesNotChangeParsing {
    Red,
    Blue,
}

#[test]
fn prefix_does_not_change_parse_aliases() {
    assert_eq!(PrefixDoesNotChangeParsing::Red, "Red".parse().unwrap());
    assert_eq!("color/Red", PrefixDoesNotChangeParsing::Red.to_string());
}

// `phf` and plain `match` must resolve conflicts identically under
// `first_wins`. Both enums below accept the same aliases; every probe input
// must produce the same variant in both.
#[cfg(feature = "test_phf")]
mod phf_consistency {
    use std::str::FromStr;
    use strum::EnumString;

    macro_rules! conflicting_enum {
        ($name:ident $(, $extra:meta)?) => {
            #[derive(Debug, Clone, Eq, PartialEq, EnumString)]
            #[strum(ascii_case_insensitive, alias_conflict = "first_wins" $(, $extra)?)]
            #[allow(dead_code)] // `Second` is intentionally shadowed by `First`.
            enum $name {
                #[strum(serialize = "Foo")]
                First,
                #[strum(serialize = "foo", serialize = "FOO")]
                Second,
                #[strum(serialize = "bar", ascii_case_insensitive = false)]
                ExactBar,
                #[strum(serialize = "BAR")]
                CiBar,
                Plain,
            }
        };
    }

    conflicting_enum!(MatchEnum);
    conflicting_enum!(PhfEnum, use_phf);

    fn index_of_match(probe: &str) -> Option<u8> {
        MatchEnum::from_str(probe).ok().map(|v| match v {
            MatchEnum::First => 0,
            MatchEnum::Second => 1,
            MatchEnum::ExactBar => 2,
            MatchEnum::CiBar => 3,
            MatchEnum::Plain => 4,
        })
    }

    fn index_of_phf(probe: &str) -> Option<u8> {
        PhfEnum::from_str(probe).ok().map(|v| match v {
            PhfEnum::First => 0,
            PhfEnum::Second => 1,
            PhfEnum::ExactBar => 2,
            PhfEnum::CiBar => 3,
            PhfEnum::Plain => 4,
        })
    }

    #[test]
    fn phf_and_match_agree_on_conflicting_aliases() {
        let probes = [
            "Foo", "foo", "FOO", "fOo", "FoO", "bar", "BAR", "Bar", "bAr", "Plain", "plain",
            "PLAIN", "unknown",
        ];
        for probe in probes {
            assert_eq!(
                index_of_match(probe),
                index_of_phf(probe),
                "phf and match disagree on input {:?}",
                probe
            );
        }

        // And the resolution is genuinely first-wins.
        assert_eq!(PhfEnum::First, "FOO".parse().unwrap());
        assert_eq!(PhfEnum::First, "foo".parse().unwrap());
        assert_eq!(PhfEnum::ExactBar, "bar".parse().unwrap());
        assert_eq!(PhfEnum::CiBar, "BAR".parse().unwrap());
        assert_eq!(PhfEnum::CiBar, "Bar".parse().unwrap());
    }
}

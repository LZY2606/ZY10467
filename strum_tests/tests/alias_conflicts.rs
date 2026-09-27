mod core {} // ensure macros call `::core`

use strum::{Display, EnumString, ParseError};

// Duplicate aliases on the *same* variant are allowed and merged: the variant
// is reachable through the alias exactly once, and `Display` still uses the
// preferred name.
#[test]
fn same_variant_duplicates_are_merged() {
    #[derive(Debug, PartialEq, Eq, EnumString, Display)]
    enum Color {
        #[strum(serialize = "red", serialize = "red", to_string = "red")]
        Red,
        Blue,
    }

    assert_eq!("red".parse::<Color>().unwrap(), Color::Red);
    assert_eq!("Blue".parse::<Color>().unwrap(), Color::Blue);
    assert_eq!("green".parse::<Color>(), Err(ParseError::VariantNotFound));
    assert_eq!(Color::Red.to_string(), "red");
    assert_eq!(Color::Blue.to_string(), "Blue");
}

// Same-variant duplicates under `ascii_case_insensitive` merge as well, and
// ASCII case folding still applies to every input.
#[test]
fn same_variant_case_insensitive_duplicates_are_merged() {
    #[derive(Debug, PartialEq, Eq, EnumString)]
    enum Color {
        #[strum(ascii_case_insensitive, serialize = "Red", serialize = "red")]
        Red,
    }

    assert_eq!("red".parse::<Color>().unwrap(), Color::Red);
    assert_eq!("RED".parse::<Color>().unwrap(), Color::Red);
    assert_eq!("rEd".parse::<Color>().unwrap(), Color::Red);
    assert_eq!("blue".parse::<Color>(), Err(ParseError::VariantNotFound));
}

// With an explicit `on_alias_conflict = "first_wins"` strategy, generation is
// allowed despite a cross-variant conflict and the first declared variant
// wins, exactly like the first arm of the generated `match`.
#[test]
fn first_wins_strategy_resolves_conflicts() {
    #[derive(Debug, PartialEq, Eq, EnumString, Display)]
    #[strum(on_alias_conflict = "first_wins")]
    enum Color {
        #[strum(serialize = "scarlet")]
        Red,
        #[strum(to_string = "scarlet")]
        Scarlet,
        Blue,
    }

    // The first declared variant wins for parsing...
    assert_eq!("scarlet".parse::<Color>().unwrap(), Color::Red);
    assert_eq!("Blue".parse::<Color>().unwrap(), Color::Blue);
    assert_eq!("crimson".parse::<Color>(), Err(ParseError::VariantNotFound));
    // ...while `Display` still emits each variant's own preferred name, so a
    // round trip through the second variant lands on the first one.
    assert_eq!(Color::Red.to_string(), "scarlet");
    assert_eq!(Color::Scarlet.to_string(), "scarlet");
    assert_eq!(
        Color::Scarlet.to_string().parse::<Color>().unwrap(),
        Color::Red
    );
}

#[test]
fn first_wins_strategy_resolves_case_insensitive_conflicts() {
    #[derive(Debug, PartialEq, Eq, EnumString)]
    #[strum(ascii_case_insensitive, on_alias_conflict = "first_wins")]
    enum Color {
        #[strum(serialize = "Red")]
        Red,
        #[strum(serialize = "RED")]
        Crimson,
    }

    assert_eq!("red".parse::<Color>().unwrap(), Color::Red);
    assert_eq!("RED".parse::<Color>().unwrap(), Color::Red);
    assert_eq!("rEd".parse::<Color>().unwrap(), Color::Red);
}

// Aliases of `disabled` variants are not input aliases: they neither conflict
// with nor shadow the aliases of active variants.
#[test]
fn disabled_variant_aliases_do_not_conflict() {
    #[derive(Debug, PartialEq, Eq, EnumString)]
    enum Color {
        #[strum(serialize = "yellow")]
        Yellow,
        #[strum(disabled, serialize = "yellow")]
        DisabledYellow,
    }

    assert_eq!("yellow".parse::<Color>().unwrap(), Color::Yellow);
}

// A `default` variant contributes no aliases and does not mask conflicts
// elsewhere; unknown input still falls through to it.
#[test]
fn default_variant_coexists_with_alias_rules() {
    #[derive(Debug, PartialEq, Eq, EnumString)]
    enum Color {
        #[strum(serialize = "red", serialize = "red")]
        Red,
        #[strum(default)]
        Unknown(String),
    }

    assert_eq!("red".parse::<Color>().unwrap(), Color::Red);
    assert_eq!(
        "magenta".parse::<Color>().unwrap(),
        Color::Unknown("magenta".to_string())
    );
}

// Unicode aliases are matched exactly; conflict normalization is ASCII-only,
// mirroring `eq_ignore_ascii_case`, so aliases that differ only in non-ASCII
// case belong to different variants and parse independently.
#[test]
fn unicode_aliases_use_ascii_only_normalization() {
    #[derive(Debug, PartialEq, Eq, EnumString)]
    #[strum(ascii_case_insensitive)]
    enum Word {
        #[strum(serialize = "Äpfel")]
        Apples,
        #[strum(serialize = "äpfel")]
        ApplesLower,
        #[strum(serialize = "日本語")]
        Japanese,
    }

    assert_eq!("Äpfel".parse::<Word>().unwrap(), Word::Apples);
    assert_eq!("äpfel".parse::<Word>().unwrap(), Word::ApplesLower);
    // ASCII case folding still applies within each alias.
    assert_eq!("ÄPFEL".parse::<Word>().unwrap(), Word::Apples);
    assert_eq!("日本語".parse::<Word>().unwrap(), Word::Japanese);
    assert_eq!("日本".parse::<Word>(), Err(ParseError::VariantNotFound));
}

// `prefix`/`suffix` only affect the display side; parse aliases (and thus the
// conflict rules) are computed from the bare serializations, unchanged.
#[test]
fn prefix_and_suffix_do_not_change_parse_aliases() {
    #[derive(Debug, PartialEq, Eq, EnumString, Display)]
    #[strum(prefix = "colour/", suffix = "!")]
    enum Color {
        #[strum(serialize = "red", serialize = "red")]
        Red,
        Blue,
    }

    assert_eq!("red".parse::<Color>().unwrap(), Color::Red);
    assert_eq!("Blue".parse::<Color>().unwrap(), Color::Blue);
    assert_eq!(
        "colour/red".parse::<Color>(),
        Err(ParseError::VariantNotFound)
    );
    assert_eq!(Color::Red.to_string(), "colour/red!");
    assert_eq!(Color::Blue.to_string(), "colour/Blue!");
}

#[cfg(feature = "test_phf")]
mod phf_consistency {
    use super::*;

    // A case-insensitive alias that is already lowercase used to break the
    // `phf` backend with a duplicate-key error from inside `phf`, while the
    // plain `match` backend worked. Both backends now merge the duplicate
    // keys and behave identically.
    #[test]
    fn phf_lowercase_alias_matches_match_backend() {
        #[derive(Debug, PartialEq, Eq, Clone, EnumString)]
        #[strum(use_phf)]
        enum PhfColor {
            #[strum(ascii_case_insensitive, serialize = "red")]
            Red,
            Blue,
        }

        #[derive(Debug, PartialEq, Eq, EnumString)]
        enum MatchColor {
            #[strum(ascii_case_insensitive, serialize = "red")]
            Red,
            Blue,
        }

        for input in ["red", "RED", "rEd", "Blue", "BLUE"] {
            let phf_result = input
                .parse::<PhfColor>()
                .map(|color| color == PhfColor::Red);
            let match_result = input
                .parse::<MatchColor>()
                .map(|color| color == MatchColor::Red);
            assert_eq!(phf_result, match_result, "input {:?}", input);
        }
        assert_eq!(
            "green".parse::<PhfColor>(),
            Err(ParseError::VariantNotFound)
        );
    }

    // Same-variant duplicates are merged out of `phf` maps exactly like they
    // are merged out of `match` arms.
    #[test]
    fn phf_same_variant_duplicates_are_merged() {
        #[derive(Debug, PartialEq, Eq, Clone, EnumString)]
        #[strum(use_phf)]
        enum Color {
            #[strum(serialize = "red", serialize = "red", to_string = "red")]
            Red,
            #[strum(ascii_case_insensitive, serialize = "blue", serialize = "BLUE")]
            Blue,
        }

        assert_eq!("red".parse::<Color>().unwrap(), Color::Red);
        assert_eq!("blue".parse::<Color>().unwrap(), Color::Blue);
        assert_eq!("bLuE".parse::<Color>().unwrap(), Color::Blue);
        assert_eq!("green".parse::<Color>(), Err(ParseError::VariantNotFound));
    }

    // Under `first_wins`, the `phf` backend resolves conflicts to the first
    // declared variant exactly like the plain `match` backend.
    #[test]
    fn phf_first_wins_matches_match_backend() {
        #[derive(Debug, PartialEq, Eq, Clone, EnumString)]
        #[strum(use_phf, on_alias_conflict = "first_wins")]
        enum PhfColor {
            #[strum(serialize = "scarlet")]
            Red,
            #[strum(ascii_case_insensitive, serialize = "SCARLET")]
            Scarlet,
            Blue,
        }

        #[derive(Debug, PartialEq, Eq, EnumString)]
        #[strum(on_alias_conflict = "first_wins")]
        enum MatchColor {
            #[strum(serialize = "scarlet")]
            Red,
            #[strum(ascii_case_insensitive, serialize = "SCARLET")]
            Scarlet,
            Blue,
        }

        for input in ["scarlet", "SCARLET", "Scarlet", "Blue", "green"] {
            let phf_result = input.parse::<PhfColor>().map(|color| match color {
                PhfColor::Red => 0,
                PhfColor::Scarlet => 1,
                PhfColor::Blue => 2,
            });
            let match_result = input.parse::<MatchColor>().map(|color| match color {
                MatchColor::Red => 0,
                MatchColor::Scarlet => 1,
                MatchColor::Blue => 2,
            });
            assert_eq!(phf_result, match_result, "input {:?}", input);
        }
    }
}

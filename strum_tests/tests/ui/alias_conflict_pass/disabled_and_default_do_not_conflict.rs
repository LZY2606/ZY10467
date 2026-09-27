use strum::EnumString;

// `disabled` and `default` variants register no parse aliases.
#[derive(Debug, EnumString)]
enum Color {
    Red,
    #[strum(disabled, serialize = "Red")]
    DisabledRed,
    #[strum(default, serialize = "Red")]
    Fallback(String),
}

fn main() {
    assert!(matches!("Red".parse::<Color>(), Ok(Color::Red)));
}

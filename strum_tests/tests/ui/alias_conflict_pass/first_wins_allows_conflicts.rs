use strum::EnumString;

// The explicit priority strategy opts out of the hard error.
#[derive(Debug, EnumString)]
#[strum(alias_conflict = "first_wins")]
enum Color {
    #[strum(serialize = "red")]
    Red,
    #[strum(serialize = "red")]
    AlsoRed,
}

fn main() {
    assert!(matches!("red".parse::<Color>(), Ok(Color::Red)));
}

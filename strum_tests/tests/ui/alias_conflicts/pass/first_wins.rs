use strum::EnumString;

#[derive(Debug, PartialEq, Eq, EnumString)]
#[strum(on_alias_conflict = "first_wins")]
enum Color {
    #[strum(serialize = "scarlet")]
    Red,
    #[strum(to_string = "scarlet")]
    Scarlet,
}

fn main() {
    assert_eq!("scarlet".parse::<Color>().unwrap(), Color::Red);
}

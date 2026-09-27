use strum::EnumString;

#[derive(Debug, PartialEq, Eq, EnumString)]
enum Color {
    #[strum(serialize = "yellow")]
    Yellow,
    #[strum(disabled, serialize = "yellow")]
    DisabledYellow,
    #[strum(default)]
    Unknown(String),
}

fn main() {
    assert_eq!("yellow".parse::<Color>().unwrap(), Color::Yellow);
}

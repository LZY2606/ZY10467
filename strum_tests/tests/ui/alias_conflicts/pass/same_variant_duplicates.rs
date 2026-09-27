use strum::EnumString;

#[derive(Debug, PartialEq, Eq, EnumString)]
enum Color {
    #[strum(serialize = "red", serialize = "red", to_string = "red")]
    Red,
    #[strum(ascii_case_insensitive, serialize = "blue", serialize = "BLUE")]
    Blue,
}

fn main() {
    assert_eq!("red".parse::<Color>().unwrap(), Color::Red);
    assert_eq!("BLUE".parse::<Color>().unwrap(), Color::Blue);
}

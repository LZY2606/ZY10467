use strum::EnumString;

// Duplicate aliases on the *same* variant are allowed and merged.
#[derive(Debug, EnumString)]
enum Color {
    #[strum(serialize = "red", serialize = "red")]
    Red,
    #[strum(serialize = "blue", serialize = "BLUE", ascii_case_insensitive)]
    Blue,
}

fn main() {
    to_parse();
}

fn to_parse() {
    let _ = "red".parse::<Color>().unwrap();
    let _ = "blue".parse::<Color>().unwrap();
}

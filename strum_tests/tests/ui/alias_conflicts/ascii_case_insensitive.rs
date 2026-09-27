use strum::EnumString;

#[derive(Debug, EnumString)]
#[strum(ascii_case_insensitive)]
enum Color {
    #[strum(serialize = "Red")]
    Red,
    #[strum(serialize = "RED")]
    Crimson,
}

fn main() {}

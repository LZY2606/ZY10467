use strum::EnumString;

#[derive(Debug, EnumString)]
enum Color {
    #[strum(serialize = "red")]
    Red,
    #[strum(serialize = "red")]
    Crimson,
    #[strum(default)]
    Unknown(String),
}

fn main() {}

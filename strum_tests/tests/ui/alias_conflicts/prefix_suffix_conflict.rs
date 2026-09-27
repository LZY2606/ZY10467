use strum::EnumString;

#[derive(Debug, EnumString)]
#[strum(prefix = "colour/", suffix = "!")]
enum Color {
    #[strum(serialize = "red")]
    Red,
    #[strum(to_string = "red")]
    Crimson,
}

fn main() {}

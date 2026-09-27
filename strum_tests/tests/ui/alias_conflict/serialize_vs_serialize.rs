use strum::EnumString;

#[derive(Debug, EnumString)]
enum Color {
    #[strum(serialize = "red")]
    Red,
    #[strum(serialize = "red")]
    AlsoRed,
}

fn main() {}

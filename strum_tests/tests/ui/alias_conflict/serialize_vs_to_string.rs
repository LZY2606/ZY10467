use strum::EnumString;

#[derive(Debug, EnumString)]
enum Color {
    #[strum(serialize = "red")]
    Red,
    #[strum(to_string = "red")]
    AlsoRed,
}

fn main() {}

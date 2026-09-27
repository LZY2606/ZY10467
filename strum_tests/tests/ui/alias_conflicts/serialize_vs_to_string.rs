use strum::EnumString;

#[derive(Debug, EnumString)]
enum Color {
    #[strum(serialize = "scarlet")]
    Red,
    #[strum(to_string = "scarlet")]
    Scarlet,
}

fn main() {}

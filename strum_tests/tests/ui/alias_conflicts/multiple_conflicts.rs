use strum::EnumString;

#[derive(Debug, EnumString)]
enum Color {
    #[strum(serialize = "red")]
    Red,
    #[strum(serialize = "red")]
    Crimson,
    #[strum(serialize = "blue")]
    Blue,
    #[strum(to_string = "blue")]
    Azure,
}

fn main() {}

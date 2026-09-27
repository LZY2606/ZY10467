use strum::EnumString;

#[derive(Debug, EnumString)]
enum Color {
    #[strum(serialize = "Black", ascii_case_insensitive)]
    Black,
    #[strum(serialize = "BLACK")]
    AlsoBlack,
}

fn main() {}

use strum::EnumString;

#[derive(Debug, EnumString)]
#[strum(ascii_case_insensitive)]
enum Color {
    #[strum(serialize = "Black")]
    Black,
    #[strum(serialize = "black")]
    AlsoBlack,
}

fn main() {}

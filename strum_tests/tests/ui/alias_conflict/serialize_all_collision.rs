use strum::EnumString;

#[derive(Debug, EnumString)]
#[strum(serialize_all = "snake_case")]
enum Brightness {
    DarkBlack,
    #[strum(serialize = "dark_black")]
    Other,
}

fn main() {}

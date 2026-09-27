use strum::EnumString;

#[derive(Debug, EnumString)]
#[strum(use_phf)]
enum Color {
    Red,
    #[strum(serialize = "Red")]
    AlsoRed,
}

fn main() {}

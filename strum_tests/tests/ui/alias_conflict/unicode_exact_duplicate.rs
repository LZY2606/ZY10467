use strum::EnumString;

#[derive(Debug, EnumString)]
enum Drink {
    #[strum(serialize = "café")]
    Coffee,
    #[strum(serialize = "café")]
    AlsoCoffee,
}

fn main() {}

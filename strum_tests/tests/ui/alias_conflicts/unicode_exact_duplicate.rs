use strum::EnumString;

#[derive(Debug, EnumString)]
enum Word {
    #[strum(serialize = "日本語")]
    Japanese,
    #[strum(serialize = "日本語")]
    Nihongo,
}

fn main() {}

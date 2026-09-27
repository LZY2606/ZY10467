use strum::EnumString;

#[derive(Debug, EnumString)]
#[strum(on_alias_conflict = "last_wins")]
enum Color {
    Red,
}

fn main() {}

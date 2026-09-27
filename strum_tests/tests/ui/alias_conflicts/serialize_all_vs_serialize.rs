use strum::EnumString;

#[derive(Debug, EnumString)]
#[strum(serialize_all = "snake_case")]
enum Color {
    FooBar,
    #[strum(serialize = "foo_bar")]
    Other,
}

fn main() {}

use strum::EnumString;

#[derive(Debug, EnumString)]
#[strum(serialize_all = "snake_case")]
#[allow(non_camel_case_types)]
enum Color {
    FooBar,
    Foo_Bar,
}

fn main() {}

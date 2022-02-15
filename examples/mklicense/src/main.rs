use ezmenu::{Menu, StructField};

#[ezmenu::parsed]
enum Type {
    MIT,
    GPL,
    BSD,
}

#[derive(Menu)]
struct License {
    #[menu(msg("Author"))]
    author: String,
    #[menu(msg("Project name"))]
    proj_name: String,
    #[menu(msg("Type"))]
    ty: Type,
}

fn main() {
    let license: u8 = StructField::from("hello").init_build().unwrap();
}

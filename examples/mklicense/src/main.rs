use ezmenu::{Menu, MenuVec};

#[derive(Debug)]
#[ezmenu::parsed]
enum Type {
    MIT,
    GPL,
    BSD,
}

#[derive(Menu)]
struct License {
    authors: MenuVec<String>,
    proj_name: String,
    ty: Type,
}

fn main() {
    let _l = License::from_menu_unwrap();
}

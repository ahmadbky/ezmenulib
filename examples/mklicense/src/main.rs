use ezmenu::{Menu, MenuVec, StructField, StructFieldFormatting, StructMenu};

#[derive(Debug)]
#[ezmenu::parsed]
enum Type {
    MIT,
    GPL,
    BSD,
}

// #[derive(Menu)]
// #[menu(
//     title("Describe the license you want to build"),
//     chip("--> "),
//     prefix = ">> ",
//     new_line = true
// )]
struct License {
    //#[menu(msg = "List of the authors")]
    authors: MenuVec<String>,
    //#[menu(msg = "The project name")]
    proj_name: String,
    //#[menu(msg = "The type of the license (MIT, GPL, BSD...)", default = "MIT")]
    ty: Type,
    //#[menu(msg = "Year of the project", default = 2022)]
    year: u16,
}

// fn _main() {
//     let License {
//         authors,
//         proj_name,
//         ty,
//         year,
//     } = License::from_menu_unwrap();
//
//     println!("authors: {:?}", authors);
//     println!("project name: {:?}", proj_name);
//     println!("type: {:?}", ty);
//     println!("year: {:?}", year);
// }

fn main() {
    let mut menu = StructMenu::from([
        StructField::from("bonsoir").default("non"),
        StructField::from("salut").fmt(StructFieldFormatting {
            chip: "--> ",
            ..Default::default()
        }),
    ])
    .fmt(StructFieldFormatting {
        chip: "==> ",
        ..Default::default()
    });
    let age: u8 = menu.next_output_unwrap();
}

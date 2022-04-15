use ezmenulib::customs::{MenuOption, MenuVec};
use ezmenulib::{prelude::*, Selectable};
use std::error::Error;
use std::io::Write;

#[derive(Debug)]
enum Type {
    MIT,
    GPL,
    BSD,
}

impl Default for Type {
    fn default() -> Self {
        Self::MIT
    }
}

impl Selectable for Type {
    fn values() -> Vec<(&'static str, Self)> {
        vec![("MIT", Self::MIT), ("GPL", Self::GPL), ("BSD", Self::BSD)]
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = MenuStream::default();
    writeln!(stream, "Describe your project")?;

    let mut license_menu = Values::from_ref(&mut stream);

    let authors: MenuVec<String> =
        license_menu.written(&Written::from("Authors").default_value("defaulmzlkejft"))?;
    let name: MenuOption<String> = license_menu.written(&Written::from("Project name"))?;
    let date: u16 = license_menu.written(&Written::from("License date").default_value("2022"))?;
    let ty: Type = license_menu.selected_or_default(Selected::from("Select a license type"));

    println!(
        "{:?} License, Copyright (C) {} {}\n{}",
        ty,
        date,
        authors.join(", "),
        name
    );

    Ok(())
}

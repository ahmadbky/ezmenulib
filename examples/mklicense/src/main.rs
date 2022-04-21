use ezmenulib::customs::{MenuOption, MenuVec};
use ezmenulib::prelude::*;
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

impl Selectable<3> for Type {
    fn values() -> [(&'static str, Self); 3] {
        [("MIT", Self::MIT), ("GPL", Self::GPL), ("BSD", Self::BSD)]
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = MenuStream::default();
    writeln!(stream, "Describe your project")?;

    let mut lic = Values::from(&mut stream).format(Format {
        prefix: "==> ",
        chip: " = ",
        ..Default::default()
    });

    let authors: MenuVec<String> = lic.written(&Written::from("Authors"))?;
    let name: MenuOption<String> = lic.written(&Written::from("Project name"))?;
    let date: u16 = lic.written(&Written::from("License date").default_value("2022"))?;
    let ty: Type = lic.selected(Selected::from("Select a license type").default(0))?;

    println!(
        "{:?} License, Copyright (C) {} {}\n{}",
        ty,
        date,
        authors.join(", "),
        name
    );

    Ok(())
}

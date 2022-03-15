use ezmenulib::customs::{MenuOption, MenuVec};
use ezmenulib::prelude::*;
use std::error::Error;
use std::str::FromStr;

#[derive(Debug)]
enum Type {
    MIT,
    GPL,
    BSD,
}

impl FromStr for Type {
    type Err = MenuError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "mit" => Ok(Self::MIT),
            "gpl" => Ok(Self::GPL),
            "bsd" => Ok(Self::BSD),
            _ => Err(MenuError::from(format!("unknown license: {}", s))),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut license = ValueMenu::from([
        Field::Value(ValueField::from("Authors").default_value("zmlfkgj")),
        Field::Value(ValueField::from("Project name")),
        Field::Value(ValueField::from("License date").default_value("2022")),
        Field::Select(
            SelectMenu::from([
                SelectField::from("MIT"),
                SelectField::from("GPL"),
                SelectField::from("BSD"),
            ])
            .title(SelectTitle::from("Select a license type"))
            .default(0),
        ),
    ]);

    let authors: MenuVec<String> = license.next_output()?;
    let name: MenuOption<String> = license.next_output()?;
    let date: u16 = license.next_output()?;
    let ty: Type = license.next_output()?;

    println!(
        "{:?} License, Copyright (C) {} {}\n{}",
        ty,
        date,
        authors.join(", "),
        name
    );

    Ok(())
}

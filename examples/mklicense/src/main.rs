use ezmenulib::customs::{MenuOption, MenuVec};
use ezmenulib::prelude::*;
use std::error::Error;

#[derive(Debug)]
enum Type {
    MIT,
    GPL,
    BSD,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut license = ValueMenu::from([
        ValueField::Value(Value::from("Authors").default_value("zmlfkgj")),
        ValueField::Value(Value::from("Project name")),
        ValueField::Value(Value::from("License date").default_value("2022")),
        ValueField::Select(
            SelectMenu::from([
                SelectField::new("MIT", Type::MIT),
                SelectField::new("GPL", Type::GPL),
                SelectField::new("BSD", Type::BSD),
            ])
            .title(SelectTitle::from("Select a license type"))
            .default(1),
        ),
    ])
    .title("Describe your project");

    let authors: MenuVec<String> = license.next_value()?;
    let name: MenuOption<String> = license.next_value()?;
    let date: u16 = license.next_value()?;
    let ty: Type = license.next_select()?;

    println!(
        "{:?} License, Copyright (C) {} {}\n{}",
        ty,
        date,
        authors.join(", "),
        name
    );

    Ok(())
}

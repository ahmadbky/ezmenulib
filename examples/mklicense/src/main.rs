#![allow(unused)]

use ezmenulib::{
    Field, MenuBuilder, MenuError, MenuResult, MenuVec, SelectField, SelectMenu, TitlePos,
    ValueField, ValueFieldFormatting, ValueMenu,
};
use std::convert::Infallible;
use std::io::Write;
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
            _ => Err(MenuError::from("bonsoir")),
        }
    }
}

fn values_test() {
    let mut menu = ValueMenu::from([
        Field::Value(ValueField::from("Authors")),
        Field::Value(ValueField::from("Project name").fmt(ValueFieldFormatting {
            chip: "--> ",
            ..Default::default()
        })),
        Field::Value(ValueField::from("Date").default("2022")),
    ])
    .fmt(ValueFieldFormatting {
        chip: "==> ",
        ..Default::default()
    });
    let _: MenuVec<String> = menu.next_output().unwrap();
    let _: String = menu.next_output().unwrap();
    let _: u16 = menu.next_output().unwrap();

    let _: i32 = ValueField::from("Give the license type")
        .build_init()
        .unwrap();
}

fn deux(e: &mut std::io::Stdout) -> MenuResult<()> {
    use std::io::Write;
    e.write_all(b"bonsoir\n").map_err(MenuError::from)
}

fn main() {
    //let amount: u8 = SelectMenu::from([SelectField::from("4")])
    //    .title("bonsoir")
    //    .title_pos(TitlePos::Bottom)
    //    .next_output()
    //    .unwrap();

    let mut test = ValueMenu::from([
        Field::Value(ValueField::from("Author name")),
        Field::Select(
            SelectMenu::from([SelectField::from("MIT"), SelectField::from("GPL")])
                .title("Choose a license type")
                .title_pos(TitlePos::Bottom)
                .default(4),
        ),
    ])
    .fmt(ValueFieldFormatting {
        chip: "--> ",
        ..Default::default()
    });

    let name: String = test.next_output().unwrap();
    let ty: Type = test.next_output().unwrap();

    println!("{:?} {:?}", name, ty);
}

#![allow(unused)]

use ezmenulib::{
    Field, MenuBuilder, MenuError, MenuResult, MenuVec, SelectField, SelectMenu, SelectTitle,
    TitlePos, ValueField, ValueFieldFormatting, ValueMenu,
};
use std::convert::Infallible;
use std::io::{Stdout, Write};
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

fn submenu_test() {
    //let amount: u8 = SelectMenu::from([SelectField::from("4")])
    //    .title("bonsoir")
    //    .title_pos(TitlePos::Bottom)
    //    .next_output()
    //    .unwrap();

    let mut test = ValueMenu::from([
        Field::Value(ValueField::from("Author name")),
        Field::Select(
            SelectMenu::from([SelectField::from("MIT"), SelectField::from("GPL")])
                .title(SelectTitle::from("Choose a license type").pos(TitlePos::Bottom))
                .default(0),
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

enum Age {
    One,
    Two,
    Three,
    More,
}

impl FromStr for Age {
    type Err = MenuError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Self::One),
            "2" => Ok(Self::Two),
            "3" => Ok(Self::Three),
            "More" => Ok(Self::More),
            _ => Err(MenuError::from("yo i dont know whats that age")),
        }
    }
}

fn submenu_primitives_test() {
    let mut amount = ValueMenu::from([
        Field::Value(ValueField::from("whats your name?")),
        Field::Select(SelectMenu::from([
            SelectField::from("1"),
            SelectField::from("2"),
            SelectField::from("3"),
            SelectField::from("More"),
        ])),
    ]);

    let name: String = amount.next_output().unwrap();
    let age: Age = amount.next_output().unwrap();
}

fn main() {
    let amount = SelectMenu::from([
        SelectField::from("5"),
        SelectField::from("6"),
        SelectField::from("54"),
    ])
    .title(SelectTitle::from("quel age avez vous").pos(TitlePos::Bottom));
}

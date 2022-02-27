#![allow(unused)]

use ezmenulib::{
    MenuBuilder, MenuError, MenuResult, MenuVec, SelectField, SelectMenu, TitlePos, ValueField,
    ValueFieldFormatting, ValueMenu,
};
use std::io::Write;

#[derive(Debug, Clone)]
enum Test {
    One,
    Two,
    Three,
}

fn values_test() {
    let mut menu = ValueMenu::from([
        ValueField::from("Authors"),
        ValueField::from("Project name").fmt(ValueFieldFormatting {
            chip: "--> ",
            ..Default::default()
        }),
        ValueField::from("Date").default("2022"),
    ])
    .fmt(ValueFieldFormatting {
        chip: "==> ",
        ..Default::default()
    });
    let _: MenuVec<String> = menu.next_output().unwrap();
    let _: String = menu.next_output().unwrap();
    let _: u16 = menu.next_output().unwrap();

    let _: i32 = ValueField::from("Give the license type")
        .init_build()
        .unwrap();
}

fn deux(e: &mut std::io::Stdout) -> MenuResult<()> {
    use std::io::Write;
    e.write_all(b"bonsoir\n").map_err(MenuError::from)
}

fn main() {
    let amount: u8 = SelectMenu::from([])
        .title("bonsoir")
        .title_pos(TitlePos::Bottom)
        .next_output()
        .unwrap();

    println!("{:?}", amount);
}

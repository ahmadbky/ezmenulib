use crate::field::StructFieldFormatting;
use crate::menu::{Menu, StructMenu};
use crate::StructField;
use std::io::{stdout, Write};

#[test]
fn basic_menu() {
    let input = "ahmad\n56";

    let mut menu = StructMenu::new(input.as_bytes(), stdout())
        .title("bonjour")
        .fmt(StructFieldFormatting {
            chip: "",
            prefix: "",
            new_line: false,
            default: false,
        })
        .with_field(StructField::from("author"))
        .with_field(StructField::from("age").fmt(StructFieldFormatting {
            chip: "",
            prefix: "",
            new_line: true,
            default: false,
        }));

    let name: String = menu
        .next_map(|s: String, w| {
            if s.to_lowercase() == "ahmad" {
                writeln!(w, "omg jte connais")?;
            }
            Ok(s)
        })
        .unwrap();
    let age: u8 = menu.next().unwrap();

    println!("name={}, age={}", name, age);
}

#[test]
fn inherited_style() {
    let input = "13\n13";
    let mut menu = StructMenu::new(input.as_bytes(), stdout())
        .title("give 2 random numbers")
        .fmt(StructFieldFormatting {
            chip: "- ",
            prefix: ">> ",
            new_line: true,
            default: false,
        })
        .with_field(
            StructField::from("give a first number").fmt(StructFieldFormatting {
                chip: "* ",
                ..Default::default()
            }),
        )
        .with_field(StructField::from("give a second number"));

    let first: i32 = menu.next().unwrap();
    let _second: i32 = menu
        .next_map(|n, w| {
            if n == first {
                writeln!(w, "you entered the same number: {}", n)?;
            }
            Ok(n)
        })
        .unwrap();
}

use ezmenu::{Menu, MenuVec, ValueField, ValueFieldFormatting, ValueMenu};

fn main() {
    let mut menu = ValueMenu::from([
        ValueField::from("bonsoir").default("non"),
        ValueField::from("salut").fmt(ValueFieldFormatting {
            chip: "--> ",
            ..Default::default()
        }),
    ])
    .fmt(ValueFieldFormatting {
        chip: "==> ",
        ..Default::default()
    });
    let age: u8 = menu.next_output().unwrap();
}

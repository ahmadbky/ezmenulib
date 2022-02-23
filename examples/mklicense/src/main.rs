use ezmenu::{Menu, MenuVec, ValueField, ValueFieldFormatting, ValueMenu};

fn main() {
    let mut menu = ValueMenu::from([
        ValueField::from("Authors"),
        ValueField::from("Project name"),
        ValueField::from("Date").default("2022"),
    ])
    .fmt(ValueFieldFormatting {
        chip: "==> ",
        ..Default::default()
    });
    let _authors: MenuVec<String> = menu.next_output().unwrap();
}

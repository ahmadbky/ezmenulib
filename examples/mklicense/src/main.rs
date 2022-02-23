use ezmenu::{Menu, MenuVec, ValueField, ValueFieldFormatting, ValueMenu};

fn main() {
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

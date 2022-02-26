use ezmenulib::{
    MenuBuilder, MenuVec, SelectField, SelectMenu, TitlePos, ValueField, ValueFieldFormatting,
    ValueMenu,
};

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

fn main() {
    let amount = SelectMenu::from([
        SelectField::new("un", 1),
        SelectField::new("deux", 2).chip(" -- "),
        SelectField::new("trois", 3).chip(" --- "),
    ])
    .title("bonsoir")
    .title_pos(TitlePos::Bottom)
    .chip(" xd ")
    .next_output()
    .unwrap();

    println!("{}", amount);
}

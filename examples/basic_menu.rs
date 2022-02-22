use ezmenu::Menu;
use ezmenu::ValueField;
use ezmenu::ValueMenu;

//#[derive(Menu)]
//#[menu(title("Describe a person"))]
struct Person {
    lastname: String,
    firstname: String,
    age: u8,
}

fn main() {
    //let Person { firstname, .. } = Person::from_menu().unwrap();
    let mut menu = ValueMenu::from([
        ValueField::from("bonsoir"),
        ValueField::from("yes"),
    ]);
    let age: u8 = menu.next_output().unwrap();
    println!("goodbye {}!", age);
}

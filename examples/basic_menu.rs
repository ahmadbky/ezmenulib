use ezmenu::Menu;

#[derive(Menu)]
#[menu(title("Describe a person"))]
struct Person {
    lastname: String,
    firstname: String,
    age: u8,
}

fn main() {
    let Person { firstname, .. } = Person::from_menu().unwrap();
    println!("goodbye {}!", firstname);
}

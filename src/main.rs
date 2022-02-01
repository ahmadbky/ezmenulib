use ezmenu::Menu;

fn play(_: &u8) {
    println!("oui");
}

#[derive(Menu)]
struct Person {
    name: String,
    #[field(msg(45), then(play))]
    age: u8,
}

// used as testing for now
fn main() {
    let Person { name, age } = Person::from_menu();
    println!("name={}, age={}", name, age);
}

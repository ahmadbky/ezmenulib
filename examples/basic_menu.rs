use ezmenu::Menu;

mod my_mod {
    pub fn check_firstname(s: &String) {
        if s == "Ahmad" {
            println!("oh you got the same name");
        }
    }
}

fn check_lastname(s: &String) {
    if s == "Baalbaky" {
        println!("are you his brother or sister?");
    }
}

fn check_age(age: &u8) {
    match age {
        0 => println!("wow you're young"),
        1..=13 => println!("still a kid"),
        14..=19 => println!("a teenager"),
        _ => println!("you're an adulte now"),
    }
}

#[derive(Menu)]
struct Person {
    #[field(msg = "What is your last name", then(check_lastname))]
    lastname: String,
    #[field(msg = "What is your first name", then(my_mod::check_firstname))]
    firstname: String,
    #[field(msg("How old are you"), then(check_age))]
    age: u8,
}

fn main() {
    let Person { firstname, .. } = Person::from_menu();
    println!("goodbye {}!", firstname);
}

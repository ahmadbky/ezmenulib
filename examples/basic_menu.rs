use ezmenulib::{Menu, ValueField, ValueMenu};
use std::error::Error;

struct Person {
    lastname: String,
    firstname: String,
    age: u8,
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut menu = ValueMenu::from([ValueField::from("bonsoir"), ValueField::from("yes")]);
    let age: u8 = menu.next_output()?;
    println!("goodbye {}!", age);
    Ok(())
}

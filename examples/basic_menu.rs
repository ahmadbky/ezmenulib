use ezmenulib::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut menu = ValueMenu::from([
        Field::Value(ValueField::from("bonsoir").default("bonsoir")),
        Field::Value(ValueField::from("yes")),
    ]);
    let age: u8 = menu.next_output()?;
    println!("goodbye {}!", age);
    Ok(())
}

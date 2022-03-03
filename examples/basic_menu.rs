use ezmenulib::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut menu = ValueMenu::from([
        Field::Value(ValueField::from("name")),
        Field::Value(ValueField::from("age").default("18")),
    ]);
    let name: String = menu.next_output()?;
    let age: u8 = menu.next_output()?;
    println!("ok you are {} y/o, goodbye {}!", age, name);
    Ok(())
}

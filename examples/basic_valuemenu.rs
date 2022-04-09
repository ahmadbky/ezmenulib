use ezmenulib::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut menu = ValueMenu::from([
        ValueField::Value(Value::from("name").example("Ahmad")),
        ValueField::Value(Value::from("age").fmt(ValueFieldFormatting::default(false))),
    ]);
    let name: String = menu.next_value()?;
    let age: u8 = menu.next_value()?;
    println!("ok you are {} y/o, goodbye {}!", age, name);
    Ok(())
}

use ezmenulib::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut menu = Values::default();
    let name: String = menu.written(&Written::from("name").example("Ahmad"))?;
    let age: u8 = menu.written(&Written::from("age").format(&Format::show_default(false)))?;
    println!("ok you are {} y/o, goodbye {}!", age, name);
    Ok(())
}

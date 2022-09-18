use ezmenulib::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut menu = Values::default();

    let name: String = menu.try_next(Written::from("name").example("Ahmad").format(Format {
        line_brk: false,
        suffix: ": ",
        ..Default::default()
    }))?;

    let age: u8 = menu.try_next(Written::from("age"))?;

    Ok(println!("ok you are {} y/o, goodbye {}!", age, name))
}

use ezmenulib::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut menu = Values::default();
    let name: String =
        menu.written_or_default(&Written::from("name").example("Ahmad").format(&Format {
            line_brk: false,
            suffix: ": ",
            ..Default::default()
        }));
    let age: u8 =
        menu.written_or_default(&Written::from("age").format(&Format::show_default(false)));
    println!("ok you are {} y/o, goodbye {}!", age, name);
    Ok(())
}

use ezmenulib::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let amount: u8 = SelectMenu::from([
        SelectField::from("0"),
        SelectField::from("1"),
        SelectField::from("2"),
    ])
    .next_output()?;

    println!("you selected {}", amount);
    Ok(())
}

use ezmenulib::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let amount: i32 = SelectMenu::from([
        SelectField::new("0", 0),
        SelectField::new("1", 1),
        SelectField::new("2", 2),
    ])
    .next_output()?;

    println!("you selected {}", amount);
    Ok(())
}

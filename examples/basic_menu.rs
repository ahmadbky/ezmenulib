use ezmenulib::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    Menu::from([
        Field::new("Hello", Kind::Unit(|_| Ok(()))),
        Field::new("Bonsoir", Kind::Unit(|_| Ok(()))),
        Field::new(
            "Settings",
            Kind::SubMenu(vec![Field::new("Add a map", Kind::Unit(|_| Ok(())))]),
        ),
        Field::new("Quit", Kind::Quit),
    ])
    .go_back(true)
    .repeat(true)
    .run()?;
    Ok(())
}

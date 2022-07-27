use std::error::Error;

use ezmenulib::{prelude::*, Select};

#[derive(Select, Debug)]
#[select("oui", fmt = Format::show_default(false))]
enum Amount {
    #[select(("One", true), default("Two", false))]
    OneTwo(bool),
    #[select(("Three", _is_three: true), ("Four", _is_three: false))]
    ThreeFour {
        _is_three: bool,
    },
    More,
}

fn main() -> Result<(), Box<dyn Error>> {
    let amount = Amount::select().prompt(MenuHandle::default())?;
    println!("{amount:?}");
    Ok(())
}

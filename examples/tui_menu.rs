use std::io::Stdout;

use ezmenulib::{
    prelude::*,
    tui::{crossterm::{Crossterm, restore_terminal}, *},
};
use tui::{backend::CrosstermBackend, Terminal};

fn first(term: &mut Terminal<CrosstermBackend<Stdout>>) -> MenuResult {
    restore_terminal(term)?;
    println!("you picked first");
    Ok(())
}

fn main() -> MenuResult {
    let mut menu = TuiMenu::<Crossterm>::try_from(&[
        ("first", TuiKind::Map(first)),
        ("second", TuiKind::Quit),
        (
            "third",
            TuiKind::Parent(&[
                ("fourth", TuiKind::Quit),
                ("fifth", TuiKind::Quit),
                ("go back", TuiKind::Back(1)),
            ]),
        ),
    ])?;

    menu.run()?;
    menu.close()
}

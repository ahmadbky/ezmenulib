use std::io::Stdout;

use ::crossterm::terminal::enable_raw_mode;
use ezmenulib::{
    prelude::*,
    tui::{
        crossterm::{read, restore_terminal, setup_terminal, Crossterm},
        event::Event,
        *,
    },
};
use tui::{backend::CrosstermBackend, Terminal};

fn first(term: &mut Terminal<CrosstermBackend<Stdout>>) -> MenuResult {
    restore_terminal(term)?;
    println!("you picked first (press any key to return)");
    // Pauses until the user presses a key
    enable_raw_mode()?;
    while !matches!(read()?, Event::Key(_)) {}
    setup_terminal(term)?;
    return Ok(());
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

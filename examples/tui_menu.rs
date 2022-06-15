use std::io::Stdout;

use ::crossterm::terminal::{disable_raw_mode, enable_raw_mode};
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
    println!("You picked first.");
    wait()?;
    setup_terminal(term)?;
    Ok(())
}

/// Pauses until the user presses a key
fn wait() -> MenuResult {
    println!("Press any key to continue.");
    enable_raw_mode()?;
    while !matches!(read()?, Event::Key(_)) {}
    disable_raw_mode()?;
    Ok(())
}

fn main() -> MenuResult {
    let mut menu = TuiMenu::<Crossterm>::try_from(&[
        ("first", TuiKind::Map(&first)),
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

    println!("This is a basic menu built with ezmenulib using tui-rs.");
    wait()?;

    menu.run()?;
    menu.close()
}

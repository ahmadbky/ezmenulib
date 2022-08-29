use ::tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use crossterm::{
    event::{read, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ezmenulib::{prelude::*, tui::*};
use std::io::{self, stdout, Write};

fn restore_terminal<B: Backend + Write>(term: &mut Terminal<B>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()
}

fn setup_terminal<B: Backend + Write>(term: &mut Terminal<B>) -> io::Result<()> {
    enable_raw_mode()?;
    execute!(term.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
    term.show_cursor()
}

fn display<B: Backend + Write>(term: &mut Terminal<B>, s: &str) -> io::Result<()> {
    restore_terminal(term)?;
    writeln!(term.backend_mut(), "{s}")?;
    wait()?;
    setup_terminal(term)
}

/// Pauses until the user presses a key
fn wait() -> io::Result<()> {
    println!("Press any key to continue.");
    enable_raw_mode()?;
    while !matches!(read()?, Event::Key(_)) {}
    disable_raw_mode()
}

fn main() -> MenuResult {
    let name = &[
        ("Firstname", tui_mapped!(display, "Editing firstname.")),
        ("Lastname", tui_mapped!(display, "Editing lastname.")),
        ("Main menu", back(2)),
    ];

    let settings = &[
        ("Name", parent(name)),
        ("Main menu", back(1)),
        ("Quit", quit()),
    ];

    let fields = &[
        ("Play", tui_mapped!(display, "Now playing.")),
        ("Settings", parent(settings)),
        ("Quit", quit()),
    ];

    let mut term = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut menu = TuiMenu::new(fields);

    writeln!(
        term.backend_mut(),
        "This is a basic menu built with ezmenulib using tui-rs."
    )?;
    wait()?;

    setup_terminal(&mut term)?;

    while {
        term.draw(|f| f.render_widget(&menu, f.size()))?;
        !menu.handle_ct_event_with(read()?, &mut term)?
    } {}

    restore_terminal(&mut term)?;

    Ok(())
}

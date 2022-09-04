use ::tui::{
    backend::{Backend, CrosstermBackend},
    Terminal,
};
use crossterm::{
    event::{read, DisableMouseCapture, EnableMouseCapture, Event},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ezmenulib::{prelude::*, tui::Menu};
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

#[derive(Menu)]
#[menu(tui)]
enum Name {
    #[menu(mapped(display, "Editing firstname"))]
    Firstname,
    #[menu(mapped(display, "Editing lastname"))]
    Lastname,
    #[menu(back(2))]
    MainMenu,
}

#[derive(Menu)]
#[menu(tui)]
enum Settings {
    #[menu(parent)]
    Name,
    #[menu(back)]
    MainMenu,
    Quit,
}

#[derive(Menu)]
#[menu(tui)]
enum MainMenu {
    #[menu(mapped(display, "Now playing."))]
    Play,
    #[menu(parent)]
    Settings,
    Quit,
}

fn main() -> MenuResult {
    let mut term = Terminal::new(CrosstermBackend::new(stdout()))?;
    let mut menu = MainMenu::tui_menu();

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

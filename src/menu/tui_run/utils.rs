use std::io::{self, stdout};
use tui::Terminal;

use super::TuiBackend;

#[cfg(feature = "crossterm")]
use tui::backend::CrosstermBackend;

#[cfg(feature = "termion")]
use tui::backend::TermionBackend;

#[cfg(feature = "crossterm")]
#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "crossterm")))]
pub fn setup_terminal() -> io::Result<Terminal<TuiBackend>> {
    use crossterm::{
        event::EnableMouseCapture,
        execute,
        terminal::{enable_raw_mode, EnterAlternateScreen},
    };

    enable_raw_mode()?;
    let mut stdout = stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    Terminal::new(backend)
}

#[cfg(feature = "crossterm")]
#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "crossterm")))]
pub fn restore_terminal(term: &mut Terminal<TuiBackend>) -> io::Result<()> {
    use crossterm::{
        event::DisableMouseCapture,
        execute,
        terminal::{disable_raw_mode, LeaveAlternateScreen},
    };

    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()
}

#[cfg(feature = "termion")]
#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "termion")))]
pub fn setup_terminal() -> io::Result<Terminal<TuiBackend>> {
    use termion::{input::MouseTerminal, raw::IntoRawMode, screen::ToAlternateScreen};

    let mut terminal = stdout().into_raw_mode()?;
    terminal.activate_raw_mode()?;
    write!(terminal, "{}", ToAlternateScreen)?;
    let m_capture = MouseTerminal::from(terminal);
    let backend = TermionBackend::new(m_capture);
    Terminal::new(backend)
}

#[cfg(feature = "termion")]
#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "termion")))]
pub fn restore_terminal(term: &mut Terminal<TuiBackend>) -> io::Result<()> {
    write!(term.backend_mut(), "{}", termion::screen::ToMainScreen)?;
    term.show_cursor()
}

//! Contains the util functions and types to manipulate the terminal
//! using the [`crossterm`](https://docs.rs/crossterm/0.23.2) backend.

use crate::{menu::Out, tui::event::*};
use crossterm::{
    event::{
        read as ct_read, DisableMouseCapture, EnableMouseCapture, Event as CTEvent, KeyCode,
        KeyEvent as CTKeyEvent, KeyModifiers, MouseButton as CTMouseButton,
        MouseEvent as CTMouseEvent, MouseEventKind,
    },
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io;
use tui::{backend::CrosstermBackend, Terminal};

/// Used to modelize the default backend type used with `crossterm` backend.
pub type Crossterm<W = Out> = CrosstermBackend<W>;

impl From<CTEvent> for Event {
    fn from(event: CTEvent) -> Self {
        use Event::*;
        use KeyEvent::*;

        match event {
            CTEvent::Key(CTKeyEvent { code, modifiers }) => match code {
                KeyCode::Backspace => Key(Backspace),
                KeyCode::Enter => Key(Enter),
                KeyCode::Left => Key(Left),
                KeyCode::Right => Key(Right),
                KeyCode::Up => Key(Up),
                KeyCode::Down => Key(Down),
                KeyCode::Home => Key(Home),
                KeyCode::End => Key(End),
                KeyCode::PageUp => Key(PageUp),
                KeyCode::PageDown => Key(PageDown),
                KeyCode::Tab => Key(Tab),
                KeyCode::BackTab => Key(BackTab),
                KeyCode::Delete => Key(Delete),
                KeyCode::Insert => Key(Insert),
                KeyCode::F(x) => Key(F(x)),
                KeyCode::Char(c) => match modifiers {
                    KeyModifiers::ALT => Key(Alt(c)),
                    KeyModifiers::CONTROL => Key(Ctrl(c)),
                    _ => Key(Char(c)),
                },
                KeyCode::Null => Key(Null),
                KeyCode::Esc => Key(Esc),
            },
            CTEvent::Mouse(CTMouseEvent {
                kind, column, row, ..
            }) => match kind {
                MouseEventKind::Down(b) => Mouse(MouseEvent::Down(
                    match b {
                        CTMouseButton::Left => MouseButton::Left,
                        CTMouseButton::Right => MouseButton::Right,
                        CTMouseButton::Middle => MouseButton::Middle,
                    },
                    column,
                    row,
                )),
                MouseEventKind::Up(_) => Mouse(MouseEvent::Up(column, row)),
                MouseEventKind::Drag(_) | MouseEventKind::Moved => Key(Null),
                MouseEventKind::ScrollDown => Mouse(MouseEvent::ScrollDown),
                MouseEventKind::ScrollUp => Mouse(MouseEvent::ScrollUp),
            },
            CTEvent::Resize(x, y) => Self::Resize(x, y),
        }
    }
}

/// Returns an [`Event`] using the `crossterm` backend.
pub fn read() -> io::Result<Event> {
    ct_read().map(Event::from)
}

/// Returns a new tui terminal using the crossterm backend type.
pub fn new_terminal() -> io::Result<Terminal<Crossterm>> {
    Ok(Terminal::new(Crossterm::new(io::stdout()))?)
}

/// Setups the terminal using the crossterm backend type.
///
/// The setup consist in entering to alternate mode, meaning clearing the screen and
/// hiding the cursor ; enabling the mouse events capture,
/// and enabling the [raw mode](https://docs.rs/crossterm/latest/crossterm/terminal/index.html#raw-mode).
pub fn setup_terminal(term: &mut Terminal<Crossterm>) -> io::Result<()> {
    enable_raw_mode()?;
    execute!(term.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
    term.hide_cursor()
}

/// Restores the terminal using the crossterm backend type.
///
/// The restoration of the terminal consist in leaving the alternate mode, meaning
/// returning to the previous screen before setuping the terminal and showing the cursor ;
/// disabling the mouse events capture and
/// disabling the [raw mode](https://docs.rs/crossterm/latest/crossterm/terminal/index.html#raw-mode)
pub fn restore_terminal(term: &mut Terminal<Crossterm>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()
}

//! Contains the util functions and types to manipulate the terminal
//! using the [`termion`](https://docs.rs/termion/1.5.0) backend.

use crate::{menu::Out, tui::event::*};
use std::io::{self, stdin, stdout, Error, Write};
use termion::{
    event::{Event as TEvent, Key as TKey, MouseButton as TMouseButton, MouseEvent as TMouseEvent},
    input::{EnterMouseSequence, ExitMouseSequence, TermRead},
    raw::{IntoRawMode, RawTerminal},
    screen::{ToAlternateScreen, ToMainScreen},
};
use tui::{backend::TermionBackend, Terminal};

/// Used to modelize the default backend type used with termion backend.
pub type Termion<W = Out> = TermionBackend<RawTerminal<W>>;

impl From<TEvent> for Event {
    fn from(event: TEvent) -> Self {
        use Event::*;
        use KeyEvent::*;

        match event {
            TEvent::Key(k) => match k {
                TKey::Backspace => Key(Backspace),
                TKey::Left => Key(Left),
                TKey::Right => Key(Right),
                TKey::Up => Key(Up),
                TKey::Down => Key(Down),
                TKey::Home => Key(Home),
                TKey::End => Key(End),
                TKey::PageUp => Key(PageUp),
                TKey::PageDown => Key(PageDown),
                TKey::BackTab => Key(BackTab),
                TKey::Delete => Key(Delete),
                TKey::Insert => Key(Insert),
                TKey::F(x) => Key(F(x)),
                TKey::Char('\n') => Key(Enter),
                TKey::Char(x) => Key(Char(x)),
                TKey::Alt(x) => Key(Alt(x)),
                TKey::Ctrl(x) => Key(Ctrl(x)),
                TKey::Null => Key(Null),
                TKey::Esc => Key(Esc),
                _ => Key(Null),
            },
            TEvent::Mouse(m) => match m {
                TMouseEvent::Press(b, x, y) => match b {
                    TMouseButton::Left => Mouse(MouseEvent::Down(MouseButton::Left, x, y)),
                    TMouseButton::Right => Mouse(MouseEvent::Down(MouseButton::Right, x, y)),
                    TMouseButton::Middle => Mouse(MouseEvent::Down(MouseButton::Middle, x, y)),
                    TMouseButton::WheelUp => Mouse(MouseEvent::ScrollUp),
                    TMouseButton::WheelDown => Mouse(MouseEvent::ScrollDown),
                },
                TMouseEvent::Release(x, y) => Mouse(MouseEvent::Up(x, y)),
                _ => Key(Null),
            },
            _ => Key(Null),
        }
    }
}

/// Returns an [`Event`] using the termion backend.
pub fn read() -> io::Result<Event> {
    for event in stdin().events() {
        return event.map(Event::from);
    }
    Err(Error::last_os_error())
}

/// Returns a new tui terminal using the termion backend type.
pub fn new_terminal() -> io::Result<Terminal<Termion>> {
    let buf = stdout().into_raw_mode()?;
    buf.suspend_raw_mode()?;
    Ok(Terminal::new(TermionBackend::new(buf))?)
}

/// Setups the terminal using the termion backend type.
///
/// The setup consist in entering to alternate mode, meaning clearing the screen and
/// hiding the cursor ; enabling the mouse events capture,
/// and enabling the [raw mode](https://docs.rs/termion/latest/termion/raw/index.html).
pub fn setup_terminal<W: Write>(term: &mut Terminal<Termion<W>>) -> io::Result<()> {
    write!(
        term.backend_mut(),
        "{}{}",
        ToAlternateScreen,
        EnterMouseSequence,
    )?;
    term.backend().activate_raw_mode()
}

/// Restores the terminal using the termion backend type.
///
/// The restoration of the terminal consist in leaving the alternate mode, meaning
/// returning to the previous screen before setuping the terminal and showing the cursor ;
/// disabling the mouse events capture and
/// disabling the [raw mode](https://docs.rs/termion/latest/termion/raw/index.html)
pub fn restore_terminal<W: Write>(term: &mut Terminal<Termion<W>>) -> io::Result<()> {
    term.backend().suspend_raw_mode()?;
    write!(term.backend_mut(), "{}{}", ExitMouseSequence, ToMainScreen)?;
    term.show_cursor()
}

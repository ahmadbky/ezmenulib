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
use std::io::{self};
use tui::{backend::CrosstermBackend, Terminal};

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

pub(crate) fn read() -> io::Result<Event> {
    ct_read().map(Event::from)
}

pub fn setup_terminal(term: &mut Terminal<Crossterm>) -> io::Result<()> {
    enable_raw_mode()?;
    execute!(term.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
    term.hide_cursor()
}

pub fn restore_terminal(term: &mut Terminal<Crossterm>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()
}

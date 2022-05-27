use std::io::{self, Read};

#[cfg(feature = "crossterm")]
use crossterm::event::{
    read as ct_read, Event as CTEvent, KeyCode, KeyEvent as CTKeyEvent, KeyModifiers,
    MouseButton as CTMouseButton, MouseEvent as CTMouseEvent, MouseEventKind,
};

#[cfg(feature = "termion")]
use termion::{
    event::{Event as TEvent, Key as TKey, MouseButton as TMouseButton, MouseEvent as TMouseEvent},
    input::TermRead,
};

pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

#[cfg(feature = "crossterm")]
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
                KeyCode::Char('\r') => Key(Enter),
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

#[cfg(feature = "termion")]
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

pub enum KeyEvent {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Insert,
    F(u8),
    Char(char),
    Null,
    Esc,
    Alt(char),
    Ctrl(char),
}

pub enum MouseButton {
    Left,
    Right,
    Middle,
}

pub enum MouseEvent {
    Down(MouseButton, u16, u16),
    Up(u16, u16),
    ScrollUp,
    ScrollDown,
}

#[cfg(feature = "termion")]
pub fn read(input: impl Read) -> io::Result<Event> {
    match input.events().next() {
        Some(event) => event.map(Event::from),
        None => Err(Error::last_os_error()),
    }
}

#[cfg(feature = "crossterm")]
pub fn read(_input: impl Read) -> io::Result<Event> {
    ct_read().map(Event::from)
}

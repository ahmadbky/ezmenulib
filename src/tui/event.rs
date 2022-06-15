//! Module defining the event types used by the library.
//!
//! It is a merged version between `crossterm` and `termion` event types.

/// The event type representing the merge between `crossterm` and `termion` event type.
///
/// This type is retrieved depending on the backend: [`crossterm::read`](crate::tui::crossterm::read)
/// or [`termion::read`](crate::tui::termion::read).
#[derive(Debug, Clone, Copy)]
pub enum Event {
    /// A key Event.
    Key(KeyEvent),
    /// A mouse event.
    Mouse(MouseEvent),
    /// A resize of the terminal event, containing the new size of the terminal.
    Resize(u16, u16),
}

/// The key event type representing the merge between `crossterm` and `termion` key event type.
#[derive(Debug, Clone, Copy)]
pub enum KeyEvent {
    /// Backspace.
    Backspace,
    /// Enter.
    Enter,
    /// Left arrow.
    Left,
    /// Right arrow.
    Right,
    /// Up arrow.
    Up,
    /// Down arrow.
    Down,
    /// Home.
    Home,
    /// End.
    End,
    /// Page Up.
    PageUp,
    /// Page Down.
    PageDown,
    /// Tab.
    Tab,
    /// The back tab (generally corresponding to Shift+Tab).
    BackTab,
    /// Delete.
    Delete,
    /// Insert.
    Insert,
    /// An F key with the digit attached to it.
    F(u8),
    /// A char key.
    Char(char),
    /// A null key event.
    Null,
    /// Escape.
    Esc,
    /// A char key pressed with Alt key.
    Alt(char),
    /// A char key pressed with Control key.
    Ctrl(char),
}

/// The mouse buttons.
#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    /// Left click.
    Left,
    /// Right click.
    Right,
    /// Middle click (generally corresponding to the wheel click).
    Middle,
}

/// The mouse event type representing the merge between `crossterm` and `termion` ket event type.
#[derive(Debug, Clone, Copy)]
pub enum MouseEvent {
    /// A mouse button has just been pressed down.
    Down(MouseButton, u16, u16),
    /// The previous map button that has been pressed down has just been released up.
    Up(u16, u16),
    /// The scroll up event.
    ScrollUp,
    /// The scroll down event.
    ScrollDown,
}

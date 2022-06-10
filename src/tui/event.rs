#[derive(Debug, Clone, Copy)]
pub enum Event {
    Key(KeyEvent),
    Mouse(MouseEvent),
    Resize(u16, u16),
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Debug, Clone, Copy)]
pub enum MouseEvent {
    Down(MouseButton, u16, u16),
    Up(u16, u16),
    ScrollUp,
    ScrollDown,
}

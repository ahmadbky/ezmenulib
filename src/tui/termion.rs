use crate::{menu::Out, tui::event::*};
use std::{
    fmt,
    io::{self, stdin, stdout, Error, Write},
    ops::{Deref, DerefMut},
};
use termion::{
    clear::All,
    cursor::{DetectCursorPos, Goto, Hide, Show},
    event::{Event as TEvent, Key as TKey, MouseButton as TMouseButton, MouseEvent as TMouseEvent},
    input::{TermRead},
    raw::{IntoRawMode, RawTerminal},
    screen::{ToAlternateScreen, ToMainScreen},
    terminal_size,
};
use tui::{
    backend::{Backend},
    buffer::Cell,
    layout::Rect,
    style::{Color, Modifier},
    Terminal,
};

/// A sequence of escape codes to enable terminal mouse support.
const ENTER_MOUSE_SEQUENCE: &'static str = "\x1B[?1000h\x1b[?1002h\x1b[?1015h\x1b[?1006h";

/// A sequence of escape codes to disable terminal mouse support.
const EXIT_MOUSE_SEQUENCE: &'static str = "\x1B[?1006l\x1b[?1015l\x1b[?1002l\x1b[?1000l";

pub struct Termion<W: Write = Out> {
    buf: RawTerminal<W>,
}

impl<W: Write> fmt::Debug for Termion<W> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Termion").finish()
    }
}

impl Termion {
    #[inline]
    pub fn new() -> io::Result<Self> {
        Self::from_writer(stdout())
    }
}

impl<W: Write> Termion<W> {
    pub fn from_writer(buf: W) -> io::Result<Self> {
        let buf = buf.into_raw_mode()?;
        buf.suspend_raw_mode()?;
        Ok(Self { buf })
    }
}

impl<W: Write> Drop for Termion<W> {
    fn drop(&mut self) {
        close_term(self).expect("unable to close terminal");
    }
}

impl<W: Write> Deref for Termion<W> {
    type Target = RawTerminal<W>;

    fn deref(&self) -> &Self::Target {
        &self.buf
    }
}

impl<W: Write> DerefMut for Termion<W> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.buf
    }
}

impl<W: Write> Write for Termion<W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Write::flush(&mut self.buf)
    }
}

impl<W: Write> Backend for Termion<W> {
    // This is the code snippet taken from the `tui::backend::termion` module.
    fn draw<'a, I>(&mut self, content: I) -> io::Result<()>
    where
        I: Iterator<Item = (u16, u16, &'a Cell)>,
    {
        use std::fmt::Write;

        let mut string = String::with_capacity(content.size_hint().0 * 3);
        let mut fg = Color::Reset;
        let mut bg = Color::Reset;
        let mut modifier = Modifier::empty();
        let mut last_pos: Option<(u16, u16)> = None;
        for (x, y, cell) in content {
            // Move the cursor if the previous location was not (x - 1, y)
            if !matches!(last_pos, Some(p) if x == p.0 + 1 && y == p.1) {
                write!(string, "{}", termion::cursor::Goto(x + 1, y + 1)).unwrap();
            }
            last_pos = Some((x, y));
            if cell.modifier != modifier {
                write!(
                    string,
                    "{}",
                    ModifierDiff {
                        from: modifier,
                        to: cell.modifier
                    }
                )
                .unwrap();
                modifier = cell.modifier;
            }
            if cell.fg != fg {
                write!(string, "{}", Fg(cell.fg)).unwrap();
                fg = cell.fg;
            }
            if cell.bg != bg {
                write!(string, "{}", Bg(cell.bg)).unwrap();
                bg = cell.bg;
            }
            string.push_str(&cell.symbol);
        }
        write!(
            self.buf,
            "{}{}{}{}",
            string,
            Fg(Color::Reset),
            Bg(Color::Reset),
            termion::style::Reset,
        )
    }

    fn hide_cursor(&mut self) -> io::Result<()> {
        write!(self.buf, "{}", Hide)?;
        self.buf.flush()
    }

    fn show_cursor(&mut self) -> io::Result<()> {
        write!(self.buf, "{}", Show)?;
        self.buf.flush()
    }

    fn get_cursor(&mut self) -> io::Result<(u16, u16)> {
        DetectCursorPos::cursor_pos(&mut self.buf).map(|(x, y)| (x - 1, y - 1))
    }

    fn set_cursor(&mut self, x: u16, y: u16) -> io::Result<()> {
        write!(self.buf, "{}", Goto(x + 1, y + 1))?;
        self.buf.flush()
    }

    fn clear(&mut self) -> io::Result<()> {
        write!(self.buf, "{}", All)?;
        write!(self.buf, "{}", Goto(1, 1))?;
        self.buf.flush()
    }

    fn size(&self) -> io::Result<Rect> {
        let (x, y) = terminal_size()?;
        Ok(Rect::new(0, 0, x, y))
    }

    fn flush(&mut self) -> io::Result<()> {
        self.buf.flush()
    }
}

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

pub fn read() -> io::Result<Event> {
    for event in stdin().events() {
        return event.map(Event::from);
    }
    Err(Error::last_os_error())
}

pub fn new_terminal() -> io::Result<Terminal<Termion>> {
    let mut term = Terminal::new(Termion::new()?)?;
    setup_terminal(&mut term)?;
    Ok(term)
}

pub fn setup_terminal<W: Write>(term: &mut Terminal<Termion<W>>) -> io::Result<()> {
    write!(
        term.backend_mut(),
        "{}{}",
        ToAlternateScreen,
        ENTER_MOUSE_SEQUENCE
    )?;
    term.backend().activate_raw_mode()
}

fn close_term<W: Write>(term: &mut Termion<W>) -> io::Result<()> {
    term.suspend_raw_mode()?;
    write!(term, "{}{}", EXIT_MOUSE_SEQUENCE, ToMainScreen)?;
    term.show_cursor()
}

pub fn restore_terminal<W: Write>(term: &mut Terminal<Termion<W>>) -> io::Result<()> {
    close_term(term.backend_mut())
}

struct Fg(Color);

struct Bg(Color);

struct ModifierDiff {
    from: Modifier,
    to: Modifier,
}

impl fmt::Display for Fg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use termion::color::Color as TermionColor;
        match self.0 {
            Color::Reset => termion::color::Reset.write_fg(f),
            Color::Black => termion::color::Black.write_fg(f),
            Color::Red => termion::color::Red.write_fg(f),
            Color::Green => termion::color::Green.write_fg(f),
            Color::Yellow => termion::color::Yellow.write_fg(f),
            Color::Blue => termion::color::Blue.write_fg(f),
            Color::Magenta => termion::color::Magenta.write_fg(f),
            Color::Cyan => termion::color::Cyan.write_fg(f),
            Color::Gray => termion::color::White.write_fg(f),
            Color::DarkGray => termion::color::LightBlack.write_fg(f),
            Color::LightRed => termion::color::LightRed.write_fg(f),
            Color::LightGreen => termion::color::LightGreen.write_fg(f),
            Color::LightBlue => termion::color::LightBlue.write_fg(f),
            Color::LightYellow => termion::color::LightYellow.write_fg(f),
            Color::LightMagenta => termion::color::LightMagenta.write_fg(f),
            Color::LightCyan => termion::color::LightCyan.write_fg(f),
            Color::White => termion::color::LightWhite.write_fg(f),
            Color::Indexed(i) => termion::color::AnsiValue(i).write_fg(f),
            Color::Rgb(r, g, b) => termion::color::Rgb(r, g, b).write_fg(f),
        }
    }
}
impl fmt::Display for Bg {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use termion::color::Color as TermionColor;
        match self.0 {
            Color::Reset => termion::color::Reset.write_bg(f),
            Color::Black => termion::color::Black.write_bg(f),
            Color::Red => termion::color::Red.write_bg(f),
            Color::Green => termion::color::Green.write_bg(f),
            Color::Yellow => termion::color::Yellow.write_bg(f),
            Color::Blue => termion::color::Blue.write_bg(f),
            Color::Magenta => termion::color::Magenta.write_bg(f),
            Color::Cyan => termion::color::Cyan.write_bg(f),
            Color::Gray => termion::color::White.write_bg(f),
            Color::DarkGray => termion::color::LightBlack.write_bg(f),
            Color::LightRed => termion::color::LightRed.write_bg(f),
            Color::LightGreen => termion::color::LightGreen.write_bg(f),
            Color::LightBlue => termion::color::LightBlue.write_bg(f),
            Color::LightYellow => termion::color::LightYellow.write_bg(f),
            Color::LightMagenta => termion::color::LightMagenta.write_bg(f),
            Color::LightCyan => termion::color::LightCyan.write_bg(f),
            Color::White => termion::color::LightWhite.write_bg(f),
            Color::Indexed(i) => termion::color::AnsiValue(i).write_bg(f),
            Color::Rgb(r, g, b) => termion::color::Rgb(r, g, b).write_bg(f),
        }
    }
}

impl fmt::Display for ModifierDiff {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let remove = self.from - self.to;
        if remove.contains(Modifier::REVERSED) {
            write!(f, "{}", termion::style::NoInvert)?;
        }
        if remove.contains(Modifier::BOLD) {
            // XXX: the termion NoBold flag actually enables double-underline on ECMA-48 compliant
            // terminals, and NoFaint additionally disables bold... so we use this trick to get
            // the right semantics.
            write!(f, "{}", termion::style::NoFaint)?;

            if self.to.contains(Modifier::DIM) {
                write!(f, "{}", termion::style::Faint)?;
            }
        }
        if remove.contains(Modifier::ITALIC) {
            write!(f, "{}", termion::style::NoItalic)?;
        }
        if remove.contains(Modifier::UNDERLINED) {
            write!(f, "{}", termion::style::NoUnderline)?;
        }
        if remove.contains(Modifier::DIM) {
            write!(f, "{}", termion::style::NoFaint)?;

            // XXX: the NoFaint flag additionally disables bold as well, so we need to re-enable it
            // here if we want it.
            if self.to.contains(Modifier::BOLD) {
                write!(f, "{}", termion::style::Bold)?;
            }
        }
        if remove.contains(Modifier::CROSSED_OUT) {
            write!(f, "{}", termion::style::NoCrossedOut)?;
        }
        if remove.contains(Modifier::SLOW_BLINK) || remove.contains(Modifier::RAPID_BLINK) {
            write!(f, "{}", termion::style::NoBlink)?;
        }

        let add = self.to - self.from;
        if add.contains(Modifier::REVERSED) {
            write!(f, "{}", termion::style::Invert)?;
        }
        if add.contains(Modifier::BOLD) {
            write!(f, "{}", termion::style::Bold)?;
        }
        if add.contains(Modifier::ITALIC) {
            write!(f, "{}", termion::style::Italic)?;
        }
        if add.contains(Modifier::UNDERLINED) {
            write!(f, "{}", termion::style::Underline)?;
        }
        if add.contains(Modifier::DIM) {
            write!(f, "{}", termion::style::Faint)?;
        }
        if add.contains(Modifier::CROSSED_OUT) {
            write!(f, "{}", termion::style::CrossedOut)?;
        }
        if add.contains(Modifier::SLOW_BLINK) || add.contains(Modifier::RAPID_BLINK) {
            write!(f, "{}", termion::style::Blink)?;
        }

        Ok(())
    }
}

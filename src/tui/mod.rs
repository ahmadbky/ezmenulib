//! Module defining the types for the `tui` feature.
//!
//! This module is mainly used to generate menu using the [`tui`](https://docs.rs/tui/) crate.

pub mod event;

use std::{
    fmt, io,
    ops::{Deref, DerefMut},
};

use tui::{
    backend::Backend,
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget},
    Terminal,
};

use crate::{
    menu::{FromMutable, Mutable, UsesMutable},
    utils::Depth,
    MenuError, MenuResult,
};

use self::event::{Event, KeyEvent};

#[cfg(feature = "crossterm")]
#[cfg_attr(nightly, doc(cfg(feature = "crossterm")))]
pub mod crossterm;
#[cfg(feature = "crossterm")]
use self::crossterm::{
    new_terminal as ct_new_terminal, read as ct_read, restore_terminal as ct_restore_terminal,
    setup_terminal as ct_setup_terminal, Crossterm,
};

#[cfg(feature = "termion")]
#[cfg_attr(nightly, doc(cfg(feature = "termion")))]
pub mod termion;
#[cfg(feature = "termion")]
use self::termion::{
    new_terminal as t_new_terminal, read as t_read, restore_terminal as t_restore_terminal,
    setup_terminal as t_setup_terminal, Termion,
};

/// Represents the style of a field in the printed menu.
///
/// The first `Style` field corresponds to the style of the text,
/// and the `Color` field corresponds to the background color of the menu field.
pub type FieldStyle = (Style, Color);

type Reader = fn() -> io::Result<Event>;

/// Defines a tui menu, with a title, and the fields.
///
/// It handles the [terminal](Terminal) and the [style](Style) of the fields.
#[derive(Debug)]
pub struct TuiMenu<'a, B: Backend> {
    block: Block<'a>,
    s_style: FieldStyle,
    f_style: FieldStyle,
    fields: TuiFields<'a, B>,
    term: Mutable<'a, Terminal<B>>,
    once: bool,
}

impl<'a, B: Backend> UsesMutable<Terminal<B>> for TuiMenu<'a, B> {
    fn take_object(self) -> Terminal<B> {
        self.term.retrieve()
    }

    fn get_object(&self) -> &Terminal<B> {
        self.term.deref()
    }

    fn get_mut_object(&mut self) -> &mut Terminal<B> {
        self.term.deref_mut()
    }
}

impl<'a, B: Backend> FromMutable<'a, Terminal<B>, TuiFields<'a, B>> for TuiMenu<'a, B> {
    fn new(term: Mutable<'a, Terminal<B>>, fields: TuiFields<'a, B>) -> Self {
        Self {
            block: Block::default()
                .borders(Borders::all())
                .title_alignment(Alignment::Center),
            s_style: (
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::White),
                Color::Black,
            ),
            f_style: (Style::default().fg(Color::Black), Color::White),
            fields,
            term,
            once: false,
        }
    }
}

#[cfg(feature = "crossterm")]
#[cfg_attr(nightly, doc(cfg(feature = "crossterm")))]
impl<'a, const N: usize> TryFrom<&'a [TuiField<'a, Crossterm>; N]> for TuiMenu<'a, Crossterm> {
    type Error = <Self as TryFrom<TuiFields<'a, Crossterm>>>::Error;

    fn try_from(fields: &'a [TuiField<'a, Crossterm>; N]) -> Result<Self, Self::Error> {
        Self::try_from(fields.as_ref())
    }
}

#[cfg(feature = "crossterm")]
#[cfg_attr(nightly, doc(cfg(feature = "crossterm")))]
impl<'a> TryFrom<TuiFields<'a, Crossterm>> for TuiMenu<'a, Crossterm> {
    type Error = MenuError;

    fn try_from(fields: TuiFields<'a, Crossterm>) -> Result<Self, Self::Error> {
        Ok(Self::owned(ct_new_terminal()?, fields))
    }
}

#[cfg(feature = "termion")]
#[cfg_attr(nightly, doc(cfg(feature = "termion")))]
impl<'a, const N: usize> TryFrom<&'a [TuiField<'a, Termion>; N]> for TuiMenu<'a, Termion> {
    type Error = <Self as TryFrom<TuiFields<'a, Termion>>>::Error;

    fn try_from(fields: &'a [TuiField<'a, Termion>; N]) -> Result<Self, Self::Error> {
        Self::try_from(fields.as_ref())
    }
}

#[cfg(feature = "termion")]
#[cfg_attr(nightly, doc(cfg(feature = "termion")))]
impl<'a> TryFrom<TuiFields<'a, Termion>> for TuiMenu<'a, Termion> {
    type Error = MenuError;

    fn try_from(fields: TuiFields<'a, Termion>) -> Result<Self, Self::Error> {
        Ok(Self::owned(t_new_terminal()?, fields))
    }
}

impl<'a, B: Backend> TuiMenu<'a, B> {
    /// Defines the style of the selected field.
    ///
    /// The style corresponds to the *text* style. If you want to modify the background color
    /// of the selected field, use [`TuiMenu::selected_bg`] method.
    pub fn selected_style(mut self, style: Style) -> Self {
        self.s_style.0 = style;
        self
    }

    /// Defines the background color of the selected field.
    pub fn selected_bg(mut self, c: Color) -> Self {
        self.s_style.1 = c;
        self
    }

    /// Defines the style of the fields.
    ///
    /// The style corresponds to the *text* style. If you want to modify the background color
    /// of the fields, use [`TuiMenu::field_bg`] method.
    pub fn field_style(mut self, style: Style) -> Self {
        self.f_style.0 = style;
        self
    }

    /// Defines the background color of the fields.
    pub fn field_bg(mut self, c: Color) -> Self {
        self.f_style.1 = c;
        self
    }

    /// Defines the block drawn by the menu.
    ///
    /// This function can be used to set a title to the menu.
    pub fn with_block(mut self, b: Block<'a>) -> Self {
        self.block = b;
        self
    }

    /// Defines if the menu should run once or loop when calling a mapped function
    /// to a field.
    pub fn run_once(mut self, once: bool) -> Self {
        self.once = once;
        self
    }

    /// Runs the menu with the given area and the function to read the events from.
    fn run_with_read(&mut self, read_fn: Reader, area: Rect) -> MenuResult {
        run_with(
            &mut RunParams {
                term: self.term.deref_mut(),
                area,
                s_style: &self.s_style,
                f_style: &self.f_style,
                read_fn,
                once: self.once,
            },
            &self.block,
            self.fields,
        )
        .map(|_| ())
    }
}

#[cfg(feature = "crossterm")]
#[cfg_attr(nightly, doc(cfg(feature = "crossterm")))]
impl<'a> TuiMenu<'a, Crossterm> {
    /// Runs the menu using the crossterm backend, using the terminal size.
    pub fn run(&mut self) -> MenuResult {
        self.run_with(self.term.size()?)
    }

    /// Runs the menu using the crossterm backend, using the given `area`.
    pub fn run_with(&mut self, area: Rect) -> MenuResult {
        ct_setup_terminal(self.term.deref_mut())?;
        self.run_with_read(ct_read, area)
    }

    /// Closes the menu using the crossterm backend
    /// by [restoring the terminal](ct_restore_terminal).
    pub fn close(&mut self) -> MenuResult {
        ct_restore_terminal(self.term.deref_mut()).map_err(MenuError::from)
    }
}

#[cfg(feature = "termion")]
#[cfg_attr(nightly, doc(cfg(feature = "termion")))]
impl<'a> TuiMenu<'a, Termion> {
    /// Runs the menu using the termion backend, using the terminal size.
    pub fn run(&mut self) -> MenuResult {
        self.run_with(self.term.size()?)
    }

    /// Runs the menu using the termion backend, using the given `area`.
    pub fn run_with(&mut self, area: Rect) -> MenuResult {
        t_setup_terminal(self.term.deref_mut())?;
        self.run_with_read(t_read, area)
    }

    /// Closes the menu using the termion backend
    /// by [restoring the terminal](t_restore_terminal).
    pub fn close(&mut self) -> MenuResult {
        t_restore_terminal(&mut self.term).map_err(MenuError::from)
    }
}

/// Contains the information displayed to the terminal at a specific moment.
struct MenuWidget<'a> {
    fields: Vec<&'a str>,
    block: Block<'a>,
    s_style: &'a FieldStyle,
    f_style: &'a FieldStyle,
    selected: usize,
}

impl<'a> Widget for MenuWidget<'a> {
    fn render(self, area @ Rect { x, y, width, .. }: Rect, buf: &mut Buffer) {
        self.block.render(area, buf);

        for (i, msg) in self.fields.into_iter().enumerate() {
            let (fg_style, bg_style) = if i == self.selected {
                (self.s_style.0, Style::default().bg(self.s_style.1))
            } else {
                (self.f_style.0, Style::default().bg(self.f_style.1))
            };

            buf.set_stringn(x + 2, y + 1 + i as u16, msg, width as usize - 4, fg_style);
            buf.set_style(Rect::new(x + 1, y + 1 + i as u16, width - 2, 1), bg_style);
        }
    }
}

/// Represents the parameters of the tui menu currently running, which are the same
/// at any state of the menu (any depth of the `run_with` recursive function).
struct RunParams<'a, B: Backend> {
    term: &'a mut Terminal<B>,
    area: Rect,
    s_style: &'a FieldStyle,
    f_style: &'a FieldStyle,
    read_fn: Reader,
    once: bool,
}

fn run_with<B: Backend>(
    params: &mut RunParams<B>,
    block: &Block,
    fields: TuiFields<B>,
) -> MenuResult<Option<usize>> {
    let mut selected = 0;

    loop {
        // The messages displayed
        let msg_list: Vec<&str> = fields.iter().map(|field| field.0).collect();

        params.term.draw(|f| {
            f.render_widget(
                MenuWidget {
                    fields: msg_list,
                    block: block.clone(),
                    s_style: params.s_style,
                    f_style: params.f_style,
                    selected,
                },
                params.area,
            );
        })?;

        if let Event::Key(k) = (params.read_fn)()? {
            match k {
                KeyEvent::Char('q') | KeyEvent::Ctrl('c') => return Ok(None),
                KeyEvent::Up | KeyEvent::Left if selected == 0 => selected = fields.len() - 1,
                KeyEvent::Up | KeyEvent::Left => selected -= 1,
                KeyEvent::Down | KeyEvent::Right if selected == fields.len() - 1 => selected = 0,
                KeyEvent::Down | KeyEvent::Right => selected += 1,
                KeyEvent::Enter => {
                    let (msg, kind) = &fields[selected];
                    match kind {
                        TuiKind::Map(b) => {
                            b(params.term)?;
                            if params.once {
                                return Ok(None);
                            }
                            params.term.clear()?;
                        }
                        TuiKind::Parent(fields) => {
                            match run_with(params, &block.clone().title(*msg), fields)? {
                                None => return Ok(None),
                                Some(0) => (),
                                Some(i) => return Ok(Some(i - 1)),
                            }
                        }
                        TuiKind::Back(0) => (),
                        TuiKind::Back(i) => return Ok(Some(i - 1)),
                        TuiKind::Quit => return Ok(None),
                    }
                }
                _ => (),
            }
        }
    }
}

pub type TuiField<'a, B> = (&'a str, TuiKind<'a, B>);

pub type TuiFields<'a, B> = &'a [TuiField<'a, B>];

pub type TuiBinding<B> = dyn Fn(&mut Terminal<B>) -> MenuResult;

pub enum TuiKind<'a, B: Backend> {
    Map(&'a TuiBinding<B>),
    Parent(TuiFields<'a, B>),
    Back(usize),
    Quit,
}

impl<'a, B: Backend> fmt::Debug for TuiKind<'a, B> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("Field::")?;
        match self {
            Self::Map(_) => f.debug_tuple("Map").finish(),
            Self::Parent(fields) => f.debug_tuple("Parent").field(fields).finish(),
            Self::Back(i) => f.debug_tuple("Back").field(i).finish(),
            Self::Quit => write!(f, "Quit"),
        }
    }
}

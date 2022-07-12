//! Module defining the types for the `tui` feature.
//!
//! This module is mainly used to generate menu using the [`tui`](https://docs.rs/tui/) crate.

use std::fmt;

use tui::{
    backend::Backend,
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget},
    Terminal,
};

use crate::{check_fields, IntoResult, MenuResult};

/// Represents the style of a field in the printed menu.
///
/// The first `Style` field corresponds to the style of the text,
/// and the `Color` field corresponds to the background color of the menu field.
pub type FieldStyle = (Style, Color);

fn consume_cb<B: Backend>(res: EventResult<B>, term: &mut Terminal<B>) -> MenuResult<bool> {
    use EventResult::*;

    match res {
        Callback(b) => {
            b(term)?;
            term.clear()?;
            Ok(false)
        }
        Consumed | Ignored => Ok(false),
        Quit => Ok(true),
    }
}

/// Defines a tui menu, with a title, and the fields.
///
/// It handles the [terminal](Terminal) and the [style](Style) of the fields.
#[derive(Debug)]
pub struct TuiMenu<'a, B: Backend> {
    s_style: FieldStyle,
    f_style: FieldStyle,
    block: Block<'a>,
    levels: Vec<(TuiFields<'a, B>, usize)>,
}

impl<'a, B: Backend> TuiMenu<'a, B> {
    pub fn new(fields: TuiFields<'a, B>) -> Self {
        check_fields(fields);
        Self {
            s_style: (
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .fg(Color::White),
                Color::Black,
            ),
            f_style: (Style::default().fg(Color::Black), Color::White),
            block: Block::default()
                .borders(Borders::all())
                .title_alignment(Alignment::Center),
            levels: vec![(fields, 0)],
        }
    }

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
    pub fn block(mut self, b: Block<'a>) -> Self {
        self.block = b;
        self
    }

    #[cfg(feature = "termion")]
    #[cfg_attr(nightly, doc(cfg(feature = "termion")))]
    pub fn handle_t_event(&mut self, e: ::termion::event::Event) -> EventResult<B> {
        use ::termion::event::{Event, Key};
        use EventResult::*;

        let (fields, mut selected) = match self.levels.pop() {
            Some(lvl) => lvl,
            None => return Quit,
        };

        let mut remaining = true;

        match e {
            Event::Key(k) => match k {
                Key::Char('q') | Key::Ctrl('c') | Key::Ctrl('d') => return Quit,
                Key::Esc if self.levels.is_empty() => return Quit,
                Key::Esc => remaining = false,
                Key::Up | Key::Left if selected == 0 => selected = fields.len() - 1,
                Key::Up | Key::Left => selected -= 1,
                Key::Down | Key::Right if selected == fields.len() - 1 => selected = 0,
                Key::Down | Key::Right => selected += 1,
                Key::Char(' ') | Key::Char('\n') => {
                    let kind = &fields[selected].1;
                    match kind {
                        TuiKind::Map(b) => {
                            self.levels.push((fields, selected));
                            return Callback(&*b);
                        }
                        TuiKind::Parent(inner_fields) => {
                            self.levels.push((fields, selected));
                            remaining = false;
                            self.levels
                                .push((inner_fields, selected.min(inner_fields.len() - 1)));
                        }
                        TuiKind::Back(0) => (),
                        TuiKind::Back(i) => {
                            for _ in 0..i - 1 {
                                self.levels.pop();
                            }
                            remaining = false;
                        }
                        TuiKind::Quit => return Quit,
                    }
                }
                _ => return Ignored,
            },
            _ => return Ignored,
        }

        if remaining {
            self.levels.push((fields, selected));
        }

        Consumed
    }

    #[cfg(feature = "termion")]
    #[cfg_attr(nightly, doc(cfg(feature = "termion")))]
    pub fn handle_t_event_with(
        &mut self,
        e: ::termion::event::Event,
        term: &mut Terminal<B>,
    ) -> MenuResult<bool> {
        consume_cb(self.handle_t_event(e), term)
    }

    #[cfg(feature = "crossterm")]
    #[cfg_attr(nightly, doc(cfg(feature = "crossterm")))]
    pub fn handle_ct_event(&mut self, e: ::crossterm::event::Event) -> EventResult<B> {
        use ::crossterm::event::{Event, KeyCode, KeyEvent};
        use crossterm::event::KeyModifiers;
        use EventResult::*;

        let (fields, mut selected) = match self.levels.pop() {
            Some(lvl) => lvl,
            None => return Quit,
        };

        let mut remaining = true;

        match e {
            Event::Key(KeyEvent { code, modifiers }) => match code {
                KeyCode::Char('q') => return Quit,
                KeyCode::Char('c') | KeyCode::Char('d') if modifiers == KeyModifiers::CONTROL => {
                    return Quit
                }
                KeyCode::Esc if self.levels.is_empty() => return Quit,
                KeyCode::Esc => remaining = false,
                KeyCode::Up | KeyCode::Left if selected == 0 => selected = fields.len() - 1,
                KeyCode::Up | KeyCode::Left => selected -= 1,
                KeyCode::Down | KeyCode::Right if selected == fields.len() - 1 => selected = 0,
                KeyCode::Down | KeyCode::Right => selected += 1,
                KeyCode::Enter | KeyCode::Char(' ') => {
                    let kind = &fields[selected].1;
                    match kind {
                        TuiKind::Map(b) => {
                            self.levels.push((fields, selected));
                            return Callback(&*b);
                        }
                        TuiKind::Parent(inner_fields) => {
                            self.levels.push((fields, selected));
                            remaining = false;
                            self.levels
                                .push((inner_fields, selected.min(inner_fields.len() - 1)));
                        }
                        TuiKind::Back(0) => (),
                        TuiKind::Back(i) => {
                            for _ in 0..i - 1 {
                                self.levels.pop();
                            }
                            remaining = false;
                        }
                        TuiKind::Quit => return Quit,
                    }
                }
                _ => {
                    self.levels.push((fields, selected));
                    return Ignored;
                }
            },
            _ => {
                self.levels.push((fields, selected));
                return Ignored;
            }
        }

        if remaining {
            self.levels.push((fields, selected));
        }

        Consumed
    }

    #[cfg(feature = "crossterm")]
    #[cfg_attr(nightly, doc(cfg(feature = "crossterm")))]
    pub fn handle_ct_event_with(
        &mut self,
        e: ::crossterm::event::Event,
        term: &mut Terminal<B>,
    ) -> MenuResult<bool> {
        consume_cb(self.handle_ct_event(e), term)
    }
}

pub enum EventResult<'a, B: Backend> {
    Quit,
    Consumed,
    Ignored,
    Callback(&'a dyn Fn(&mut Terminal<B>) -> MenuResult),
}

impl<'a, B: Backend> Widget for &TuiMenu<'a, B> {
    fn render(self, area @ Rect { x, y, width, .. }: Rect, buf: &mut Buffer) {
        let (fields, selected) = self.levels.last().unwrap();

        if self.levels.len() == 1 {
            // Render the main menu block
            self.block.clone()
        } else {
            // Render the nested menu block
            let (fields, index) = &self.levels[self.levels.len() - 2];
            let msg = &fields[*index].0;
            self.block.clone().title(*msg)
        }
        .render(area, buf);

        for (i, (msg, _)) in fields.into_iter().enumerate() {
            let (fg_style, bg_style) = if i == *selected {
                (self.s_style.0, Style::default().bg(self.s_style.1))
            } else {
                (self.f_style.0, Style::default().bg(self.f_style.1))
            };

            buf.set_stringn(x + 2, y + 1 + i as u16, msg, width as usize - 4, fg_style);
            buf.set_style(Rect::new(x + 1, y + 1 + i as u16, width - 2, 1), bg_style);
        }
    }
}

/// A tui menu field.
///
/// The string slice corersponds to the message displayed in the list,
/// and the kind corresponds to its behavior.
///
/// See [`TuiKind`] for more information.
pub type TuiField<'a, B> = (&'a str, TuiKind<'a, B>);

/// The tui menu fields.
///
/// It simply corresponds to a slice of fields.
/// It is used for more convenience in the library.
pub type TuiFields<'a, B> = &'a [TuiField<'a, B>];

/// Corresponds to the function mapped to a field.
///
/// It can be viewed as a callback for a menu button.
/// This function is called right after the user selected the corresponding field.
pub type TuiCallback<B> = Box<dyn Fn(&mut Terminal<B>) -> MenuResult>;

/// Defines the behavior of a [tui field](TuiField).
pub enum TuiKind<'a, B: Backend> {
    /// Maps a function to call right after the user selects the field.
    Map(TuiCallback<B>),
    /// Defines the current field as a parent menu of a sub-menu defined by its given fields.
    Parent(TuiFields<'a, B>),
    /// Allows the user to go back to the given depth level from the current running page.
    ///
    /// The depth level of the current running page is at `0`, meaning it will stay at
    /// the current level if the index is at `0` when the user will select the field.
    Back(usize),
    /// Closes all the nested menu pages to the top when the user selects the field.
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

#[macro_export]
macro_rules! tui_mapped {
    ($f:expr, $($s:expr),* $(,)?) => {{
        $crate::tui::map(move |s| $f(s, $($s),*))
    }};
}

pub fn map<'a, B, F, Res>(f: F) -> TuiKind<'a, B>
where
    B: Backend,
    Res: IntoResult,
    F: Fn(&mut Terminal<B>) -> Res + 'static,
{
    TuiKind::Map(Box::new(move |s| f(s).into_result()))
}

#[inline(always)]
pub fn parent<'a, B: Backend>(f: TuiFields<'a, B>) -> TuiKind<'a, B> {
    TuiKind::Parent(f)
}

#[inline(always)]
pub fn back<'a, B: Backend>(i: usize) -> TuiKind<'a, B> {
    TuiKind::Back(i)
}

#[inline(always)]
pub fn quit<'a, B: Backend>() -> TuiKind<'a, B> {
    TuiKind::Quit
}

mod event;
pub mod utils;

use crate::prelude::*;
use std::io::{stdin, Read, Stdin};
use std::ops::{Deref, DerefMut};

use tui::{
    backend::Backend,
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Widget},
    Terminal,
};

#[cfg(feature = "crossterm")]
#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "crossterm")))]
pub type TuiBackend<W = Out> = tui::backend::CrosstermBackend<W>;

#[cfg(feature = "termion")]
#[cfg_attr(feature = "doc-cfg", doc(cfg(feature = "termion")))]
pub type TuiBackend<W = Out> =
    tui::backend::TermionBackend<termion::input::MouseTerminal<termion::raw::RawTerminal<W>>>;

use self::event::{Event, KeyEvent};
use self::utils::{restore_terminal, setup_terminal};

use super::stream::Stream;
use super::RefStream;

pub type FieldStyle = (Style, Color);

pub struct TuiMenu<'a, R = Stdin, B: Backend = TuiBackend> {
    block: Block<'a>,
    s_style: FieldStyle,
    f_style: FieldStyle,
    fields: TuiFields<'a, B>,
    term: Stream<'a, Terminal<B>>,
    reader: fn() -> R,
}

impl<'a, R, B: Backend> Streamable<'a, Terminal<B>> for TuiMenu<'a, R, B> {
    fn take_stream(self) -> Terminal<B> {
        self.term.retrieve()
    }

    fn get_stream(&self) -> &Terminal<B> {
        self.term.deref()
    }

    fn get_mut_stream(&mut self) -> &mut Terminal<B> {
        self.term.deref_mut()
    }
}

impl<'a, B: Backend> RefStream<'a, Terminal<B>, TuiFields<'a, B>> for TuiMenu<'a, Stdin, B> {
    fn new(term: Stream<'a, Terminal<B>>, fields: TuiFields<'a, B>) -> Self {
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
            reader: stdin,
        }
    }
}

impl<'a> TryFrom<TuiFields<'a>> for TuiMenu<'a> {
    type Error = MenuError;

    fn try_from(fields: TuiFields<'a>) -> MenuResult<Self> {
        Ok(Self::owned(setup_terminal()?, fields))
    }
}

impl<'a, const N: usize> TryFrom<&'a [TuiField<'a>; N]> for TuiMenu<'a> {
    type Error = MenuError;

    fn try_from(fields: &'a [TuiField<'a>; N]) -> MenuResult<Self> {
        Self::try_from(fields.as_ref())
    }
}

impl<'a, R, B: Backend> TuiMenu<'a, R, B> {
    pub fn selected_style(mut self, style: Style) -> Self {
        self.s_style.0 = style;
        self
    }

    pub fn selected_bg(mut self, c: Color) -> Self {
        self.s_style.1 = c;
        self
    }

    pub fn field_style(mut self, style: Style) -> Self {
        self.f_style.0 = style;
        self
    }

    pub fn field_bg(mut self, c: Color) -> Self {
        self.f_style.1 = c;
        self
    }

    pub fn with_block(mut self, b: Block<'a>) -> Self {
        self.block = b;
        self
    }
}

impl<'a, R, B> TuiMenu<'a, R, B>
where
    R: Read,
    B: Backend,
{
    pub fn run(&mut self) -> MenuResult {
        self.run_with(self.term.size()?)
    }

    pub fn run_with(&mut self, area: Rect) -> MenuResult {
        run_with(
            &mut RunParams {
                term: self.term.deref_mut(),
                area,
                s_style: &self.s_style,
                f_style: &self.f_style,
                reader: self.reader,
            },
            &self.block,
            self.fields,
        )
        .map(|_| ())
    }
}

impl<'a, R> TuiMenu<'a, R, TuiBackend> {
    pub fn restore_term(&mut self) -> MenuResult {
        restore_terminal(self.term.deref_mut()).map_err(MenuError::from)
    }
}

struct MenuWidget<'a> {
    fields: Vec<&'a str>,
    block: Block<'a>,
    s_style: &'a FieldStyle,
    f_style: &'a FieldStyle,
    selected: usize,
}

impl<'a> Widget for MenuWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        self.block.render(area, buf);

        for (i, msg) in self.fields.into_iter().enumerate() {
            let (fg_style, bg_style) = if i == self.selected {
                (self.s_style.0, Style::default().bg(self.s_style.1))
            } else {
                (self.f_style.0, Style::default().bg(self.f_style.1))
            };

            buf.set_stringn(2, 1 + i as u16, msg, area.width as usize - 3, fg_style);
            buf.set_style(Rect::new(1, 1 + i as u16, area.width - 2, 1), bg_style);
        }
    }
}

struct RunParams<'a, B: Backend, R> {
    term: &'a mut Terminal<B>,
    area: Rect,
    s_style: &'a FieldStyle,
    f_style: &'a FieldStyle,
    reader: fn() -> R,
}

fn run_with<B: Backend, R: Read>(
    params: &mut RunParams<B, R>,
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

        if let Event::Key(k) = event::read((params.reader)())? {
            match k {
                KeyEvent::Char('q') | KeyEvent::Ctrl('c') => return Ok(None),
                KeyEvent::Up | KeyEvent::Left => {
                    selected = selected.checked_sub(1).unwrap_or(fields.len() - 1);
                    continue;
                }
                KeyEvent::Down | KeyEvent::Right => {
                    selected = if selected == fields.len() - 1 {
                        0
                    } else {
                        selected + 1
                    };
                    continue;
                }
                KeyEvent::Enter => {
                    let (msg, kind) = &fields[selected];
                    match kind {
                        TuiKind::Map(b) => {
                            b(params.term)?;
                            return Ok(None);
                        }
                        TuiKind::Parent(fields) => {
                            match run_with(params, &block.clone().title(*msg), fields)? {
                                None => return Ok(None),
                                Some(0) => continue,
                                Some(i) => return Ok(Some(i - 1)),
                            }
                        }
                        TuiKind::Back(0) => continue,
                        TuiKind::Back(i) => return Ok(Some(i - 1)),
                        TuiKind::Quit => return Ok(None),
                    }
                }
                _ => (),
            }
        }
    }
}

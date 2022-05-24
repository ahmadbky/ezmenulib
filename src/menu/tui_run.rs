use crate::prelude::*;
use crossterm::event::{
    read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers,
};
use crossterm::execute;
use crossterm::terminal::{enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use std::io::{stdout, Write};
use std::ops::{Deref, DerefMut};
use tui::backend::CrosstermBackend;
use tui::style::{Color, Modifier, Style};

use tui::buffer::Buffer;
use tui::layout::{Alignment, Rect};
use tui::widgets::{Block, Borders, Widget};
use tui::{backend::Backend, Terminal};

use super::stream::Stream;
use super::RefStream;

type FieldStyle = (Style, Color);

pub struct TuiMenu<'a, B: Backend = CrosstermBackend<Out>> {
    block: Block<'a>,
    fields: TuiFields<'a, B>,
    s_style: FieldStyle,
    f_style: FieldStyle,
    term: Stream<'a, Terminal<B>>,
}

impl<'a, B: Backend> Streamable<'a, Terminal<B>> for TuiMenu<'a, B> {
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

impl<'a, B: Backend> RefStream<'a, Terminal<B>, TuiFields<'a, B>> for TuiMenu<'a, B> {
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
        }
    }
}

impl<'a> TryFrom<TuiFields<'a>> for TuiMenu<'a> {
    type Error = MenuError;

    fn try_from(fields: TuiFields<'a>) -> MenuResult<Self> {
        // Setup terminal
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let term = Terminal::new(backend)?;

        Ok(Self::new_owned(term, fields))
    }
}

impl<'a, const N: usize> TryFrom<&'a [TuiField<'a>; N]> for TuiMenu<'a> {
    type Error = MenuError;

    fn try_from(fields: &'a [TuiField<'a>; N]) -> MenuResult<Self> {
        Self::try_from(fields.as_ref())
    }
}

impl<'a, B: Backend> TuiMenu<'a, B> {
    pub fn new_owned(term: Terminal<B>, fields: TuiFields<'a, B>) -> Self {
        Self::owned(term, fields)
    }

    pub fn new_borrowed(term: &'a mut Terminal<B>, fields: TuiFields<'a, B>) -> Self {
        Self::borrowed(term, fields)
    }

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
            },
            &self.block,
            self.fields,
        )
        .map(|_| ())
    }
}

impl<'a, B> TuiMenu<'a, B>
where
    B: Backend + Write,
{
    pub fn restore_term(&mut self) -> MenuResult {
        execute!(
            self.term.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture,
        )?;
        self.term.show_cursor()?;
        Ok(())
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

struct RunParams<'a, B: Backend> {
    term: &'a mut Terminal<B>,
    area: Rect,
    s_style: &'a FieldStyle,
    f_style: &'a FieldStyle,
}

fn run_with<B: Backend>(
    params: &mut RunParams<B>,
    block: &Block,
    fields: TuiFields<B>,
) -> MenuResult<Option<usize>> {
    let mut selected = 0usize;

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

        if let Event::Key(KeyEvent { code, modifiers }) = read()? {
            match code {
                KeyCode::Char('q') => return Ok(None),
                KeyCode::Char('c') if modifiers == KeyModifiers::CONTROL => return Ok(None),
                KeyCode::Up | KeyCode::Left => {
                    selected = selected.checked_sub(1).unwrap_or(fields.len() - 1);
                    continue;
                }
                KeyCode::Down | KeyCode::Right => {
                    selected = if selected == fields.len() - 1 {
                        0
                    } else {
                        selected + 1
                    };
                    continue;
                }
                KeyCode::Enter => {
                    let (msg, kind) = &fields[selected];
                    match kind {
                        TuiKind::Map(b) => return b(params.term).map(|_| None),
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

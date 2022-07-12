use std::{
    cell::RefCell,
    io::{self, stdout, Write},
    marker::PhantomData,
    rc::Rc,
    time::{Duration, Instant},
};

use ::tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use crossterm::{
    event::{poll, read, DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ezmenulib::{
    prelude::*,
    tui::{back, map, parent, quit, EventResult, TuiMenu},
};

struct App {
    firstname: String,
    lastname: String,
    playing: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            firstname: "Ahmad".to_owned(),
            lastname: "Baalbaky".to_owned(),
            playing: false,
        }
    }
}

impl App {
    fn change_firstname<B: Backend + Write>(&mut self, term: &mut Terminal<B>) -> MenuResult {
        change_name(term, &mut self.firstname, "first")
    }

    fn change_lastname<B: Backend + Write>(&mut self, term: &mut Terminal<B>) -> MenuResult {
        change_name(term, &mut self.lastname, "last")
    }

    fn handle_play(&mut self) {
        self.playing = !self.playing;
    }
}

fn restore_terminal<B: Backend + Write>(term: &mut Terminal<B>) -> io::Result<()> {
    disable_raw_mode()?;
    execute!(
        term.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    term.show_cursor()
}

fn setup_terminal<B: Backend + Write>(term: &mut Terminal<B>) -> io::Result<()> {
    enable_raw_mode()?;
    execute!(term.backend_mut(), EnterAlternateScreen, EnableMouseCapture)?;
    term.show_cursor()
}

fn change_name<B: Backend + Write>(
    term: &mut Terminal<B>,
    name: &mut String,
    span: &str,
) -> MenuResult {
    restore_terminal(term)?;
    writeln!(term.backend_mut(), "Current {span}name: {}", name)?;
    let new: Option<String> = Written::from(format!("Enter the new {span}name").as_str())
        .optional_value(&mut MenuStream::from_writer(term.backend_mut()))?;
    if let Some(new) = new {
        *name = new;
    }
    setup_terminal(term)?;
    Ok(())
}

fn main() -> MenuResult {
    let app = Rc::new(RefCell::new(App::default()));
    let edit_first = app.clone();
    let edit_last = app.clone();
    let edit_play = app.clone();

    let mut term = Terminal::new(CrosstermBackend::new(stdout()))?;
    setup_terminal(&mut term)?;

    let name = &[
        (
            "Firstname",
            map(move |t| edit_first.borrow_mut().change_firstname(t)),
        ),
        (
            "Lastname",
            map(move |t| edit_last.borrow_mut().change_lastname(t)),
        ),
        ("Main menu", back(2)),
    ];

    let settings = &[
        ("Name", parent(name)),
        ("Main menu", back(1)),
        ("Quit", quit()),
    ];

    let fields = &[
        ("Play", map(move |_| edit_play.borrow_mut().handle_play())),
        ("Settings", parent(settings)),
        ("Quit", quit()),
    ];

    // Left part: menu; right part: information
    let root = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Min(0)])
        .split(term.size()?);

    // Top part: names; bottom part: game status (playing or not)
    let info = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Min(0)])
        .split(root[1]);

    let mut menu = TuiMenu::new(fields).block(
        Block::default()
            .title("A dynamic TUI menu")
            .borders(Borders::all()),
    );

    let tick_rate = Duration::from_millis(100);
    let mut status = ['/', '-', '\\', '|'].into_iter().cycle();

    let mut last_tick = Instant::now();
    let mut current = status.next().unwrap();

    loop {
        if last_tick.elapsed() >= tick_rate {
            current = status.next().unwrap();
            last_tick = Instant::now();
        }

        term.draw(|f| {
            f.render_widget(&menu, root[0]);

            let app = app.borrow();

            let list = List::new([
                ListItem::new(app.firstname.as_str()),
                ListItem::new(app.lastname.as_str()),
            ])
            .block(Block::default().title("Names").borders(Borders::all()))
            .highlight_symbol(">>");

            f.render_widget(list, info[0]);

            let txt = Paragraph::new(format!(
                "{} {}",
                if app.playing {
                    "Currently playing"
                } else {
                    "Not yet playing"
                },
                current,
            ))
            .block(Block::default().title("Status").borders(Borders::all()));

            f.render_widget(txt, info[1]);
        })?;

        if poll(tick_rate)? {
            if menu.handle_ct_event_with(read()?, &mut term)? {
                // User chose to quit
                break;
            }
        }
    }

    restore_terminal(&mut term)?;

    Ok(())
}

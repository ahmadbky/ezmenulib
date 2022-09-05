use std::{
    cell::RefCell,
    io::{self, stdout, Write},
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
use ezmenulib::{field::Promptable, prelude::*, tui::Menu};

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
    if let Some(new) = Written::from(format!("Enter the new {span}name").as_str())
        .optional_prompt(MenuHandle::from_writer(term.backend_mut()))?
    {
        *name = new;
    }
    setup_terminal(term)?;
    Ok(())
}

thread_local! {
    static APP: Rc<RefCell<App>> = Default::default();
}

#[derive(Menu)]
#[menu(tui)]
enum Name {
    #[menu(map_with(mut APP: |h| app.change_firstname(h)))]
    Firstname,
    #[menu(map_with(mut APP: |h| app.change_lastname(h)))]
    Lastname,
    #[menu(back(2))]
    MainMenu,
}

#[derive(Menu)]
#[menu(tui)]
enum Settings {
    #[menu(parent)]
    Name,
    #[menu(back)]
    MainMenu,
    Quit,
}

#[derive(Menu)]
#[menu(tui, title = "A dynamic TUI menu")]
enum MainMenu {
    #[menu(map_with(mut APP: |_| app.handle_play()))]
    Play,
    #[menu(parent)]
    Settings,
    Quit,
}

fn main() -> MenuResult {
    let mut term = Terminal::new(CrosstermBackend::new(stdout()))?;
    setup_terminal(&mut term)?;

    // Left part: menu; right part: information
    let root = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Min(0)])
        .split(term.size()?);

    // Top part: names; bottom part: game status (playing or not)
    let info = Layout::default()
        .constraints([Constraint::Percentage(50), Constraint::Min(0)])
        .split(root[1]);

    let mut menu = MainMenu::tui_menu();

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

            let app = APP.with(|app| app.clone());
            let app = app.borrow();

            let list = List::new([
                ListItem::new(format!("Firstname:   {}", app.firstname)),
                ListItem::new(format!("Lastname:    {}", app.lastname)),
            ])
            .block(Block::default().title("Names").borders(Borders::all()));

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

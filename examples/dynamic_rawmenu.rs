use ezmenulib::{field::*, menu::Handle, prelude::*};
use std::{cell::RefCell, fmt, io, rc::Rc};

struct App {
    firstname: String,
    lastname: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            firstname: "Ahmad".to_owned(),
            lastname: "Baalbaky".to_owned(),
        }
    }
}

impl App {
    fn change_firstname<H: Handle>(&mut self, s: H) -> MenuResult {
        change_name(s, &mut self.firstname, "first")
    }

    fn change_lastname<H: Handle>(&mut self, s: H) -> MenuResult {
        change_name(s, &mut self.lastname, "last")
    }
}

impl fmt::Display for App {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.firstname, self.lastname)
    }
}

fn change_name<H: Handle>(mut s: H, name: &mut String, span: &str) -> MenuResult {
    writeln!(s, "Current {span}name: {}", name)?;
    if let Some(new) = Written::from(format!("Enter the new {span}name").as_str())
        .format(Format {
            line_brk: false,
            suffix: ": ",
            ..Default::default()
        })
        .optional_prompt(&mut s)?
    {
        *name = new;
    } else {
        writeln!(s, "The {span}name hasn't been modified.")?;
    }
    Ok(())
}

thread_local! {
    static APP: Rc<RefCell<App>> = Default::default();
}

#[derive(Menu)]
enum Name {
    #[menu(map_with(mut APP: |h| app.change_firstname(h)))]
    EditFirstname,
    #[menu(map_with(mut APP: |h| app.change_lastname(h)))]
    EditLastname,
    #[menu(back(2))]
    MainMenu,
}

#[derive(Menu)]
enum Settings {
    #[menu(parent)]
    Name,
    #[menu(back)]
    MainMenu,
    Quit,
}

#[derive(Menu)]
enum MainMenu {
    #[menu(map_with(APP: play))]
    Play,
    #[menu(parent)]
    Settings,
    Quit,
}

fn play<H: Handle>(mut s: H, play: &App) -> io::Result<()> {
    writeln!(s, "Now playing with {play}")
}

fn main() {
    MainMenu::run();
}

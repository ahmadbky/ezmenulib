use std::{cell::RefCell, fmt, io, rc::Rc};

use ezmenulib::{field::*, menu::Handle, prelude::*};

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
    fn change_firstname<H: Handle>(&mut self, s: D<H>) -> MenuResult {
        change_name(s, &mut self.firstname, "first")
    }

    fn change_lastname<H: Handle>(&mut self, s: D<H>) -> MenuResult {
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

fn play<H: Handle>(mut s: D<H>, play: &App) -> io::Result<()> {
    writeln!(s, "Now playing with {play}")
}

fn main() -> MenuResult {
    let app = Rc::new(RefCell::new(App::default()));
    let edit_play = app.clone();
    let edit_first = app.clone();
    let edit_last = app.clone();

    RawMenu::from(&[
        ("Play", mapped!(play, &*edit_play.borrow())),
        (
            "Settings",
            parent(&[
                (
                    "Name",
                    parent(&[
                        (
                            "Firstname",
                            map(move |s| edit_first.borrow_mut().change_firstname(s)),
                        ),
                        (
                            "Lastname",
                            map(move |s| edit_last.borrow_mut().change_lastname(s)),
                        ),
                        ("Main menu", back(2)),
                    ]),
                ),
                ("Main menu", back(1)),
                ("Quit", quit()),
            ]),
        ),
        ("Quit", quit()),
    ])
    .title("Basic menu")
    .run()
}

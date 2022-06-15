use std::{cell::RefCell, fmt, io::Write, rc::Rc};

use ezmenulib::prelude::*;

type Res<T> = Rc<RefCell<T>>;

#[derive(Clone)]
struct Data {
    first: Res<String>,
    last: Res<String>,
}

impl Data {
    fn new(first: Res<String>, last: Res<String>) -> Self {
        Self { first, last }
    }
}

impl fmt::Display for Data {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.first.borrow(), self.last.borrow())
    }
}

fn playing(s: &mut MenuStream, data: Data) -> MenuResult {
    writeln!(s, "Now playing with {}", data)?;
    Ok(())
}

fn change_name(s: &mut MenuStream, name: Res<String>, span: &str) -> MenuResult {
    writeln!(s, "Current {span}name: {}", name.borrow())?;
    let new: Option<String> =
        Written::from(format!("Enter the new {span}name").as_str()).optional_value(s)?;
    if let Some(new) = new {
        *name.borrow_mut() = new;
    } else {
        writeln!(s, "The {span}name hasn't been modified.")?;
    }
    Ok(())
}

fn main() -> MenuResult {
    let first = Rc::new(RefCell::new("Ahmad".to_owned()));
    let last = Rc::new(RefCell::new("Baalbaky".to_owned()));

    let data = Data::new(first.clone(), last.clone());

    RawMenu::from(&[
        ("Play", Kind::Map(&move |s| playing(s, data.clone()))),
        (
            "Settings",
            Kind::Parent(&[
                (
                    "Name",
                    Kind::Parent(&[
                        (
                            "Firstname",
                            Kind::Map(&move |s| change_name(s, first.clone(), "first")),
                        ),
                        (
                            "Lastname",
                            Kind::Map(&move |s| change_name(s, last.clone(), "last")),
                        ),
                        ("Main menu", Kind::Back(2)),
                    ]),
                ),
                ("Main menu", Kind::Back(1)),
                ("Quit", Kind::Quit),
            ]),
        ),
        ("Quit", Kind::Quit),
    ])
    .title("Basic menu")
    .run()
}

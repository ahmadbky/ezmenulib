use ezmenulib::{field::kinds::*, menu::Handle, prelude::*};
use std::io;

fn play<H: Handle>(s: &mut H) -> io::Result<()> {
    writeln!(s, "Now playing.")
}

fn edit_name<H: Handle>(s: &mut H, span: &str) -> io::Result<()> {
    writeln!(s, "Editing {span}stname")
}

fn main() {
    RawMenu::from(&[
        ("Play", map(play)),
        (
            "Settings",
            parent(&[
                (
                    "Name",
                    parent(&[
                        ("Firstname", mapped!(edit_name, "fir")),
                        ("Lastname", mapped!(edit_name, "la")),
                        ("Main menu", back(2)),
                    ]),
                ),
                ("Main menu", back(1)),
                ("Quit", quit()),
            ]),
        ),
        ("Quit", quit()),
    ])
    .run()
    .unwrap();
}

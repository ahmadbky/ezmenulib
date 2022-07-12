use std::io::Write;

use ezmenulib::{field::*, prelude::*};

fn main() {
    RawMenu::from(&[
        ("Play", map(|s| writeln!(s, "Now playing."))),
        (
            "Settings",
            parent(&[
                (
                    "Name",
                    Kind::Parent(&[
                        ("Firstname", map(|s| writeln!(s, "Editing firstname."))),
                        ("Lastname", map(|s| writeln!(s, "Editing lastname."))),
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

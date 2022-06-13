use std::io::Write;

use ezmenulib::prelude::*;

fn playing(s: &mut MenuStream) -> MenuResult {
    writeln!(s, "now playing")?;
    Ok(())
}

fn firstname(s: &mut MenuStream) -> MenuResult {
    writeln!(s, "editing firstname")?;
    Ok(())
}

fn lastname(s: &mut MenuStream) -> MenuResult {
    writeln!(s, "editing lastname")?;
    Ok(())
}

fn main() {
    RawMenu::from(&[
        ("Play", Kind::Map(&playing)),
        (
            "Settings",
            Kind::Parent(&[
                (
                    "Name",
                    Kind::Parent(&[
                        ("Firstname", Kind::Map(&firstname)),
                        ("Lastname", Kind::Map(&lastname)),
                        ("Main menu", Kind::Back(2)),
                    ]),
                ),
                ("Main menu", Kind::Back(1)),
                ("Quit", Kind::Quit),
            ]),
        ),
        ("Quit", Kind::Quit),
    ])
    .run()
    .unwrap();
}

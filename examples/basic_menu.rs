use ezmenulib::prelude::*;

fn name(_: &mut MenuStream) -> MenuResult {
    Ok(())
}

fn main() {
    Menu::from(&[
        ("Play", Kind::Unit(|_| Ok(()))),
        (
            "Settings",
            Kind::Parent(&[("Name", Kind::Unit(name)), ("Main menu", Kind::Back)]),
        ),
    ])
    .run()
    .unwrap();
}

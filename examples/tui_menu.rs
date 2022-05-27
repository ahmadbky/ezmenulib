use ezmenulib::{menu::tui_run::TuiMenu, prelude::*};

fn main() -> MenuResult {
    let mut menu = TuiMenu::try_from(&[
        ("hello", TuiKind::Quit),
        ("hey", TuiKind::Quit),
        (
            "no",
            TuiKind::Parent(&[
                ("first", TuiKind::Quit),
                ("second", TuiKind::Quit),
                ("go back", TuiKind::Back(1)),
            ]),
        ),
    ])?;

    menu.run()?;
    menu.restore_term()?;

    Ok(())
}

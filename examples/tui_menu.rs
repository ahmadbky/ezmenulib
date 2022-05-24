use ezmenulib::prelude::*;
use tui::{
    style::{Color, Style},
    widgets::{Block, BorderType, Borders},
};

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
    ])?
    .with_block(
        Block::default()
            .title("Hey")
            .borders(Borders::all())
            .border_type(BorderType::Double)
            .border_style(Style::default().fg(Color::Green)),
    );

    menu.run()?;
    menu.restore_term()
}

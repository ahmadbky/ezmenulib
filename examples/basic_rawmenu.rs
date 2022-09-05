use ezmenulib::{menu::Handle, prelude::*};
use std::io;

mod hehe {
    use super::*;

    #[derive(Menu)]
    pub enum Name {
        #[menu(mapped(edit_name, "fir"))]
        Firstname,
        #[menu(mapped(edit_name, "la"))]
        Lastname,
        #[menu(back(2))]
        MainMenu,
    }
}

#[derive(Menu)]
enum Settings {
    #[menu(path = hehe::Name, parent)]
    Name,
    #[menu(back)]
    MainMenu,
    Quit,
}

#[derive(Menu)]
enum MainMenu {
    #[menu(map(play))]
    Play,
    #[menu(parent)]
    Settings,
    Quit,
}

fn play<H: Handle>(s: &mut H) -> io::Result<()> {
    writeln!(s, "Now playing.")
}

fn edit_name<H: Handle>(s: &mut H, span: &str) -> io::Result<()> {
    writeln!(s, "Editing {span}stname")
}

fn main() {
    MainMenu::run();
}

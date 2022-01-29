//use std::io::{stdin, stdout, Write};

pub use menu::Menu;
use std::str::FromStr;

mod test_inits {
    pub fn play(name: &String) {
        if name.to_lowercase() == "ahmad" {
            println!("oh i know u");
        }
    }
}

fn exit(_t: &Type) {
    println!("lol ok exit")
}

#[derive(Menu)]
pub struct MyStructMenu {
    #[field(msg = "Give the author name", then = test_inits::play)]
    pub author: String,
    #[field(msg = "Give the type of the license", then = exit)]
    pub t: Type,
}

#[derive(Debug)]
pub enum Type {
    MIT,
    GPL,
    BSD,
}

// this should be expanded by macro too
impl FromStr for Type {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mit" | "MIT" => Ok(Self::MIT),
            "gpl" | "GPL" => Ok(Self::GPL),
            "bsd" | "BSD" => Ok(Self::BSD),
            _ => Err(()),
        }
    }
}

//#[derive(menu::Menu)]
pub enum MyEnumMenu {
    //#[menu(msg = "Play now", init = test_inits::play)]
    Play,
    //#[menu(init = exit)]
    Exit,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_run() {
        //MyMenu::run();
        //MyMenu::loltest();
    }

    #[test]
    fn struct_base() {
        println!("test");
        let values = MyStructMenu::from_menu();
    }
}

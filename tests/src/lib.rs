use menu::Menu;
//use std::io::{stdin, stdout, Write};

fn play() {}
fn exit() {}

#[derive(Menu)]
#[main(msg = "Hello this is a test menu")]
pub enum MyMenu {
    #[field(msg = "Play now", init = test::play)]
    Play,
    #[field(init = exit)]
    Exit,
}

/*
impl MyMenu {
    pub fn run() {
        use std::io::{stdin, stdout, Write};
        let mut stdout = stdout();
        let stdin = stdin();
        println!("{}", "Hello this is a test menu");

        let msgs = ["Play", "Exit"];
        for (i, msg) in msgs.iter().enumerate() {
            println!("{} - {}", i + 1, msg);
        }

        let i = loop {
            println!(">> ");
            stdout.flush().expect("Unable to flush stdout");
            let mut buf = String::new();
            stdin.read_line(&mut buf).expect("Unable to read line");
            match buf.parse::<usize>() {
                Ok(x) if (1..=msgs.len()).contains(&x) => break x,
                _ => continue,
            }
        };
        match i {
            1 => play(),
            2 => exit(),
            _ => unreachable!(),
        };
    }
}
*/
#[cfg(test)]
mod tests {
    use crate::MyMenu;

    #[test]
    fn call_run() {
        MyMenu::run();
    }
}

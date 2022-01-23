# Menu-derive

The final idea is to code this :
```rust
fn play() {}
fn do_a() {}
fn exit() {}

#[derive(Menu)]
pub enum MyMenu {
    #[menu(msg = "Play now", exe = play)]
    Play,
    #[menu(msg = "Do A", exe = do_a)]
    DoA,
    // Default message displayed ("Exit")
    #[menu(exe = exit)]
    Exit,
}

// function called by macro after checking if parsing the initial string input
// into a number works
// each time a check doesn't work, the menu reasks the user to give a correct value
fn check_nonzero(n: i32) -> bool {
    n != 0
}

#[derive(Menu)]
struct GetUserValues {
    #[menu(
        msg = "Please give a random nonzero integer",
        check = check_nonzero
    )]
    rand: i32,

    #[menu(msg = "Now give a string")]
    s: String,
}

fn main() {
    // either run like this
    MyMenu::run();
    // or like this
    let values = GetUserValues::run();
    println!("given {} and {:?}", values.rand, values.s);
}
```

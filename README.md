# EZMenu

Fast designing menus for your Rust CLI programs.

This crate provides a `derive(Menu)` procedural macro to easily build menus.
It includes type-checking from the user input, and a formatting customization.

## Documentation

You can find all the crate documentation on [Docs.rs](https://docs.rs/ezmenu).
You can also check the [examples](examples) to learn with a practical way.

## Example

Here is an example of how to use it:

```rust
use std::io::Write;
use ezmenu::{Menu, MenuResult};

fn map(val: u32, w: &mut impl Write) -> MenuResult<u32> {
    if val == 1000 {
        w.write(b"ok!\n")?;
    }
    Ok(val)
}

#[derive(Menu)]
#[menu(title = "Hello there!")]
struct MyMenu {
    author: String,
    #[menu(msg = "Give a number", then(map))]
    number: u32,
}
```

To display the menu, you instantiate the struct by calling its `from_menu` method:

```rust
let MyMenu { author, number } = MyMenu::from_menu();
println!("values provided: author={}, number={}", author, number);
```

This sample code prints the standard menu like above:

```
Hello there!
* author: Ahmad
* Give a number: 1000
ok!
values provided: author=Ahmad, number=1000
```

## WIP

This project is still in development.
You can check the [EZMenu project](https://github.com/users/ahbalbk/projects/4) to see all the next features.
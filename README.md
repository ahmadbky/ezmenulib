# EZMenuLib

Fast designing menus for your Rust CLI programs.

This crate provides a library with structs and traits to easily build menus.
It includes type-checking from the user input, and a formatting customization.

## Example

Here is an example of how to use the library:

```rust
use ezmenulib::{Menu, ValueField, ValueMenu};

fn main() {
    let mut my_menu = ValueMenu::from([
        ValueField::from("Give your name"),
        ValueField::from("Give a number"),
    ]);
    
    let name: String = my_menu.next_output().unwrap();
    let number: i32 = my_menu.next_output().unwrap();
}
```

This sample code prints the standard menu like above:

```
Hello there!
* Give your name: Ahmad
* Give a number: 1000
values provided: author=Ahmad, number=1000
```

## Documentation

You can find all the crate documentation on [Docs.rs](https://docs.rs/ezmenulib).
You can also check the [examples](examples) to learn with a practical way.

## WIP

This project is still in development.
You can check the [EZMenu project](https://github.com/users/ahbalbk/projects/4) to look at my todolist :D
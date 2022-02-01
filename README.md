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
use ezmenu::Menu;

fn check(n: &i32) {
    if *n == 0 {
        println!("yo respect me plz :'(");
    } else {
        println!("good. :)");
    }
}

#[derive(Menu)]
struct MyMenu {
    author: String,
    #[field(msg = "Give a nonzero number", then(check))]
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
- author: Ahmad
- Give a nonzero number: 0
yo respect me plz :'(
values provided: author=Ahmad, number=0
```

## WIP

This project is not finished yet, and is at its first release.
You can check the [EZMenu project](https://github.com/users/ahbalbk/projects/4) to see all the next features.
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
    if n < 0 {
        println!("well that's not positive but anyway.");
    } else {
        println!("good :)");
    }
}

#[derive(Menu)]
struct MyMenu {
    author: String,
    #[field(msg = "Give a positive number", then(check))]
    number: i32,
}
```

To display the menu, you construct the struct by calling its `from_menu` method:
```rust
let values = MyMenu::from_menu();
println!("values provided: author={}, number={}", values.author, values.number);
```

This sample code prints the standard menu like above:
```shell
- author: Ahmad
- Give a positive number: -43
well that's not positive but anyway.
values provided: author=Ahmad, number=-43
```
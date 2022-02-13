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
use ezmenu::{Menu, MenuResult};

fn map(val: u32, w: &mut impl Write) -> MenuResult<u32> {
    if val == 1000 {
        w.write(b"ok!\n")?;
    }
    Ok(val)
}

#[derive(Menu)]
struct MyMenu {
    author: String,
    #[field(msg = "Give a nonzero number", then(map))]
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

### Custom I/O types

If you are not using `std::io::Stdin` and `std::io::Stdout` types, you can provide your own
types by enabling the `custom_io` feature in your Cargo.toml file:

```toml
[dependencies]
ezmenu = { version = "0.2.0", features = ["custom_io"] }
```

Then you can instantiate your struct with:

```rust
use std::io::stdout;
let input = b"Ahmad\n1000\n" as &[u8];
let MyMenu { author, number } = MyMenu::from_io(input, stdout());
```

## WIP

This project is not finished yet, and is at its first release.
You can check the [EZMenu project](https://github.com/users/ahbalbk/projects/4) to see all the next features.
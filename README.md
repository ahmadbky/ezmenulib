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
    #[field(msg = "Give a number", then(map))]
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

## Format it as you wish

You can apply several formatting rules on a menu and on a field specifically.
You can edit:
* the chip: `* ` by default.
* the prefix: `: ` by default.
* insert a new line before prefix and user input: `false` by default.
* display default values or not: `true` by default.

### Example

For a custom format on a field and a main formatting rule on a menu, you can build this with:
```rust
#[derive(Menu)]
#[menu(chip = ">> ")]
struct License {
    #[menu(chip = "- ")]
    author: String,
    date: u16,
}
```

The custom `>> ` will be applied on every field except those with custom formatting rules.
In this case, it will format the text like above:

```
- author: ...
>> date: ...
```

## Skip fields with default values

You can provide default values to a field like above:

```rust
#[derive(Menu)]
struct License {
    author: String,
    #[menu(default = 2022)]
    date: u16,
}
```

If the user provided an incorrect input, the program will not re-ask a value to the user,
but will directly return the default value instead.

By default, the default value is visible. If you want to hide it, you can do so:
```rust
#[menu(display_default = false)]
```
on the struct or on a field.

## Custom I/O types

If you are not using `std::io::Stdin` and `std::io::Stdout` types, you can provide your own
types by enabling the `custom_io` feature in your Cargo.toml file:

```toml
[dependencies]
ezmenu = { version = "0.2.3", features = ["custom_io"] }
```

Then you can instantiate your struct with:

```rust
use std::io::stdout;
let input = b"Ahmad\n1000\n" as &[u8];
let values = MyMenu::from_io(input, stdout());
```

## Use custom value types

If the user has to provide a value which corresponds to your specific type,
you can use the `ezmenu::parsed` on this type.
For example, in the case of a mk-license program, the menu can be built like above:

```rust
#[ezmenu::parsed]
enum Type {
    MIT,
    BSD,
    GPL,
}

#[derive(Menu)]
struct License {
    author: String,
    date: u16,
    #[menu(default = "mit")]
    ty: Type,
}
```

This will restrict the user to enter "MIT", "BSD" or "GPL" inputs ignoring the case.

## Derive feature

The `derive(Menu)` is available with the `derive` feature, enabled by default.
You can disable it in your Cargo.toml file:
```toml
[dependencies]
ezmenu = { version = "0.2.3", default-features = false }
```

You can still use the provided library to build your menus.

### Example

To ask a simple value, you can use `StructField::build` method by giving the `Stdin`
and `Stdout` types.

```rust
use std::io::{stdin, stdout};
use ezmenu::StructField;
let age: u8 = StructField::from("How old are you?")
    .build(&stdin(), &mut stdout()).unwrap();
```

If you want to build a menu with all the previous features (default values, formatting rules...),
you can refer to this code below:
```rust
use ezmenu::{StructField, StructFieldFormatting};
let mut menu = StructMenu::default()
    .title("-- Mklicense --")
    .fmt(StructFieldFormatting {
        chip: "* Give the ",
        ..Default::default()
    })
    .with_field(StructField::from("project author name"))
    .with_field(StructField::from("project name"))
    .with_field(StructField::from("Give the year of the license")
        .default("2022")
        .fmt(StructFieldFormatting {
            prefix: ">> ",
            new_line: true,
            ..Default::default()
        })
    );

let name: String = menu.next_map(|s: String, w| {
    if s.to_lowercase() == "ahmad" {
        w.write(b"Nice name!!")?;
    }
    Ok(s)
}).unwrap();
let proj_name: String = menu.next().unwrap();
let proj_year: i64 = menu.next().unwrap();
```

## WIP

This project is still in development.
You can check the [EZMenu project](https://github.com/users/ahbalbk/projects/4) to see all the next features.
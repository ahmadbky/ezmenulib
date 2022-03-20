# EZMenuLib

Fast designing menus for your Rust CLI programs.

> Caution: This library is not completely stable yet.
Many changes may occur depending on versions. I am still looking for a sustainable design
of the library.

This crate provides a library with structs and traits to easily build menus.
It includes type-checking from the user input, and a formatting customization.

This crate is really useful if you use [structopt](https://docs.rs/structopt/)
or [clap](https://docs.rs/clap/) crates beside this, so you can get the matches safely, and
build a menu on your own after.

It can also be used as a mode selection, for a game for example.

### Note

If you want to use the `derive(Menu)` macro,
you must use the [ezmenu](https://docs.rs/ezmenu/) crate instead.
This crate may however contain features that are not available on the ezmenu crate.

## Example

Here is an example of how to use the library:

```rust
use ezmenulib::prelude::*;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let mut my_menu = ValueMenu::from([
        Field::Value(ValueField::from("Give your name")),
        Field::Value(ValueField::from("Give a number")),
        Field::Select(SelectMenu::from([
            SelectField::from("0"),
            SelectField::from("1"),
            SelectField::from("2"),
        ])
        .title(SelectTitle::from("Select a number"))),
    ]);
    
    let name: String = my_menu.next_output()?;
    let number: i32 = my_menu.next_output()?;
    let amount: u8 = my_menu.next_output()?;
    
    println!("hello {}, you entered {} and you selected {}.", name, number, amount);
}
```

This sample code prints the standard menu like above:

```
Hello there!
--> Give your name
>> Ahmad

--> Give a number
>> 1000

--> Select a number:
1 - 0
2 - 1
3 - 2
>> 2

hello Ahmad, you entered 1000 and you selected 2.
```

## Documentation

You can find all the crate documentation on [Docs.rs](https://docs.rs/ezmenulib).
You can also check the [examples](examples) to learn with a practical way.

## WIP

This project is still in development.
You can check the [EZMenu project](https://github.com/users/ahbalbk/projects/4) to take a look at my todolist :D
<div style="text-align: center;">

# EZMenuLib

[![Crates.io](https://img.shields.io/crates/l/ezmenulib?style=flat-square)](./LICENSE)
[![Crates.io](https://img.shields.io/crates/v/ezmenulib?style=flat-square)](https://crates.io/crates/ezmenulib)
[![docs.rs](https://img.shields.io/docsrs/ezmenulib?style=flat-square)](https://docs.rs/ezmenulib)
</div>

Fast designing menus for your Rust CLI programs.

This crate provides a library with structs and traits to easily build menus.
It includes type-checking from the user input, and a formatting customization.

This crate is really useful if you use [structopt](https://docs.rs/structopt/)
or [clap](https://docs.rs/clap/) crates beside this, so you can get the matches safely, and
build a menu on your own after.

It can also be used as a mode selection, for a game for example.

### Note

If you want to use the `derive(Menu)` macro,
you must use the [ezmenu](https://docs.rs/ezmenu/) crate instead.
This crate may however contain features that are not yet available on the ezmenu crate.

## Examples

### Menus

You can construct CLI menus with the library:

```rust
use ezmenulib::prelude::*;
use std::io::Write;

fn playing(s: &mut MenuStream) -> MenuResult {
    writeln!(s, "PLAYING")?;
    Ok(())
}

fn firstnaming(s: &mut MenuStream) -> MenuResult {
    writeln!(s, "EDITING FIRSTNAME")?;
    Ok(())
}

fn lastnaming(s: &mut MenuStream) -> MenuResult {
    writeln!(s, "EDITING LASTNAME")?;
    Ok(())
}

Menu::from(&[
    ("Play", Kind::Map(playing)),
    (
        "Settings",
        Kind::Parent(&[
            ("Name", Kind::Parent(&[
                ("Firstname", Kind::Map(firstnaming)),
                ("Lastname", Kind::Map(lastnaming)),
                ("Main menu", Kind::Back(2)),
            ]))
            ("Go back", Kind::Back(1)),
        ]),
    ),
    ("Quit", Kind::Quit),
])
.title("Basic menu")
.run()?;
}
```

This sample code prints the standard menu like above:

```
Basic menu
1 - Play
2 - Settings
3 - Quit
>> 2
Settings
1 - Name
2 - Go back
>> 1
Name
1 - Firstname
2 - Lastname
3 - Main Menu
>> 3
Basic menu
1 - Play
2 - Settings
3 - Quit
>> 1
PLAYING
```

### Retrieve values

You can get values from the user, by asking him to write the value, or to select among valid values. Follow the `gen_license` example, a sample code to get information about a project to generate a license.

```rust
#[derive(Debug)]
enum Type {
    MIT,
    GPL,
    BSD,
}

impl Selectable<3> for Type {
    fn values() -> [(&'static str, Self); 3] {
        use Type::*;
        [("MIT", MIT), ("GPL", GPL), ("BSD", BSD)]
    }

    fn default() -> Option<usize> {
        Some(0)
    }
}

let mut lic = Values::default();

let authors: Vec<String> =
    lic.many_written(&Written::from("Authors").example("Ahmad, ..."), ", ")?;
let name: Option<String> = lic.optional_written(&Written::from("Project name"))?;
let date: u16 = lic.written(&Written::from("License date").default("2022"))?;
let ty: Type = lic.selected(Selected::from("Select a license type"))?;

println!(
    "{:?} License, Copyright (C) {} {}\n{}",
    ty,
    date,
    authors.join(", "),
    if let Some(n) = name { n } else { "".to_owned() },
);
```

This sample code prints the standard menu like above:

```
--> Authors (example: Ahmad, ...)
>> Ahmad Baalbaky, Hello
--> Project name (optional)
>> 
--> License date (default: 2022)
>> 
--> Select a license type
1 - MIT (default)
2 - GPL
3 - BSD
>> 2
GPL License, Copyright (C) 2022 Ahmad Baalbaky, Hello
```

The user can skip the prompt if it is optional, otherwise the prompt will be reprinted until the entered value is correct.

## Formatting customization

The library allows you to customize the text format behavior in many ways. The rules are defined in the [`Format` ](https://docs.rs/ezmenulib/latest/ezmenulib/field/struct.Format.html) struct.

You may remove the line break between the prompt and the suffix before the user input for example:

```rust
let name: String = Written::from("Name")
    .format(Format {
        line_brk: false,
        suffix: ": ",
        ..Default::default()
    })
    .prompt(&mut MenuStream::default())?;
```

The format can be global and inherited by the [`Values`](https://docs.rs/ezmenulib/latest/ezmenulib/menu/struct.Values.html) container on the following prompts ([`Written`](https://docs.rs/ezmenulib/latest/ezmenulib/field/struct.Written.html) and [`Selected`](https://docs.rs/ezmenulib/latest/ezmenulib/field/struct.Selected.html)).

## Documentation

You can find all the crate documentation on [Docs.rs](https://docs.rs/ezmenulib).
You can also check the [examples](examples) to learn with a practical way.

## WIP

This project is still in development.
You can check the [EZMenu project](https://github.com/users/ahbalbk/projects/4) to take a look at my todolist :D
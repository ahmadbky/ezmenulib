//! # EZMenu
//!
//! Fast designing menus for your Rust CLI programs.
//!
//! This crate provides a `derive(Menu)` procedural macro to easily build menus.
//! It includes type-checking from the user input, and a formatting customization.
//!
//! ## Example
//!
//! Here is an example of how to use it:
//!
//! ```rust
//! use std::io::Write;
//! use ezmenu::{Menu, MenuResult};
//!
//! fn map(val: u32, w: &mut impl Write) -> MenuResult<u32> {
//!     if val == 1000 {
//!         w.write(b"ok!\n")?;
//!     }
//!     Ok(val)
//! }
//!
//! #[derive(Menu)]
//! #[menu(title = "Hello there!")]
//! struct MyMenu {
//!     author: String,
//!     #[menu(msg = "Give a number", then(map))]
//!     number: u32,
//! }
//! ```
//!
//! To display the menu, you instantiate the struct by calling its `from_menu` method:
//!
//! ```rust
//! let MyMenu { author, number } = MyMenu::from_menu();
//! println!("values provided: author={}, number={}", author, number);
//! ```
//!
//! This sample code prints the standard menu like above:
//!
//! ```text
//! Hello there!
//! * author: Ahmad
//! * Give a number: 1000
//! ok!
//! values provided: author=Ahmad, number=1000
//! ```
//!
//! ## Format it as you wish
//!
//! You can apply several formatting rules on a menu and on a field specifically.
//! You can edit:
//! * the chip: `* ` by default.
//! * the prefix: `: ` by default.
//! * insert a new line before prefix and user input: `false` by default.
//! * display default values or not: `true` by default.
//!
//! ### Example
//!
//! For a custom format on a field and a main formatting rule on a menu, you can build this with:
//! ```rust
//! #[derive(Menu)]
//! #[menu(chip = ">> ")]
//! struct License {
//!     #[menu(chip = "- ")]
//!     author: String,
//!     date: u16,
//! }
//! ```
//!
//! The custom `>> ` will be applied on every field except those with custom formatting rules.
//! In this case, it will format the text like above:
//!
//! ```text
//! - author: ...
//! >> date: ...
//! ```
//!
//! ## Skip fields with default values
//!
//! You can provide default values to a field like above:
//!
//! ```rust
//! #[derive(Menu)]
//! struct License {
//!     author: String,
//!     #[menu(default = 2022)]
//!     date: u16,
//! }
//! ```
//!
//! If the user provided an incorrect input, the program will not re-ask a value to the user,
//! but will directly return the default value instead.
//!
//! By default, the default value is visible. If you want to hide it, you can do so:
//! ```rust
//! #[derive(Menu)]
//! #[menu(display_default = false)]
//! struct License {
//!     author: String,
//!     #[menu(default = 2022, display_default = true)]
//!     date: u16,
//! }
//! ```
//!
//! ## Custom I/O types
//!
//! If you are not using `std::io::Stdin` and `std::io::Stdout` types, you can provide your own
//! types by enabling the `custom_io` feature in your Cargo.toml file:
//!
//! ```toml
//! [dependencies]
//! ezmenu = { version = "0.2.3", features = ["custom_io"] }
//! ```
//!
//! Then you can instantiate your struct with:
//!
//! ```rust
//! use std::io::stdout;
//! let input = b"Ahmad\n1000\n" as &[u8];
//! let values = MyMenu::from_io(input, stdout());
//! ```
//!
//! ## Use custom value types
//!
//! If the user has to provide a value which corresponds to your specific type,
//! you can use the `ezmenu::parsed` on this type.
//! For example, in the case of a mk-license program, the menu can be built like above:
//!
//! ```rust
//! #[ezmenu::parsed]
//! enum Type {
//!     MIT,
//!     BSD,
//!     GPL,
//! }
//!
//! #[derive(Menu)]
//! struct License {
//!     author: String,
//!     date: u16,
//!     #[menu(default = "mit")]
//!     ty: Type,
//! }
//! ```
//!
//! This will restrict the user to enter "MIT", "BSD" or "GPL" inputs ignoring the case.
//!
//! ## Derive feature
//!
//! The `derive(Menu)` is available with the `derive` feature, enabled by default.
//! You can disable it in your Cargo.toml file:
//! ```toml
//! [dependencies]
//! ezmenu = { version = "0.2.3", default-features = false }
//! ```
//!
//! You can still use the provided library to build your menus.
//!
//! ### Example
//!
//! To ask a simple value, you can use `StructField::build` method by giving the `Stdin`
//! and `Stdout` types.
//!
//! ```rust
//! use std::io::{stdin, stdout};
//! use ezmenu::ValueField;
//! let age: u8 = ValueField::from("How old are you?")
//!    .build(&stdin(), &mut stdout()).unwrap();
//! ```
//!
//! If you want to build a menu with all the previous features (default values, formatting rules...),
//! you can refer to this code below:
//! ```rust
//! use ezmenu::{ValueField, ValueFieldFormatting};
//! let mut menu = StructMenu::default()
//!     .title("-- Mklicense --")
//!     .fmt(ValueFieldFormatting {
//!         chip: "* Give the ",
//!        ..Default::default()
//!     })
//!     .with_field(ValueField::from("project author name"))
//!     .with_field(ValueField::from("project name"))
//!     .with_field(
//!         ValueField::from("Give the year of the license")
//!             .default("2022")
//!             .fmt(ValueFieldFormatting {
//!                 prefix: ">> ",
//!                 new_line: true,
//!                 ..Default::default()
//!             }),
//!     );
//!
//! let name: String = menu
//!     .next_map(|s: String, w| {
//!         if s.to_lowercase() == "ahmad" {
//!             w.write(b"Nice name!!")?;
//!         }
//!         Ok(s)
//!     }).unwrap();
//!
//! let proj_name: String = menu.next().unwrap();
//! let proj_year: i64 = menu.next().unwrap();
//! ```
mod customs;
mod field;
mod menu;

/// The `derive(Menu)` macro
#[cfg(feature = "derive")]
pub use ezmenu_derive::Menu;

/// The `ezmenu::parsed` attribute macro
#[cfg(feature = "parsed_attr")]
pub use ezmenu_derive::parsed;

pub use customs::{MenuBool, MenuVec};
pub use field::{ValueField, ValueFieldFormatting};
pub use menu::{Menu, ValueMenu};

use std::fmt::Debug;
use std::{fmt, io};

/// The error type used by the menu builder.
pub enum MenuError {
    /// An IO error, when flushing, reading or writing values,
    IOError(io::Error),
    /// An incorrect type of value has been used as default value.
    IncorrectType(Box<dyn Debug>),
    /// There is no more field to call for an output.
    /// This error appears when calling `<StructMenu as Menu>::next` method whereas
    /// the menu building has finished for example.
    NoMoreField,
    /// A custom error type.
    /// You can define this type when mapping the output value of the `Menu::next_map` method,
    /// by returning an `Err(MenuError::Other(...))`
    Other(Box<dyn Debug>),
}

impl fmt::Debug for MenuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "{}",
            match self {
                Self::IOError(e) => format!("IO error: {:?}", e),
                Self::IncorrectType(e) => format!(
                    "an incorrect value type has been used as default value: {:?}",
                    *e
                ),
                Self::NoMoreField =>
                    "attempted to get the next output while there is no more field in the menu"
                        .to_owned(),
                Self::Other(e) => format!("an error occurred: {:?}", e),
            }
        )
    }
}

impl From<io::Error> for MenuError {
    fn from(e: io::Error) -> Self {
        Self::IOError(e)
    }
}

/// The main result type used in the EZMenu library.
pub type MenuResult<T> = Result<T, MenuError>;

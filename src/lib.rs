//! Fast designing menus for your Rust CLI programs.
//!
//! This crate provides a library with structs and traits to easily build menus.
//! It includes type-checking from the user input, and a formatting customization.
//!
//! ### Note
//!
//! If you want to use the derive Menu macro,
//! you must use the [`ezmenu`](https://docs.rs/ezmenu/) crate instead.
//!
//! ## Example
//!
//! Here is an example of how to use the library:
//!
//! ```rust
//! use ezmenulib::{Menu, ValueField, ValueMenu};
//!
//! fn main() {
//!     let mut my_menu = ValueMenu::from([
//!         ValueField::from("Give your name"),
//!         ValueField::from("Give a number"),
//!     ])
//!    .title("Hello there!");
//!
//!     let name: String = my_menu.next_output().unwrap();
//!     let number: i32 = my_menu.next_output().unwrap();
//!
//!     println!("values provided: name={}, number={}", name, number);
//! }
//! ```
//!
//! This sample code prints the standard menu like above:
//!
//! ```text
//! Hello there!
//! * Give your name: Ahmad
//! * Give a number: 1000
//! values provided: name=Ahmad, number=1000
//! ```
//!
//! ## Format it as you wish
//!
//! You can apply several formatting rules on a menu or on a field specifically.
//! You can edit:
//! * the chip: `* ` by default.
//! * the prefix: `: ` by default.
//! * insert a new line before prefix and user input: `false` by default.
//! * display default values or not: `true` by default.
//! These parameters are defined in the [`ValueFieldFormatting`] struct.
//!
//! ### Example
//!
//! For a custom format on a field and a main formatting rule on a menu, you can build this with:
//! ```rust
//! use ezmenulib::{ValueField, ValueFieldFormatting};
//! fn main() {
//!     let mut license = ValueMenu::from([
//!         ValueField::from("Authors"),
//!         ValueField::from("Project name")
//!             .fmt(ValueFieldFormatting {
//!                 chip: "--> ",
//!                 ..Default::default()
//!             }),
//!         ValueField::from("Date"),
//!     ])
//!     .fmt(ValueFieldFormatting {
//!         chip: "==> ",
//!         ..Default::default()
//!     });
//!
//!     // ...
//! }
//! ```
//!
//! The custom `==> ` chip will be applied on every field except those with custom formatting rules,
//! In this case, it will format the text like above:
//!
//! ```text
//! ==> Authors: ...
//! --> Project name: ...
//! ==> Date: ...
//! ```
//!
//! ## Skip fields with default values
//!
//! You can provide a default input value to a field with the `default` method:
//! ```rust
//! ValueField::from("Date").default("2022")
//! ```
//!
//! If the user provided an incorrect input, the program will not re-ask a value to the user,
//! but will directly return the default value instead.
//!
//! By default, the default value is visible. If you want to hide it, you can do so
//! with formatting rules:
//! ```rust
//! ValueField::from("...")
//!     .fmt(ValueFieldFormatting {
//!         default: false,
//!         ..Default::default()
//!     })
//! ```
//!
//! ## Use custom value types
//!
//! If the user has to provide a value which corresponds to your specific type,
//! you only need to implement the `FromStr` trait on that type.
//! The error type only needs to implement `Debug` trait, for error displaying purposes.
//!
//! If the error is infallible, you can use simple data types such as unit `()`
//! or `std::convert::Infallible`.
//!
//! ### Example
//!
//! ```rust
//! use std::str::FromStr;
//! use ezmenulib::ValueField;
//!
//! enum Type {
//!     MIT,
//!     BSD,
//!     GPL,
//! }
//!
//! impl FromStr for Type {
//!     type Err = String;
//!     fn from_str(s: &str) -> Result<Self, Self::Err> {
//!         Ok(Self::MIT)
//!     }
//! }
//!
//! fn main() {
//!     let license_type: Type = ValueField::from("Give the license type")
//!         .init_build()
//!         .unwrap();
//! }
//! ```
//!
//! ## Provided custom value types
//!
//! The EZMenu library already provides custom value types to handle user input.
//! Check out the [`customs`]
//! module to see all available custom value types.
//!
//! For instance, the [`MenuBool`]
//! is used to override the boolean parsing method, allowing "yes" or "no" as inputs.
//!
//! The [`MenuVec<T>`] type allows the user
//! to enter many values separated by spaces and collect them into a `Vec<T>`.
//! Of course, `T` must implement `FromStr` trait.
#![deny(missing_docs)]

/// The module defining the provided custom value types to handle user input.
pub mod customs;
mod field;
mod menu;

pub use customs::{MenuBool, MenuVec};
pub use field::{ValueField, ValueFieldFormatting};
pub use menu::{Menu, ValueMenu};

use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::{fmt, io};

/// The error type used by the menu builder.
pub enum MenuError {
    /// An IO error, when flushing, reading or writing values.
    IOError(io::Error),
    /// An incorrect type of value has been used as default value.
    IncorrectType(Box<dyn Debug>),
    /// There is no more field to call for an output.
    ///
    /// This error appears when calling `<ValueMenu as Menu>::next_output` method whereas
    /// the menu building has finished.
    NoMoreField,
    /// A custom error type.
    /// You can define this type when mapping the output value of the `Menu::next_map` method,
    /// by returning an `Err(MenuError::Other(...))`
    Other(Box<dyn Debug>),
}

impl Error for MenuError {}

impl Display for MenuError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Debug::fmt(self, f)
    }
}

impl fmt::Debug for MenuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}",
            match self {
                Self::IOError(e) => format!("IO error: {}", e),
                Self::IncorrectType(e) => format!(
                    "an incorrect value type has been used as default value: {:?}",
                    *e
                ),
                Self::NoMoreField =>
                    "attempted to get the next output while there is no more field in the menu"
                        .to_owned(),
                Self::Other(e) => format!("an error occurred: {:?}", e),
            }
        ))
    }
}

impl From<io::Error> for MenuError {
    fn from(e: io::Error) -> Self {
        Self::IOError(e)
    }
}

/// The main result type used in the EZMenu library.
pub type MenuResult<T> = Result<T, MenuError>;

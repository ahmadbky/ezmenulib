//! Fast designing menus for your Rust CLI programs.
//!
//! > Caution: This library is not completely stable yet.
//! Many changes may occur depending on versions. I am still looking for a sustainable design
//! of the library.
//!
//! This crate provides a library with structs and traits to easily build menus.
//! It includes type-checking from the user input, and a formatting customization.
//!
//! This crate is useful if you use [structopt](https://docs.rs/structopt/)
//! or [clap](https://docs.rs/clap/) crates beside it, so you can get the matches safely, and
//! build a menu on your own after.
//!
//! It can also be used as a mode selection, for a game for example.
//!
//! ## Note
//!
//! If you want to use the derive Menu macro,
//! you must use the [ezmenu](https://docs.rs/ezmenu/) crate instead.
//! This crate may however contain features that are not available on the ezmenu crate.
//!
//! # Value-menus
//!
//! The first type of menu this library provides is a [value-menu](crate::menu::ValueMenu).
//! These menus are used to retrieve data values from the user by iterating on the next outputs.
//! At each iteration, it prompts the user a value, parses it and prompts until it is correct,
//! then returns it.
//!
//! ## Example
//!
//! Here is an example of how to use this menu:
//!
//! ```
//! use ezmenulib::prelude::*;
//!
//! let mut my_menu = ValueMenu::from([
//!     Field::Value(ValueField::from("Give your name")),
//!     Field::Value(ValueField::from("Give a number")),
//! ])
//! .title("Hello there!");
//!
//! let name: String = my_menu.next_output().unwrap();
//! let number: i32 = my_menu.next_output().unwrap();
//!
//! println!("values provided: name={}, number={}", name, number);
//! ```
//!
//! This sample code prints the standard menu like above:
//!
//! ```text
//! Hello there!
//! --> Give your name
//! >> Ahmad
//!
//! --> Give a number
//! >> 1000
//!
//! values provided: name=Ahmad, number=1000
//! ```
//!
//! ## Format it as you wish
//!
//! You can apply several formatting rules on a menu or on a field specifically.
//! You can edit:
//! * the chip: `"--> "` by default.
//! * the prefix: `">> "` by default.
//! * insert a new line before prefix and user input: `true` by default.
//! * display default values or not: `true` by default.
//! These parameters are defined in the [`ValueFieldFormatting`](crate::field::ValueFieldFormatting) struct.
//!
//! ### Example
//!
//! For a custom format on a field and a main formatting rule on a menu, you can build this with:
//! ```rust
//! use ezmenulib::prelude::*;
//!
//! let mut license = ValueMenu::from([
//!     Field::Value(ValueField::from("Authors")),
//!     Field::Value(ValueField::from("Project name")
//!         .fmt(ValueFieldFormatting::chip("--- "))),
//!     Field::Value(ValueField::from("Date")),
//! ])
//! .fmt(ValueFieldFormatting::chip("==> "));
//!
//! // ...
//! ```
//!
//! The custom `"==> "` chip will be applied on every field except those with custom formatting rules,
//! In this case, it will format the text like above:
//!
//! ```text
//! ==> Authors
//! >> ...
//!
//! --- Project name
//! >> ...
//!
//! ==> Date
//! >> ...
//! ```
//!
//! ## Skip fields with default values
//!
//! You can provide a default input value to a field with the `ValueField::default` method:
//! ```rust
//! # use ezmenulib::field::ValueField;
//! # fn get_field() -> ValueField<'static> {
//! ValueField::from("Date").default_value("2022")
//! # }
//! ```
//!
//! If the user provided an incorrect input, the program will not re-ask a value to the user,
//! but will directly return the default value instead.
//!
//! By default, the default value is visible by the user, like above:
//!
//! ```text
//! --> Date (default: 2022)
//! ```
//!
//! If you want to hide it, you can do so
//! with formatting rules:
//!
//! ```rust
//! # use ezmenulib::prelude::*;
//! # fn get_field() -> ValueField<'static> {
//! ValueField::from("Date")
//!     .fmt(ValueFieldFormatting::default(false))
//! # }
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
//! use ezmenulib::field::ValueField;
//!
//! enum Type {
//!     MIT,
//!     BSD,
//!     GPL,
//! }
//!
//! impl FromStr for Type {
//!     type Err = String;
//!
//!     fn from_str(s: &str) -> Result<Self, Self::Err> {
//!         match s.to_lowercase().as_str() {
//!             "mit" => Ok(Self::MIT),
//!             "gpl" => Ok(Self::GPL),
//!             "bsd" => Ok(Self::BSD),
//!             s => Err(format!("unknown license type: {}", s)),
//!         }
//!     }
//! }
//!
//! let license_type: Type = ValueField::from("Give the license type")
//!     .build_init()
//!     .unwrap();
//! ```
//!
//! ## Provided custom value types
//!
//! The EZMenu library already provides custom value types to handle user input.
//! Check out the [`customs`]
//! module to see all available custom value types.
//!
//! For instance, the [`MenuBool`](crate::customs::MenuBool)
//! is used to override the boolean parsing method, allowing "yes" or "no" as inputs.
//!
//! The [`MenuVec<T>`](crate::customs::MenuVec) type allows the user
//! to enter many values separated by spaces and collect them into a `Vec<T>`.
//! Of course, `T` must implement `FromStr` trait.
//!
//! # Selectable menus
//!
//! Beside the value-menus, there is also the [selectable menus](crate::menu::SelectMenu).
//! These menus, unlike value-menus, displays the list of possible values to the user,
//! to let him select one among them.
//!
//! ## Example
//!
//! ```
//! use std::str::FromStr;
//! use ezmenulib::prelude::*;
//!
//! enum Type {
//!     MIT,
//!     GPL,
//!     BSD,
//! }
//!
//! impl FromStr for Type {
//!     type Err = MenuError;
//!
//!     fn from_str(s: &str) -> MenuResult<Self> {
//!         match s.to_lowercase().as_str() {
//!             "mit" => Ok(Self::MIT),
//!             "gpl" => Ok(Self::GPL),
//!             "bsd" => Ok(Self::BSD),
//!             s => Err(MenuError::from(format!("unknown license type: {}", s))),
//!         }
//!     }
//! }
//!
//! let license_type: Type = SelectMenu::from([
//!     SelectField::from("MIT"),
//!     SelectField::from("GPL"),
//!     SelectField::from("BSD"),
//! ])
//! .title(SelectTitle::from("Choose a license type"))
//! .default(0)
//! .next_output()
//! .unwrap();
//! ```
//!
//! This code prints the output like above:
//! ```text
//! Choose a license type:
//! 1 - MIT (default)
//! 2 - GPL
//! 3 - BSD
//! ```
//! > Note that the `:` character right next to the title is on purpose
//! (check the [`SelectTitle`](crate::menu::SelectTitle) for more information).
//!
//! You can also use this menu on primitive types or types already implementing `FromStr` trait.
//! The menu accepts an index or the literal value as input.
//!
//! ## Formatting rules
//!
//! Like the [`ValueMenu`](crate::menu::ValueMenu), you can edit many formatting rules
//! to stylish the menu as you want.
//!
//! ### The menu format
//!
//! The selective menu itself has two editable formatting rules.
//! Like [`ValueFieldFormatting`](crate::field::ValueFieldFormatting), it contains a
//! `chip` and a `prefix`:
//! ```text
//! X<chip><message>
//! X<chip><message>
//! ...
//! <prefix>
//! ```
//!
//! The default chip is `" - "`, and the default prefix is `">> "`.
//!
//! ### The title format
//!
//! The selective has also its own title format.
//! Because the title can be seen as a field of a value-menu, it has its own instance of
//! `ValueFieldFormatting` struct.
//!
//! This is useful for sub-menu management, where the formatting rules of the title inherits from the
//! formatting rules of the parent menu, for more convenience.
//!
//! ## Skip the menu with a default field value
//!
//! The user can skip the selectable menu if it has a default value provided.
//! To do so, you must use the [`SelectMenu::default`](crate::menu::SelectMenu::default) method.
//!
//! This will mark the indexed field as `"(default)"`.
//!
//! ## Sub-menus
//!
//! You can set a selectable menu as a field of a value-menu.
//! This is really useful if you want to design sub-menu. The selectable field format
//! will inherit from the formatting rules of the value-menu.
//!
//! ### Example
//!
//! ```
//! use ezmenulib::prelude::*;
//!
//! let mut license = ValueMenu::from([
//!     Field::Value(ValueField::from("Authors")),
//!     Field::Value(ValueField::from("Project name")),
//!     Field::Select(SelectMenu::from([
//!         SelectField::from("MIT"),
//!         SelectField::from("GPL"),
//!         SelectField::from("BSD"),
//!     ])
//!     .title(SelectTitle::from("License type"))
//!     .default(0)),
//! ])
//! .title("Describe the project license");
//! ```

#![warn(missing_docs, missing_copy_implementations, unused_allocation)]

pub mod customs;
pub mod field;
pub mod menu;

/// Module used to import common structs, to build menus with their fields.
pub mod prelude {
    pub use crate::field::*;
    pub use crate::menu::*;

    pub use crate::MenuError;
    pub use crate::MenuResult;
}

use std::env::VarError;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use std::{fmt, io};

/// The error type used by the menu builder.
#[non_exhaustive]
pub enum MenuError {
    /// An IO error, when flushing, reading or writing values.
    IOError(io::Error),
    /// A parsing error for a value.
    Parse(String, Box<dyn Debug>),
    /// An environment variable error.
    EnvVar(String, VarError),
    /// An incorrect selection input has been provided.
    Select(String),
    /// A custom error type.
    /// You can define this type when mapping the output value of the `Menu::next_map` method,
    /// by returning an `Err(MenuError::Other(...))`
    Other(Box<dyn Debug>),
}

impl Error for MenuError {}

impl Debug for MenuError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for MenuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}",
            match self {
                Self::IOError(e) => format!("IO error: {}", e),
                Self::Parse(v, e) =>
                    format!("the input value provided `{}` is incorrect: {:?}", v, e),
                Self::EnvVar(v, e) => format!(
                    "attempted to get a default value from the environment variable `{}`: {}",
                    v, e
                ),
                Self::Select(s) => format!("incorrect selection input: `{}`", s),
                Self::Other(e) => format!("an error occurred: {:?}", e),
            }
        ))
    }
}

impl From<io::Error> for MenuError {
    #[inline]
    fn from(e: io::Error) -> Self {
        Self::IOError(e)
    }
}

impl From<&'static str> for MenuError {
    #[inline]
    fn from(s: &'static str) -> Self {
        Self::Other(Box::new(s))
    }
}

impl From<String> for MenuError {
    #[inline]
    fn from(s: String) -> Self {
        Self::Other(Box::new(s))
    }
}

/// The main result type used in the EZMenu library.
pub type MenuResult<T> = Result<T, MenuError>;

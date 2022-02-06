//! Fast designing menus for your Rust CLI programs.
//!
//! This crate provides a `derive(Menu)` procedural macro to easily build menus.
//! It includes type-checking from the user input, and a formatting customization.
//!
//! ## Example
//!
//! Here is an example of how to use it:
//! ```rust
//! use ezmenu::Menu;
//!
//! fn check(n: &i32) {
//!     if *n == 0 {
//!         println!("yo respect me plz :'(");
//!     } else {
//!         println!("good. :)");
//!     }
//! }
//!
//! #[derive(Menu)]
//! struct MyMenu {
//!     author: String,
//!     #[field(msg = "Give a nonzero number", then(check))]
//!     number: u32,
//! }
//! ```
//!
//! To display the menu, you instantiate the struct by calling its `from_menu` method:
//! ```rust
//! let MyMenu { author, number } = MyMenu::from_menu();
//! println!("values provided: author={}, number={}", author, number);
//! ```
//!
//! This sample code prints the standard menu like above:
//! ```md
//! - author: Ahmad
//! - Give a nonzero number: 0
//! yo respect me plz :'(
//! values provided: author=Ahmad, number=0
//! ```

mod field;
mod menu;

/// Menu derive macro
pub use ezmenu_derive::Menu;
use std::error::Error;

pub use field::{StructField, StructFieldFormatting};
pub use menu::{Menu, StructMenu};

use std::fmt::Debug;
use std::io;

/// The error type used by the menu builder.
#[derive(Debug)]
pub enum MenuError {
    /// An IO error, when flushing, reading or writing values,
    IOError(io::Error),
    /// An incorrect type has been used as default.
    IncorrectType(Box<dyn Debug>),
    /// There is no more field to call for an output.
    /// This error appears when calling `<StructMenu as Menu>::next` method whereas
    /// the menu building has finished.
    NoMoreField,
    /// A custom error type.
    /// You can define this type when mapping the output value of the `Menu::next_map` method,
    /// by returning an `Err(MenuError::Custom(...))`
    Custom(Box<dyn Error>),
}

impl From<io::Error> for MenuError {
    fn from(e: io::Error) -> Self {
        Self::IOError(e)
    }
}

/// The main result type used in the EZMenu library.
pub type MenuResult<T> = Result<T, MenuError>;

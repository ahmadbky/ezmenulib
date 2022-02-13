//! Fast designing menus for your Rust CLI programs.
//!
//! This crate provides a `derive(Menu)` procedural macro to easily build menus.
//! It includes type-checking from the user input, and a formatting customization.
//!
//! ## Example
//!
//! Here is an example of how to use it:
//! ```rust
//! use ezmenu::{Menu, MenuResult};
//! use std::io::Write;
//!
//! fn map(val: u32, w: &mut impl Write) -> MenuResult<u32> {
//!     if val == 1000 {
//!         w.write(b"ok!\n")?;
//!     }
//!     Ok(val)
//! }
//!
//! #[derive(Menu)]
//! struct MyMenu {
//!     author: String,
//!     #[menu(msg = "Give a nonzero number", then(map))]
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
//! ```md
//! - author: Ahmad
//! - Give a nonzero number: 0
//! yo respect me plz :'(
//! values provided: author=Ahmad, number=0
//! ```
//!
//! ## Custom I/O types
//!
//! If you are not using `std::io::Stdin` or `std::io::Stdout` types, you can provide
//! your types by enabling `custom_io` feature in your Cargo.toml file:
//!
//! ```toml
//! [dependencies]
//! ezmenu = { version = "0.2.0", features = ["custom_io"] }
//! ```
//!
//! Then you can instantiate your struct with:
//!
//! ```
//! use std::io::stdout;
//! let input = b"Ahmad\n1000\n" as &[u8];
//! let MyMenu { author, number } = MyMenu::from_io(input, stdout());
//! ```

mod field;
mod menu;

/// Menu derive macro
pub use ezmenu_derive::Menu;

pub use field::{StructField, StructFieldFormatting};
pub use menu::{Menu, StructMenu};

use std::fmt::Debug;
use std::io;

/// The error type used by the menu builder.
#[derive(Debug)]
pub enum MenuError {
    /// An IO error, when flushing, reading or writing values,
    IOError(io::Error),
    /// An incorrect type of value has been used as default value.
    IncorrectType(Box<dyn Debug>),
    /// There is no more field to call for an output.
    /// This error appears when calling `<StructMenu as Menu>::next` method whereas
    /// the menu building has finished.
    NoMoreField,
    /// A custom error type.
    /// You can define this type when mapping the output value of the `Menu::next_map` method,
    /// by returning an `Err(MenuError::Custom(...))`
    Custom(Box<dyn Debug>),
}

impl From<io::Error> for MenuError {
    fn from(e: io::Error) -> Self {
        Self::IOError(e)
    }
}

/// The main result type used in the EZMenu library.
pub type MenuResult<T> = Result<T, MenuError>;

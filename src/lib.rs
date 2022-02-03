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

pub use field::Field;

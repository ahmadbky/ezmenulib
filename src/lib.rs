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

pub use ezmenu_derive::*;
use std::io::{BufRead, Stdin, Stdout, Write};
use std::str::FromStr;

/// Prompts an input and returns the string value
fn prompt<R, W>(reader: &mut R, writer: &mut W, msg: &str) -> Result<String, std::io::Error>
where
    R: BufRead,
    W: Write,
{
    write!(writer, "- {}: ", msg)?;
    // flushes writer so it prints the prefix
    writer.flush()?;

    // read the user input
    let mut out = String::new();
    reader.read_line(&mut out)?;

    Ok(out)
}

/// Asks the user a value, then returns it.
/// It prints the text according to the given parameters formatting.
pub fn ask<T, R, W, F>(reader: &mut R, writer: &mut W, msg: &str, then: F) -> T
where
    T: FromStr,
    R: BufRead,
    W: Write,
    F: FnOnce(&T),
{
    //loops while incorrect input
    loop {
        let out = match prompt(reader, writer, msg) {
            Ok(s) => s,
            Err(e) => panic!("An error occurred while prompting input: {:?}", e),
        };

        // user input type checking
        match out.trim().parse::<T>() {
            Ok(t) => {
                then(&t);
                break t;
            }
            _ => continue,
        }
    }
}

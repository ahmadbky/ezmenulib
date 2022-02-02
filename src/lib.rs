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

#[cfg(test)]
mod field_tests;

pub use ezmenu_derive::*;
use std::fmt::Debug;
use std::io::{BufRead, Write};
use std::str::FromStr;

struct MainFormatting<'a> {
    title: Option<&'a str>,
    pos: TitlePos,
    prefix: &'a str,
    new_line: bool,
    defaults: bool,
}

impl Default for MainFormatting<'_> {
    fn default() -> Self {
        Self {
            title: None,
            pos: Default::default(),
            prefix: ": ",
            new_line: false,
            defaults: true,
        }
    }
}

#[derive(Clone)]
struct FieldFormatting<'a> {
    chip: Option<&'a str>,
    prefix: Option<&'a str>,
    new_line: bool,
    default: bool,
}

impl<'a> Default for FieldFormatting<'a> {
    fn default() -> Self {
        Self {
            chip: Some("* "),
            prefix: Some(": "),
            new_line: false,
            default: true,
        }
    }
}

pub struct Field<'a, T> {
    msg: &'a str,
    fmt: FieldFormatting<'a>,
    default: Option<&'a str>,
    then: Option<Box<dyn FnOnce(&T, &mut dyn Write)>>,
}

impl<'a, T> From<&'a str> for Field<'a, T> {
    fn from(msg: &'a str) -> Self {
        Self {
            msg,
            fmt: Default::default(),
            default: None,
            then: None,
        }
    }
}

/// Constructor methods
impl<'a, T> Field<'a, T> {
    /// Define the chip at the left of the message displayed
    /// (default: "* ").
    pub fn chip(mut self, chip: Option<&'a str>) -> Self {
        self.fmt.chip = chip;
        self
    }

    /// Define the prefix for the prompt (default: ": ").
    pub fn prefix(mut self, prefix: Option<&'a str>) -> Self {
        self.fmt.prefix = prefix;
        self
    }

    /// Set the prefix and user input on a new line (false by default).
    pub fn new_line(mut self, new_line: bool) -> Self {
        self.fmt.new_line = new_line;
        self
    }

    /// Displays default values or not (true by default).
    pub fn display_default(mut self, default: bool) -> Self {
        self.fmt.default = default;
        self
    }

    /// Sets the default value as a &str.
    pub fn default_value(mut self, default: &'a str) -> Self {
        self.default = Some(default);
        self
    }

    pub fn then<F: 'static + FnOnce(&T, &mut dyn Write)>(mut self, then: F) -> Self {
        self.then = Some(Box::new(then));
        self
    }
}

impl<'a, T> Field<'a, T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    /// Builds the field.
    /// It prints the message according to its formatting, then returns the corresponding value.
    pub fn build<R, W>(self, reader: &mut R, writer: &mut W) -> T
    where
        R: BufRead,
        W: Write,
    {
        /// Function that parses the default value without checking.
        /// It it useful to break the loop if there is some default value,
        /// and no value was provided, or if the value is incorrect.
        #[inline]
        fn unchecked_default_parse<T>(default: Option<&str>) -> T
        where
            T: FromStr,
            <T as FromStr>::Err: Debug,
        {
            let default = default.unwrap();
            default
                .parse()
                .unwrap_or_else(|e| panic!("Invalid default value `{}`. Error: {:?}", default, e))
        }

        // loops while incorrect value
        loop {
            prompt_fmt(writer, self.msg, self.default, &self.fmt)
                .unwrap_or_else(|e| panic!("An error occurred while prompting a value: {:?}", e));

            // read input to string
            let mut out = String::new();
            reader.read_line(&mut out).expect("Unable to read line");

            // if testing, append input to output
            if cfg!(test) {
                writeln!(writer, "{}", out)
                    .expect("An error occurred while appending input to output for testing");
            }

            let out = out.trim();

            if out.is_empty() && self.default.is_some() {
                let out = unchecked_default_parse(self.default);
                if let Some(func) = self.then {
                    func(&out, writer);
                }
                break out;
            }

            // try to parse to T, else repeat
            match out.parse() {
                Ok(t) => {
                    if let Some(func) = self.then {
                        func(&t, writer);
                    }
                    break t;
                }
                // TODO: feature allowing control over default values behavior
                Err(_) if self.default.is_some() => {
                    let out = unchecked_default_parse(self.default);
                    if let Some(func) = self.then {
                        func(&out, writer);
                    }
                    break out;
                }
                _ => continue,
            }
        }
    }
}

pub enum TitlePos {
    Top,
    Bottom,
}

impl Default for TitlePos {
    fn default() -> Self {
        Self::Top
    }
}

/// Prompts an input according to the formatting options of the field
fn prompt_fmt<W: Write>(
    writer: &mut W,
    msg: &str,
    default: Option<&str>,
    fmt: &FieldFormatting<'_>,
) -> Result<(), std::io::Error> {
    write!(
        writer,
        "{chip}{msg}{def}{nl}{prefix}",
        chip = fmt.chip.unwrap_or(""),
        msg = msg,
        def = if fmt.default {
            format!(" (default: {})", default.unwrap_or("none"))
        } else {
            "".to_owned()
        },
        nl = if fmt.new_line { "\n" } else { "" },
        prefix = fmt.prefix.unwrap_or("")
    )?;
    // flushes writer so it prints the message if it's on the same line
    writer.flush()
}

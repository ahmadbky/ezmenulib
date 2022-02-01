//! This crate provides several tools to build a CLI menu easily.
//! It also defines its Menu derive.

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

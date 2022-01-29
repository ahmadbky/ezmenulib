pub use menu_derive::*;
use std::fmt::Debug;
use std::io::{Stdin, Stdout, Write};
use std::str::FromStr;

/// Function that asks the user a value, then returns it.
/// It prints the text according to the parameters formatting.
pub fn ask<T: FromStr>(stdin: &Stdin, stdout: &mut Stdout, msg: &str) -> T {
    //loops while incorrect input
    loop {
        print!("- {}: ", msg);
        // flushes stdout so it prints the prefix
        stdout.flush().expect("Unable to flush stdout");

        // read the user input
        let mut out = String::new();
        stdin
            .read_line(&mut out)
            .expect("Unable to read line from stdin");

        // user input type checking
        match out.trim().parse::<T>() {
            Ok(t) => break t,
            Err(_) => continue,
        }
    }
}

/// A trait used to construct a menu from a struct fields
pub trait Menu {
    /// Constructs the struct menu from its fields
    fn from_fields() -> Self;
}

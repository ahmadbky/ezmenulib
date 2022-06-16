//! This module contains many utils functions used by the library.

use crate::prelude::*;

use std::any::type_name;
use std::fmt::Display;
use std::io::BufRead;
use std::io::Write;

/// Type to handle the depth of the running menus.
pub(crate) enum Depth {
    /// We go back to `n` level.
    Back(usize),
    /// We stay at the current page.
    Current,
    /// We quit all the nested pages to the top.
    Quit,
}

/// Function used by the fields associated functions.
///
/// It is useful because it always returns true no matter the output value,
/// "keeping" the value in the context of the associated functions
/// (see [`Written::prompt`] or [`Written::many_values`] methods).
pub(crate) fn keep<T>(_val: &T) -> bool {
    true
}

/// Shows the text using the given stream and maps the `io::Error` into a `MenuError`.
pub(crate) fn show<T: ?Sized + Display, S: Write>(text: &T, stream: &mut S) -> MenuResult {
    write!(stream, "{}", text)?;
    stream.flush().map_err(MenuError::from)
}

/// Shows the text using the given stream, then prompts a value to the user and
/// returns the corresponding String.
pub(crate) fn prompt<T: ?Sized + Display, R: BufRead, W: Write>(
    text: &T,
    stream: &mut MenuStream<R, W>,
) -> MenuResult<String> {
    show(text, stream)?;
    read_input(stream)
}

/// Panics at runtime, emphasizing that the given `default` value is incorrect for `T` type.
pub(crate) fn default_failed<T>(default: &str) -> ! {
    panic!(
        "`{}` has been used as default value but is incorrect for `{}` type",
        default,
        type_name::<T>(),
    )
}

/// Returns the input value as a String from the given input stream.
pub(crate) fn read_input<R: BufRead, W>(stream: &mut MenuStream<R, W>) -> MenuResult<String> {
    let mut out = String::new();
    stream.read_line(&mut out)?;
    Ok(out.trim().to_owned())
}

/// Prompts the user to enter an index to select a value among the available values.
///
/// The available values are in theory printed before calling this function.
pub(crate) fn select<R: BufRead, W: Write>(
    stream: &mut MenuStream<R, W>,
    suffix: &str,
    max: usize,
) -> MenuResult<Option<usize>> {
    let s = prompt(suffix, stream)?;
    Ok(match s.parse::<usize>() {
        Ok(i) if i >= 1 && i <= max => Some(i - 1),
        _ => None,
    })
}

/// Checks that the menu fields are not empty at runtime.
pub(crate) fn check_fields<T>(fields: &[T]) {
    if fields.is_empty() {
        panic!("empty fields for the selectable values");
    }
}

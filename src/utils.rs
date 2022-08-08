//! This module contains many utils functions used by the library.

use crate::{menu::Handle, prelude::*};
use std::any::type_name;

/// Type to handle the depth of the running menus.
pub(crate) enum Depth {
    /// We go back to `n` level.
    Back(usize),
    /// We stay at the current page.
    Current,
    /// We quit all the nested pages to the top.
    Quit,
}

/// Panics at runtime, emphasizing that the given `default` value is incorrect for `T` type.
pub(crate) fn default_failed<T>(default: &str) -> ! {
    panic!(
        "`{}` has been used as default value but is incorrect for `{}` type",
        default,
        type_name::<T>(),
    )
}

/// Prompts the user to enter an index to select a value among the available values.
///
/// The available values are in theory printed before calling this function.
pub(crate) fn select<H: Handle>(
    mut handle: H,
    suffix: &str,
    max: usize,
) -> MenuResult<Option<usize>> {
    handle.write_all(suffix.as_bytes())?;
    handle.flush()?;
    let s = handle.read_input()?;
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

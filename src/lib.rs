//! Fast designing menus for your Rust CLI programs.
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
//! This crate may however contain features that are not yet available on the ezmenu crate.

#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    unreachable_pub,
    unused_lifetimes,
    future_incompatible
)]
#![cfg_attr(nightly, feature(doc_cfg))]

#[cfg(feature = "tui")]
#[cfg_attr(nightly, doc(cfg(feature = "tui")))]
pub mod tui;

#[cfg(feature = "derive")]
#[doc(hidden)]
pub use ezmenu_macros::Menu;

#[cfg(feature = "derive")]
#[doc(hidden)]
pub use ezmenu_macros::Prompted;

#[cfg(feature = "derive")]
#[cfg_attr(nightly, doc(cfg(feature = "derive")))]
pub use ezmenu_macros::bound;

pub mod customs;
pub mod field;
pub mod menu;

pub(crate) mod utils;

/// Module used to import common structs, to build menus with their fields.
pub mod prelude {
    pub use super::*;
    #[doc(inline)]
    pub use crate::{field::*, menu::*};
}

use crate::field::Format;
use std::env::VarError;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::io::{self, BufRead, Read, Write};
use std::ops::{Deref, DerefMut};

pub(crate) const DEFAULT_FMT: Format<'static> = Format {
    prefix: "--> ",
    left_sur: "[",
    right_sur: "]",
    chip: " - ",
    show_default: true,
    suffix: ">> ",
    line_brk: true,
};

/// The error type used by the menu builder.
#[non_exhaustive]
pub enum MenuError {
    /// An IO error, when flushing, reading or writing values.
    IOError(io::Error),
    /// A parsing error for a value.
    Input,
    /// An environment variable error.
    EnvVar(String, VarError),
    /// An error occurred when formatting a field.
    Format(fmt::Error),
    /// A custom error.
    Other(Box<dyn Debug>),
}

#[cfg(test)]
impl PartialEq for MenuError {
    fn eq(&self, other: &Self) -> bool {
        // We are simply checking that the variants are the same.
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
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
                Self::Input => "an incorrect input has been provided".to_owned(),
                Self::EnvVar(v, e) => format!(
                    "attempted to get a default value from the environment variable `{}`: {}",
                    v, e
                ),
                Self::Format(e) => format!("an error occurred while formatting a field: {:?}", e),
                Self::Other(d) => format!("{:?}", d),
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

impl From<fmt::Error> for MenuError {
    #[inline]
    fn from(e: fmt::Error) -> Self {
        Self::Format(e)
    }
}

/// The main result type used in the EZMenu library.
pub type MenuResult<T = ()> = Result<T, MenuError>;

pub trait IntoResult {
    fn into_result(self) -> MenuResult;
}

impl<E: Into<MenuError>> IntoResult for Result<(), E> {
    fn into_result(self) -> MenuResult {
        self.map_err(E::into)
    }
}

impl IntoResult for () {
    fn into_result(self) -> MenuResult {
        Ok(())
    }
}

//! The module containing the default custom types implementing [`FromStr`](std::str::FromStr) trait.
//!
//! When retrieving values from the user inputs, we need to accept more human values to parse.
//! For instance, the default implementation of the `FromStr` trait for the `bool` primitive type
//! requires the string slice to be either exactly `true` or `false`. Therefore, in this module,
//! there exists the [`MenuBool`] type overriding this implementation, to accept more human values,
//! such as `"yes"` or `"no"`.
//!
//! ## Example
//!
//! ```no_run
//! use ezmenulib::{prelude::*, customs::*};
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let mut project = Values::default();
//! let is_sure: MenuBool = project.next(Written::from("Are you sure?"))?;
//! # Ok(()) }
//! ```

#[cfg(test)]
mod tests;

use crate::MenuError;
use std::{
    fmt::{Debug, Display, Formatter, Result as FmtResult},
    ops::{Deref, DerefMut},
    str::FromStr,
};

macro_rules! impl_inner {
    ($name:ident$(<$($generic:ident),*>)?: $ty:ty$(, $meta:meta)*) => {
        $(#[$meta])*
        impl$(<$($generic),*>)? AsRef<$ty> for $name$(<$($generic),*>)? {
            #[inline]
            fn as_ref(&self) -> &$ty {
                &self.0
            }
        }

        $(#[$meta])*
        impl$(<$($generic),*>)? AsMut<$ty> for $name$(<$($generic),*>)?  {
            #[inline]
            fn as_mut(&mut self) -> &mut $ty {
                &mut self.0
            }
        }

        $(#[$meta])*
        impl$(<$($generic),*>)? Deref for $name$(<$($generic),*>)?  {
            type Target = $ty;

            #[inline]
            fn deref(&self) -> &Self::Target {
                self.as_ref()
            }
        }

        $(#[$meta])*
        impl$(<$($generic),*>)? DerefMut for $name$(<$($generic),*>)?  {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.as_mut()
            }
        }

        $(#[$meta])*
        impl$(<$($generic),*>)? From<$name$(<$($generic),*>)? > for $ty {
            #[inline]
            fn from(t: $name$(<$($generic),*>)?) -> Self {
                t.0
            }
        }
    }
}

/// Wrapper type used to handle a boolean user input value.
///
/// Its main feature is to implemented `FromStr` trait,
/// accepting "yes" or "no" input for example.
///
/// You can still access the bool inner value with
/// `&x.0`, or `*x`, which is same as `x.as_ref()`.
#[derive(Clone, Copy, Eq, PartialEq, Debug, Default)]
pub struct MenuBool(pub bool);

impl Display for MenuBool {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl_inner!(MenuBool: bool);

impl FromStr for MenuBool {
    type Err = MenuError;

    /// Parses the string slice to a boolean accepting more human values,
    /// than only `"true"` or `"false"`, like `"yes"` or `"no"`..
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "y" | "yes" | "ye" | "yep" | "yeah" | "yea" | "yup" | "true" | "1" => Ok(Self(true)),
            "n" | "no" | "non" | "nop" | "nah" | "nan" | "nani" | "false" | "0" => Ok(Self(false)),
            _ => Err(MenuError::Input),
        }
    }
}

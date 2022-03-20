//! The module containing the default custom types implementing [`FromStr`](std::str::FromStr) trait.
//!
//! When retrieving values from the user inputs, we need to accept more human values to parse.
//! For instance, the default implementation of the `FromStr` trait for the `bool` primitive type
//! requires the string slice to be either `true` or `false`. Therefore, in this module, there is
//! the [`MenuBool`] type overriding this implementation, to accept more human values,
//! like `yes` or `no`.
//!
//! An other example is about multiple values providing. The [`Vec<T>`] struct does not implement
//! the `FromStr` trait, and this is why there is the [`MenuVec<T>`] struct for this.
//!
//! ## Example
//!
//! ```no_run
//! use ezmenulib::{prelude::*, customs::*};
//!
//! let mut project = ValueMenu::from([
//!     Field::Value(ValueField::from("Authors")),
//!     Field::Value(ValueField::from("Was it hard?")),
//! ])
//! .title("Describe a project.");
//!
//! let authors: MenuVec<String> = project.next_output().unwrap();
//! let hard: MenuBool = project.next_output().unwrap();
//! ```

use crate::MenuError;
use std::fmt::{Debug, Display, Formatter, Result as FmtResult};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

macro_rules! impl_inner {
    ($name:ident$(<$($generic:ident),*>)?: $ty:ty$(, $meta:meta)?) => {
        $(#[$meta])?
        impl$(<$($generic),*>)? AsRef<$ty> for $name$(<$($generic),*>)? {
            #[inline]
            fn as_ref(&self) -> &$ty {
                &self.0
            }
        }

        $(#[$meta])?
        impl$(<$($generic),*>)? AsMut<$ty> for $name$(<$($generic),*>)?  {
            #[inline]
            fn as_mut(&mut self) -> &mut $ty {
                &mut self.0
            }
        }

        $(#[$meta])?
        impl$(<$($generic),*>)? Deref for $name$(<$($generic),*>)?  {
            type Target = $ty;

            #[inline]
            fn deref(&self) -> &Self::Target {
                self.as_ref()
            }
        }

        $(#[$meta])?
        impl$(<$($generic),*>)? DerefMut for $name$(<$($generic),*>)?  {
            #[inline]
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.as_mut()
            }
        }

        $(#[$meta])?
        impl$(<$($generic),*>)? From<$name$(<$($generic),*>)? > for $ty {
            #[inline]
            fn from(t: $name$(<$($generic),*>)?) -> Self {
                t.0
            }
        }
    }
}

/// Wrapper type used to handle multiple user input.
///
/// Its main feature is to implement FromStr trait,
/// by splitting input by spaces.
///
/// You can access the inner value by `&x.0`, `*x`, which is same as `x.as_ref()`.
///
/// ## Example
///
/// ```
/// use ezmenulib::customs::MenuVec;
///
/// let a = "23 -54 456";
/// let a: MenuVec<i32> = a.parse().unwrap();
/// assert_eq!(*a, vec![23, -54, 456]);
/// ```
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MenuVec<T>(pub Vec<T>);

impl_inner!(MenuVec<T>: Vec<T>);

/// The error type used for parsing user input into a [`MenuVec<T>`].
///
/// The `E` generic parameter means `<T as FromStr>::Err`.
#[derive(PartialEq)]
pub enum MenuVecParseError<E> {
    /// The user input is empty.
    Empty,
    /// An incorrect input has been provided among the values.
    ItemParsed(E),
}

impl<E> From<E> for MenuVecParseError<E> {
    #[inline]
    fn from(err: E) -> Self {
        Self::ItemParsed(err)
    }
}

impl<E: Debug> Debug for MenuVecParseError<E> {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let msg = match self {
            Self::Empty => "empty vector".to_owned(),
            Self::ItemParsed(e) => format!("parsing error: {:?}", e),
        };
        f.write_str(msg.as_str())
    }
}

/// Wrapper implementation of FromStr for Output providing.
impl<T: FromStr> FromStr for MenuVec<T> {
    type Err = MenuVecParseError<T::Err>;

    /// The implementation uses space as pattern for separation of inputs.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            return Err(MenuVecParseError::Empty);
        }
        let result: Result<Vec<T>, T::Err> = s.split(' ').map(T::from_str).collect();
        Ok(Self(result?))
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
            "y" | "yes" | "ye" | "yep" | "yeah" | "yea" | "yup" | "true" => Ok(Self(true)),
            "n" | "no" | "non" | "nop" | "nah" | "nan" | "nani" | "false" => Ok(Self(false)),
            _ => Err(MenuError::Other(Box::new("incorrect boolean value"))),
        }
    }
}

/// Wrapper type used to handle an optional user input value.
///
/// It implements `FromStr` trait, returning `Some(value)` if a value is
/// indeed present in the string slice, else it returns `None`.
///
/// You can still access the `Option<T>` inner value with
/// `&x.0`, or `*x`, which is same as `x.as_ref()`.
#[derive(Debug, PartialEq, Clone, Default)]
pub struct MenuOption<T>(pub Option<T>);

impl<T: Display> Display for MenuOption<T> {
    /// Displays T if present, else nothing (`""`).
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match &self.0 {
            Some(e) => Display::fmt(e, f),
            None => f.write_str(""),
        }
    }
}

impl_inner!(MenuOption<T>: Option<T>);

impl<T: FromStr> FromStr for MenuOption<T> {
    type Err = T::Err;

    /// Returns `Some(value)` if the string contains a value, else `None`.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.trim().is_empty() {
            Ok(Self(None))
        } else {
            Ok(Self(Some(s.parse()?)))
        }
    }
}

/// Wrapper type used to handle a math expression from the user input.
///
/// It uses the [`meval`](https://docs.rs/meval/0.2.0) crate to parse the math expression, thus requires
/// to set `expr` feature in the `Cargo.toml` file.
///
/// You can access the inner value by `&x.0`, `*x`, which is same as `x.as_ref()`.
///
/// ## Example
///
/// ```
/// use ezmenulib::customs::MenuNumber;
///
/// let a = "43 + 34 - 6";
/// let a: MenuNumber = a.parse().unwrap();
/// assert_eq!(*a, 71.);
/// ```
#[derive(Clone, Copy, PartialEq, Debug, Default)]
#[cfg(feature = "expr")]
pub struct MenuNumber(pub f64);

#[cfg(feature = "expr")]
impl Display for MenuNumber {
    #[inline]
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        Display::fmt(&self.0, f)
    }
}

impl_inner!(MenuNumber: f64, cfg(feature = "expr"));

#[cfg(feature = "expr")]
impl FromStr for MenuNumber {
    type Err = meval::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self(meval::eval_str(s)?))
    }
}

use crate::MenuError;
use std::fmt;
use std::fmt::Formatter;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

/// Wrapper type used to handle multiple user input.
/// Its main feature is to implement FromStr trait,
/// by splitting input by spaces.
///
/// You can access the inner value by `&x.0`, `*x`, which is same as `x.as_ref()`.
///
/// ## Example
///
/// ```
/// use ezmenu::MenuVec;
/// let a = "23 -54 456";
/// let a: MenuVec<i32> = a.parse().unwrap();
/// assert_eq!(*a, vec![23, -54, 456]);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct MenuVec<T>(pub Vec<T>);

impl<T> AsRef<Vec<T>> for MenuVec<T> {
    fn as_ref(&self) -> &Vec<T> {
        &self.0
    }
}

impl<T> AsMut<Vec<T>> for MenuVec<T> {
    fn as_mut(&mut self) -> &mut Vec<T> {
        &mut self.0
    }
}

impl<T> Deref for MenuVec<T> {
    type Target = Vec<T>;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl<T> DerefMut for MenuVec<T> {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

/// Wrapper implementation of FromStr for Output providing.
impl<T: FromStr> FromStr for MenuVec<T> {
    type Err = T::Err;

    /// The implementation uses space as pattern for separation of inputs.
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result: Result<Vec<T>, T::Err> = s.split(' ').map(T::from_str).collect();
        Ok(Self(result?))
    }
}

/// Wrapper type used to handle a boolean value.
/// Its main feature is to implemented `FromStr` trait,
/// accepting "yes" or "no" input for example.
///
/// You can still access the bool inner value with
/// `&x.0`, or `*x`, which is same as `x.as_ref()`.
#[derive(Clone, Copy, Eq, PartialEq, Debug)]
pub struct MenuBool(pub bool);

impl fmt::Display for MenuBool {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<bool> for MenuBool {
    fn as_ref(&self) -> &bool {
        &self.0
    }
}

impl AsMut<bool> for MenuBool {
    fn as_mut(&mut self) -> &mut bool {
        &mut self.0
    }
}

impl Deref for MenuBool {
    type Target = bool;

    #[inline]
    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}

impl DerefMut for MenuBool {
    #[inline]
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut()
    }
}

impl FromStr for MenuBool {
    type Err = MenuError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "y" | "yes" | "true" => Ok(Self(true)),
            "n" | "no" | "false" => Ok(Self(false)),
            _ => Err(MenuError::Other(Box::new("incorrect boolean value"))),
        }
    }
}

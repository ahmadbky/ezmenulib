use std::ops::{Deref, DerefMut};
use std::str::FromStr;

/// Vector used to handle multiple user input.
/// Its main feature is to implement FromStr trait.
pub struct MenuVec<T>(Vec<T>);

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

impl<T: FromStr> FromStr for MenuVec<T> {
    type Err = T::Err;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let result: Result<Vec<T>, T::Err> = s.split(' ').map(T::from_str).collect();
        Ok(Self(result?))
    }
}

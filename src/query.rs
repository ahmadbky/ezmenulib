use crate::{MenuError, MenuResult};

/// The return type of each iteration when prompting a value to the user.
///
/// `T` represents the output type of the field or menu.
pub enum Query<T> {
    /// The user entered an incorrect value.
    Continue,
    /// The user entered a correct value, so it has been parsed to its corresponding type.
    Finished(T),
    /// An error occurred when prompting a value.
    Err(MenuError),
}

impl<T> Query<T> {
    /// Calls `op` if the result is [`Finished`](Query::Finished), otherwise returns the [`Err`]
    /// or [`Loop`] value of `self`.
    ///
    /// This function is used for control flow based on `Query` values.
    pub fn then<U, O: FnOnce(T) -> Query<U>>(self, op: O) -> Query<U> {
        match self {
            Self::Finished(t) => op(t),
            Self::Err(e) => Query::Err(e),
            _ => Query::Continue,
        }
    }
}

impl<T: Default> Query<T> {
    pub fn or_default(self) -> T {
        match self {
            Self::Finished(out) => out,
            _ => T::default(),
        }
    }
}

/// The inner Result represents the parsing result of the output type.
/// The outer Result represents the other error types.
impl<T> From<MenuResult<MenuResult<T>>> for Query<T> {
    fn from(res: MenuResult<MenuResult<T>>) -> Self {
        match res {
            Ok(Ok(out)) => Self::Finished(out),
            Ok(Err(_)) => Self::Continue,
            Err(e) => Self::Err(e),
        }
    }
}

impl<T, E> From<Result<T, E>> for Query<T>
where
    MenuError: From<E>,
{
    fn from(res: Result<T, E>) -> Self {
        match res {
            Ok(t) => Self::Finished(t),
            Err(e) => Self::Err(MenuError::from(e)),
        }
    }
}

impl<T> From<Query<T>> for MenuResult<T> {
    fn from(q: Query<T>) -> Self {
        match q {
            Query::Finished(out) => Ok(out),
            Query::Err(e) => Err(e),
            Query::Continue => Err(MenuError::from("incorrect input")),
        }
    }
}

//! Module defining the different types of menus.
//!
//! ## The types
//!
//! There are two types of menus:
//! - the [value-menus](ValueMenu): they corresponds to the menu where the user has to enter
//! the value himself, for example for a name providing.
//! - the [selectable menus](SelectMenu): they corresponds to the menu where the user has to select
//! a value among proposed values by a list.
//!
//! ## Fields
//!
//! The `ValueMenu` can contain [`ValueField`s](crate::field::ValueField)
//! and [`SelectMenu`s](SelectMenu).
//! This behavior allows to use the selectable menus as sub-menus to retrieve
//! values.
//!
//! The `SelectMenu` contains [`SelectField`s](crate::field::SelectField).
//!
//! ## Formatting
//!
//! The formatting rules are defined by the [`ValueFieldFormatting`](crate::field::ValueFieldFormatting) struct.
//! It manages how the output of a message before the user input should be displayed.
//!
//! For a value-menu, you can apply global formatting rules with the [`ValueMenu::fmt`] method,
//! which will be applied on all the fields it contains. You can also apply rules on
//! specific fields.
//!
//! When a `SelectMenu` inherits the rules of its parent `ValueMenu`, they are applied on its title.
//!
//! ## Outputs
//!
//! The values entered by the user are provided by the [`Promptable`] trait.
//! This trait is implemented on both menus type and uses the [`MenuBuilder::prompt`] method
//! to return the next output provided by the user.
//!
//! When calling this method, you need to provide your own type to convert the input from.
//!
//! The next output of a value-menu corresponds to its next fields, so if it is, for example, a
//! selectable menu field, it will display the list of output values, then return the value the user
//! selected. Attention: if all the fields have been retrieved, the value-menu will be empty, and the
//! next call of this method will panic.
//!
//! Therefore, a selectable menu can return many times the value selected by the user at different
//! points of the code.
//!
//! ## Example
//!
//! ```no_run
//! use std::str::FromStr;
//! use ezmenulib::customs;
//! use ezmenulib::prelude::*;
//!
//! enum Type {
//!     MIT,
//!     GPL,
//!     BSD,
//! }
//!
//! let mut license = ValueMenu::from([
//!     Field::Value(ValueField::from("Authors")),
//!     Field::Select(SelectMenu::from([
//!         SelectField::new("MIT", Type::MIT),
//!         SelectField::new("GPL", Type::GPL),
//!         SelectField::new("BSD", Type::BSD),
//!     ])
//!     .default(1)
//!     .title(SelectTitle::from("Select the license type"))),
//! ]);
//!
//! let authors: customs::MenuVec<String> = license.next_value().unwrap();
//! let ty: Type = license.next_select().unwrap();
//! ```

#[cfg(test)]
mod tests;

mod stream;

pub use crate::menu::stream::MenuStream;
use crate::menu::stream::Stream;
use crate::prelude::*;
use std::fmt::Display;
use std::io::{BufRead, BufReader, Stdin, Stdout, Write};
use std::ops::DerefMut;
use std::str::FromStr;

/// The default input stream used by a menu, using the standard input stream.
pub type In = BufReader<Stdin>;

/// The default output stream used by a menu, using the standard output stream.
pub type Out = Stdout;

/// Shows the text using the given stream and maps the `io::Error` into a `MenuError`.
pub(crate) fn show<T, S>(text: &T, stream: &mut S) -> MenuResult
where
    T: ?Sized + Display,
    S: Write,
{
    write!(stream, "{}", text)?;
    stream.flush()?;
    Ok(())
}

/// Shows the text using the given stream, then prompts a value to the user and
/// returns the corresponding String.
pub(crate) fn prompt<S, R, W>(text: &S, stream: &mut MenuStream<R, W>) -> MenuResult<String>
where
    R: BufRead,
    W: Write,
    S: ?Sized + Display,
{
    show(text, stream)?;
    raw_read_input(stream)
}

/// Represents a value-menu type, which means a menu that retrieves values from the user inputs.
///
/// The `R` type parameter represents its reader type, and the `W` type parameter means its writer type.
/// By default, it uses the standard input and output streams to get values from the user.
/// It wraps the streams into a [`MenuStream`].
///
/// It has a global formatting applied to the fields it contains by inheritance.
pub struct Values<'a, R = In, W = Out> {
    fmt: Format<'a>,
    stream: Stream<'a, MenuStream<'a, R, W>>,
}

// Cannot use the derivable implementation of `Default`
// because generic parameters R and W need to implement `Default`.
// Here, we use the `Default` implementation of `MenuStream`, which
// uses `BufReader<Stdin>` as `R` and `Stdout` as `W`.
#[allow(clippy::derivable_impls)]
impl Default for Values<'_> {
    fn default() -> Self {
        Self {
            fmt: Format::default(),
            stream: Stream::default(),
        }
    }
}

impl<'a, R, W> From<MenuStream<'a, R, W>> for Values<'a, R, W> {
    fn from(stream: MenuStream<'a, R, W>) -> Self {
        Self::inner_new(Stream::Owned(stream))
    }
}

impl<'a, R, W> From<&'a mut MenuStream<'a, R, W>> for Values<'a, R, W> {
    fn from(stream: &'a mut MenuStream<'a, R, W>) -> Self {
        Self::inner_new(Stream::Borrowed(stream))
    }
}

impl<'a, R, W> Values<'a, R, W> {
    fn inner_new(stream: Stream<'a, MenuStream<'a, R, W>>) -> Self {
        Self {
            fmt: Default::default(),
            stream,
        }
    }

    /// Give the global formatting applied to all the fields the menu contains.
    /// If a field has a custom formatting, it will uses the formatting rules of the field
    /// when printing to the writer.
    pub fn format(mut self, fmt: &Format<'a>) -> Self {
        self.fmt.merge(fmt);
        self
    }

    pub fn take_stream(self) -> MenuStream<'a, R, W> {
        self.stream.retrieve()
    }

    pub fn take_io(self) -> (R, W) {
        self.stream.retrieve().retrieve()
    }

    pub fn get_stream(&self) -> &MenuStream<'a, R, W> {
        &self.stream
    }

    pub fn get_mut_stream(&mut self) -> &mut MenuStream<'a, R, W> {
        &mut self.stream
    }
}

impl<'a, R, W> Values<'a, R, W>
where
    R: BufRead,
    W: Write,
{
    /// Returns the next output, if the next output corresponds to an inner selectable menu output.
    ///
    /// If this is the case, it returns the selectable menu output
    /// (See [`<SelectMenu as MenuBuilder>::next_output`](SelectMenu::next_value)).
    ///
    /// ## Panic
    ///
    /// If the next field is not a selectable menu, this function will panic.
    pub fn selected<T, const N: usize>(&mut self, sel: Selected<'a, T, N>) -> MenuResult<T> {
        show(&self.fmt.prefix, self.stream.deref_mut())?;
        sel.format(&self.fmt).select(&mut self.stream)
    }

    pub fn selected_or_default<T, const N: usize>(&mut self, sel: Selected<'a, T, N>) -> T
    where
        T: Default,
    {
        show(&self.fmt.prefix, self.stream.deref_mut())
            .map(|_| sel.format(&self.fmt).select_or_default(&mut self.stream))
            .unwrap_or_default()
    }

    /// Returns the next output provided by the user.
    ///
    /// It prompts the user until the value entered is correct.
    /// If the next field is a selectable menu, it will prompt the selectable menu,
    /// requiring the output type to be `'static`.
    /// Otherwise, it will prompt the value-field, requiring the output type to implement
    /// `FromStr` trait.
    ///
    /// If there is no more field to prompt in the menu, this function will return an error
    /// (see [`MenuError::EndOfMenu`](crate::MenuError::EndOfMenu)).
    pub fn written<T: FromStr>(&mut self, written: &Written<'a>) -> MenuResult<T> {
        written.prompt_with(&mut self.stream, &self.fmt)
    }

    pub fn written_until<T: FromStr, F: Fn(&T) -> bool>(
        &mut self,
        written: &Written<'a>,
        til: F,
    ) -> MenuResult<T> {
        written.prompt_until_with(&mut self.stream, til, &self.fmt)
    }

    pub fn written_or_default<T: FromStr + Default>(&mut self, written: &Written<'a>) -> T {
        written.prompt_or_default_with(&mut self.stream, &self.fmt)
    }
}

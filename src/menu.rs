//! Module that defines several types to handle menus, streams and values retrieving.

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
    stream.flush().map_err(MenuError::from)
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

/// Container used to handle the [stream](MenuStream) and the global [format](Format).
///
/// The `R` type parameter represents its reader type,
/// and the `W` type parameter represents its writer type.
/// By default, it uses the standard input and output streams to get values from the user.
/// It wraps the streams into a [`MenuStream`].
///
/// It has a global formatting applied to the fields it gets values from by inheritance.
/// The inheritance saves the custom format specifications of the field.
///
/// # Example
///
/// ```no_run
/// use ezmenulib::prelude::*;
/// let mut menu = Values::from(Format::prefix("->> "));
/// // Inherits the prefix specification on the written field
/// let name: String = menu.written(&Written::from("What is your name")).unwrap();
/// // Uses the custom prefix specification of the selectable field
/// let amount: u8 = menu
///     .selected(
///         Selected::new("Select an amount", [("one", 1), ("two", 2), ("three", 3)])
///         .format(Format::prefix("-- "))
///     )
///     .unwrap();
/// ```
///
/// # Streams
///
/// By default, the container uses the standard input and output stream.
/// You can provide your own stream types, wrapped in a [`MenuStream`], and
/// borrow them to the container, or take the stream by ownership at the end.
///
/// ## Example
///
/// Taking the stream from the container by ownership:
/// ```no_run
/// # use ezmenulib::prelude::*;
/// let mut menu = Values::default();
/// // ...
/// let stream = menu.take_stream();
/// // or:
/// # let mut menu = Values::default();
/// let (reader, writer) = menu.take_io();
/// ```
///
/// Giving a mutable reference to the stream to the container:
/// ```no_run
/// # use ezmenulib::prelude::*;
/// let mut my_stream = MenuStream::default();
/// let mut menu = Values::from(&mut my_stream);
/// // We can also give the ownership:
/// // let mut menu = Values::from(my_stream);
/// ```
pub struct Values<'a, R = In, W = Out> {
    /// The global format of the container.
    pub fmt: Format<'a>,
    stream: Stream<'a, MenuStream<'a, R, W>>,
}

/// Returns the default container, which corresponds to the
/// [default format](Format::default) and the [owned default stream](MenuStream::default).
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

/// Creates the container from an owned stream.
///
/// You can still take the stream at the end of the usage, with [`Values::take_stream`].
impl<'a, R, W> From<MenuStream<'a, R, W>> for Values<'a, R, W> {
    fn from(stream: MenuStream<'a, R, W>) -> Self {
        Self::inner_new(Stream::Owned(stream), Format::default())
    }
}

/// Creates the container from a mutably borrowed stream.
///
/// This is useful if you still want to access the given streams while using the
/// container to retrieve values.
impl<'a, R, W> From<&'a mut MenuStream<'a, R, W>> for Values<'a, R, W> {
    fn from(stream: &'a mut MenuStream<'a, R, W>) -> Self {
        Self::inner_new(Stream::Borrowed(stream), Format::default())
    }
}

impl<'a> From<Format<'a>> for Values<'a> {
    fn from(fmt: Format<'a>) -> Self {
        Self::inner_new(Stream::default(), fmt)
    }
}

impl<'a, R, W> Values<'a, R, W> {
    fn inner_new(stream: Stream<'a, MenuStream<'a, R, W>>, fmt: Format<'a>) -> Self {
        Self { fmt, stream }
    }

    /// Defines the global formatting applied to all the fields the menu retrieves the values from.
    ///
    /// If the field contains custom formatting specifications, it will save them
    /// when printing to the writer.
    pub fn format(mut self, fmt: Format<'a>) -> Self {
        self.fmt = fmt;
        self
    }

    /// Returns the ownership of the stream it contains, consuming `self`.
    ///
    /// # Panics
    ///
    /// If the container does not own the stream (meaning it has been constructed
    /// with the `From<&mut MenuStream<R, W>>` implementation), this function panics.
    pub fn take_stream(self) -> MenuStream<'a, R, W> {
        self.stream.retrieve()
    }

    /// Returns the ownership of the reader and writer, consuming `self`.
    ///
    /// # Panics
    ///
    /// If the container does not own the stream (meaning it has been constructed
    /// with the `From<&mut MenuStream<R, W>>` implementation), this function panics.
    pub fn take_io(self) -> (R, W) {
        self.take_stream().retrieve()
    }

    /// Returns a reference to the stream the container uses.
    pub fn get_stream(&self) -> &MenuStream<'a, R, W> {
        &self.stream
    }

    /// Returns a mutable reference to the stream the container uses.
    pub fn get_mut_stream(&mut self) -> &mut MenuStream<'a, R, W> {
        &mut self.stream
    }
}

/// Associated functions that concerns retrieving values from the user,
/// thus using the reader and writer stream.
impl<R, W> Values<'_, R, W>
where
    R: BufRead,
    W: Write,
{
    /// Returns the next value selected by the user.
    ///
    /// It merges the [format](Format) of the field with the global format of the container.
    /// The merge saves the custom formatting specification of the selectable fields.
    ///
    /// See [`Selected::select`] function fore more information.
    pub fn selected<T, const N: usize>(&mut self, sel: Selected<'_, T, N>) -> MenuResult<T> {
        let fmt = sel.fmt.merged(&self.fmt);
        sel.format(fmt).select(self.stream.deref_mut())
    }

    pub fn optional_selected<T, const N: usize>(
        &mut self,
        sel: Selected<'_, T, N>,
    ) -> MenuResult<Option<T>> {
        let fmt = sel.fmt.merged(&self.fmt);
        sel.format(fmt).optional_select(self.stream.deref_mut())
    }

    pub fn selected_or_default<T, const N: usize>(&mut self, sel: Selected<'_, T, N>) -> T
    where
        T: Default,
    {
        let fmt = self.fmt.merged(&self.fmt);
        sel.format(fmt).select_or_default(self.stream.deref_mut())
    }

    /// Returns the next value written by the user.
    ///
    /// It merges the [format](Format) of the field with the global format of the container.
    /// The merge saves the custom formatting specification of the written field.
    ///
    /// See [`Written::prompt`] for more information.
    ///
    /// # Panic
    ///
    /// If the given written field has an incorrect default value,
    /// this function will panic at runtime.
    pub fn written<T>(&mut self, written: &Written<'_>) -> MenuResult<T>
    where
        T: FromStr,
    {
        written.prompt_with(self.stream.deref_mut(), &self.fmt)
    }

    /// Returns the next value written by the user by prompting him the field
    /// until the given constraint is applied.
    ///
    /// It merges the [format](Format) of the field with the global format of the container.
    /// The merge saves the custom formatting specification of the written field.
    ///
    /// See [`Written::prompt_until`] for more information.
    ///
    /// # Panic
    ///
    /// If the given written field has an incorrect default value,
    /// this function will panic at runtime.
    pub fn written_until<T, F>(&mut self, written: &Written<'_>, til: F) -> MenuResult<T>
    where
        T: FromStr,
        F: Fn(&T) -> bool,
    {
        written.prompt_until_with(self.stream.deref_mut(), til, &self.fmt)
    }

    pub fn optional_written<T>(&mut self, written: &Written<'_>) -> MenuResult<Option<T>>
    where
        T: FromStr,
    {
        written.optional_prompt_with(self.stream.deref_mut(), &self.fmt)
    }

    pub fn written_or_default<T>(&mut self, written: &Written<'_>) -> T
    where
        T: FromStr + Default,
    {
        written.prompt_or_default_with(self.stream.deref_mut(), &self.fmt)
    }
}

//! Module that defines several types to handle menus, streams and values retrieving.

#[cfg(test)]
mod tests;

mod stream;

pub use crate::menu::stream::{MenuStream, Mutable};
use crate::prelude::*;
use crate::utils::{check_fields, select, Depth};

use std::fmt::{self, Display, Formatter};
use std::io::{BufRead, BufReader, Stdin, Stdout, Write};
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

/// The default input stream used by a menu, using the standard input stream.
pub type In = BufReader<Stdin>;

/// The default output stream used by a menu, using the standard output stream.
pub type Out = Stdout;

/// Used to retrieve the object from a container.
///
/// The object may be either owned or mutably borrowed.
/// See [`Mutable`] for more information.
pub trait UsesMutable<T> {
    /// Returns the ownership of the stream it contains, consuming `self`.
    ///
    /// # Panics
    ///
    /// Because the object may not be owned by the container, this function may panic
    /// at runtime, because it attempts to retrieve the ownership of the stream it does not own.
    fn take_object(self) -> T;

    /// Returns a reference to the object the container uses.
    fn get_object(&self) -> &T;

    /// Returns a mutable reference to the object the container uses.
    fn get_mut_object(&mut self) -> &mut T;
}

/// Used to instantiate a container from a mutable object,
/// and the rest of the required arguments.
///
/// A mutable object in the library means a generic object (here `S`)
/// that can be either owned by the container, or mutably borrowed.
pub trait FromMutable<'a, S: 'a, Arg>: Sized {
    /// Returns `Self` from the mutable object and the rest of the required arguments
    /// for the container.
    fn new(stream: Mutable<'a, S>, arg: Arg) -> Self;

    /// Returns `Self` from a mutably borrowed object
    /// the rest of the required arguments for the container.
    fn borrowed(stream: &'a mut S, arg: Arg) -> Self {
        Self::new(Mutable::Borrowed(stream), arg)
    }

    /// Returns `Self` from an owned object and the rest of the required arguments
    /// for the container.
    fn owned(stream: S, arg: Arg) -> Self {
        Self::new(Mutable::Owned(stream), arg)
    }
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
///
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
/// The mutability of the struct when calling its methods is due to the mutability
/// of the stream when doing operations with it.
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
/// let (reader, writer) = menu.take_stream().retrieve();
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
#[derive(Debug)]
pub struct Values<'a, R = In, W = Out> {
    /// The global format of the container.
    pub fmt: Format<'a>,
    stream: Mutable<'a, MenuStream<'a, R, W>>,
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
            stream: Mutable::default(),
        }
    }
}

/// Creates the container from an owned stream.
///
/// You can still take the stream at the end of the usage, with [`Values::take_object`].
impl<'a, R, W> From<MenuStream<'a, R, W>> for Values<'a, R, W> {
    fn from(stream: MenuStream<'a, R, W>) -> Self {
        Self::owned(stream, Format::default())
    }
}

/// Creates the container from a mutably borrowed stream.
///
/// This is useful if you still want to access the given streams while using the
/// container to retrieve values.
impl<'a, R, W> From<&'a mut MenuStream<'a, R, W>> for Values<'a, R, W> {
    fn from(stream: &'a mut MenuStream<'a, R, W>) -> Self {
        Self::borrowed(stream, Format::default())
    }
}

impl<'a> From<Format<'a>> for Values<'a> {
    fn from(fmt: Format<'a>) -> Self {
        Self::owned(MenuStream::default(), fmt)
    }
}

impl<'a, R, W> FromMutable<'a, MenuStream<'a, R, W>, Format<'a>> for Values<'a, R, W> {
    fn new(stream: Mutable<'a, MenuStream<'a, R, W>>, fmt: Format<'a>) -> Self {
        Self { fmt, stream }
    }
}

impl<'a, R, W> Values<'a, R, W> {
    /// Defines the global formatting applied to all the fields
    /// the container retrieves the values from.
    ///
    /// If the field contains custom formatting specifications, it will save them
    /// when printing to the writer.
    pub fn format(mut self, fmt: Format<'a>) -> Self {
        self.fmt = fmt;
        self
    }
}

impl<'a, R, W> UsesMutable<MenuStream<'a, R, W>> for Values<'a, R, W> {
    /// Returns the ownership of the stream it contains, consuming `self`.
    ///
    /// # Panics
    ///
    /// If the container does not own the stream (meaning it has been constructed
    /// with the `From<&mut MenuStream<R, W>>` implementation), this function panics.
    fn take_object(self) -> MenuStream<'a, R, W> {
        self.stream.retrieve()
    }

    fn get_object(&self) -> &MenuStream<'a, R, W> {
        self.stream.deref()
    }

    fn get_mut_object(&mut self) -> &mut MenuStream<'a, R, W> {
        self.stream.deref_mut()
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

    /// Returns the next value selected by the user wrapped as `Some(value)`,
    /// else `None`.
    ///
    /// It merges the [format](Format) of the field with the global format of the container.
    /// The merge saves the custom formatting specification of the selectable fields.
    ///
    /// See [`Selected::optional_select`] function fore more information.
    pub fn optional_selected<T, const N: usize>(
        &mut self,
        sel: Selected<'_, T, N>,
    ) -> MenuResult<Option<T>> {
        let fmt = sel.fmt.merged(&self.fmt);
        sel.format(fmt).optional_select(self.stream.deref_mut())
    }

    /// Returns the next value selected by the user, or the default value of the output type
    /// if any error occurred.
    ///
    /// It merges the [format](Format) of the field with the global format of the container.
    /// The merge saves the custom formatting specification of the selectable fields.
    ///
    /// See [`Selected::select_or_default`] function for more information.
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

    /// Returns the next value written by the user wrapped as `Some(value)`
    /// if the input is correct, else `None`.
    ///
    /// It merges the [format](Format) of the field with the global format of the container.
    /// The merge saves the custom formatting specification of the written field.
    ///
    /// See [`Written::optional_value`] for more information.
    ///
    /// # Panic
    ///
    /// If the given written field has an incorrect default value,
    /// this function will panic at runtime.
    pub fn optional_written<T>(&mut self, written: &Written<'_>) -> MenuResult<Option<T>>
    where
        T: FromStr,
    {
        written.optional_value_with(self.stream.deref_mut(), &self.fmt)
    }

    /// Returns the next many values written by the user wrapped as a `Vec<T>`, separated by
    /// `sep`, until the given constraint is applied to all the values.
    ///
    /// It merges the [format](Format) of the field with the global format of the container.
    /// The merge saves the custom formatting specification of the written field.
    ///
    /// See [`Written::many_values_until`] for more information.
    ///
    /// # Panic
    ///
    /// If the given written field has an incorrect default value,
    /// this function will panic at runtime.
    pub fn many_written_until<T, S, F>(
        &mut self,
        written: &Written<'_>,
        sep: S,
        til: F,
    ) -> MenuResult<Vec<T>>
    where
        T: FromStr,
        S: AsRef<str>,
        F: Fn(&T) -> bool,
    {
        written.many_values_until_with(self.stream.deref_mut(), sep, til, &self.fmt)
    }

    /// Returns the next many values written by the user wrapped as a `Vec<T>`,
    /// separated by `sep`.
    ///
    /// It merges the [format](Format) of the field with the global format of the container.
    /// The merge saves the custom formatting specification of the written field.
    ///
    /// See [`Written::many_values`] for more information.
    ///
    /// # Panic
    ///
    /// If the given written field has an incorrect default value,
    /// this function will panic at runtime.
    pub fn many_written<T, S>(&mut self, written: &Written<'_>, sep: S) -> MenuResult<Vec<T>>
    where
        T: FromStr,
        S: AsRef<str>,
    {
        written.many_values_with(self.stream.deref_mut(), sep, &self.fmt)
    }

    /// Returns the next value written by the user, or the default value of the
    /// output type if any error occurred.
    ///
    /// It merges the [format](Format) of the field with the global format of the container.
    /// The merge saves the custom formatting specification of the written field.
    ///
    /// See [`Written::prompt_or_default`] for more information.
    ///
    /// # Panic
    ///
    /// If the given written field has an incorrect default value,
    /// this function will panic at runtime.
    pub fn written_or_default<T>(&mut self, written: &Written<'_>) -> T
    where
        T: FromStr + Default,
    {
        written.prompt_or_default_with(self.stream.deref_mut(), &self.fmt)
    }
}

/// Defines a menu, with a title, the fields, and the reader and writer types.
///
/// It handles the [stream](MenuStream) and a [format](Format).
///
/// The `R` type parameter represents its reader type,
/// and the `W` type parameter represents its writer type.
/// By default, it uses the standard input and output streams to get values from the user.
/// It wraps the streams into a [`MenuStream`].
///
/// ## Example
///
/// ```no_run
/// use ezmenulib::prelude::*;
/// use std::io::Write;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// Menu::from(&[
///     ("Alice", Kind::Quit),
///     ("Bob", Kind::Quit),
///     ("Charlie", Kind::Quit),
///     ("SubMenu", Kind::Parent(&[
///         ("Foo", Kind::Map(|s| Ok(writeln!(s, "foo")?))),
///         ("Bar", Kind::Map(|s| Ok(writeln!(s, "bar")?))),
///         ("Go back!", Kind::Back(1)),
///     ])),
/// ])
/// .run()?;
/// # Ok(()) }
/// ```
#[derive(Debug)]
pub struct RawMenu<'a, R = In, W = Out> {
    /// The global format of the menu.
    pub fmt: Format<'a>,
    title: Option<&'a str>,
    fields: Fields<'a, R, W>,
    stream: Mutable<'a, MenuStream<'a, R, W>>,
    once: bool,
}

impl<'a, R, W> UsesMutable<MenuStream<'a, R, W>> for RawMenu<'a, R, W> {
    /// Returns the ownership of the stream the menu contains, consuming `self`.
    ///
    /// # Panics
    ///
    /// If the menu does not own the stream (meaning it has been constructed
    /// with the `From<&mut MenuStream<R, W>>` implementation), this function panics.
    fn take_object(self) -> MenuStream<'a, R, W> {
        self.stream.retrieve()
    }

    fn get_object(&self) -> &MenuStream<'a, R, W> {
        self.stream.deref()
    }

    fn get_mut_object(&mut self) -> &mut MenuStream<'a, R, W> {
        self.stream.deref_mut()
    }
}

impl<R, W> Display for RawMenu<'_, R, W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Title
        if let Some(title) = self.title {
            f.write_str(title)?;
        }

        // Fields
        // The chip representation is managed by the field itself.
        for (i, field) in self.fields.iter().enumerate() {
            writeln!(f, "{}{}{}", i + 1, self.fmt.chip, field.0)?;
        }

        Ok(())
    }
}

impl<'a> From<Fields<'a>> for RawMenu<'a> {
    fn from(fields: Fields<'a>) -> Self {
        Self::owned(MenuStream::default(), fields)
    }
}

impl<'a, const N: usize> From<&'a [Field<'a>; N]> for RawMenu<'a> {
    fn from(fields: &'a [Field<'a>; N]) -> Self {
        Self::from(fields.as_ref())
    }
}

impl<'a, R, W> FromMutable<'a, MenuStream<'a, R, W>, Fields<'a, R, W>> for RawMenu<'a, R, W> {
    fn new(stream: Mutable<'a, MenuStream<'a, R, W>>, fields: Fields<'a, R, W>) -> Self {
        check_fields(fields);

        Self {
            title: None,
            fmt: Format::default(),
            fields,
            stream,
            once: false,
        }
    }
}

impl<'a, R, W> RawMenu<'a, R, W> {
    /// Defines the global formatting applied to all the fields the menu displays.
    pub fn format(mut self, fmt: Format<'a>) -> Self {
        self.fmt = fmt;
        self
    }

    /// Defines the title of the menu, which corresponds to the string slice displayed
    /// at the top when running the menu.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    /// Defines if the menu should run once or loop when calling a mapped function
    /// to a field.
    pub fn run_once(mut self, once: bool) -> Self {
        self.once = once;
        self
    }
}

impl<R, W> RawMenu<'_, R, W>
where
    R: BufRead,
    W: Write,
{
    /// Runs the menu.
    ///
    /// The mutability is from the operations done with the stream.
    ///
    /// It prints to the stream the fields next to their indexes, then asks the user to
    /// select a field. Then, it runs the corresponding procedure
    /// matching the selected field [kind](Kind).
    pub fn run(&mut self) -> MenuResult {
        run_with(
            &mut RunParams {
                stream: self.stream.deref_mut(),
                fmt: &self.fmt,
                once: self.once,
            },
            self.title,
            self.fields,
        )
        .map(|_| ())
    }
}

/// Represents the parameters of the menu currently running, which are the same
/// at any state of the menu (any depth of the `run_with` recursive function).
struct RunParams<'a, 'b: 'a, R, W> {
    stream: &'a mut MenuStream<'b, R, W>,
    fmt: &'a Format<'b>,
    once: bool,
}

/// Recursive function used to run the current prompt state of the menu.
///
/// It prints out to the stream the fields next to their indexes, then asks the user to
/// select a field. Then, it runs the corresponding procedure matching the selected field kind.
///
/// The function returns a wrapped `Option<usize>`. The index inside corresponds to the current
/// level of depth of the menu. With recursion, it allows to go back to the indexed depth
/// level from the current running prompt.
fn run_with<R: BufRead, W: Write>(
    params: &mut RunParams<R, W>,
    msg: Option<&str>,
    fields: Fields<R, W>,
) -> MenuResult<Option<usize>> {
    let quit = Ok(None);
    let back = |i| Ok(Some(i - 1));

    loop {
        // Title of current selective menu.
        if let Some(s) = msg {
            writeln!(params.stream, "{}{s}", params.fmt.prefix)?;
        }

        // Fields of current selective menu.
        for (i, (field_msg, _)) in (1..=fields.len()).zip(fields.iter()) {
            writeln!(
                params.stream,
                "{}{i}{}{}{field_msg}",
                params.fmt.left_sur, params.fmt.right_sur, params.fmt.chip
            )?;
        }

        // Gets the message and the field kind selected by the user.
        let (msg, kind) = loop {
            match select(params.stream, params.fmt.suffix, fields.len())?
                .and_then(|i| fields.get(i))
            {
                Some(field) => break field,
                None => continue,
            }
        };

        match kind {
            Kind::Map(f) => {
                f(params.stream)?;
                if params.once {
                    return quit;
                }
            }
            Kind::Parent(fields) => match run_with(params, Some(msg), fields)? {
                None => return quit,
                Some(0) => (),
                Some(i) => return back(i),
            },
            Kind::Back(0) => (),
            Kind::Back(i) => return back(*i),
            Kind::Quit => return quit,
        }
    }
}

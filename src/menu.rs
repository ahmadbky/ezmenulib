//! Module that defines several types to handle menus, streams and values retrieving.

mod handle;
#[cfg(test)]
mod tests;

pub use self::handle::{Handle, MenuHandle};

use crate::{
    field::Promptable,
    prelude::*,
    utils::{check_fields, select, Depth},
};

use std::{
    fmt::{self, Display, Formatter},
    io::{Stdin, Stdout},
};

/// The default input stream used by a menu, using the standard input stream.
pub type In = Stdin;

/// The default output stream used by a menu, using the standard output stream.
pub type Out = Stdout;

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
pub struct Values<'a, H = MenuHandle> {
    /// The global format of the container.
    pub fmt: Format<'a>,
    /// The global handle of the container.
    pub handle: H,
}

impl Default for Values<'_> {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> From<Format<'a>> for Values<'a> {
    #[inline]
    fn from(fmt: Format<'a>) -> Self {
        Self::from_format(fmt)
    }
}

impl<'a, H> From<H> for Values<'a, H> {
    fn from(handle: H) -> Self {
        Self::from_handle(handle)
    }
}

impl<'a> Values<'a> {
    #[inline]
    pub fn new() -> Self {
        Self::from_format(Format::default())
    }

    pub fn from_format(fmt: Format<'a>) -> Self {
        Self {
            handle: Default::default(),
            fmt,
        }
    }
}

impl<'a, H> Values<'a, H> {
    pub fn from_handle(handle: H) -> Self {
        Self {
            handle,
            fmt: Format::default(),
        }
    }

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

/// Associated functions that concerns retrieving values from the user,
/// thus using the reader and writer stream.
impl<H: Handle> Values<'_, H> {
    pub fn next<T, P>(&mut self, p: P) -> MenuResult<T>
    where
        P: Promptable<T>,
    {
        p.prompt_with(&mut self.handle, &self.fmt)
    }

    pub fn next_or_default<T, P>(&mut self, p: P) -> T
    where
        T: Default,
        P: Promptable<T>,
    {
        p.prompt_or_default_with(&mut self.handle, &self.fmt)
    }

    pub fn next_optional<T, P>(&mut self, p: P) -> MenuResult<Option<T>>
    where
        P: Promptable<T>,
    {
        p.optional_prompt_with(&mut self.handle, &self.fmt)
    }
}

const ERR_MSG: &str = "an error occurred while retrieving values";

pub trait Prompted: Sized {
    fn prompt() -> Self {
        Self::try_prompt().expect(ERR_MSG)
    }

    fn try_prompt() -> MenuResult<Self> {
        Self::try_prompt_with(MenuHandle::default())
    }

    fn prompt_with<H: Handle>(handle: H) -> Self {
        Self::try_prompt_with(handle).expect(ERR_MSG)
    }

    fn try_prompt_with<H: Handle>(handle: H) -> MenuResult<Self> {
        Self::from_values(&mut Values::from_handle(handle))
    }

    fn from_values<H: Handle>(vals: &mut Values<H>) -> MenuResult<Self>;
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
pub struct RawMenu<'a, H = MenuHandle> {
    /// The global format of the menu.
    pub fmt: Format<'a>,
    /// The global handle of the menu.
    pub handle: H,
    title: Option<&'a str>,
    fields: Fields<'a, H>,
    once: bool,
}

impl<H> Display for RawMenu<'_, H> {
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

impl<'a> RawMenu<'a> {
    pub fn new(fields: Fields<'a>) -> Self {
        check_fields(fields);
        Self::with_handle(MenuHandle::default(), fields)
    }
}

impl<'a> From<Fields<'a>> for RawMenu<'a> {
    #[inline]
    fn from(fields: Fields<'a>) -> Self {
        Self::new(fields)
    }
}

impl<'a, const N: usize> From<&'a [Field<'a>; N]> for RawMenu<'a> {
    #[inline]
    fn from(fields: &'a [Field<'a>; N]) -> Self {
        Self::from(fields.as_ref())
    }
}

impl<'a, H> RawMenu<'a, H> {
    pub fn with_handle(handle: H, fields: Fields<'a, H>) -> Self {
        Self {
            fmt: Format::default(),
            title: None,
            fields,
            handle,
            once: false,
        }
    }

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

impl<H: Handle> RawMenu<'_, H> {
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
                handle: &mut self.handle,
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
struct RunParams<'a, 'b: 'a, H> {
    handle: &'a mut H,
    fmt: &'a Format<'b>,
    once: bool,
}

/// Prints out the menu to the terminal.
fn show_menu<H: Handle>(
    params: &mut RunParams<H>,
    msg: Option<&str>,
    fields: Fields<H>,
) -> MenuResult {
    // Title of current selective menu.
    if let Some(s) = msg {
        writeln!(params.handle, "{}{s}", params.fmt.prefix)?;
    }

    // Fields of current selective menu.
    for (i, (field_msg, _)) in (1..=fields.len()).zip(fields.iter()) {
        writeln!(
            params.handle,
            "{}{i}{}{}{field_msg}",
            params.fmt.left_sur, params.fmt.right_sur, params.fmt.chip
        )?;
    }

    Ok(())
}

/// Handles the field selected by the user.
fn handle_field<H: Handle>(
    params: &mut RunParams<H>,
    msg: &str,
    kind: &Kind<H>,
) -> MenuResult<Depth> {
    use Depth::*;

    Ok(match kind {
        Kind::Map(f) => {
            f(D::new(params.handle))?;
            if params.once {
                Quit
            } else {
                Current
            }
        }
        Kind::Parent(fields) => match run_with(params, Some(msg), fields)? {
            Current | Back(0) => Current,
            Quit => Quit,
            Back(i) => Back(i - 1),
        },
        Kind::Back(0) => Current,
        Kind::Back(i) => Back(i - 1),
        Kind::Quit => Quit,
    })
}

/// Recursive function used to run the current prompt state of the menu.
///
/// It prints out to the stream the fields next to their indexes, then asks the user to
/// select a field. Then, it runs the corresponding procedure matching the selected field kind.
///
/// The function returns a wrapped `Option<usize>`. The index inside corresponds to the current
/// level of depth of the menu. With recursion, it allows to go back to the indexed depth
/// level from the current running prompt.
fn run_with<H: Handle>(
    params: &mut RunParams<H>,
    msg: Option<&str>,
    fields: Fields<H>,
) -> MenuResult<Depth> {
    loop {
        show_menu(params, msg, fields)?;

        // Gets the message and the field kind selected by the user.
        let (msg, kind) = loop {
            match select(&mut params.handle, params.fmt.suffix, fields.len())?
                .and_then(|i| fields.get(i))
            {
                Some(field) => break field,
                None => continue,
            }
        };

        match handle_field(params, msg, kind)? {
            Depth::Quit => return Ok(Depth::Quit),
            Depth::Back(i) => return Ok(Depth::Back(i)),
            Depth::Current => (),
        }
    }
}

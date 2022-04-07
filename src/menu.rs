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
//! The values entered by the user are provided by the [`MenuBuilder`] trait.
//! This trait is implemented on both menus type and uses the [`MenuBuilder::next_output`] method
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

mod stream;

pub use crate::menu::stream::MenuStream;
use crate::menu::stream::Stream;
use crate::prelude::*;
use crate::DEFAULT_FMT;
use std::fmt::{self, Debug, Display, Error, Formatter};
use std::io::{BufRead, BufReader, Stdin, Stdout, Write};
use std::rc::Rc;
use std::str::FromStr;
use std::vec::IntoIter;

/// Used to retrieve the stream contained in a menu.
pub trait Streamable<'s, R: 's, W: 's>: Sized {
    /// Returns the menu stream, consuming the menu.
    fn get_stream(self) -> MenuStream<'s, R, W>;

    /// Returns a reference of the menu stream.
    fn get_stream_ref(&self) -> &MenuStream<'s, R, W>;

    /// Returns a mutable reference of the menu stream.
    fn get_stream_ref_mut(&mut self) -> &mut MenuStream<'s, R, W>;

    /// Returns the input and output streams, consuming the menu.
    /// See [`MenuStream::retrieve`](stream::MenuStream::retrieve) for more information.
    ///
    /// ## Panics
    ///
    /// If it hasn't been given a reader and a writer, this method will panic, because it needs
    /// to own the reader and writer to retrieve it at the end.
    #[inline]
    fn retrieve(self) -> (R, W) {
        self.get_stream().retrieve()
    }
}

/// The default input stream used by a menu, using the standard input stream.
pub type In = BufReader<Stdin>;

/// The default output stream used by a menu, using the standard output stream.
pub type Out = Stdout;

pub(crate) fn show<S, R, W>(text: &S, stream: &mut MenuStream<R, W>) -> MenuResult<()>
where
    W: Write,
    S: ?Sized + Display,
{
    write!(stream, "{}", text)?;
    stream.flush()?;
    Ok(())
}

pub(crate) fn prompt<S, R, W>(text: &S, stream: &mut MenuStream<R, W>) -> MenuResult<String>
where
    R: BufRead,
    W: Write,
    S: ?Sized + Display,
{
    show(text, stream)?;
    raw_read_input(stream)
}

/// The position of the title for an enum menu.
/// By default, the title position is at the top.
///
/// ## Example
///
/// ```no_run
/// use ezmenulib::prelude::*;
///
/// let amount: MenuResult<u8> = SelectMenu::from([
///     SelectField::new("first", 1u8),
///     SelectField::new("second", 2u8),
///     SelectField::new("third", 3u8),
/// ])
/// .title(SelectTitle::from("set the podium").pos(TitlePos::Bottom))
/// .select();
/// ```
#[derive(Clone, Copy)]
pub enum TitlePos {
    /// Position at the top of the menu:
    /// ```text
    /// <title>
    /// 1 - field0
    /// 2 - field1
    /// ...
    /// >>
    /// ```
    Top,
    /// Position at the bottom of the menu:
    /// ```text
    /// 1 - field0
    /// 2 - field1
    /// ...
    /// <title>
    /// >>
    /// ```
    Bottom,
}

impl Default for TitlePos {
    /// Default position for the menu title is at the top.
    fn default() -> Self {
        Self::Top
    }
}

/// Represents the title of a selectable menu.
///
/// It has its own type because it manages its position, its formatting,
/// and the formatting of the fields inside the selectable menu.
///
/// ## Example
///
/// ```
/// use ezmenulib::{prelude::*, customs::MenuBool};
///     
/// let is_adult: MenuBool = SelectMenu::from([
///     SelectField::new("yes", MenuBool(true)),
///     SelectField::new("no", MenuBool(false)),
/// ])
/// .title(SelectTitle::from("Are you an adult?")
///     .fmt(ValueFieldFormatting::chip("==> "))
///     .pos(TitlePos::Top))
/// .default(1)
/// .select()
/// .unwrap();
/// ```
pub struct SelectTitle<'a> {
    inner: &'a str,
    fmt: ValueFieldFormatting<'a>,
    custom_fmt: bool,
    pub(crate) pos: TitlePos,
}

impl Default for SelectTitle<'_> {
    fn default() -> Self {
        Self {
            inner: "",
            fmt: ValueFieldFormatting {
                chip: "",
                prefix: "",
                ..Default::default()
            },
            custom_fmt: false,
            pos: Default::default(),
        }
    }
}

impl<'a> From<&'a str> for SelectTitle<'a> {
    fn from(inner: &'a str) -> Self {
        Self {
            inner,
            fmt: ValueFieldFormatting::prefix(":"),
            custom_fmt: false,
            pos: Default::default(),
        }
    }
}

impl<'a> SelectTitle<'a> {
    /// Sets the formatting of the selectable menu title.
    ///
    /// The formatting type is the same as the [`ValueField`](crate::field::ValueField) is using.
    pub fn fmt(mut self, fmt: ValueFieldFormatting<'a>) -> Self {
        self.fmt = fmt;
        self.custom_fmt = true;
        self
    }

    /// Sets the position of the title.
    ///
    /// By default, the title position is at the top (see [`TitlePos`]).
    pub fn pos(mut self, pos: TitlePos) -> Self {
        self.pos = pos;
        self
    }

    /// Inherits the formatting rules from a parent menu (the [`ValueMenu`](crate::menu::ValueMenu)).
    ///
    /// It saves the prefix, because the default prefix is `>> ` and is not compatible with the
    /// title displaying.
    pub(crate) fn inherit_fmt(&mut self, fmt: Rc<ValueFieldFormatting<'a>>) {
        self.fmt = ValueFieldFormatting {
            chip: fmt.chip,
            default: fmt.default,
            // saving prefix
            prefix: self.fmt.prefix,
        };
        self.custom_fmt = false;
    }
}

impl Display for SelectTitle<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.inner.is_empty() {
            return Ok(());
        }

        writeln!(
            f,
            "{chip}{title}{prefix}",
            chip = self.fmt.chip,
            title = self.inner,
            prefix = self.fmt.prefix,
        )
    }
}

/// Struct modeling a selective menu.
///
/// ## Example
///
/// ```no_run
/// use std::str::FromStr;
/// use ezmenulib::prelude::*;
///
/// enum Type {
///     MIT,
///     GPL,
///     BSD,
/// }
///
/// let license_type: Type = SelectMenu::from([
///     SelectField::new("MIT", Type::MIT),
///     SelectField::new("GPL", Type::GPL),
///     SelectField::new("BSD", Type::BSD),
/// ])
/// .title(SelectTitle::from("Choose a license type"))
/// .default(1)
/// .select()
/// .unwrap();
/// ```
///
/// ## Formatting
///
/// The selective menu has two editable formatting rules.
/// Like [`ValueFieldFormatting`], it contains a `chip` and a `prefix`:
/// ```text
/// X<chip><message>
/// X<chip><message>
/// ...
/// <prefix>
/// ```
///
/// The default chip is `" - "`, and the default prefix is `">> "`.
pub struct SelectMenu<'a, R = In, W = Out> {
    prefix: &'a str,
    title: SelectTitle<'a>,
    fields: Vec<SelectField<'a, R, W>>,
    default: Option<usize>,
    stream: Stream<'a, MenuStream<'a, R, W>>,
}

impl<'a, R, W, Output> Promptable<'a, Output, R, W> for SelectMenu<'a, R, W>
where
    R: BufRead,
    W: Write,
    Output: 'static,
{
    fn prompt_once(&mut self, stream: &mut MenuStream<'a, R, W>) -> Query<Output> {
        select_once(stream, self.prefix, self.default, &mut self.fields)
    }
}

impl<'a> From<Vec<SelectField<'a>>> for SelectMenu<'a> {
    /// Builds the menu from its fields vector.
    #[inline]
    fn from(fields: Vec<SelectField<'a>>) -> Self {
        Self::with_owned(MenuStream::default(), fields)
    }
}

impl<'a, const N: usize> From<[SelectField<'a>; N]> for SelectMenu<'a> {
    /// Builds the menu from an array of fields.
    #[inline]
    fn from(fields: [SelectField<'a>; N]) -> Self {
        Self::from(Vec::from(fields))
    }
}

impl<'a, R, W> SelectMenu<'a, R, W> {
    fn inner_new(
        stream: Stream<'a, MenuStream<'a, R, W>>,
        fields: Vec<SelectField<'a, R, W>>,
    ) -> Self {
        Self {
            prefix: DEFAULT_FMT.prefix,
            title: Default::default(),
            fields,
            default: None,
            stream,
        }
    }

    /// Builds the menu from its owned menu stream, with its fields vector.
    #[inline]
    pub fn with_owned(stream: MenuStream<'a, R, W>, fields: Vec<SelectField<'a, R, W>>) -> Self {
        Self::inner_new(Stream::Owned(stream), fields)
    }

    /// Builds the menu from a mutable reference of a menu stream, with its fields vector.
    #[inline]
    pub fn with_ref(
        stream: &'a mut MenuStream<'a, R, W>,
        fields: Vec<SelectField<'a, R, W>>,
    ) -> Self {
        Self::inner_new(Stream::Borrowed(stream), fields)
    }

    /// Builds the menu from its owned reader and writer, with its fields vector.
    #[inline]
    pub fn new(reader: R, writer: W, fields: Vec<SelectField<'a, R, W>>) -> Self {
        Self::with_owned(MenuStream::new(reader, writer), fields)
    }

    /// Builds the menu from mutable references of the reader and writer, with its fields vector.
    #[inline]
    pub fn new_ref(
        reader: &'a mut R,
        writer: &'a mut W,
        fields: Vec<SelectField<'a, R, W>>,
    ) -> Self {
        Self::with_owned(MenuStream::with(reader, writer), fields)
    }

    /// Sets the title of the selective menu.
    ///
    /// The title is by default displayed at the top of the selective fields,
    /// but you can edit this behavior by setting the title position to `TitlePos::Bottom`, with
    /// `SelectMenu::title_pos` method.
    #[inline]
    pub fn title(mut self, title: SelectTitle<'a>) -> Self {
        self.title = title;
        self
    }

    /// Sets the default selective field with a given index.
    ///
    /// If you have specified a default field, the latter will be marked as `"(default)"`.
    /// Thus, if the user skips the selective menu (by pressing enter without input), it will return
    /// the default selective field.
    ///
    /// # Note
    ///
    /// You have to give the index as the user has to enter, so it is shifted up by 1.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ezmenulib::prelude::*;
    /// let hello = SelectMenu::from([
    ///     SelectField::new("hey", 0),
    ///     SelectField::new("hello", 1),
    ///     SelectField::new("bonjour", 2),
    /// ])
    /// .default(1); // binds the default value to "hey"
    /// ```
    pub fn default(mut self, default: usize) -> Self {
        self.default = Some(default);
        self
    }

    /// Sets the user input prefix of the selective menu.
    ///
    /// By default, the prefix used is `">> "`.
    pub fn prefix(mut self, prefix: &'a str) -> Self {
        self.prefix = prefix;
        self
    }

    /// Sets the chip of the selective menu.
    ///
    /// The chip is the short string slice placed between the field index and the field message.
    /// It acts as a list style attribute.
    pub fn chip(mut self, chip: &'a str) -> Self {
        for field in self.fields.as_mut_slice() {
            field.set_chip(chip);
        }
        self
    }

    #[inline]
    pub(crate) fn inherit_fmt(&mut self, fmt: Rc<ValueFieldFormatting<'a>>) {
        self.title.inherit_fmt(fmt);
    }
}

impl<'a, R, W> SelectMenu<'a, R, W>
where
    R: BufRead,
    W: Write,
{
    /// Prompts the selectable menu to the user, then returns the
    /// value he selected.
    ///
    /// The output value corresponds to the bound value of the field the user selected.
    ///
    /// # Panic
    ///
    /// If the bound value has a different type than the output value, this function
    /// will panic at runtime.
    pub fn select<Output: 'static>(&mut self) -> MenuResult<Output> {
        {
            let s = format!("{}", self);
            show(&s, &mut self.stream)
        }?;
        loop {
            match select_once(
                &mut self.stream,
                self.prefix,
                self.default,
                &mut self.fields,
            ) {
                Query::Finished(out) => break Ok(out),
                Query::Err(e) => break Err(e),
                _ => continue,
            }
        }
    }

    /// Prompts the selectable menu to the user, then returns the
    /// value he selected, using the given `MenuStream`.
    ///
    /// The output value corresponds to the bound value of the field the user selected.
    ///
    /// # Panic
    ///
    /// If the bound value has a different type than the output value, this function
    /// will panic at runtime.
    pub fn select_with<Output: 'static>(
        &mut self,
        stream: &mut MenuStream<'a, R, W>,
    ) -> MenuResult<Output> {
        self.prompt(stream)
    }

    /// Prompts the selectable menu to the user, then returns the
    /// value he selected, or returns the default value.
    ///
    /// The output value corresponds to the bound value of the field the user selected.
    /// If an error occurred (see [`MenuError`](crate::MenuError)), or if the user
    /// provided an incorrect input, it will return the default value.
    ///
    /// # Panic
    ///
    /// If the bound value has a different type than the output value, this function
    /// will panic at runtime.
    pub fn select_or_default<Output>(&mut self) -> Output
    where
        Output: 'static + Default,
    {
        {
            let s = format!("{}", self);
            show(&s, &mut self.stream)
        }
        .and(
            select_once(
                &mut self.stream,
                self.prefix,
                self.default,
                &mut self.fields,
            )
            .into(),
        )
        .unwrap_or_default()
    }

    /// Prompts the selectable menu to the user, then returns the
    /// value he selected, or returns the default value, using the given
    /// `MenuStream`.
    ///
    /// The output value corresponds to the bound value of the field the user selected.
    /// If an error occurred (see [`MenuError`](crate::MenuError)), or if the user
    /// provided an incorrect input, it will return the default value.
    ///
    /// # Panic
    ///
    /// If the bound value has a different type than the output value, this function
    /// will panic at runtime.
    pub fn select_or_default_with<Output>(&mut self, stream: &mut MenuStream<'a, R, W>) -> Output
    where
        Output: 'static + Default,
    {
        self.prompt_or_default(stream)
    }
}

impl<'a, R: 'a, W: 'a> Streamable<'a, R, W> for SelectMenu<'a, R, W> {
    #[inline]
    fn get_stream(self) -> MenuStream<'a, R, W> {
        self.stream.retrieve()
    }

    fn get_stream_ref(&self) -> &MenuStream<'a, R, W> {
        &self.stream
    }

    fn get_stream_ref_mut(&mut self) -> &mut MenuStream<'a, R, W> {
        &mut self.stream
    }
}

impl<R, W> Display for SelectMenu<'_, R, W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // displays fields
        fn list_fields<R, W>(
            fields: &[SelectField<R, W>],
            default: Option<usize>,
            f: &mut Formatter<'_>,
        ) -> fmt::Result {
            if fields.is_empty() {
                return Err(Error);
            }

            for (i, field) in (1..=fields.len()).zip(fields.iter()) {
                writeln!(
                    f,
                    "{x}{msg}{def}",
                    x = i,
                    msg = field,
                    def = if matches!(default, Some(d) if d == i) {
                        " (default)"
                    } else {
                        ""
                    },
                )?;
            }
            Ok(())
        }

        match self.title.pos {
            TitlePos::Top => {
                write!(f, "{}", self.title).and(list_fields(&self.fields, self.default, f))
            }
            TitlePos::Bottom => {
                list_fields(&self.fields, self.default, f).and(writeln!(f, "{}", self.title))
            }
        }
    }
}

// We need to use this function from the owned selectable menu stream,
// or from a provided one, so we don't mutate the whole selectable menu struct.
fn select_once<Output, R, W>(
    stream: &mut MenuStream<'_, R, W>,
    prefix: &str,
    default: Option<usize>,
    fields: &mut Vec<SelectField<R, W>>,
) -> Query<Output>
where
    R: BufRead,
    W: Write,
    Output: 'static,
{
    prompt(prefix, stream)
        .map(|s| {
            parse_value(&s, default).and_then(|idx| match idx.checked_sub(1) {
                Some(i) if fields.get(i).is_some() => fields.remove(i).select(stream),
                _ => Err(MenuError::Select(s)),
            })
        })
        .into()
}

/// Represents a value-menu type, which means a menu that retrieves values from the user inputs.
///
/// The `R` type parameter represents its reader type, and the `W` type parameter means its writer type.
/// By default, it uses the standard input and output streams to get values from the user.
/// It wraps the streams into a [`MenuStream`].
///
/// It has a global formatting applied to the fields it contains by inheritance.
pub struct ValueMenu<'a, R = In, W = Out> {
    title: &'a str,
    fmt: Rc<ValueFieldFormatting<'a>>,
    fields: IntoIter<Field<'a, R, W>>,
    stream: Stream<'a, MenuStream<'a, R, W>>,
    began: bool,
}

impl<'a, const N: usize> From<[Field<'a>; N]> for ValueMenu<'a> {
    /// Instantiate the value-menu from its value-fields array.
    #[inline]
    fn from(fields: [Field<'a>; N]) -> Self {
        Self::from(Vec::from(fields))
    }
}

impl<'a> From<Vec<Field<'a>>> for ValueMenu<'a> {
    #[inline]
    fn from(fields: Vec<Field<'a>>) -> Self {
        Self::with_owned(MenuStream::default(), fields)
    }
}

impl<'a, R, W> ValueMenu<'a, R, W> {
    fn inner_new(stream: Stream<'a, MenuStream<'a, R, W>>, fields: Vec<Field<'a, R, W>>) -> Self {
        // inherits fmt on submenus title
        let fmt: Rc<ValueFieldFormatting> = Rc::default();
        let mut fields = fields.into_iter();
        for field in fields.as_mut_slice() {
            field.inherit_fmt(fmt.clone());
        }

        Self {
            fields,
            title: "",
            fmt,
            stream,
            began: false,
        }
    }

    /// Builds the menu from its owned menu stream, with its fields vector.
    #[inline]
    pub fn with_owned(stream: MenuStream<'a, R, W>, fields: Vec<Field<'a, R, W>>) -> Self {
        Self::inner_new(Stream::Owned(stream), fields)
    }

    /// Builds the menu from a mutable reference of a menu stream, with its fields vector.
    #[inline]
    pub fn with_ref(stream: &'a mut MenuStream<'a, R, W>, fields: Vec<Field<'a, R, W>>) -> Self {
        Self::inner_new(Stream::Borrowed(stream), fields)
    }

    /// Builds the menu from its owned input and output streams, with its fields vector.
    #[inline]
    pub fn new(reader: R, writer: W, fields: Vec<Field<'a, R, W>>) -> Self {
        Self::with_owned(MenuStream::new(reader, writer), fields)
    }

    /// Builds the menu from mutable references of the reader and writer, with its fields vector.
    #[inline]
    pub fn new_ref(reader: &'a mut R, writer: &'a mut W, fields: Vec<Field<'a, R, W>>) -> Self {
        Self::with_owned(MenuStream::with(reader, writer), fields)
    }

    /// Give the global formatting applied to all the fields the menu contains.
    /// If a field has a custom formatting, it will uses the formatting rules of the field
    /// when printing to the writer.
    pub fn fmt(mut self, fmt: ValueFieldFormatting<'a>) -> Self {
        self.fmt = Rc::new(fmt);
        for field in self.fields.as_mut_slice() {
            field.inherit_fmt(self.fmt.clone());
        }
        self
    }

    /// Give the main title of the menu.
    /// It is printed at the beginning, right before the first field.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    #[inline]
    fn next_field(&mut self) -> MenuResult<Field<'a, R, W>> {
        self.fields.next().ok_or(MenuError::EndOfMenu)
    }
}

impl<'a, R, W: Write> ValueMenu<'a, R, W> {
    #[inline]
    fn show_title(&mut self) -> MenuResult<()> {
        show_title(&mut self.stream, self.title, &mut self.began)
    }
}

impl<'a, R, W> ValueMenu<'a, R, W>
where
    R: BufRead,
    W: Write,
{
    /// Returns the next output, if the next output corresponds to an inner selectable menu output.
    ///
    /// If this is the case, it returns the selectable menu output
    /// (See [`<SelectMenu as MenuBuilder>::next_output`](SelectMenu::next_output)).
    ///
    /// ## Panic
    ///
    /// If the next field is not a selectable menu, this function will panic.
    pub fn next_select<Output: 'static>(&mut self) -> MenuResult<Output> {
        self.show_title()?;
        next_select(self.next_field()?, &mut self.stream)
    }

    /// Returns the next output, if the next output corresponds to an inner selectable menu,
    /// using the given menu stream.
    ///
    /// If this is the case, it returns the selectable menu output
    /// (See [`<SelectMenu as MenuBuilder>::next_output`](SelectMenu::next_output)).
    ///
    /// ## Panic
    ///
    /// If the next field is not a selectable menu, this function will panic.
    pub fn next_select_with<Output: 'static>(
        &mut self,
        stream: &mut MenuStream<'a, R, W>,
    ) -> MenuResult<Output> {
        show_title(stream, self.title, &mut self.began)?;
        next_select(self.next_field()?, stream)
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
    pub fn next_value<Output>(&mut self) -> MenuResult<Output>
    where
        Output: 'static + FromStr,
        Output::Err: 'static + Debug,
    {
        self.show_title()?;
        self.next_field()?.prompt(&mut self.stream)
    }

    /// Returns the next output provided by the user, using the given menu stream.
    ///
    /// It prompts the user until the value entered is correct.
    /// If the next field is a selectable menu, it will prompt the selectable menu,
    /// requiring the output type to be `'static`.
    /// Otherwise, it will prompt the value-field, requiring the output type to implement
    /// `FromStr` trait.
    ///
    /// If there is no more field to prompt in the menu, this function will return an error
    /// (see [`MenuError::EndOfMenu`](crate::MenuError::EndOfMenu)).
    pub fn next_value_with<Output>(
        &mut self,
        stream: &mut MenuStream<'a, R, W>,
    ) -> MenuResult<Output>
    where
        Output: 'static + FromStr,
        Output::Err: 'static + Debug,
    {
        show_title(stream, self.title, &mut self.began)?;
        self.next_field()?.prompt(stream)
    }

    /// Returns the next output provided by the user, or its default value if an error occurred.
    ///
    /// It prompts the user once, and if the value entered is incorrect, it returns the
    /// default value.
    ///
    /// If the next field is a selectable menu, it will prompt the selectable menu,
    /// requiring the output type to be `'static`.
    /// Otherwise, it will prompt the value-field, requiring the output type to implement
    /// `FromStr` trait.
    ///
    /// If there is no more field to prompt in the menu, this function will return an error
    /// (see [`MenuError::EndOfMenu`](crate::MenuError::EndOfMenu)).
    pub fn next_value_or_default<Output>(&mut self) -> Output
    where
        Output: 'static + FromStr + Default,
        Output::Err: 'static + Debug,
    {
        self.show_title()
            .and(self.next_field())
            .map(|mut f| f.prompt_or_default(&mut self.stream))
            .unwrap_or_default()
    }

    /// Returns the next output provided by the user, or its default value if an error occurred,
    /// using the given menu stream.
    ///
    /// It prompts the user once, and if the value entered is incorrect, it returns the
    /// default value.
    ///
    /// If the next field is a selectable menu, it will prompt the selectable menu,
    /// requiring the output type to be `'static`.
    /// Otherwise, it will prompt the value-field, requiring the output type to implement
    /// `FromStr` trait.
    ///
    /// If there is no more field to prompt in the menu, this function will return an error
    /// (see [`MenuError::EndOfMenu`](crate::MenuError::EndOfMenu)).
    pub fn next_value_or_default_with<Output>(
        &mut self,
        stream: &mut MenuStream<'a, R, W>,
    ) -> Output
    where
        Output: 'static + FromStr + Default,
        Output::Err: 'static + Debug,
    {
        show_title(stream, self.title, &mut self.began)
            .and(self.next_field())
            .map(|mut f| f.prompt_or_default(stream))
            .unwrap_or_default()
    }

    /// Returns the valid next output, if the next output is not a selectable menu, according
    /// to the given function.
    ///
    /// Read [`ValueField::build_until`](crate::field::ValueField::build_until) for more information.
    ///
    /// ## Panic
    ///
    /// If the next field is not a value-field, this function will panic.
    /// In fact, you can prompt the selectable menu until a given operation is accomplished,
    /// as it implements the [`Promptable`](crate::Promptable) trait
    /// (see [`prompt_until`](crate::Promptable::prompt_until) function), but it is a falsity,
    /// as a selectable menu has to list all the correct values to the user.
    pub fn next_value_until<Output, F>(&mut self, w: F) -> MenuResult<Output>
    where
        Output: 'static + FromStr,
        Output::Err: 'static + Debug,
        F: Fn(&Output) -> bool,
    {
        self.show_title()?;
        self.next_field()?.prompt_until(&mut self.stream, w)
    }

    /// Returns the valid next output, if the next output is not a selectable menu, according
    /// to the given function, using the given stream.
    ///
    /// Read [`ValueField::build_until`](crate::field::ValueField::build_until) for more information.
    ///
    /// ## Panic
    ///
    /// If the next field is not a value-field, this function will panic.
    /// In fact, you can prompt the selectable menu until a given operation is accomplished,
    /// as it implements the [`Promptable`](crate::Promptable) trait
    /// (see [`prompt_until`](crate::Promptable::prompt_until) function), but it is a falsity,
    /// as a selectable menu has to list all the correct values to the user.
    pub fn next_value_until_with<Output, F>(
        &mut self,
        stream: &mut MenuStream<'a, R, W>,
        w: F,
    ) -> MenuResult<Output>
    where
        Output: 'static + FromStr,
        Output::Err: 'static + Debug,
        F: Fn(&Output) -> bool,
    {
        show_title(stream, self.title, &mut self.began)?;
        self.next_field()?.prompt_until(stream, w)
    }
}

fn next_select<'a, Output, R, W>(
    field: Field<'a, R, W>,
    stream: &mut MenuStream<'a, R, W>,
) -> MenuResult<Output>
where
    Output: 'static,
    R: BufRead,
    W: Write,
{
    if let Field::Select(mut s) = field {
        s.prompt(stream)
    } else {
        panic!("next output of the value-menu is not from a selectable menu")
    }
}

impl<R, W> Display for ValueMenu<'_, R, W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self.title, f)
    }
}

fn show_title<R, W>(stream: &mut MenuStream<R, W>, title: &str, began: &mut bool) -> MenuResult<()>
where
    W: Write,
{
    if !*began && !title.is_empty() {
        writeln!(stream, "{}", title)?;
        *began = true;
    }
    Ok(())
}

impl<'a, R, W> Streamable<'a, R, W> for ValueMenu<'a, R, W> {
    /// Returns the menu stream, consuming the menu.
    ///
    /// ## Panic
    ///
    /// If it hasn't been given a menu stream, this method will panic,
    /// because it needs to own its stream to retrieve it at the end.
    #[inline]
    fn get_stream(self) -> MenuStream<'a, R, W> {
        self.stream.retrieve()
    }

    fn get_stream_ref(&self) -> &MenuStream<'a, R, W> {
        &self.stream
    }

    fn get_stream_ref_mut(&mut self) -> &mut MenuStream<'a, R, W> {
        &mut self.stream
    }
}

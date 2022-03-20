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
//!     .default(0)
//!     .title(SelectTitle::from("Select the license type"))),
//! ]);
//!
//! let authors: customs::MenuVec<String> = license.next_output().unwrap();
//! let ty: Type = license.next_select().unwrap();
//! ```

mod stream;

pub use crate::menu::stream::MenuStream;
use crate::menu::stream::Stream;
use crate::prelude::*;
use std::fmt::{self, Debug, Display, Formatter};
use std::io::{BufRead, BufReader, Stdin, Stdout, Write};
use std::rc::Rc;
use std::str::FromStr;
use std::vec::IntoIter;

/// Trait used to return the next output of the menu.
pub trait MenuBuilder<'a, Output, R, W> {
    /// Returns the next output from the menu.
    fn next_output(&mut self) -> MenuResult<Output>;

    /// Returns the next output from the menu using a given menu stream in parameter.
    fn next_output_with(&mut self, stream: &mut MenuStream<'a, R, W>) -> MenuResult<Output>;

    /// Returns the next output from the menu, or its default value if an error occurred.
    fn next_or_default(&mut self) -> Output
    where
        Output: Default,
    {
        self.next_output().unwrap_or_default()
    }

    /// Returns the next output from the menu using a given menu stream in parameter,
    /// or its default value if an error occurred.
    fn next_or_default_with(&mut self, stream: &mut MenuStream<'a, R, W>) -> Output
    where
        Output: Default,
    {
        self.next_output_with(stream).unwrap_or_default()
    }
}

/// The default input stream used by a menu, using the standard input stream.
pub type In = BufReader<Stdin>;

/// The default output stream used by a menu, using the standard output stream.
pub type Out = Stdout;

/// Used to retrieve the stream contained in a menu.
pub trait GetStream<'s, R: 's, W: 's>: Sized {
    /// Returns the menu stream, consuming the menu.
    fn get_stream(self) -> MenuStream<'s, R, W>;

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
/// .next_output();
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
/// .next_output()
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
                new_line: true,
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
            new_line: fmt.new_line,
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

        let disp = format!(
            "{chip}{title}{prefix}",
            chip = self.fmt.chip,
            title = self.inner,
            prefix = self.fmt.prefix,
        );

        if self.fmt.new_line {
            writeln!(f, "{}", disp)
        } else {
            write!(f, "{}", disp)
        }
    }
}

/// Struct modeling a selective menu.
///
/// ## Example
///
/// ```
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
/// .default(0)
/// .next_output()
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
    title: SelectTitle<'a>,
    fields: Vec<SelectField<'a, R, W>>,
    stream: Stream<'a, MenuStream<'a, R, W>>,
    default: Option<usize>,
    prefix: &'a str,
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
            title: Default::default(),
            fields,
            stream,
            default: None,
            prefix: ">> ",
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

    /// Sets the default selective field.
    ///
    /// If you have specified a default field, the latter will be marked as `"(default)"`.
    /// Thus, if the user skips the selective menu (by pressing enter without input), it will return
    /// the default selective field.
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
    fn run_output<Output: 'static>(
        &mut self,
        provided: Option<&mut MenuStream<'a, R, W>>,
    ) -> MenuResult<Output> {
        let s = format!("{}", self);

        let stream = if let Some(stream) = provided {
            stream
        } else {
            &mut self.stream
        };

        stream.write_all(s.as_bytes())?;

        // loops while incorrect input
        loop {
            match select_once(stream, self.prefix, self.default, &mut self.fields) {
                Query::Err(e) => break Err(e),
                Query::Finished(out) => break Ok(out),
                Query::Loop => {
                    if let Some(d) = self.default {
                        break Ok(default_parse(d, &mut self.fields, &mut self.stream)?);
                    }
                }
            }
        }
    }
}

impl<'a, R, W> GetStream<'a, R, W> for SelectMenu<'a, R, W> {
    #[inline]
    fn get_stream(self) -> MenuStream<'a, R, W> {
        self.stream.retrieve()
    }
}

impl<R, W> Display for SelectMenu<'_, R, W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.fields.is_empty() {
            panic!("empty fields vector for the selection menu");
        }

        // displays title at top
        if let TitlePos::Top = self.title.pos {
            write!(f, "{}", self.title)?;
        }

        // displays fields
        for (i, field) in self.fields.iter().enumerate() {
            writeln!(
                f,
                "{i}{msg}{def}",
                i = i + 1,
                msg = field,
                def = if matches!(self.default, Some(d) if d == i) {
                    " (default)"
                } else {
                    ""
                },
            )?;
        }

        // displays title at bottom
        if let TitlePos::Bottom = self.title.pos {
            write!(f, "{}", self.title)?;
        }
        Ok(())
    }
}

#[inline(never)]
fn assert_default_idx(default: usize, max: usize) {
    assert!(
        default < max,
        "incorrect index: length is {}, index is {}",
        max,
        default
    );
}

/// Returns the default value among the fields.
///
/// If the default value index or the aimed default field type is incorrect,
/// it will return an error (See [`MenuError::IncorrectType`]).
fn default_parse<Output, R, W>(
    default: usize,
    fields: &mut Vec<SelectField<'_, R, W>>,
    stream: &mut MenuStream<R, W>,
) -> MenuResult<Output>
where
    Output: 'static,
{
    assert_default_idx(default, fields.len());
    let field = fields.remove(default);
    field.call_bind(stream)?;
    Ok(field.select())
}

impl<'a, Output, R, W> MenuBuilder<'a, Output, R, W> for SelectMenu<'a, R, W>
where
    Output: 'static,
    R: BufRead,
    W: Write,
{
    /// Displays the selective menu to the user, then return the field he selected.
    ///
    /// ## Example
    ///
    /// ```no_run
    /// use std::str::FromStr;
    /// use ezmenulib::prelude::*;
    ///
    /// enum Amount {
    ///     Exact(u8),
    ///     More,
    /// }
    ///
    /// impl FromStr for Amount {
    ///     type Err = MenuError;
    ///
    ///     fn from_str(s: &str) -> Result<Self, Self::Err> {
    ///         match s {
    ///             "one" => Ok(Self::Exact(1)),
    ///             "two" => Ok(Self::Exact(2)),
    ///             "three" => Ok(Self::Exact(3)),
    ///             "more" => Ok(Self::More),
    ///             _ => Err(MenuError::from("no")),
    ///         }
    ///     }
    /// }
    ///
    /// let amount: Amount = SelectMenu::from([
    ///     SelectField::new("one", Amount::Exact(1)),
    ///     SelectField::new("two", Amount::Exact(2)),
    ///     SelectField::new("three", Amount::Exact(3)),
    ///     SelectField::new("more", Amount::More),
    /// ])
    /// .next_output()
    /// .unwrap();
    /// ```
    ///
    /// ## Panic
    ///
    /// This method panics if an incorrect index has been specified as default.
    fn next_output(&mut self) -> MenuResult<Output> {
        self.run_output(None)
    }

    /// Displays the selective menu to the user, then return the field he selected,
    /// using the given menu stream.
    ///
    /// ## Panic
    ///
    /// This method panics if an incorrect index has been specified as default.
    fn next_output_with(&mut self, stream: &mut MenuStream<'a, R, W>) -> MenuResult<Output> {
        self.run_output(Some(stream))
    }

    /// Displays the selective menu to the user, then return the field he selected,
    /// or return the default value of the type specified.
    ///
    /// ## Panic
    ///
    /// This method panics if an incorrect index has been specified as default.
    fn next_or_default(&mut self) -> Output
    where
        Output: Default,
    {
        if {
            let s = format!("{}", self);
            self.stream.write_all(s.as_bytes())
        }
        .is_ok()
        {
            let res: MenuResult<Output> = select_once(
                &mut self.stream,
                self.prefix,
                self.default,
                &mut self.fields,
            )
            .into();
            res.unwrap_or_default()
        } else {
            Output::default()
        }
    }

    /// Displays the selective menu to the user, then return the field he selected,
    /// or return the default value of the type specified, using the given menu stream.
    ///
    /// ## Panic
    ///
    /// This method panics if an incorrect index has been specified as default.
    fn next_or_default_with(&mut self, stream: &mut MenuStream<'a, R, W>) -> Output
    where
        Output: Default,
    {
        if stream.write_all(format!("{}", self).as_bytes()).is_ok() {
            let res: MenuResult<Output> =
                select_once(stream, self.prefix, self.default, &mut self.fields).into();
            res.unwrap_or_default()
        } else {
            Output::default()
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
    fn show<R, W>(prefix: &[u8], stream: &mut MenuStream<R, W>) -> MenuResult<String>
    where
        R: BufRead,
        W: Write,
    {
        stream.write_all(prefix)?;
        stream.flush()?;
        raw_read_input(stream)
    }

    show(prefix.as_bytes(), stream)
        .map(|s| {
            parse_value(&s).and_then(|idx: usize| {
                if idx == 0 {
                    return Err(MenuError::Select(s));
                }

                if fields.get(idx - 1).is_some() {
                    Ok(fields.remove(idx - 1).select())
                } else {
                    if let Some(d) = default {
                        assert_default_idx(d, fields.len());
                        Ok(fields.remove(d).select())
                    } else {
                        Err(MenuError::Select(s))
                    }
                }
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
    popped: bool,
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
    fn inner_new(
        stream: Stream<'a, MenuStream<'a, R, W>>,
        mut fields: Vec<Field<'a, R, W>>,
    ) -> Self {
        let fmt: Rc<ValueFieldFormatting> = Rc::default();

        // inherits fmt on submenus title
        for field in fields.iter_mut() {
            field.inherit_fmt(fmt.clone());
        }

        Self {
            fields: fields.into_iter(),
            title: "",
            fmt,
            stream,
            popped: false,
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
    fn next_field(&mut self) -> Field<'a, R, W> {
        self.fields.next().expect("no more field in the value-menu")
    }
}

impl<'a, R, W: Write> ValueMenu<'a, R, W> {
    #[inline]
    fn show_title(&mut self) -> MenuResult<()> {
        show_title(&mut self.stream, self.title, &mut self.popped)
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
    /// (See [`<SelectMenu as MenuBuilder>::next_output`](<SelectMenu as MenuBuilder>::next_output)).
    ///
    /// ## Panic
    ///
    /// If the next field is not a selectable menu, this function will panic.
    pub fn next_select<Output: 'static>(&mut self) -> MenuResult<Output> {
        self.show_title()?;
        next_select(self.next_field(), &mut self.stream)
    }

    /// Returns the next output, if the next output corresponds to an inner selectable menu,
    /// using the given menu stream.
    ///
    /// If this is the case, it returns the selectable menu output
    /// (See [`<SelectMenu as MenuBuilder>::next_output`](<SelectMenu as MenuBuilder>::next_output)).
    ///
    /// ## Panic
    ///
    /// If the next field is not a selectable menu, this function will panic.
    pub fn next_select_with<Output: 'static>(
        &mut self,
        stream: &mut MenuStream<'a, R, W>,
    ) -> MenuResult<Output> {
        self.show_title()?;
        next_select(self.next_field(), stream)
    }

    /// Returns the valid next output, if the next output is not a selectable menu, according
    /// to the given function.
    ///
    /// Read [`ValueField::build_until`](crate::field::ValueField::build_until) for more information.
    ///
    /// ## Panic
    ///
    /// If the next field is not a value-field, this function will panic.
    pub fn next_output_until<Output, F>(&mut self, w: F) -> MenuResult<Output>
    where
        Output: FromStr,
        Output::Err: 'static + Debug,
        F: Fn(&Output) -> bool,
    {
        self.show_title()?;
        self.next_field().build_until(&mut self.stream, w)
    }

    /// Returns the valid next output, if the next output is not a selectable menu, according
    /// to the given function, using the given stream.
    ///
    /// Read [`ValueField::build_until`](crate::field::ValueField::build_until) for more information.
    ///
    /// ## Panic
    ///
    /// If the next field is not a value-field, this function will panic.
    pub fn next_output_with_until<Output, F>(
        &mut self,
        stream: &mut MenuStream<'a, R, W>,
        w: F,
    ) -> MenuResult<Output>
    where
        Output: FromStr,
        Output::Err: 'static + Debug,
        F: Fn(&Output) -> bool,
    {
        show_title(stream, self.title, &mut self.popped)?;
        self.next_field().build_until(stream, w)
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
        s.next_output_with(stream)
    } else {
        panic!("next output of the value-menu is not from a selectable menu")
    }
}

impl<'a, Output, R, W> MenuBuilder<'a, Output, R, W> for ValueMenu<'a, R, W>
where
    Output: FromStr + 'static,
    Output::Err: 'static + Debug,
    R: BufRead,
    W: Write,
{
    /// Returns the output of the next field if present.
    ///
    /// ## Panic
    ///
    /// This function panics if there is no more field in the value-menu,
    /// or if an incorrect value type has been used as default.
    fn next_output(&mut self) -> MenuResult<Output> {
        self.show_title()?;
        self.next_field().build(&mut self.stream)
    }

    /// Returns the output of the next field if present using the given menu stream.
    ///
    /// ## Panic
    ///
    /// This function panics if there is no more field in the value-menu,
    /// or if an incorrect value type has been used as default.
    fn next_output_with(&mut self, stream: &mut MenuStream<'a, R, W>) -> MenuResult<Output> {
        show_title(stream, self.title, &mut self.popped)?;
        self.next_field().build(stream)
    }

    /// Returns the output of the next field if present or its default type.
    ///
    /// ## Panic
    ///
    /// This function panics if there is no more field in the value-menu,
    /// or if an incorrect value type has been used as default.
    fn next_or_default(&mut self) -> Output
    where
        Output: Default,
    {
        let field = self.next_field();
        next_or_default(&mut self.stream, field, &mut self.popped, self.title)
    }

    /// Returns the output of the next field if present or its default type,
    /// using the given menu stream.
    ///
    /// ## Panic
    ///
    /// This function panics if there is no more field in the value-menu,
    /// or if an incorrect value type has been used as default.
    fn next_or_default_with(&mut self, stream: &mut MenuStream<'a, R, W>) -> Output
    where
        Output: Default,
    {
        let field = self.next_field();
        next_or_default(stream, field, &mut self.popped, self.title)
    }
}

fn next_or_default<'a, Output, R, W>(
    stream: &mut MenuStream<'a, R, W>,
    mut field: Field<'a, R, W>,
    popped: &mut bool,
    title: &str,
) -> Output
where
    Output: FromStr + Default + 'static,
    Output::Err: 'static + Debug,
    W: Write,
    R: BufRead,
{
    if show_title(stream, title, popped).is_ok() {
        field.build_or_default(stream)
    } else {
        Output::default()
    }
}

fn show_title<R, W>(stream: &mut MenuStream<R, W>, title: &str, popped: &mut bool) -> MenuResult<()>
where
    W: Write,
{
    if !*popped && !title.is_empty() {
        writeln!(stream, "{}", title)?;
        *popped = true;
    }
    Ok(())
}

impl<'a, R, W> GetStream<'a, R, W> for ValueMenu<'a, R, W> {
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
}

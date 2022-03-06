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
//! next call of this method will return an error (See [`MenuError::NoMoreField`](crate::MenuError::NoMoreField)).
//!
//! Therefore, a selectable menu can return many times the value selected by the user at different
//! points of the code.
//!
//! ## Example
//!
//! ```
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
//! impl FromStr for Type {
//!     type Err = MenuError;
//!
//!     fn from_str(s: &str) -> MenuResult<Self> {
//!         match s.to_lowercase().as_str() {
//!             "mit" => Ok(Self::MIT),
//!             "gpl" => Ok(Self::GPL),
//!             "bsd" => Ok(Self::BSD),
//!             s => Err(MenuError::from(format!("unknown license type: {}", s))),
//!         }
//!     }
//! }
//!
//! let mut license = ValueMenu::from([
//!     Field::Value(ValueField::from("Authors")),
//!     Field::Select(SelectMenu::from([
//!         SelectField::from("MIT"),
//!         SelectField::from("GPL"),
//!         SelectField::from("BSD"),
//!     ])
//!     .default(0)
//!     .title(SelectTitle::from("Select the license type"))),
//! ]);
//!
//! let authors: customs::MenuVec<String> = license.next_output().unwrap();
//! let ty: Type = license.next_output().unwrap();
//! ```

use crate::prelude::*;
use std::fmt::{self, Debug, Display, Formatter};
use std::io::{stdin, stdout, BufReader, Read, Stdin, Stdout, Write};
use std::rc::Rc;
use std::str::FromStr;
use std::vec::IntoIter;

/// The position of the title for an enum menu.
/// By default, the title position is at the top.
///
/// ## Example
///
/// ```
/// use ezmenulib::prelude::*;
///
/// let amount: MenuResult<u8> = SelectMenu::from([
///     SelectField::from("first"),
///     SelectField::from("second"),
///     SelectField::from("third"),
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
/// impl FromStr for Type {
///     type Err = MenuError;
///
///     fn from_str(s: &str) -> MenuResult<Self> {
///         match s.to_lowercase().as_str() {
///             "mit" => Ok(Self::MIT),
///             "gpl" => Ok(Self::GPL),
///             "bsd" => Ok(Self::BSD),
///             s => Err(MenuError::from(format!("unknown license type: {}", s))),
///         }
///     }
/// }
///
/// let license_type = SelectMenu::from([
///     SelectField::new("MIT"),
///     SelectField::new("GPL"),
///     SelectField::new("BSD"),
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
pub struct SelectMenu<'a, R = Stdin, W = Stdout> {
    title: SelectTitle<'a>,
    fields: Vec<SelectField<'a, W>>,
    reader: BufReader<R>,
    writer: W,
    default: Option<usize>,
    prefix: &'a str,
    raw: bool,
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
///     SelectField::from("yes"),
///     SelectField::from("no"),
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

impl<'a> From<Vec<SelectField<'a, Stdout>>> for SelectMenu<'a> {
    /// Builds the menu from its fields vector.
    #[inline]
    fn from(fields: Vec<SelectField<'a, Stdout>>) -> Self {
        Self::inner_new(stdin(), stdout(), fields, false)
    }
}

impl<'a, const N: usize> From<[SelectField<'a, Stdout>; N]> for SelectMenu<'a> {
    /// Builds the menu from an array of fields.
    #[inline]
    fn from(fields: [SelectField<'a, Stdout>; N]) -> Self {
        Self::from(Vec::from(fields))
    }
}

impl<'a, R, W> SelectMenu<'a, R, W>
where
    R: Read,
{
    fn inner_new(reader: R, writer: W, fields: Vec<SelectField<'a, W>>, raw: bool) -> Self {
        Self {
            title: Default::default(),
            fields,
            reader: BufReader::new(reader),
            writer,
            default: None,
            prefix: ">> ",
            raw,
        }
    }

    /// Builds the menu from its reader and writer streams, and with its fields vector.
    #[inline]
    pub fn new(reader: R, writer: W, fields: Vec<SelectField<'a, W>>) -> Self {
        Self::inner_new(reader, writer, fields, true)
    }
}

impl<'a> SelectMenu<'a> {
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

    #[inline]
    pub(crate) fn inherit_fmt(&mut self, fmt: Rc<ValueFieldFormatting<'a>>) {
        self.title.inherit_fmt(fmt);
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
}

impl<'a, R, W: Write> SelectMenu<'a, R, W> {
    #[inline]
    fn disp_list(&mut self) -> MenuResult<()> {
        self.writer
            .write_all(format!("{}", self).as_bytes())
            .map_err(MenuError::from)
    }
}

impl<'a, R, W> SelectMenu<'a, R, W>
where
    R: Read,
    W: Write,
{
    fn select<Output>(&mut self) -> MenuResult<Output>
    where
        Output: FromStr,
        Output::Err: 'static + Debug,
    {
        // printing prefix
        self.writer.write_all(self.prefix.as_bytes())?;
        self.writer.flush()?;

        let out = raw_read_input(&mut self.reader, &mut self.writer, self.raw)?;

        if let Some(field) = self
            .fields
            .iter()
            .find(|field| field.msg.to_lowercase() == out.to_lowercase())
        {
            // value entered as literal
            match out.parse::<Output>() {
                Ok(out) => {
                    field.call_bind(&mut self.writer)?;
                    Ok(out)
                }
                Err(_) => {
                    if let Some(default) = self.default {
                        default_parse(default, &self.fields, &mut self.writer)
                    } else {
                        Err(MenuError::Select(out))
                    }
                }
            }
        } else {
            // value entered as index
            match out.parse::<usize>() {
                Ok(idx) if idx >= 1 => {
                    if let Some(field) = self.fields.get(idx - 1) {
                        field.call_bind(&mut self.writer)?;
                        field.msg.parse().map_err(err_ty)
                    } else {
                        Err(MenuError::Select(out))
                    }
                }
                Err(_) => {
                    if let Some(default) = self.default {
                        default_parse(default, &self.fields, &mut self.writer)
                    } else {
                        Err(MenuError::Select(out))
                    }
                }
                _ => Err(MenuError::Select(out)),
            }
        }
    }
}

impl<'a, R, W> SelectMenu<'a, R, W> {
    /// Returns its input and output streams, consuming the selectable menu.
    pub fn get_io(self) -> (R, W) {
        (self.reader.into_inner(), self.writer)
    }
}

impl<R, W> Display for SelectMenu<'_, R, W> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // displays title at top
        if let TitlePos::Top = self.title.pos {
            write!(f, "{}", self.title)?;
        }

        // displays fields
        for (i, field) in self.fields.iter().enumerate() {
            writeln!(
                f,
                "{i}{msg} {def}",
                i = i + 1,
                msg = field,
                def = if matches!(self.default, Some(d) if d == i) {
                    "(default)"
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

/// Returns an error meaning that the default selection index is incorrect.
#[inline]
fn err_idx(default: usize, len: usize) -> MenuError {
    MenuError::from(format!(
        "incorrect default value index: index is {} but selective menu length is {}",
        default, len
    ))
}

/// Returns an error meaning that the value type contained in the string slice is incorrect.
#[inline]
fn err_ty<E: 'static + Debug>(e: E) -> MenuError {
    MenuError::from(format!("incorrect default value type: {:?}", e))
}

/// Returns the default value among the fields.
///
/// If the default value index or the aimed default field type is incorrect,
/// it will return an error (See [`MenuError::IncorrectType`]).
fn default_parse<Output, W>(
    default: usize,
    fields: &[SelectField<'_, W>],
    writer: &mut W,
) -> MenuResult<Output>
where
    Output: FromStr,
    Output::Err: 'static + Debug,
{
    let field = fields
        .get(default)
        .ok_or_else(|| err_idx(default, fields.len()))?;
    field.call_bind(writer)?;
    field.msg.parse().map_err(err_ty)
}

impl<Output, R, W> MenuBuilder<Output> for SelectMenu<'_, R, W>
where
    Output: FromStr,
    Output::Err: 'static + Debug,
    R: Read,
    W: Write,
{
    /// Displays the selective menu to the user, then return the field he selected.
    ///
    /// ## Example
    ///
    /// ```
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
    ///     SelectField::from("one"),
    ///     SelectField::from("two"),
    ///     SelectField::from("three"),
    ///     SelectField::from("more"),
    /// ])
    /// .next_output()
    /// .unwrap();
    /// ```
    fn next_output(&mut self) -> MenuResult<Output> {
        self.disp_list()?;

        // loops while incorrect input
        loop {
            match self.select() {
                Ok(out) => break Ok(out),
                Err(_) => {
                    if let Some(default) = self.default {
                        break Ok(default_parse(default, &self.fields, &mut self.writer)?);
                    }
                }
            }
        }
    }

    fn next_or_default(&mut self) -> Output
    where
        Output: Default,
    {
        if self.disp_list().is_ok() {
            self.select().unwrap_or_default()
        } else {
            Output::default()
        }
    }
}

/// Represents a value-menu type, which means a menu that retrieves values from the user inputs.
///
/// The `N` const parameter represents the amount of [`ValueField`](crate::field::ValueField)
/// It has a global formatting applied to the fields it contains by inheritance.
pub struct ValueMenu<'a, R = Stdin, W = Stdout> {
    title: &'a str,
    fmt: Rc<ValueFieldFormatting<'a>>,
    fields: IntoIter<Field<'a>>,
    reader: BufReader<R>,
    writer: W,
    first_popped: bool,
    raw: bool,
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
        Self::inner_new(stdin(), stdout(), fields, false)
    }
}

impl<'a, R, W> ValueMenu<'a, R, W>
where
    R: Read,
{
    fn inner_new(reader: R, writer: W, mut fields: Vec<Field<'a>>, raw: bool) -> Self {
        let fmt: Rc<ValueFieldFormatting> = Rc::default();

        // inherits fmt on submenus title
        for field in fields.iter_mut() {
            field.inherit_fmt(fmt.clone());
        }

        Self {
            fields: fields.into_iter(),
            title: "",
            fmt,
            reader: BufReader::new(reader),
            writer,
            first_popped: false,
            raw,
        }
    }

    /// Builds the menu from its input and output streams, with its fields vector.
    #[inline]
    pub fn new(reader: R, writer: W, fields: Vec<Field<'a>>) -> Self {
        Self::inner_new(reader, writer, fields, true)
    }
}

impl<'a, R, W> ValueMenu<'a, R, W>
where
    W: Write,
{
    fn print_title(&mut self) -> MenuResult<()> {
        if !self.first_popped && !self.title.is_empty() {
            writeln!(self.writer, "{}", self.title)?;
            self.first_popped = true;
        }
        Ok(())
    }
}

impl<'a> ValueMenu<'a> {
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
}

impl<'a, R, W> ValueMenu<'a, R, W> {
    /// Returns its input and output streams, consuming the value-menu.
    pub fn get_io(self) -> (R, W) {
        (self.reader.into_inner(), self.writer)
    }

    fn next_field(&mut self) -> MenuResult<Field<'a>> {
        self.fields.next().ok_or(MenuError::NoMoreField)
    }
}

/// Trait used to return the next output of the menu.
pub trait MenuBuilder<Output> {
    /// Returns the next output from the menu.
    fn next_output(&mut self) -> MenuResult<Output>;

    /// Returns the next output from the menu, or its default value.
    #[inline(always)]
    fn next_or_default(&mut self) -> Output
    where
        Output: Default,
    {
        self.next_output().unwrap_or_default()
    }
}

impl<Output, R, W> MenuBuilder<Output> for ValueMenu<'_, R, W>
where
    Output: FromStr,
    Output::Err: 'static + Debug,
    R: Read,
    W: Write,
{
    /// Returns the output of the next field if present.
    fn next_output(&mut self) -> MenuResult<Output> {
        self.print_title()?;
        self.next_field()?
            .menu_build(&mut self.reader, &mut self.writer, self.raw)
    }

    fn next_or_default(&mut self) -> Output
    where
        Output: Default,
    {
        if self.print_title().is_ok() {
            self.next_field()
                .map(|mut f| f.menu_build_or_default(&mut self.reader, &mut self.writer, self.raw))
                .unwrap_or_default()
        } else {
            Output::default()
        }
    }
}

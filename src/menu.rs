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
//!
//! ## Stdin and Stdout
//!
//! After using a menu, you can drop it and recover the [`Stdin`](std::io::Stdin)
//! and [`Stdout`](std::io::Stdout) instances it used, thank to the [`MenuBuilder::get_io`] method.

use crate::prelude::*;
use std::array::IntoIter;
use std::fmt::{self, Debug, Display, Formatter};
use std::io::{stdin, stdout, Stdin, Stdout, Write};
use std::rc::Rc;
use std::str::FromStr;

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
pub enum TitlePos {
    /// Position at the top of the menu:
    /// ```md
    /// <title>
    /// 1 - field1
    /// 2 - field2
    /// ...
    /// >>
    /// ```
    Top,
    /// Position at the bottom of the menu:
    /// ```md
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
/// The generic type `Output` means the output type of the selective menu, while `N` means the
/// amount of selective fields it contains.
///
/// ## Example
///
/// ```
/// use ezmenulib::prelude::*;
///
/// // Debug and PartialEq trait impl are used for the `assert_eq` macro.
/// #[derive(Clone, Debug, PartialEq)]
/// enum Type {
///     MIT,
///     GPL,
///     BSD,
/// }
///
/// let license_type = SelectMenu::from([
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
/// Supposing the user skipped the selective menu, it will return by default the selective field
/// at index `0`.
/// ```
/// assert_eq!(license_type, Type::MIT);
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
/// Default chip is `" - "`, and default prefix is `">> "`.
pub struct SelectMenu<'a> {
    title: SelectTitle<'a>,
    fields: Vec<SelectField<'a>>,
    reader: Stdin,
    writer: Stdout,
    default: Option<usize>,
    prefix: &'a str,
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

impl<'a> From<Vec<SelectField<'a>>> for SelectMenu<'a> {
    fn from(fields: Vec<SelectField<'a>>) -> Self {
        Self {
            title: Default::default(),
            fields,
            reader: stdin(),
            writer: stdout(),
            default: None,
            prefix: ">> ",
        }
    }
}

impl<'a, const N: usize> From<[SelectField<'a>; N]> for SelectMenu<'a> {
    fn from(fields: [SelectField<'a>; N]) -> Self {
        Self::from(Vec::from(fields))
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

impl Display for SelectMenu<'_> {
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

impl AsRef<Stdout> for SelectMenu<'_> {
    fn as_ref(&self) -> &Stdout {
        &self.writer
    }
}

impl AsMut<Stdout> for SelectMenu<'_> {
    fn as_mut(&mut self) -> &mut Stdout {
        &mut self.writer
    }
}

impl<Output> MenuBuilder<Output> for SelectMenu<'_>
where
    Output: FromStr,
    Output::Err: 'static + Debug,
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
        fn default_parse<Output>(
            default: usize,
            fields: &[SelectField<'_>],
            writer: &mut Stdout,
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

        // displays the menu once
        self.writer.write_all(format!("{}", self).as_bytes())?;

        // loops while incorrect input
        loop {
            // printing prefix
            self.writer.write_all(self.prefix.as_bytes())?;
            self.writer.flush()?;

            // reading input
            let mut out = String::new();
            self.reader.read_line(&mut out)?;
            let out = out.trim();

            if let Some(field) = self
                .fields
                .iter()
                .find(|field| field.msg.to_lowercase() == out.to_lowercase())
            {
                // value entered as literal
                match out.parse::<Output>() {
                    Ok(out) => {
                        break Ok({
                            field.call_bind(&mut self.writer)?;
                            out
                        })
                    }
                    Err(_) => {
                        if let Some(default) = self.default {
                            break default_parse(default, &self.fields, &mut self.writer);
                        }
                    }
                }
            } else {
                // value entered as index
                match out.parse::<usize>() {
                    Ok(idx) if idx >= 1 => {
                        if let Some(field) = self.fields.get(idx - 1) {
                            break {
                                field.call_bind(&mut self.writer)?;
                                field.msg.parse().map_err(err_ty)
                            };
                        }
                    }
                    Err(_) => {
                        if let Some(default) = self.default {
                            break default_parse(default, &self.fields, &mut self.writer);
                        }
                    }
                    _ => continue,
                }
            }
        }
    }

    fn get_io(self) -> (Stdin, Stdout) {
        (self.reader, self.writer)
    }
}

/// Represents a value-menu type, which means a menu that retrieves values from the user inputs.
///
/// The `N` const parameter represents the amount of [`ValueField`](crate::field::ValueField)
/// It has a global formatting applied to the fields it contains by inheritance.
pub struct ValueMenu<'a, const N: usize> {
    title: &'a str,
    fmt: Rc<ValueFieldFormatting<'a>>,
    fields: IntoIter<Field<'a>, N>,
    reader: Stdin,
    writer: Stdout,
    // used to know when to print the title
    first_popped: bool,
}

impl<'a, const N: usize> From<[Field<'a>; N]> for ValueMenu<'a, N> {
    /// Instantiate the value-menu from its value-fields array.
    fn from(mut fields: [Field<'a>; N]) -> Self {
        let fmt: Rc<ValueFieldFormatting> = Rc::default();

        // inherits fmt on submenus title
        for field in fields.iter_mut() {
            field.inherit_fmt(fmt.clone());
        }

        Self {
            fields: fields.into_iter(),
            title: "",
            fmt,
            reader: stdin(),
            writer: stdout(),
            first_popped: false,
        }
    }
}

impl<'a, const N: usize> ValueMenu<'a, N> {
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

/// Trait used to return the next output of the menu.
pub trait MenuBuilder<Output>: AsRef<Stdout> + AsMut<Stdout> {
    /// Returns the next output from the menu.
    fn next_output(&mut self) -> MenuResult<Output>;

    /// Returns the input and output streams of the menu and drops it.
    fn get_io(self) -> (Stdin, Stdout);
}

impl<const N: usize> AsRef<Stdout> for ValueMenu<'_, N> {
    fn as_ref(&self) -> &Stdout {
        &self.writer
    }
}

impl<const N: usize> AsMut<Stdout> for ValueMenu<'_, N> {
    fn as_mut(&mut self) -> &mut Stdout {
        &mut self.writer
    }
}

impl<Output, const N: usize> MenuBuilder<Output> for ValueMenu<'_, N>
where
    Output: FromStr,
    Output::Err: 'static + Debug,
{
    /// Returns the output of the next field if present.
    fn next_output(&mut self) -> MenuResult<Output> {
        // prints the title
        if !self.first_popped && !self.title.is_empty() {
            writeln!(self.writer, "{}", self.title)?;
            self.first_popped = true;
        }

        self.fields
            .next()
            .ok_or(MenuError::NoMoreField)?
            .build(&self.reader, &mut self.writer)
    }

    fn get_io(self) -> (Stdin, Stdout) {
        (self.reader, self.writer)
    }
}

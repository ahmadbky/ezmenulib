use crate::field::{Field, SelectField, ValueFieldFormatting};
use crate::{MenuError, MenuResult};
use std::array::IntoIter;
use std::fmt::{self, Debug, Display, Formatter};
use std::io::{stdin, stdout, Stdin, Stdout, Write};
use std::rc::Rc;
use std::str::FromStr;

/// The position of the title for an enum menu.
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
    /// 1 - field1
    /// 2 - field2
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
/// use ezmenulib::{SelectField, SelectMenu, MenuBuilder};
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
/// .title("License type")
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
    // used for title displaying
    // if used as submenu
    fmt: Rc<ValueFieldFormatting<'a>>,
    custom_fmt: bool,

    title: &'a str,
    pos: TitlePos,
    fields: Vec<SelectField<'a>>,
    reader: Stdin,
    writer: Stdout,
    default: Option<usize>,
    prefix: &'a str,
}

impl<'a> From<Vec<SelectField<'a>>> for SelectMenu<'a> {
    fn from(fields: Vec<SelectField<'a>>) -> Self {
        Self {
            fmt: Rc::new(ValueFieldFormatting {
                chip: "",
                prefix: ":",
                ..Default::default()
            }),
            custom_fmt: false,
            title: "",
            pos: Default::default(),
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
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    /// Gives the formatting to apply to the title.
    pub fn fmt(mut self, fmt: ValueFieldFormatting<'a>) -> Self {
        self.fmt = Rc::new(fmt);
        self.custom_fmt = true;
        self
    }

    pub(crate) fn inherit_fmt(&mut self, fmt: Rc<ValueFieldFormatting<'a>>) {
        if !self.custom_fmt {
            self.fmt = fmt;
        }
    }

    /// Sets the title position of the selective menu.
    ///
    /// The title position is either at the top of the selective fields (by default),
    /// or at the bottom.
    pub fn title_pos(mut self, pos: TitlePos) -> Self {
        self.pos = pos;
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
            if !field.custom_fmt {
                field.chip = chip;
            }
        }
        self
    }
}

impl Display for SelectMenu<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // displays title at top
        if let TitlePos::Top = self.pos {
            disp_title(f, &self.fmt, self.title)?;
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
        if let TitlePos::Bottom = self.pos {
            disp_title(f, &self.fmt, self.title)?;
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
    /// use ezmenulib::{SelectMenu, MenuBuilder};
    ///
    /// let amount = SelectMenu::from([
    ///     
    /// ])
    /// .next_output()
    /// .unwrap();
    /// ```
    // FIXME: i find it really awful
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
        fn default_parse<Output>(default: usize, fields: &[SelectField<'_>]) -> MenuResult<Output>
        where
            Output: FromStr,
            Output::Err: 'static + Debug,
        {
            fields
                .get(default)
                .ok_or_else(|| err_idx(default, fields.len()))?
                .msg
                .parse()
                .map_err(err_ty)
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

            if self
                .fields
                .iter()
                .any(|field| field.msg.to_lowercase() == out.to_lowercase())
            {
                // value entered as literal
                match out.parse::<Output>() {
                    Ok(out) => break Ok(out),
                    Err(_) => {
                        if let Some(default) = self.default {
                            break default_parse(default, &self.fields);
                        }
                    }
                }
            } else {
                // value entered as index
                match out.parse::<usize>() {
                    Ok(idx) if idx >= 1 => {
                        if let Some(field) = self.fields.get(idx - 1) {
                            break Ok(field.msg.parse().map_err(err_ty)?);
                        }
                    }
                    Err(_) => {
                        if let Some(default) = self.default {
                            break default_parse(default, &self.fields);
                        }
                    }
                    _ => continue,
                }
            }
        }
    }
}

#[inline(never)]
fn disp_title(
    f: &mut Formatter<'_>,
    fmt: &Rc<ValueFieldFormatting<'_>>,
    title: &str,
) -> fmt::Result {
    if !title.is_empty() {
        writeln!(
            f,
            "{chp}{ttl}{prfx}",
            chp = fmt.chip,
            ttl = title,
            prfx = fmt.prefix
        )?;
    }
    Ok(())
}

/// Represents a value-menu type, which means a menu that retrieves values from the user inputs.
///
/// The `N` const parameter represents the amount of [`ValueField`]
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
}

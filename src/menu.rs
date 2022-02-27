use crate::field::{SelectField, ValueFieldFormatting};
use crate::{MenuError, MenuResult, ValueField};
use std::array::IntoIter;
use std::fmt::Debug;
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
/// fn main() {
///     let license_type = SelectMenu::from([
///         SelectField::new("MIT", Type::MIT),
///         SelectField::new("GPL", Type::GPL),
///         SelectField::new("BSD", Type::BSD),
///     ])
///     .title("License type")
///     .default(0)
///     .next_output()
///     .unwrap();
/// }
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
pub struct SelectMenu<'a, Output, const N: usize> {
    title: &'a str,
    pos: TitlePos,
    fields: [SelectField<'a, Output>; N],
    reader: Stdin,
    writer: Stdout,
    default: Option<usize>,
    prefix: &'a str,
}

impl<'a, Output, const N: usize> From<[SelectField<'a, Output>; N]> for SelectMenu<'a, Output, N> {
    fn from(fields: [SelectField<'a, Output>; N]) -> Self {
        Self {
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

impl<'a, Output, const N: usize> SelectMenu<'a, Output, N> {
    /// Sets the title of the selective menu.
    ///
    /// The title is by default displayed at the top of the selective fields,
    /// but you can edit this behavior by setting the title position to `TitlePos::Bottom`, with
    /// `SelectMenu::title_pos` method.
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
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

impl<Output, const N: usize> AsRef<Stdout> for SelectMenu<'_, Output, N> {
    fn as_ref(&self) -> &Stdout {
        &self.writer
    }
}

impl<Output, const N: usize> AsMut<Stdout> for SelectMenu<'_, Output, N> {
    fn as_mut(&mut self) -> &mut Stdout {
        &mut self.writer
    }
}

impl<Output, const N: usize> MenuBuilder<Output> for SelectMenu<'_, Output, N>
where
    Output: Clone,
{
    /// Displays the selective menu to the user, then return the field he selected.
    ///
    /// ## Example
    ///
    /// ```
    /// use ezmenulib::{SelectMenu, MenuBuilder};
    ///
    /// fn main() {
    ///     let amount = SelectMenu::from([
    ///         
    ///     ])
    ///     .next_output()
    ///     .unwrap();
    /// }
    /// ```
    fn next_output(&mut self) -> MenuResult<Output> {
        disp_sel_menu(
            &self.pos,
            &mut self.writer,
            self.title,
            self.fields.as_ref(),
            &self.default,
        )?;

        // loops while incorrect input
        loop {
            // printing prefix
            self.writer.write_all(self.prefix.as_bytes())?;
            self.writer.flush()?;

            // reading input
            let mut out = String::new();
            self.reader.read_line(&mut out)?;

            // converts user input into Output type
            match out.trim().parse::<usize>() {
                Ok(n @ 1..=usize::MAX) => {
                    if let Some(sf) = self.fields.get(n - 1) {
                        break Ok(sf.select(&mut self.writer)?);
                    }
                }
                _ => {
                    if let Some(default) = self.default {
                        break Ok(self
                            .fields
                            .get(default)
                            .ok_or(MenuError::IncorrectType(Box::new(format!(
                                "default index is {} but menu length is {}",
                                default, N
                            ))))?
                            .select(&mut self.writer)?);
                    } else {
                        continue;
                    }
                }
            }
        }
    }
}

/// Writes the whole selective menu according to its formatting rules
/// to the writer.
fn disp_sel_menu<Output>(
    pos: &TitlePos,
    writer: &mut Stdout,
    title: &str,
    fields: &[SelectField<'_, Output>],
    default: &Option<usize>,
) -> MenuResult<()> {
    // displays the title at the top
    if let TitlePos::Top = pos {
        disp_title(writer, title)?;
    }

    // displays the select-fields
    for (i, field) in fields.iter().enumerate() {
        disp_sel_field(writer, i, field, matches!(default, Some(d) if *d == i))?;
    }

    // displays the title at the bottom
    if let TitlePos::Bottom = pos {
        disp_title(writer, title)?;
    }

    Ok(())
}

#[inline(never)]
fn disp_title(writer: &mut Stdout, title: &str) -> MenuResult<()> {
    if !title.is_empty() {
        writeln!(writer, "{}", title).map_err(MenuError::from)?;
    }
    Ok(())
}

#[inline(never)]
fn disp_sel_field<Output>(
    writer: &mut Stdout,
    idx: usize,
    field: &SelectField<'_, Output>,
    default: bool,
) -> MenuResult<()> {
    writeln!(
        writer,
        "{i}{msg}{def}",
        i = idx + 1,
        msg = field,
        def = if default { " (default)" } else { "" },
    )
    .map_err(MenuError::from)
}

/// Represents a value-menu type, which means a menu that retrieves values from the user inputs.
///
/// The `N` const parameter represents the amount of [`ValueField`]
/// It has a global formatting applied to the fields it contains by inheritance.
pub struct ValueMenu<'a, const N: usize> {
    title: &'a str,
    fmt: Rc<ValueFieldFormatting<'a>>,
    fields: IntoIter<ValueField<'a>, N>,
    reader: Stdin,
    writer: Stdout,
    // used to know when to print the title
    first_popped: bool,
}

impl<'a, const N: usize> From<[ValueField<'a>; N]> for ValueMenu<'a, N> {
    /// Instantiate the value-menu from its value-fields array.
    fn from(fields: [ValueField<'a>; N]) -> Self {
        Self {
            fields: fields.into_iter(),
            title: "",
            fmt: Rc::default(),
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
            if !field.custom_fmt {
                field.fmt = self.fmt.clone();
            }
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

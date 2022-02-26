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

/// Default position for the menu title is at the top.
impl Default for TitlePos {
    fn default() -> Self {
        Self::Top
    }
}

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
    pub fn title(mut self, title: &'a str) -> Self {
        self.title = title;
        self
    }

    pub fn title_pos(mut self, pos: TitlePos) -> Self {
        self.pos = pos;
        self
    }

    pub fn default(mut self, default: usize) -> Self {
        self.default = Some(default);
        self
    }

    pub fn prefix(mut self, prefix: &'a str) -> Self {
        self.prefix = prefix;
        self
    }

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
    fn next_output(&mut self) -> MenuResult<Output> {
        // displays the title at the top
        if let TitlePos::Top = self.pos {
            disp_title(&mut self.writer, self.title)?;
        }

        // displays the select-fields
        for (i, field) in self.fields.iter().enumerate() {
            disp_select_field(
                &mut self.writer,
                i,
                field,
                matches!(self.default, Some(d) if d == i),
            )?;
        }

        // displays the title at the bottom
        if let TitlePos::Bottom = self.pos {
            disp_title(&mut self.writer, self.title)?;
        }

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
                Ok(n) => {
                    if let Some(SelectField { select, .. }) = self.fields.get(n).cloned() {
                        break Ok(select);
                    }
                }
                _ => {
                    if let Some(default) = self.default {
                        break Ok(self
                            .fields
                            .get(default)
                            .cloned()
                            .ok_or(MenuError::IncorrectType(Box::new(format!(
                                "default index is {} but menu length is {}",
                                default, N
                            ))))?
                            .select);
                    } else {
                        continue;
                    }
                }
            }
        }
    }
}

#[inline(never)]
fn disp_title(writer: &mut Stdout, title: &str) -> MenuResult<()> {
    writeln!(writer, "{}", title).map_err(MenuError::from)
}

#[inline(never)]
fn disp_select_field<Output>(
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
                field.inherit_fmt(self.fmt.clone());
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
        if !self.first_popped {
            writeln!(self.writer, "{}", self.title)?;
            self.first_popped = true;
        }

        self.fields
            .next()
            .ok_or(MenuError::NoMoreField)?
            .build(&self.reader, &mut self.writer)
    }
}

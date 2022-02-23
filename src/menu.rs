use crate::field::ValueFieldFormatting;
use crate::{MenuError, MenuResult, ValueField};
use std::array::IntoIter;
use std::fmt::Debug;
use std::io::{stdin, stdout, Stdin, Stdout, Write};
use std::rc::Rc;
use std::str::FromStr;

/// Represents a value-menu type, which means a menu that retrieves values from the user inputs.
///
/// The `N` const parameter represents the amount of [`ValueField`s](https://docs.rs/ezmenu/latest/ezmenu/struct.ValueField.html)
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
pub trait Menu<Output>: AsRef<Stdout> + AsMut<Stdout>
where
    Output: FromStr,
    Output::Err: Debug,
{
    /// Returns the next output from the reader.
    fn next_output(&mut self) -> MenuResult<Output>;

    /// Returns the value mapped by the function specified as argument.
    ///
    /// The function takes `(Output, &mut Stdout)` as argument, where `Output` is the type of the output.
    ///
    /// It returns a `MenuResult<Output>` to prevent from any error or return a custom error, with:
    /// `MenuError::Other(Box<dyn std::error::Debug>)`.
    fn next_map<F>(&mut self, f: F) -> MenuResult<Output>
    where
        F: FnOnce(Output, &mut Stdout) -> MenuResult<Output>,
    {
        f(self.next_output()?, self.as_mut())
    }
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

impl<Output, const N: usize> Menu<Output> for ValueMenu<'_, N>
where
    Output: FromStr,
    Output::Err: 'static + Debug,
{
    /// Returns the output of the next field if present.
    fn next_output(&mut self) -> MenuResult<Output> {
        // prints the title
        if !self.first_popped {
            let title = self.title.to_owned() + "\n";
            self.writer.write_all(title.as_bytes())?;
            self.first_popped = true;
        }

        self.fields
            .next()
            .ok_or(MenuError::NoMoreField)?
            .build(&self.reader, &mut self.writer)
    }
}

/// The position of the title for an enum menu.
// TODO: implement enum menu to use this
#[allow(unused)]
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

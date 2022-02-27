use crate::{MenuError, MenuResult};
use std::fmt::{self, Debug, Display, Formatter};
use std::io::{stdin, stdout, Stdin, Stdout, Write};
use std::rc::Rc;
use std::str::FromStr;

/// Type used to handle the binding function executed right after
/// the corresponding field has been selected by the user.
pub type Binding = fn(&mut Stdout) -> MenuResult<()>;

/// Struct modeling a field of a selective menu.
///
/// The generic type `Output` means the output type of the selective menu.
/// Unlike [`ValueField`], this struct should not be used alone, without a context.
/// You must instantiate it in an array in the constructor of the [`SelectMenu`](crate::SelectMenu) struct.
///
/// Just like [`ValueFieldFormatting`], it contains an editable `chip` string slice, placed
/// after the selection index (`X`):
/// ```text
/// X<chip><message>
/// ```
///
/// ## Example
///
/// ```
/// use ezmenulib::{SelectField, SelectMenu};
///
/// fn main() {
///     let get_amount = SelectMenu::from([
///         SelectField::new("one", 1),
///         SelectField::new("two", 2),
///     ]);
/// }
/// ```
pub struct SelectField<'a, Output> {
    msg: &'a str,
    pub(crate) chip: &'a str,
    select: Output,
    pub(crate) custom_fmt: bool,
    bind: Option<Binding>,
}

impl<Output> Display for SelectField<'_, Output> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self.chip, f)?;
        Display::fmt(self.msg, f)
    }
}

impl<'a, Output> SelectField<'a, Output> {
    /// Initializes the selection field for the menu.
    ///
    /// This associated function should not be used without context.
    /// The `SelectField` type must be instantiated inside a
    pub fn new(msg: &'a str, select: Output) -> Self {
        Self {
            msg,
            chip: " - ",
            select,
            custom_fmt: false,
            bind: None,
        }
    }

    /// Edits the chip of the selection field.
    ///
    /// The default chip is `" - "`. It includes spaces by default, so you can remove them.
    /// It is placed next right to the selection menu index (`X`):
    /// ```text
    /// X<chip><message>
    /// ```
    pub fn chip(mut self, chip: &'a str) -> Self {
        self.chip = chip;
        self.custom_fmt = true;
        self
    }

    /// Defines the function to execute right after the user selected this field.
    ///
    /// It is optional, and is useful if you have many modes in your program with
    /// procedures defined for each mode.
    ///
    /// The function must take a `&mut Stdout` as parameter and return a `MenuResult<()>`.
    /// The result return is used to spread the error to the point you built the menu.
    ///
    /// If you want to return a string (or static string slice) error, you can use `MenuError::from` associated function.
    /// For an IO Error when using `Stdout` methods, you can map the error type by using `Result::map_err`.
    ///
    /// ## Example
    ///
    /// ```
    /// use ezmenulib::{MenuResult, MenuError, SelectField, SelectMenu};
    /// use std::io::{Stdout, Write};
    ///
    /// fn bsd(e: &mut Stdout) -> MenuResult<()> {
    ///     e.write_all(b"coucou this is from bsd\n").map_err(MenuError::from)
    /// }
    ///
    /// fn main() {
    ///     let selected = SelectMenu::from([
    ///         SelectField::new("MIT", Type::MIT).bind(|_| Ok(println!("you selected MIT!!!"))),
    ///         SelectField::new("BSD", Type::BSD).bind(bsd),
    ///         // ...
    ///     ]);
    /// }
    /// ```
    ///
    /// For other error, you can simply use the `MenuError::Other` variant and box your custom error type.
    pub fn bind(mut self, bind: Binding) -> Self {
        self.bind = Some(bind);
        self
    }
}

impl<'a, Output> SelectField<'a, Output>
where
    Output: Clone,
{
    /// Selects this menu and returns the corresponding output.
    ///
    /// This function is used by the [`SelectMenu`] struct when building the menu.
    /// It calls the inner function binding, then returns a clone of the output.
    pub(crate) fn select(&self, writer: &mut Stdout) -> MenuResult<Output> {
        if let Some(func) = self.bind {
            func(writer)?;
        }
        Ok(self.select.clone())
    }
}

/// Defines the formatting of a value-menu field.
///
/// The final text formatting looks like above:
/// ```md
/// <chip><message>{[ (default: <default>)]}{new line}<prefix>
/// ```
/// where:
/// - `<...>` means a given string slice
/// - `{...}` means the value inside is displayed or not (boolean)
/// - `[...]` means the value inside is displayed if it is available
#[derive(Clone)]
pub struct ValueFieldFormatting<'a> {
    /// The small string slice displayed before the message acting as a list style attribute
    /// (by default set as `* `).
    pub chip: &'a str,
    /// The small string slice displayed before the user input (by default set as `: `).
    pub prefix: &'a str,
    /// Sets the user input with its prefix on a new line or not (by default set as `false`).
    pub new_line: bool,
    /// Display default value or not (by default set as `true`).
    pub default: bool,
}

/// Default formatting for a field is `* ` as a chip and `: ` as prefix.
///
/// This being, the field is printed like above (text between `[` and `]` is optional
/// depending on default value providing:
/// ```md
/// * <message>[ (default: <default>)]:
/// ```
impl<'a> Default for ValueFieldFormatting<'a> {
    fn default() -> Self {
        Self {
            chip: "* ",
            prefix: ": ",
            new_line: false,
            default: true,
        }
    }
}

/// Defines behavior for a value-menu field.
///
/// It manages
/// - its message
/// - its formatting
/// - the return value type
/// - the default value
///
/// # Examples
///
/// For a make-license CLI program for example, you can use
/// ```
/// use ezmenulib::ValueField;
/// let author: String = ValueField::from("Give the author of the license")
///     .init_build()
///     .unwrap();
/// ```
pub struct ValueField<'a> {
    msg: &'a str,
    // pointer to the parent fmt or its own fmt
    pub(crate) fmt: Rc<ValueFieldFormatting<'a>>,
    pub(crate) custom_fmt: bool,
    default: Option<&'a str>,
}

impl<'a> From<&'a str> for ValueField<'a> {
    fn from(msg: &'a str) -> Self {
        Self {
            msg,
            fmt: Rc::default(),
            custom_fmt: false,
            default: None,
        }
    }
}

impl<'a> Display for ValueField<'a> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{chip}{msg}{def}{nl}{prefix}",
            chip = self.fmt.chip,
            msg = self.msg,
            def = match self.default {
                Some(val) if self.fmt.default => format!(" (default: {})", val),
                _ => "".to_owned(),
            },
            nl = if self.fmt.new_line { "\n" } else { "" },
            prefix = self.fmt.prefix,
        )
    }
}

/// Constructor methods defining how the field behaves
impl<'a> ValueField<'a> {
    /// Give a custom formatting for the field.
    pub fn fmt(mut self, fmt: ValueFieldFormatting<'a>) -> Self {
        self.fmt = Rc::new(fmt);
        self.custom_fmt = true;
        self
    }

    /// Give the default value accepted by the field.
    ///
    /// If the value type is incorrect, the `ValueField::build` or `ValueField::init_build`
    /// method will return an `Err` value emphasizing that the default value type is incorrect.
    ///
    /// So when instantiating the field with a provided default value, it will not panic and it will
    /// print the default value even if it is incorrect (if the format rule `default` is set to `true`).
    pub fn default(mut self, default: &'a str) -> Self {
        self.default = Some(default);
        self
    }

    /// Builds the field without specifying standard input and output files.
    ///
    /// It initializes instance of `Stdin` and `Stdout`.
    ///
    /// ## Example
    ///
    /// ```
    /// use ezmenulib::{MenuResult, ValueField};
    /// let age: MenuResult<u8> = ValueField::from("How old are you")
    ///     .init_build();
    /// ```
    #[inline]
    pub fn init_build<T>(&self) -> MenuResult<T>
    where
        T: FromStr,
        T::Err: 'static + Debug,
    {
        self.build(&stdin(), &mut stdout())
    }

    /// Builds the field. It prints the message according to its formatting,
    /// then returns the corresponding value.
    ///
    /// You need to provide the reader for the input, and the writer for the output.
    /// It returns a MenuResult if there was an IO error, or if the default value type is incorrect.
    ///
    /// # Example
    ///
    /// Supposing you have declared your own `Stdin` and `Stdout` in your program, you can do so:
    /// ```
    /// use std::io::{stdin, stdout};
    /// let stdin = stdin();
    /// let mut stdout = stdout();
    ///
    /// let author: String = ValueField::from("author")
    ///     .build(&stdin(), &mut stdout())
    ///     .unwrap();
    /// ```
    pub fn build<T>(&self, reader: &Stdin, writer: &mut Stdout) -> MenuResult<T>
    where
        T: FromStr,
        T::Err: 'static + Debug,
    {
        /// Function that parses the default value without checking.
        /// It it useful to break the loop if there is some default value,
        /// and no value was provided, or if the value is incorrect.
        fn default_parse<T>(default: Option<&str>) -> MenuResult<T>
        where
            T: FromStr,
            <T as FromStr>::Err: 'static + Debug,
        {
            let default = default.unwrap();
            default
                .parse()
                .map_err(|e| MenuError::IncorrectType(Box::new(e)))
        }

        // loops while incorrect value
        loop {
            // outputs the field message with its formatting
            // see `<ValueField as Display>::fmt`
            write!(writer, "{}", self)?;
            writer.flush()?;

            // read input to string
            let mut out = String::new();
            reader.read_line(&mut out)?;

            // try to parse to T, else repeat
            match out.trim().parse() {
                Ok(t) => break Ok(t),
                Err(_) if self.default.is_some() => break default_parse(self.default),
                _ => continue,
            }
        }
    }
}

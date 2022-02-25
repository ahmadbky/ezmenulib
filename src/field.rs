use crate::{MenuError, MenuResult};
use std::fmt::Debug;
use std::io::{stdin, stdout, Stdin, Stdout, Write};
use std::rc::Rc;
use std::str::FromStr;

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
    fmt: Rc<ValueFieldFormatting<'a>>,
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

/// Constructor methods defining how the field behaves
impl<'a> ValueField<'a> {
    /// Give a custom formatting for the field.
    pub fn fmt(mut self, fmt: ValueFieldFormatting<'a>) -> Self {
        self.fmt = Rc::new(fmt);
        self.custom_fmt = true;
        self
    }

    /// Method used to make the field inherit from the formatting rules from a parent attribute
    /// such as a `ValueMenu`.
    pub(crate) fn inherit_fmt(&mut self, fmt: Rc<ValueFieldFormatting<'a>>) {
        self.fmt = fmt;
        self.custom_fmt = false;
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
            prompt_fmt(writer, self.msg, self.default, self.fmt.as_ref())?;

            // read input to string
            let mut out = String::new();
            reader.read_line(&mut out)?;

            // try to parse to T, else repeat
            match out.trim().parse() {
                Ok(t) => break Ok(t),
                // TODO: feature allowing control over default values behavior
                Err(_) if self.default.is_some() => break default_parse(self.default),
                _ => continue,
            }
        }
    }
}

/// Prompts an input according to the formatting options of the field
fn prompt_fmt<W: Write>(
    writer: &mut W,
    msg: &str,
    default: Option<&str>,
    fmt: &ValueFieldFormatting<'_>,
) -> Result<(), std::io::Error> {
    write!(
        writer,
        "{chip}{msg}{def}{nl}{prefix}",
        chip = fmt.chip,
        msg = msg,
        def = match default {
            Some(val) if fmt.default => format!(" (default: {})", val),
            _ => "".to_owned(),
        },
        nl = if fmt.new_line { "\n" } else { "" },
        prefix = fmt.prefix,
    )?;
    // flushes writer so it prints the message if it's on the same line
    writer.flush()
}

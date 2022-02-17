use crate::{MenuError, MenuResult};
use std::fmt::Debug;
use std::io::{stdin, stdout, Stdin, Stdout, Write};
use std::str::FromStr;

/// Defines the formatting of a menu field.
///
/// It manages:
/// - the chip: the small string displayed before the message acting as a list style attribute
/// - the prefix: the small string displayed before the user input
/// - new line: sets the user input with its prefix on a new line or not (by default no)
/// - default: display default value or not (by default yes)
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
pub struct StructFieldFormatting<'a> {
    pub chip: &'a str,
    pub prefix: &'a str,
    pub new_line: bool,
    pub default: bool,
}

/// Default formatting for a field is "* " as a chip and ": " as prefix.
/// This being, the field is printed like above (text between `[` and `]` is optional
/// depending on default value providing:
/// ```md
/// * <message>[ (default: <default>)]:
/// ```
impl<'a> Default for StructFieldFormatting<'a> {
    fn default() -> Self {
        Self {
            chip: "* ",
            prefix: ": ",
            new_line: false,
            default: true,
        }
    }
}

/// Defines behavior for a menu field.
///
/// It manages
/// - its message
/// - its formatting
/// - the return value type
/// - the default value
/// - the function to execute after the user provided a value
///
/// # Examples
///
/// For a make-license CLI program for example, you can use
/// ```
/// use std::io::{stdin, stdout};
/// let author: String = StructField::from("Give the author of the license")
///     .build(&stdin(), &mut stdout()).unwrap();
/// ```
pub struct StructField<'a> {
    msg: &'a str,
    // FIXME: use for<'m: 'a> Cow<'m, StructFieldFormatting<'a>>
    // for arrangement between inherited or owned fmt
    // and memory space optimization.
    // problem is that we can't give an immutable reference from a StructMenu
    // because it needs to be mutable so it can fill its inner fields VecDeque :/
    // for now we're cloning it.
    fmt: StructFieldFormatting<'a>,
    // used by StructMenu to know if we inherit fmt or not
    pub(crate) custom_fmt: bool,
    default: Option<&'a str>,
}

impl<'a> From<&'a str> for StructField<'a> {
    fn from(msg: &'a str) -> Self {
        Self {
            msg,
            fmt: Default::default(),
            custom_fmt: false,
            default: None,
        }
    }
}

/// Constructor methods defining how the field behaves
impl<'a> StructField<'a> {
    /// Give a custom formatting for the field.
    pub fn fmt(mut self, fmt: StructFieldFormatting<'a>) -> Self {
        self.fmt = fmt;
        self.custom_fmt = true;
        self
    }

    /// Give an inherited fmt from a menu for example.
    /// In the future versions, this method will take a reference as parameter.
    pub fn inherit_fmt(mut self, fmt: StructFieldFormatting<'a>) -> Self {
        self.fmt = fmt;
        self.custom_fmt = false;
        self
    }

    /// Give the default value accepted by the field.
    /// If the value type is incorrect, the `StructField::build` method will return an `Err` value,
    /// emphasizing that the default value type is incorrect.
    pub fn default(mut self, default: &'a str) -> Self {
        self.default = Some(default);
        self
    }

    /// Builds the field without specifying input and output streams.
    /// It initializes instance of `Stdin` and `Stdout`.
    ///
    /// ## Example
    ///
    /// ```
    /// use ezmenu::{MenuResult, StructField};
    /// let age: MenuResult<u8> = StructField::from("How old are you")
    ///     .init_build();
    /// ```
    #[inline]
    pub fn init_build<T>(self) -> MenuResult<T>
    where
        T: FromStr,
        T::Err: 'static + Debug,
    {
        self.build(&stdin(), &mut stdout())
    }

    /// Builds the field without specifying input and output streams,
    /// and return directly the value provided.
    ///
    /// ## Panic
    ///
    /// This method panics if an error occurred. (See [`MenuError`](https://docs.rs/ezmenu/latest/ezmenu/enum.MenuError.html))
    ///
    /// ## Example
    ///
    /// ```
    /// use ezmenu::StructField;
    /// let age: u8 = StructField::from("How old are you?").unwrap_init_build();
    /// ```
    #[inline]
    pub fn unwrap_init_build<T>(self) -> T
    where
        T: FromStr,
        T::Err: 'static + Debug,
    {
        self.init_build()
            .expect("An error occurred while prompting a value")
    }

    /// Builds the field by specifying input and output streams.
    /// It directly returns the value provided.
    ///
    /// ## Panic
    ///
    /// This method panics if an error occurred while prompting the field.
    /// (See [`MenuError`](https://docs.rs/ezmenu/latest/ezmenu/enum.MenuError.html))
    ///
    /// ## Example
    ///
    /// ```
    /// use std::io::{stdin, stdout};
    /// use ezmenu::{MenuResult, StructField};
    /// let stdin = stdin();
    /// let mut stdout = stdout();
    /// let age: MenuResult<u8>  = StructField::from("How old are you?")
    ///     .unwrap_build(&stdin, &mut stdout);
    /// ```
    #[inline]
    pub fn unwrap_build<T>(self, reader: &Stdin, writer: &mut Stdout) -> T
    where
        T: FromStr,
        T::Err: 'static + Debug,
    {
        self.build(reader, writer)
            .expect("An error occurred while prompting a value")
    }

    /// Builds the field.
    /// It prints the message according to its formatting, then returns the corresponding value.
    /// You need to provide the reader for the input, and the writer for the output.
    /// It returns a MenuResult if there was an IO error, or if the default value type is incorrect.
    ///
    /// # Examples
    ///
    /// If you want to use the standard input and output stream,
    /// you can do so:
    /// ```
    /// use std::io::{stdin, stdout};
    /// let author: String = Field::from("author")
    ///     .build(&stdin(), &mut stdout()).unwrap();
    /// ```
    pub fn build<T>(self, reader: &Stdin, writer: &mut Stdout) -> MenuResult<T>
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
            prompt_fmt(writer, self.msg, self.default, &self.fmt)?;

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
    fmt: &StructFieldFormatting<'_>,
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

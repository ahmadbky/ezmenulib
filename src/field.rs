#[cfg(test)]
#[path = "tests/field.rs"]
mod field;

use std::fmt::Debug;
use std::io::{BufRead, Write};
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
/// let author: String = Field::from("Give the author of the license")
///     .build(&mut stdin().lock(), &mut stdout());
/// ```
pub struct StructField<'a> {
    msg: &'a str,
    // TODO: use Cow<'m, StructFieldFormatting<'a>>
    // for arrangement between inherited or owned fmt
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
    /// If the value type is incorrect, the program will panic at the building.
    pub fn default(mut self, default: &'a str) -> Self {
        self.default = Some(default);
        self
    }

    // TODO: use "custom_io" feature to enable polymorphic IO
    /// Builds the field.
    /// It prints the message according to its formatting, then returns the corresponding value.
    /// You need to provide the reader for the input, and the writer for the output.
    ///
    /// # Examples
    ///
    /// If you want to use the standard input and output stream,
    /// you can do so:
    /// ```
    /// use std::io::{stdin, stdout};
    /// let author: String = Field::from("author")
    ///     .build(&mut stdin().lock(), &mut stdout());
    /// ```
    /// The `lock()` method returns an `impl BufRead` for monomorphism purposes.
    ///
    /// # Panics
    ///
    /// This method may panic if the default value type is invalid, or if
    /// it has encountered an IO problem.
    pub fn build<T, R, W>(self, reader: &mut R, writer: &mut W) -> T
    where
        R: BufRead,
        W: Write,
        T: FromStr,
        <T as FromStr>::Err: Debug,
    {
        /// Function that parses the default value without checking.
        /// It it useful to break the loop if there is some default value,
        /// and no value was provided, or if the value is incorrect.
        #[inline]
        fn unchecked_default_parse<T>(default: Option<&str>) -> T
        where
            T: FromStr,
            <T as FromStr>::Err: Debug,
        {
            let default = default.unwrap();
            default
                .parse()
                .unwrap_or_else(|e| panic!("Invalid default value `{}`. Error: {:?}", default, e))
        }

        // loops while incorrect value
        loop {
            prompt_fmt(writer, self.msg, self.default, &self.fmt)
                .unwrap_or_else(|e| panic!("An error occurred while prompting a value: {:?}", e));

            // read input to string
            let mut out = String::new();
            reader.read_line(&mut out).expect("Unable to read line");

            let out = out.trim();

            // if testing, append input to output
            if cfg!(test) {
                writeln!(writer, "{}", out)
                    .expect("An error occurred while appending input to output for testing");
            }

            if out.is_empty() && self.default.is_some() {
                break unchecked_default_parse(self.default);
            }

            // try to parse to T, else repeat
            match out.parse() {
                Ok(t) => break t,
                // TODO: feature allowing control over default values behavior
                Err(_) if self.default.is_some() => break unchecked_default_parse(self.default),
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

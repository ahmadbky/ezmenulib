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
struct FieldFormatting<'a> {
    chip: Option<&'a str>,
    prefix: Option<&'a str>,
    new_line: bool,
    default: bool,
}

/// Default formatting for a field is "* " as a chip and ": " as prefix.
/// This being, the field is printed like above (text between `[` and `]` is optional
/// depending on default value providing:
/// ```md
/// * <message>[ (default: <default>)]:
/// ```
impl<'a> Default for FieldFormatting<'a> {
    fn default() -> Self {
        Self {
            chip: Some("* "),
            prefix: Some(": "),
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
/// ## Example
///
/// For a make-license CLI program for example, you can use
/// ```rust
/// use std::io::{stdin, stdout};
/// let author: String = Field::from("Give the author of the license")
///     .then(|s, _| println!("what a beautiful name!"))
///     .build(&mut stdin().lock(), &mut stdout());
/// ```
pub struct Field<'a, T> {
    msg: &'a str,
    fmt: FieldFormatting<'a>,
    default: Option<&'a str>,
    then: Option<Box<dyn FnOnce(&T, &mut dyn Write)>>,
}

impl<'a, T> From<&'a str> for Field<'a, T> {
    fn from(msg: &'a str) -> Self {
        Self {
            msg,
            fmt: Default::default(),
            default: None,
            then: None,
        }
    }
}

/// Constructor methods defining how the field behaves
impl<'a, T> Field<'a, T> {
    /// Define the chip at the left of the message displayed
    /// (default: "* ").
    /// The chip is the small string that acts as a list style attribute.
    /// If you don't want any chip for your prompt, you can pass None as value.
    pub fn chip(mut self, chip: Option<&'a str>) -> Self {
        self.fmt.chip = chip;
        self
    }

    /// Define the prefix for the prompt (default: ": ").
    /// The prefix is displayed just before the user input.
    /// If you don't want any prefix for you prompt, you can pass None as value.
    pub fn prefix(mut self, prefix: Option<&'a str>) -> Self {
        self.fmt.prefix = prefix;
        self
    }

    /// Set the prefix and user input on a new line (false by default).
    pub fn new_line(mut self, new_line: bool) -> Self {
        self.fmt.new_line = new_line;
        self
    }

    /// Displays default values or not (true by default).
    /// Default values are displayed after the message like this:
    /// ```md
    /// <message> (default: <default>)
    /// ```
    pub fn display_default(mut self, default: bool) -> Self {
        self.fmt.default = default;
        self
    }

    /// Sets the default value as a &str.
    /// If a default value is provided, it will, by default, by displayed with the message.
    /// For example, this code:
    /// ```rust
    /// let field = Field::from("give a number")
    ///     .default_value("0");
    /// ```
    /// will display the field like below:
    /// ```md
    /// - give a number (default: 0):
    /// ```
    /// If you want to disable this behavior, you can set "display_default" method to false:
    /// ```rust
    /// let field = Field::from("give a number")
    ///     .default_value("0")
    ///     .display_default(false);
    /// ```
    /// And the default value will be skipped at printing, but still available
    /// if the user hasn't provided any correct value.
    // TODO: use a T type for default value instead of &str
    pub fn default_value(mut self, default: &'a str) -> Self {
        self.default = Some(default);
        self
    }

    /// Sets the function to call once, after the user provided an input.
    /// This function takes :
    /// - a `&T` depending of the value type
    /// - a `&mut dyn std::io::Write`, to write the output on it
    ///
    /// ## Example
    ///
    /// Here is an example of how to implement the `then` function:
    /// ```rust
    /// let field = Field::from("author")
    ///     .then(|s: &String, w| {
    ///         if s == "ahmed" {
    ///             writeln!(w, "what a beautiful name");
    ///         }
    ///     });
    /// ```
    ///
    /// ## Behavior
    ///
    /// The function is also called if the field contains a default value,
    /// and the user did not provide any value, or provided an incorrect one.
    pub fn then<F: 'static + FnOnce(&T, &mut dyn Write)>(mut self, then: F) -> Self {
        self.then = Some(Box::new(then));
        self
    }
}

impl<'a, T> Field<'a, T>
where
    T: FromStr,
    <T as FromStr>::Err: Debug,
{
    /// Builds the field.
    /// It prints the message according to its formatting, then returns the corresponding value.
    /// You need to provide the reader for the input, and the writer for the output.
    ///
    /// If you want to use the standard input and output stream,
    /// you can do so:
    /// ```rust
    /// use std::io::{stdin, stdout};
    /// let author: String = Field::from("author")
    ///     .build(&mut stdin().lock(), &mut stdout());
    /// ```
    /// The `lock()` method returns an `impl BufRead` for monomorphism purposes.
    /// Then, you don't need to use the `&mut dyn Write` parameter for you `then` function.
    /// You can directly use the `println!` macro.
    pub fn build<R, W>(self, reader: &mut R, writer: &mut W) -> T
    where
        R: BufRead,
        W: Write,
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

            // if testing, append input to output
            if cfg!(test) {
                writeln!(writer, "{}", out)
                    .expect("An error occurred while appending input to output for testing");
            }

            let out = out.trim();

            if out.is_empty() && self.default.is_some() {
                let out = unchecked_default_parse(self.default);
                if let Some(func) = self.then {
                    func(&out, writer);
                }
                break out;
            }

            // try to parse to T, else repeat
            match out.parse() {
                Ok(t) => {
                    if let Some(func) = self.then {
                        func(&t, writer);
                    }
                    break t;
                }
                // TODO: feature allowing control over default values behavior
                Err(_) if self.default.is_some() => {
                    let out = unchecked_default_parse(self.default);
                    if let Some(func) = self.then {
                        func(&out, writer);
                    }
                    break out;
                }
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
    fmt: &FieldFormatting<'_>,
) -> Result<(), std::io::Error> {
    write!(
        writer,
        "{chip}{msg}{def}{nl}{prefix}",
        chip = fmt.chip.unwrap_or(""),
        msg = msg,
        def = match default {
            Some(val) if fmt.default => format!(" (default: {}", val),
            _ => "".to_owned(),
        },
        nl = if fmt.new_line { "\n" } else { "" },
        prefix = fmt.prefix.unwrap_or("")
    )?;
    // flushes writer so it prints the message if it's on the same line
    writer.flush()
}

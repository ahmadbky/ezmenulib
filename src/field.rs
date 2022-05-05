//! Module that defines several types about retrieving values from the user.

#[cfg(test)]
mod tests;

use crate::prelude::*;
use crate::utils::*;
use crate::DEFAULT_FMT;
use std::env;
use std::fmt::{self, Display, Formatter};
use std::io::{BufRead, Write};
use std::str::FromStr;

/// Builds the associated functions of the [`Format`] struct
/// according to its fields.
macro_rules! impl_fmt {
    ($(#[doc = $main_doc:expr])*
    $(
        $i:ident: $t:ty,
        $(#[doc = $doc:expr])*
    )*) => {
        $(#[doc = $main_doc])*
        #[derive(Clone)]
        pub struct Format<'a> {$(
            $(#[doc = $doc])*
            pub $i: $t,
        )*}

        impl<'a> Format<'a> {
            /// Returns a merged version of the format between `self` and `r`.
            ///
            /// The merged version saves the custom formatting specifications of `self`.
            /// If it a specification corresponds to the default specification
            /// (see [`Format::default`]), for instance `prefix`, it will be replaced
            /// by the `r` specification of `prefix`.
            pub(crate) fn merged(&self, r: &Format<'a>) -> Self {
                Self {$(
                    $i: {
                        match self.$i == DEFAULT_FMT.$i {
                            true => r.$i,
                            false => self.$i,
                        }
                    },
                )*}
            }

            pub(crate) fn merge(&mut self, r: &Format<'a>) {
                *self = self.merged(r);
            }

            // Constructors
            $(
            $(#[doc = $doc])*
            pub fn $i($i: $t) -> Self {
                Self {
                    $i,
                    ..Default::default()
                }
            }
            )*
        }
    }
}

impl_fmt!(
    /// Defines the formatting of a value-menu field.
    ///
    /// The final text format for a written field looks literally like above:
    /// ```md
    /// <prefix><message>[ ({[default: <default>]}, [example: <example>])]{\n}<suffix>
    /// ```
    /// For a selectable value, it looks like above:
    /// ```md
    /// <prefix><message>
    /// X<chip><field message>{[ (default)]}
    /// X<chip><field message>{[ (default)]}
    /// ...
    /// <suffix>
    /// ```
    /// where:
    /// - `<...>` means a given string slice.
    /// - `{...}` means that the value inside is chose to be displayed or not (boolean).
    /// - `[...]` means that the value inside is displayed if it is available.
    prefix: &'a str,
    /// Sets the prefix of the formatting (`"--> "` by default).
    ///
    /// It corresponds to the string slice displayed at the beginning of the field message.
    chip: &'a str,
    /// Defines the chip as marker type for lists (`" - "` by default).
    ///
    /// It is displayed between the index and the field message among the selectable fields.
    show_default: bool,
    /// Defines if it displays the default value or not (`true` by default).
    ///
    /// If an example is provided in the current written field,
    /// the latter will always be displayed.
    suffix: &'a str,
    /// Sets the prefix of the formatting (`">> "` by default).
    ///
    /// It is displayed right before the user input, on the same line.
    line_brk: bool,
    /// Defines if it breaks the line right before the suffix (`true` by default).
    ///
    /// If it does, re-prompting the field will not display the message again,
    /// but only the suffix. Otherwise, because it is on the same line, it will display
    /// the whole message again.
    ///
    /// For selectable fields, if `new_line` format specification is set as `false`,
    /// it will use the default suffix, and always use a line break, for more convenience.
);

/// Default formatting for a field is `"--> "` as a chip and `">> "` as prefix.
///
/// This being, the field is printed like above (text between `[` and `]` is optional
/// depending on default value providing:
/// ```md
/// * <message>[ (default: <default>)]:
/// ```
impl<'a> Default for Format<'a> {
    fn default() -> Self {
        DEFAULT_FMT
    }
}

struct WrittenDetails<'a> {
    example: Option<&'a str>,
    default: Option<String>,
}

impl Display for WrittenDetails<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        // Either `opt` is true or `show_d` is true, but never both,
        // because it is checked by the `Written::fmt_with` method.

        // `true` if asked to print the "optional" string slice.
        let opt = f.alternate();

        // `true` if asked to print the "default" string slice.
        let show_d = f.sign_plus() && self.default.is_some();

        if !opt && !show_d && self.example.is_none() {
            return Ok(());
        }

        // The previous condition guarantees that there is at least
        // something to write inside the parenthesis.
        f.write_str(" (")?;

        // - Example
        if let Some(e) = self.example {
            write!(f, "example: {}", e)?;
            if show_d || opt {
                f.write_str(", ")?;
            }
        }

        // - Default
        match self.default {
            Some(ref d) if show_d => write!(f, "default: {}", d)?,
            _ => (),
        }

        // - Optional
        // We don't check if `show_d` is false
        // because this is done by the `Written::fmt_with` method.
        if opt {
            f.write_str("optional")?;
        }

        f.write_str(")")
    }
}

/// Defines the behavior for a written value provided by the user.
///
/// Like the [selected](Selected) values, it contains its own [format](Format),
/// and it can be inherited, saving the custom format specifications.
///
/// It displays the message, with a given example and default value if it is provided
/// (see [`Written::example`] or [`Written::default_value`] functions).
///
/// It provides functions to define how to retrieve the value from the user.
/// You have to provide a mutable reference to a [`MenuStream`] to retrieve the value.
///
/// # Example
///
/// For a make-license CLI program for example, you can use it like below:
///
/// ```no_run
/// use ezmenulib::{
///     prelude::*,
///     customs::MenuVec,
/// };
/// let author: MenuVec<String> = Written::from("Give the author of the license")
///     .prompt(&mut MenuStream::default())
///     .unwrap();
/// ```
pub struct Written<'a> {
    msg: &'a str,
    /// The format of the written field value.
    pub fmt: Format<'a>,
    details: WrittenDetails<'a>,
}

impl<'a> From<&'a str> for Written<'a> {
    fn from(msg: &'a str) -> Self {
        Self {
            msg,
            fmt: Format::default(),
            details: WrittenDetails {
                example: None,
                default: None,
            },
        }
    }
}

impl Display for Written<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.fmt_with(f, &self.fmt, false)?;
        f.write_str(match self.fmt.line_brk {
            true => "\n",
            false => self.fmt.suffix,
        })
    }
}

/// Constructor methods defining how the field behaves
impl<'a> Written<'a> {
    /// Displays the message of the written field with a given prefix.
    ///
    /// This is used to prompt the written field with a given [`Format`]
    /// (see [`Written::prompt_with`] function for example).
    fn fmt_with<S: fmt::Write>(&self, s: &mut S, fmt: &Format<'_>, opt: bool) -> fmt::Result {
        s.write_str(fmt.prefix)?;
        s.write_str(self.msg)?;

        match (opt, fmt.show_default, self.details.default.as_ref()) {
            (_, true, Some(_)) => write!(s, "{:+}", self.details)?,
            (true, _, _) | (false, false, Some(_)) => write!(s, "{:#}", self.details)?,
            (false, _, None) => write!(s, "{}", self.details)?,
        };

        match fmt.line_brk {
            true => s.write_char('\n'),
            false => Ok(()),
        }
    }

    /// Displays the second line according to the format, and returns the output
    /// of the prompt.
    fn prompt_line<R: BufRead, W: Write>(
        &self,
        stream: &mut MenuStream<R, W>,
        fmt: &Format<'_>,
        opt: bool,
    ) -> MenuResult<String> {
        if !fmt.line_brk {
            self.fmt_with(stream, fmt, opt)?;
        }

        prompt(fmt.suffix, stream)
    }

    /// Gives a custom formatting for the written field.
    ///
    /// # Example
    ///
    /// ```
    /// # use ezmenulib::prelude::*;
    /// let w = Written::from("hello").format(Format::prefix("==> "));
    /// ```
    pub fn format(mut self, fmt: Format<'a>) -> Self {
        self.fmt = fmt;
        self
    }

    /// Gives the default value accepted by the field.
    ///
    /// If the value type is incorrect, the [`Written::prompt`] function and its variations
    /// will panic at runtime.
    ///
    /// The default value and the example (see the [`example`](Written::example) method documentation)
    /// will be displayed inside parenthesis according to its formatting (see [`Format`]
    /// for more information).
    pub fn default_value(mut self, default: &'a str) -> Self {
        self.details.default = Some(default.to_owned());
        self
    }

    /// Gives the default value of the field, passed by an environment variable.
    ///
    /// If the provided environment variable is incorrect, it will return an error
    /// (See [`MenuError::EnvVar`] variant).
    ///
    /// If the value type of the variable is incorrect, the [`Written::prompt`] function
    /// and its variations will panic at runtime.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ezmenulib::prelude::*;
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let user: String = Written::from("What is your name?")
    ///     .default_env("USERNAME")?
    ///     .prompt(&mut MenuStream::default())?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn default_env(mut self, var: &'a str) -> MenuResult<Self> {
        self.details.default =
            Some(env::var(var).map_err(|e| MenuError::EnvVar(var.to_owned(), e))?);
        Ok(self)
    }

    /// Gives an example of correct value for the field.
    ///
    /// Obviously, it is better to give a correct value for the user as example,
    /// but if the value is incorrect, it will only mislead the user,
    /// and unlike the default value providing, the program will not panic at runtime
    /// to emphasize the problem.
    ///
    /// The example will be shown inside parenthesis according to its formatting
    /// (see [`Format`] for more information).
    pub fn example(mut self, example: &'a str) -> Self {
        self.details.example = Some(example);
        self
    }

    /// Prompts the field once, using the given prefix.
    ///
    /// It checks the `line_brk` specification. If it is on `true`, the suffix is displayed
    /// on a separate line, thus it will only display the suffix. Otherwise, it prints out
    /// the whole message with the suffix.
    fn prompt_once<R: BufRead, W: Write, T: FromStr>(
        &self,
        stream: &mut MenuStream<R, W>,
        fmt: &Format<'_>,
        opt: bool,
    ) -> MenuResult<Option<T>> {
        Ok(self
            .prompt_line(stream, fmt, opt)?
            .parse()
            .ok()
            .or_else(|| {
                self.details.default.as_deref().map(|default| {
                    default
                        .parse()
                        .unwrap_or_else(|_| default_failed::<T>(default))
                })
            }))
    }

    /// Prompts the field and returns the input, or `None` if the input is incorrect,
    /// using the given format.
    ///
    /// It uses the merged version between the format of the written field and the given format.
    ///
    /// It prompts the field once, and if the user entered a correct input,
    /// it returns `Some(value)`, otherwise, it attempts to return the default value
    /// (see [`Written::default_value`] or [`Written::default_env`]), and if there is no
    /// default value, it returns `None`.
    ///
    /// The output is wrapped in a [`MenuResult`] to prevent from any error (see [`MenuError`]).
    ///
    /// # Panics
    ///
    /// If the default value has an incorrect type, this function will panic.
    pub fn optional_value_with<R, W, T>(
        &self,
        stream: &mut MenuStream<R, W>,
        fmt: &Format<'_>,
    ) -> MenuResult<Option<T>>
    where
        R: BufRead,
        W: Write,
        T: FromStr,
    {
        let fmt = self.fmt.merged(fmt);
        self.fmt_with(stream, &fmt, true)?;
        self.prompt_once(stream, &fmt, true)
    }

    /// Prompts the field and returns the input, or `None` if the input is incorrect.
    ///
    /// It prompts the field once, and if the user entered a correct input,
    /// it returns `Some(value)`, otherwise, it attempts to return the default value
    /// (see [`Written::default_value`] or [`Written::default_env`]), and if there is no
    /// default value, it returns `None`.
    ///
    /// The output is wrapped in a [`MenuResult`] to prevent from any error (see [`MenuError`]).
    ///
    /// # Panics
    ///
    /// If the default value has an incorrect type, this function will panic.
    pub fn optional_value<R, W, T>(&self, stream: &mut MenuStream<R, W>) -> MenuResult<Option<T>>
    where
        R: BufRead,
        W: Write,
        T: FromStr,
    {
        self.optional_value_with(stream, &self.fmt)
    }

    /// Prompts the field and returns the inputs as a `Vec<T>` until the given
    /// constraint is applied to all the values, using `sep` to split the input
    /// into the output values, and using the given format.
    ///
    /// It uses the merged version between the format of the written field and the given format.
    ///
    /// After checking and parsing the values provided by the user, it calls the `til` function.
    /// The output is wrapped in a [`MenuResult`] to prevent from any error (see [`MenuError`]);
    ///
    /// # Panics
    ///
    /// If the default value has an incorrect type, this function will panic.
    pub fn many_values_until_with<R, W, T, S, F>(
        &self,
        stream: &mut MenuStream<R, W>,
        sep: S,
        til: F,
        fmt: &Format<'_>,
    ) -> MenuResult<Vec<T>>
    where
        R: BufRead,
        W: Write,
        T: FromStr,
        S: AsRef<str>,
        F: Fn(&T) -> bool,
    {
        fn inner_prompt_once<R: BufRead, W: Write, T: FromStr>(
            w: &Written<'_>,
            stream: &mut MenuStream<R, W>,
            sep: &str,
            fmt: &Format<'_>,
        ) -> MenuResult<Option<Vec<T>>> {
            let s = w.prompt_line(stream, fmt, false)?;
            let res: Result<Vec<T>, T::Err> = s.split(sep).map(T::from_str).collect();

            Ok(res.ok().or_else(|| {
                let default = w.details.default.as_ref()?;
                let res: Result<Vec<T>, T::Err> = default.split(sep).map(T::from_str).collect();
                Some(res.unwrap_or_else(|_| default_failed::<T>(default)))
            }))
        }

        let fmt = self.fmt.merged(fmt);
        self.fmt_with(stream, &fmt, false)?;
        let s = sep.as_ref();

        // Loops while incorrect input.
        loop {
            match inner_prompt_once(self, stream, s, &fmt)? {
                Some(v) if v.iter().all(&til) => return Ok(v),
                _ => continue,
            }
        }
    }

    /// Prompts the field and returns the inputs as a `Vec<T>` until the given
    /// constraint is applied to all the values, using `sep` to split the input
    /// into the output values.
    ///
    /// After checking and parsing the values provided by the user, it calls the `til` function.
    /// The output is wrapped in a [`MenuResult`] to prevent from any error (see [`MenuError`]);
    ///
    /// # Panics
    ///
    /// If the default value has an incorrect type, this function will panic.
    pub fn many_values_until<R, W, T, S, F>(
        &self,
        stream: &mut MenuStream<R, W>,
        sep: S,
        til: F,
    ) -> MenuResult<Vec<T>>
    where
        R: BufRead,
        W: Write,
        T: FromStr,
        S: AsRef<str>,
        F: Fn(&T) -> bool,
    {
        self.many_values_until_with(stream, sep, til, &self.fmt)
    }

    /// Prompts the field and returns the inputs as a `Vec<T>` using `sep` to split the input
    /// into the output values, and using the given format.
    ///
    /// It uses the merged version between the format of the written field and the given format.
    ///
    /// After checking and parsing the values provided by the user, it calls the `til` function.
    /// The output is wrapped in a [`MenuResult`] to prevent from any error (see [`MenuError`]);
    ///
    /// # Panics
    ///
    /// If the default value has an incorrect type, this function will panic.
    pub fn many_values_with<R, W, T, S>(
        &self,
        stream: &mut MenuStream<R, W>,
        sep: S,
        fmt: &Format<'_>,
    ) -> MenuResult<Vec<T>>
    where
        R: BufRead,
        W: Write,
        T: FromStr,
        S: AsRef<str>,
    {
        self.many_values_until_with(stream, sep, keep, fmt)
    }

    /// Prompts the field and returns the inputs as a `Vec<T>` using `sep` to split the input
    /// into the output values.
    ///
    /// After checking and parsing the values provided by the user, it calls the `til` function.
    /// The output is wrapped in a [`MenuResult`] to prevent from any error (see [`MenuError`]);
    ///
    /// # Panics
    ///
    /// If the default value has an incorrect type, this function will panic.
    pub fn many_values<R, W, T, S>(
        &self,
        stream: &mut MenuStream<R, W>,
        sep: S,
    ) -> MenuResult<Vec<T>>
    where
        R: BufRead,
        W: Write,
        T: FromStr,
        S: AsRef<str>,
    {
        self.many_values_with(stream, sep, &self.fmt)
    }

    /// Prompts the field until the constraint is applied, using the given format.
    ///
    /// It uses the merged version between the format of the written field and the given format.
    /// After checking and parsing the value provided by the user, it calls the `til` function.
    /// The output is wrapped in a [`MenuResult`] to prevent from any error (see [`MenuError`]);
    ///
    /// # Panic
    ///
    /// If the default value has an incorrect type, this function will panic.
    pub fn prompt_until_with<R, W, T, F>(
        &self,
        stream: &mut MenuStream<R, W>,
        til: F,
        fmt: &Format<'a>,
    ) -> MenuResult<T>
    where
        R: BufRead,
        W: Write,
        T: FromStr,
        F: Fn(&T) -> bool,
    {
        let fmt = self.fmt.merged(fmt);
        self.fmt_with(stream, &fmt, false)?;

        // Loops while incorrect input.
        loop {
            match self.prompt_once(stream, &fmt, false)? {
                Some(out) if til(&out) => return Ok(out),
                _ => continue,
            }
        }
    }

    /// Prompts the field until the constraint is applied.
    ///
    /// After checking and parsing the value provided by the user, it calls the `til` function.
    /// The output is wrapped in a [`MenuResult`] to prevent from any error (see [`MenuError`]);
    ///
    /// # Panic
    ///
    /// If the default value has an incorrect type, this function will panic.
    pub fn prompt_until<R, W, T, F>(&self, stream: &mut MenuStream<R, W>, til: F) -> MenuResult<T>
    where
        R: BufRead,
        W: Write,
        T: FromStr,
        F: Fn(&T) -> bool,
    {
        self.prompt_until_with(stream, til, &self.fmt)
    }

    /// Prompts the field, using the given format.
    ///
    /// It uses the merged version between the format of the written field and the given format.
    /// It prompts the field until the value provided by the user is correct, then parses it.
    /// The output is wrapped in a [`MenuResult`] to prevent from any error (see [`MenuError`]);
    ///
    /// # Panic
    ///
    /// If the default value has an incorrect type, this function will panic.
    pub fn prompt_with<R, W, T>(
        &self,
        stream: &mut MenuStream<R, W>,
        fmt: &Format<'a>,
    ) -> MenuResult<T>
    where
        R: BufRead,
        W: Write,
        T: FromStr,
    {
        self.prompt_until_with(stream, keep, fmt)
    }

    /// Prompts the field.
    ///
    /// It prompts the field until the value provided by the user is correct, then parses it.
    /// The output is wrapped in a [`MenuResult`] to prevent from any error (see [`MenuError`]);
    ///
    /// # Panic
    ///
    /// If the default value has an incorrect type, this function will panic.
    pub fn prompt<R, W, T>(&self, stream: &mut MenuStream<R, W>) -> MenuResult<T>
    where
        R: BufRead,
        W: Write,
        T: FromStr,
    {
        self.prompt_with(stream, &self.fmt)
    }

    /// Prompts the field and returns the input value, or the default value of the type
    /// if the input is incorrect, using the given format.
    ///
    /// It uses the merged version between the format of the written field and the given format.
    ///
    /// It prompts the value once, and if the user entered a correct input, it returns the value,
    /// otherwise, it attempts to return the default value (see [`Written::default_value`]
    /// or [`Written::default_env`]), and if there is no default value, it returns
    /// the [default](Default) implementation of `T`.
    ///
    /// # Panics
    ///
    /// If the default value has an incorrect type, this function will panic.
    pub fn prompt_or_default_with<R, W, T>(
        &self,
        stream: &mut MenuStream<R, W>,
        fmt: &Format<'_>,
    ) -> T
    where
        R: BufRead,
        W: Write,
        T: FromStr + Default,
    {
        self.optional_value_with(stream, fmt)
            .map(Option::unwrap_or_default)
            .unwrap_or_default()
    }

    /// Prompts the field and returns the input value, or the default value of the type
    /// if the input is incorrect.
    ///
    /// It prompts the value once, and if the user entered a correct input, it returns the value,
    /// otherwise, it attempts to return the default value (see [`Written::default_value`]
    /// or [`Written::default_env`]), and if there is no default value, it returns
    /// the [default](Default) implementation of `T`.
    ///
    /// # Panics
    ///
    /// If the default value has an incorrect type, this function will panic.
    pub fn prompt_or_default<R, W, T>(&self, stream: &mut MenuStream<R, W>) -> T
    where
        R: BufRead,
        W: Write,
        T: FromStr + Default,
    {
        self.prompt_or_default_with(stream, &self.fmt)
    }
}

/// Used to define a selectable type.
///
/// It provides the fields, corresponding to a message and the return value.
/// It is used by the [`Selected`] struct with its `From<&str>` implementation.
///
/// The `N` const generic parameter represents the amount of available selectable values.
///
/// # Example
///
/// ```
/// # use ezmenulib::field::Selectable;
/// enum Type {
///     MIT,
///     GPL,
///     BSD,
/// }
///
/// impl Selectable<3> for Type {
///     fn values() -> [(&'static str, Self); 3] {
///         [
///             ("MIT", Self::MIT),
///             ("GPL", Self::GPL),
///             ("BSD", Self::BSD),
///         ]
///     }
/// }
/// ```
pub trait Selectable<const N: usize>: Sized {
    /// Provides the fields, corresponding to a message and the return value.
    fn values() -> [(&'static str, Self); N];
}

/// Defines the behavior for a selected value provided by the user.
///
/// Like the [written](Written) values, it contains its own [format](Format),
/// and it can be inherited, saving the custom format specifications.
///
/// It displays the message with the available fields to select, with the
/// default field marked as "(default)" if it is provided (see [`Selected::default`] function).
/// You have to provide a mutable reference to a [`MenuStream`] to retrieve the selected value.
///
/// You can use beside it the [`Selectable`] trait to list the available values to select.
///
/// The `N` const generic parameter represents the amount of available selectable values.
///
/// # Example
///
/// For a make-license CLI program for example, you can use it like below:
///
/// ```no_run
/// use ezmenulib::prelude::*;
///
/// enum Type {
///     MIT,
///     GPL,
///     BSD,
/// }
///
/// impl Selectable<3> for Type {
///     fn values() -> [(&'static str, Self); 3] {
///         use Type::*;
///         [
///             ("MIT", MIT),
///             ("GPL", GPL),
///             ("BSD", BSD),
///         ]
///     }
/// }
///
/// let s: Type = Selected::from("Select the license type")
///     .select(&mut MenuStream::default())
///     .unwrap();
/// ```
// Clone is implemented on it because it is moved when the user selected the value.
#[derive(Clone)]
pub struct Selected<'a, T, const N: usize> {
    /// The format used by the selected field value.
    pub fmt: Format<'a>,
    msg: &'a str,
    fields: [(&'a str, T); N],
    default: Option<usize>,
}

impl<'a, T, const N: usize> From<&'a str> for Selected<'a, T, N>
where
    T: Selectable<N>,
{
    fn from(msg: &'a str) -> Self {
        Self::new(msg, T::values())
    }
}

impl<'a, T, const N: usize> Selected<'a, T, N> {
    /// Returns the Selected wrapper using the given message and
    /// selectable fields.
    ///
    /// # Note
    ///
    /// If `T` implements `Selectable`, you may use the `From<&str>` implementation
    /// for `Selected`, to not write again the available selectable fields.
    ///
    /// # Panic
    ///
    /// If the fields array is empty, this function will panic. Indeed,
    /// when prompting the index to the user to select with an empty list, it will generate an
    /// infinite loop.
    pub fn new(msg: &'a str, fields: [(&'a str, T); N]) -> Self {
        if fields.is_empty() {
            panic!("empty fields for the selectable value");
        }

        Self {
            fmt: Default::default(),
            msg,
            fields,
            default: None,
        }
    }

    /// Gives a custom formatting for the selected value.
    ///
    /// # Example
    ///
    /// ```
    /// # use ezmenulib::field::{Selected, Format, Selectable};
    /// enum Type {
    ///     MIT,
    ///     GPL,
    ///     BSD,
    /// }
    ///
    /// let w = Selected::new("Select the license type", [
    ///     ("MIT", Type::MIT),
    ///     ("GPL", Type::GPL),
    ///     ("BSD", Type::BSD),
    /// ])
    /// .format(Format::prefix("==> "));
    /// ```
    pub fn format(mut self, fmt: Format<'a>) -> Self {
        self.fmt = fmt;
        // Saves the default suffix if asked to break the line,
        // because it would be ugly to have for instance ": " as suffix.
        // This is useful if the format is inherited (see [`Values::selected`] function).
        if !self.fmt.line_brk {
            self.fmt.suffix = DEFAULT_FMT.suffix;
        }
        self
    }

    /// Defines the default value among the the selectable values, by its index.
    ///
    /// # Note
    ///
    /// If the index is out of bounds, it will not panic at runtime. Therefore,
    /// if the user enters an incorrect index, it will not use the default index.
    pub fn default(mut self, default: usize) -> Self {
        self.default = Some(default + 1);
        self
    }

    /// Prompts the selectable fields once.
    ///
    /// In fact, it only displays the suffix, and gets the user input, then returns
    /// the correct index wrapped in a `Query`.
    fn prompt_once<R: BufRead, W: Write>(
        &self,
        stream: &mut MenuStream<R, W>,
    ) -> MenuResult<Option<usize>> {
        select(stream, self.fmt.suffix, self.default, N)
    }

    /// Prompts the selectable fields and returns the value at the input index,
    /// or `None` if the index is incorrect.
    ///
    /// It prompts the selectable fields once, and if the user entered a correct index,
    /// it returns `Some(value)` where `value` corresponds to the value mapped to this index,
    /// otherwise, it attempts to return the value mapped by the default index
    /// (see `Selected::default`), and if there is no default index, it returns `None`.
    ///
    /// The output is wrapped in a [`MenuResult`] to prevent from any error (see [`MenuError`]).
    pub fn optional_select<R, W>(self, stream: &mut MenuStream<R, W>) -> MenuResult<Option<T>>
    where
        R: BufRead,
        W: Write,
    {
        // Uses the alternate form of selection field display
        // to display the "(optional)" string slice message.
        show(&format!("{:#}", self), stream)?;

        Ok(self.prompt_once(stream)?.map(|i| {
            // SAFETY: the `Selected::prompt_once` guarantees that the index is in bounds.
            unsafe { self.take(i) }
        }))
    }

    /// Gives the value stored at index `i`, consuming `self`.
    ///
    /// The index must be in bounds, or this will cause an undefined behavior.
    ///
    /// # Safety
    ///
    /// The `i` index must be in bounds, meaning `i < N`.
    /// Otherwise, this function results in an undefined behavior.
    unsafe fn take(self, i: usize) -> T {
        assert!(i < N);
        self.fields.into_iter().nth(i).unwrap_unchecked().1
    }

    /// Prompts the selectable values to the user.
    ///
    /// It prompts the fields once and the suffix until the index provided, then returns the selected value.
    /// The output is wrapped in a [`MenuResult`] to prevent from any error (see [`MenuError`]);
    ///
    /// This function consumes `self` because it returns the ownership of a contained value
    /// (`T`) defined earlier in the [`Selected::new`] function.
    pub fn select<R, W>(self, stream: &mut MenuStream<R, W>) -> MenuResult<T>
    where
        R: BufRead,
        W: Write,
    {
        show(&self, stream)?;
        loop {
            match self.prompt_once(stream)? {
                // SAFETY: the `Selected::prompt_once` guarantees that the index is in bounds.
                Some(out) => return Ok(unsafe { self.take(out) }),
                None => continue,
            }
        }
    }

    /// Prompts the selectable values to the user, and returns the value at the input index,
    /// or the default index if the input is incorrect.
    ///
    /// It prompts the selectable values once, and if the user entered a correct index,
    /// it returns the value mapped to this index, otherwise, it attempts to return the value
    /// mapped to the default index (see [`Selected::default`]), and if there is no default value,
    /// it returns the [default](Default) implementation of `T`.
    pub fn select_or_default<R, W>(self, stream: &mut MenuStream<R, W>) -> T
    where
        R: BufRead,
        W: Write,
        T: Default,
    {
        self.optional_select(stream)
            .map(Option::unwrap_or_default)
            .unwrap_or_default()
    }
}

impl<T, const N: usize> Display for Selected<'_, T, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.fmt.prefix)?;
        f.write_str(self.msg)?;
        if f.alternate() && self.default.is_none()
            || self.default.is_some() && !self.fmt.show_default
        {
            f.write_str(" (optional)")?;
        }

        for (i, (msg, _)) in (1..=N).zip(self.fields.iter()) {
            write!(f, "{}", i)?;
            f.write_str(self.fmt.chip)?;
            f.write_str(msg)?;
            match self.default {
                Some(x) if x == i && self.fmt.show_default => f.write_str(" (default)")?,
                _ => (),
            }
        }

        Ok(())
    }
}

pub type Field<'a, R = In, W = Out> = (&'a str, Kind<'a, R, W>);

pub type Fields<'a, R = In, W = Out> = &'a [Field<'a, R, W>];

pub type SizedFields<'a, const LEN: usize, R = In, W = Out> = &'a [Field<'a, R, W>; LEN];

pub type Binding<R, W> = fn(&mut MenuStream<R, W>) -> MenuResult;

pub enum Kind<'a, R = In, W = Out> {
    Unit(Binding<R, W>),
    Parent(Fields<'a, R, W>),
    Back,
    Quit,
}

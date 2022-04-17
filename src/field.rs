//! Module defining different types of fields.
//!
//! The selection fields [`SelectField`] corresponds to a [`SelectMenu`](crate::menu::SelectMenu),
//! while the value fields [`ValueField`] corresponds to a [`ValueMenu`](crate::menu::ValueMenu).
//!
//! A `ValueMenu` can however contain both a `ValueField` and a `SelectMenu`,
//! to be used as a sub-menu (check out the [`Field`] enum).
//!
//! ## Formatting
//!
//! You can edit the [formatting rules](ValueFieldFormatting) of a `ValueField` or set the global
//! formatting rules for the `ValueMenu`.
//!
//! If a `SelectMenu` is used as a sub-menu to a `ValueMenu`, the global formatting rules of the `ValueMenu`
//! will be applied on the title of the `SelectMenu` to integrate it in the structure.
//!
//! You can still edit the formatting rules of its title (see [`SelectTitle`](crate::menu::SelectTitle))
//! independently from the global formatting rules.

mod query;

#[cfg(test)]
mod tests;

use self::query::Query;
use crate::prelude::*;
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
            pub fn merged(&self, r: &Format<'a>) -> Self {
                Self {$(
                    $i: {
                        match self.$i == DEFAULT_FMT.$i {
                            true => r.$i,
                            false => self.$i,
                        }
                    },
                )*}
            }

            /// Merges `self` with `r` format.
            ///
            /// See [`Format::merged`] for more information.
            pub fn merge(&mut self, r: &Format<'a>) {
                *self = self.merged(r)
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

// Should not be accessed from outer module
pub(crate) enum WrittenDefaultValue<'a> {
    Value(&'a str),
    Env(String),
}

impl<'a> WrittenDefaultValue<'a> {
    pub fn env(var: &'a str) -> MenuResult<Self> {
        Ok(Self::Env(
            env::var(var).map_err(|e| MenuError::EnvVar(var.to_string(), e))?,
        ))
    }
}

impl Display for WrittenDefaultValue<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        #[inline(never)]
        fn write_str(f: &mut Formatter<'_>, s: impl AsRef<str>) -> fmt::Result {
            write!(f, "default: {}", s.as_ref())
        }

        match self {
            Self::Value(s) => write_str(f, s),
            Self::Env(s) => write_str(f, s),
        }
    }
}

struct WrittenDetails<'a> {
    example: Option<&'a str>,
    default: Option<WrittenDefaultValue<'a>>,
    show_d: bool,
}

impl Display for WrittenDetails<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        if self.example.is_none() && (self.default.is_none() || !self.show_d) {
            return Ok(());
        }

        write!(
            f,
            " ({})",
            match (&self.default, self.example) {
                (Some(d), None) if self.show_d => format!("{}", d),
                (Some(d), Some(e)) if self.show_d => format!("example: {}, {}", e, d),
                (None, Some(e)) => format!("example: {}", e),
                (Some(_), Some(e)) => format!("example: {}", e),
                _ => unreachable!(),
            }
        )
    }
}

/// Defines the behavior for a written value provided by the user.
///
/// Like the [selected](Selected) values, it contains its own [format](Format),
/// and it can be inherited, saving the custom format specifications
/// (see [`Format::merged`] function).
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
///     field::Written,
///     customs::MenuVec,
/// };
/// let author: MenuVec<String> = Written::from("Give the author of the license")
///     .prompt(&mut MenuStream::default())
///     .unwrap();
/// ```
pub struct Written<'a> {
    msg: &'a str,
    fmt: Format<'a>,
    details: WrittenDetails<'a>,
}

impl<'a> From<&'a str> for Written<'a> {
    fn from(msg: &'a str) -> Self {
        let fmt = Format::default();
        let show_d = fmt.show_default;
        Self {
            msg,
            fmt,
            details: WrittenDetails {
                example: None,
                default: None,
                show_d,
            },
        }
    }
}

impl Display for Written<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.show_with_pref(f, self.fmt.prefix)?;
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
    fn show_with_pref<S: fmt::Write>(&self, s: &mut S, pref: &'a str) -> fmt::Result {
        write!(
            s,
            "{pref}{msg}{det}",
            pref = pref,
            msg = self.msg,
            det = self.details,
        )
    }

    /// Gives a custom formatting for the written field.
    ///
    /// # Example
    ///
    /// ```
    /// let w = Written::from("hello").format(&Format::prefix("==> "));
    /// ```
    pub fn format(mut self, fmt: &Format<'a>) -> Self {
        self.fmt.merge(fmt);
        self.details.show_d = fmt.show_default;
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
        self.details.default = Some(WrittenDefaultValue::Value(default));
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
    /// # use std::error::Error;
    /// # fn main() -> Result<(), Box<dyn Error>> {
    /// let user = Written::from("What is your name?")
    ///     .default_env("USERNAME")?
    ///     .prompt(&mut MenuStream::default());
    /// # Ok(())
    /// # }
    /// ```
    pub fn default_env(mut self, var: &'a str) -> MenuResult<Self> {
        self.details.default = Some(WrittenDefaultValue::env(var)?);
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
    ) -> Query<T> {
        fn prompted<R: BufRead, W: Write>(
            written: &Written<'_>,
            fmt: &Format<'_>,
            stream: &mut MenuStream<R, W>,
        ) -> MenuResult<String> {
            let msg = if fmt.line_brk {
                fmt.suffix.to_owned()
            } else {
                let mut s = String::new();
                written.show_with_pref(&mut s, fmt.prefix)?;
                s.push_str(fmt.suffix);
                s
            };
            prompt(&msg, stream)
        }

        prompted(self, fmt, stream)
            .map(|s| parse_value(s, self.details.default.as_ref().map(default_parse)))
            .into()
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

        // Displays the field message.
        if fmt.line_brk {
            self.show_with_pref(stream, fmt.prefix)?;
            stream.write_all("\n".as_bytes())?;
        }

        // Loops while incorrect input.
        loop {
            match self.prompt_once(stream, &fmt) {
                Query::Finished(out) if til(&out) => break Ok(out),
                Query::Err(e) => break Err(e),
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
        self.prompt_until_with(stream, |_| true, fmt)
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

    /// Prompts the field, or return the default value if any error occurred,
    /// using the given format.
    ///
    /// It uses the merged version between the format of the written field and the given format.
    /// It prompts the field message once, and if any error occurred, such as an incorrect input
    /// (see [`MenuError`]), it returns the default value.
    ///
    /// # Panic
    ///
    /// If the default value (provided by [`Written::default_value`] or [`Written::default_env`]
    /// functions) has an incorrect type, this function will panic.
    pub fn prompt_or_default_with<R, W, T>(
        &self,
        stream: &mut MenuStream<R, W>,
        fmt: &Format<'a>,
    ) -> T
    where
        R: BufRead,
        W: Write,
        T: FromStr + Default,
    {
        fn inner_prompt<R: BufRead, W: Write, T: FromStr + Default>(
            stream: &mut MenuStream<R, W>,
            fmt: Format<'_>,
            w: &Written<'_>,
        ) -> MenuResult<T> {
            if fmt.line_brk {
                w.show_with_pref(stream, fmt.prefix)?;
                stream.write_all("\n".as_bytes())?;
            }
            // This will only print out the suffix, because it is on a separate line.
            w.prompt_once(stream, &fmt).into()
        }

        inner_prompt(stream, self.fmt.merged(fmt), self).unwrap_or_default()
    }

    /// Prompts the field, or return the default value if any error occurred.
    ///
    /// It prompts the field message once, and if any error occurred, such as an incorrect input
    /// (see [`MenuError`]), it returns the default value.
    ///
    /// # Panic
    ///
    /// If the default value (provided by [`Written::default_value`] or [`Written::default_env`]
    /// functions) has an incorrect type, this function will panic.
    pub fn prompt_or_default<R, W, T>(&self, stream: &mut MenuStream<R, W>) -> T
    where
        R: BufRead,
        W: Write,
        T: FromStr + Default,
    {
        self.prompt_or_default_with(stream, &self.fmt)
    }
}

/// Returns the input value as a String from the standard input stream.
pub(crate) fn raw_read_input<R, W>(stream: &mut MenuStream<R, W>) -> MenuResult<String>
where
    R: BufRead,
    W: Write,
{
    let mut out = String::new();
    stream.read_line(&mut out)?;
    Ok(out.trim().to_owned())
}

/// Parses the input value.
///
/// It is useful because it maps the error according to the [`MenuError`](crate::MenuError)
/// type definition.
pub(crate) fn parse_value<T, S>(s: S, default: Option<T>) -> MenuResult<T>
where
    T: FromStr,
    S: AsRef<str>,
{
    match (s.as_ref().parse(), default) {
        (Ok(out), _) | (Err(_), Some(out)) => Ok(out),
        (Err(_), None) => Err(MenuError::Input),
    }
}

/// Function that parses the default value with a check if the default value is incorrect.
/// It it used to return a value if there is some default value,
/// and if no value was provided, or if the value provided is incorrect.
fn default_parse<T: FromStr>(default: &WrittenDefaultValue<'_>) -> T {
    fn unwrap<T: FromStr>(s: impl AsRef<str>) -> T {
        let s = s.as_ref();
        s.parse().unwrap_or_else(|_| {
            panic!(
                "`{}` has been used as default value but its type is incorrect",
                s
            )
        })
    }

    match default {
        WrittenDefaultValue::Value(s) => unwrap(s),
        WrittenDefaultValue::Env(s) => unwrap(s),
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
/// and it can be inherited, saving the custom format specifications
/// (see [`Format::merged`] function).
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
/// use ezmenulib::field::{Selected, Selectable};
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
pub struct Selected<'a, T, const N: usize> {
    fmt: Format<'a>,
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
    /// If the fields vector is empty, this function will panic. Indeed,
    /// when prompting the index to the user to select, it will generate an
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
    /// .format(&Format::prefix("==> "));
    /// ```
    pub fn format(mut self, fmt: &Format<'a>) -> Self {
        self.fmt.merge(fmt);
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

    /// Prompts the selected menu once.
    ///
    /// In fact, it only displays the suffix, and gets the user input, then returns
    /// the correct index wrapped in a `Query`.
    fn prompt_once<R: BufRead, W: Write>(&self, stream: &mut MenuStream<R, W>) -> Query<usize> {
        prompt(self.fmt.suffix, stream)
            .map(|s| match parse_value(&s, self.default) {
                Ok(i) if i >= 1 && i <= N => Ok(i - 1),
                _ => Err(MenuError::Input),
            })
            .into()
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
            match self.prompt_once(stream) {
                Query::Finished(out) => break Ok(Vec::from(self.fields).remove(out).1),
                Query::Err(e) => break Err(e),
                Query::Continue => continue,
            }
        }
    }
}

impl<'a, T, const N: usize> Selected<'a, T, N>
where
    T: Default,
{
    /// Prompts the selectable values to the user, or return the default value
    /// if any error occurred.
    ///
    /// It prompts the fields and suffix once, and if any error occurred, such as an incorrect
    /// input (see [`MenuError`]), it returns the default value.
    ///
    /// This function consumes `self` because it returns the ownership of a contained value
    /// (`T`) defined earlier in the [`Selected::new`] function.
    pub fn select_or_default<R, W>(self, stream: &mut MenuStream<R, W>) -> T
    where
        R: BufRead,
        W: Write,
    {
        show(&self, stream)
            .and(self.prompt_once(stream).into())
            .map(|i| Vec::from(self.fields).remove(i).1)
            .unwrap_or_default()
    }
}

impl<T, const N: usize> Display for Selected<'_, T, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "{}", self.msg)?;

        for (i, (msg, _)) in self.fields.iter().enumerate() {
            writeln!(
                f,
                "{i}{chip}{msg}{default}",
                i = i + 1,
                chip = self.fmt.chip,
                msg = msg,
                default = match self.default {
                    Some(x) if x == i + 1 && self.fmt.show_default => " (default)",
                    _ => "",
                }
            )?;
        }

        Ok(())
    }
}

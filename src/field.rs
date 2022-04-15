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

#[cfg(test)]
mod tests;

use crate::query::Query;
use crate::DEFAULT_FMT;
use crate::{prelude::*, Selectable};
use std::env;
use std::fmt::{self, Display, Formatter};
use std::io::{BufRead, Write};
use std::str::FromStr;

/// Type used to handle the binding function executed right after
/// the corresponding field has been selected by the user.
pub type Binding<R, W> = fn(&mut MenuStream<R, W>) -> MenuResult;

/// Builds the associated functions of the [`Format`] struct
/// according to its fields.
macro_rules! impl_fmt {
    ($(#[doc = $main_doc:expr])*
    ,
    $(
        $(#[doc = $doc:expr])*
        $i:ident: $t:ty $(,)?
    ),*) => {
        $(#[doc = $main_doc])*
        #[derive(Clone)]
        pub struct Format<'a> {$(
            $(#[doc = $doc])*
            pub $i: $t,
        )*}

        impl<'a> Format<'a> {
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

            pub(crate) fn merge(&mut self, r: &Format<'a>) {
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
    /// The final text formatting looks literally like above:
    /// ```md
    /// <chip><message>[ ({[default: <default>]}, [example: <example>])]\\n<prefix>
    /// ```
    /// where:
    /// - `<...>` means a given string slice.
    /// - `{...}` means that the value inside is chose to be displayed or not (boolean).
    /// - `[...]` means that the value inside is displayed if it is available.
    ,
    /// Sets the chip of the formatting (`"--> "` by default).
    prefix: &'a str,
    /// Defines the chip as marker type for lists (`" - "` by default).
    chip: &'a str,
    /// Defines if it displays the default value or not (`true` by default).
    show_default: bool,
    /// Sets the prefix of the formatting (`">> "` by default).
    suffix: &'a str,
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
pub(crate) enum DefaultValue<'a> {
    Value(&'a str),
    Env(String),
}

impl<'a> DefaultValue<'a> {
    pub fn env(var: &'a str) -> MenuResult<Self> {
        Ok(Self::Env(
            env::var(var).map_err(|e| MenuError::EnvVar(var.to_string(), e))?,
        ))
    }
}

impl Display for DefaultValue<'_> {
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

struct FieldDetails<'a> {
    example: Option<&'a str>,
    default: Option<DefaultValue<'a>>,
    show_d: bool,
}

impl Display for FieldDetails<'_> {
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
/// ```no_run
/// use ezmenulib::field::ValueField;
/// let author: String = ValueField::from("Give the author of the license")
///     .build_init()
///     .unwrap();
/// ```
pub struct Written<'a> {
    msg: &'a str,
    fmt: Format<'a>,
    details: FieldDetails<'a>,
}

impl<'a> From<&'a str> for Written<'a> {
    fn from(msg: &'a str) -> Self {
        let fmt = Format::default();
        let show_d = fmt.show_default;
        Self {
            msg,
            fmt,
            details: FieldDetails {
                example: None,
                default: None,
                show_d,
            },
        }
    }
}

impl Display for Written<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        self.show_with_pref(f, self.fmt.prefix)
    }
}

/// Constructor methods defining how the field behaves
impl<'a> Written<'a> {
    fn show_with_pref<S: fmt::Write>(&self, s: &mut S, pref: &'a str) -> fmt::Result {
        writeln!(
            s,
            "{pref}{msg}{det}",
            pref = pref,
            msg = self.msg,
            det = self.details,
        )
    }

    /// Give a custom formatting for the field.
    pub fn format(mut self, fmt: &Format<'a>) -> Self {
        self.fmt.merge(fmt);
        self.details.show_d = fmt.show_default;
        self
    }

    /// Give the default value accepted by the field.
    ///
    /// If the value type is incorrect, the [`ValueField::build`] or [`ValueField::build_init`]
    /// methods will panic at runtime.
    ///
    /// The default value and the example (see the [`example`](ValueField::example) method documentation)
    /// will be displayed inside parenthesis according to its formatting (see [`ValueFieldFormatting`]
    /// for more information).
    pub fn default_value(mut self, default: &'a str) -> Self {
        self.details.default = Some(DefaultValue::Value(default));
        self
    }

    /// Give the default value of the field, passing by an environment variable.
    ///
    /// If the provided environment variable is incorrect, it will return an error
    /// (See [`MenuError::EnvVar`] variant).
    ///
    /// If the value type of the variable is incorrect, the [`ValueField::build`] or [`ValueField::build_init`]
    /// method will panic at runtime.
    pub fn default_env(mut self, var: &'a str) -> MenuResult<Self> {
        self.details.default = Some(DefaultValue::env(var)?);
        Ok(self)
    }

    /// Give an example of correct value for the field.
    ///
    /// Obviously, it is better to give a correct value for the user, but if the value is incorrect,
    /// it will only mislead the user, and unlike the default value providing,
    /// the program will not panic at runtime to emphasize the problem.
    ///
    /// The example will be shown inside parenthesis according to its formatting
    /// (see [`ValueFieldFormatting`] for more information).
    pub fn example(mut self, example: &'a str) -> Self {
        self.details.example = Some(example);
        self
    }

    fn prompt_once<R: BufRead, W: Write, T: FromStr>(
        &self,
        stream: &mut MenuStream<R, W>,
        suffix: &'a str,
    ) -> Query<T> {
        prompt(suffix, stream)
            .map(|s| parse_value(s, self.details.default.as_ref().map(default_parse)))
            .into()
    }

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

        self.show_with_pref(stream, fmt.prefix)?;
        loop {
            match self.prompt_once(stream, fmt.suffix) {
                Query::Finished(out) if til(&out) => break Ok(out),
                Query::Err(e) => break Err(e),
                _ => continue,
            }
        }
    }

    pub fn prompt_until<R, W, T, F>(&self, stream: &mut MenuStream<R, W>, til: F) -> MenuResult<T>
    where
        R: BufRead,
        W: Write,
        T: FromStr,
        F: Fn(&T) -> bool,
    {
        self.prompt_until_with(stream, til, &self.fmt)
    }

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

    pub fn prompt<R, W, T>(&self, stream: &mut MenuStream<R, W>) -> MenuResult<T>
    where
        R: BufRead,
        W: Write,
        T: FromStr,
    {
        self.prompt_with(stream, &self.fmt)
    }

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
        let fmt = self.fmt.merged(fmt);

        self.show_with_pref(stream, fmt.prefix)
            .map_err(MenuError::from)
            .and(self.prompt_once(stream, fmt.suffix).into())
            .unwrap_or_default()
    }

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
    let s = s.as_ref();
    match (s.parse(), default) {
        (Ok(out), _) | (Err(_), Some(out)) => Ok(out),
        (Err(_), None) => Err(MenuError::Input),
    }
}

/// Function that parses the default value with a check if the default value is incorrect.
/// It it used to return a value if there is some default value,
/// and if no value was provided, or if the value provided is incorrect.
fn default_parse<T>(default: &DefaultValue<'_>) -> T
where
    T: FromStr,
{
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
        DefaultValue::Value(s) => unwrap(s),
        DefaultValue::Env(s) => unwrap(s),
    }
}

pub struct Selected<'a, T> {
    fmt: Format<'a>,
    msg: &'a str,
    fields: Vec<(&'a str, T)>,
    default: Option<usize>,
}

impl<'a, T> From<&'a str> for Selected<'a, T>
where
    T: Selectable,
{
    fn from(msg: &'a str) -> Self {
        Self::new(msg, T::values())
    }
}

impl<'a, T> Selected<'a, T> {
    pub fn new(msg: &'a str, fields: Vec<(&'a str, T)>) -> Self {
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

    pub fn format(mut self, fmt: &Format<'a>) -> Self {
        self.fmt.merge(fmt);
        self
    }

    pub fn default(mut self, default: usize) -> Self {
        self.default = Some(default);
        self
    }

    fn prompt_once<R, W>(&self, stream: &mut MenuStream<R, W>) -> Query<usize>
    where
        R: BufRead,
        W: Write,
    {
        prompt(self.fmt.suffix, stream)
            .map(|s| match parse_value(&s, self.default) {
                Ok(i) if i >= 1 && i <= self.fields.len() => Ok(i - 1),
                Ok(_) => Err(MenuError::Input),
                Err(e) => Err(e),
            })
            .into()
    }

    pub fn select<R, W>(mut self, stream: &mut MenuStream<R, W>) -> MenuResult<T>
    where
        R: BufRead,
        W: Write,
    {
        show(&self, stream)?;
        loop {
            match self.prompt_once(stream) {
                Query::Finished(out) => break Ok(self.fields.swap_remove(out).1),
                Query::Err(e) => break Err(e),
                Query::Continue => continue,
            }
        }
    }
}

impl<'a, T> Selected<'a, T>
where
    T: Default,
{
    pub fn select_or_default<R: BufRead, W: Write>(mut self, stream: &mut MenuStream<R, W>) -> T {
        show(&self, stream)
            .and(self.prompt_once(stream).into())
            .map(|i| self.fields.swap_remove(i).1)
            .unwrap_or_default()
    }
}

impl<T> Display for Selected<'_, T> {
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
                    Some(x) if x == i + 1 => " (default)",
                    _ => "",
                }
            )?;
        }

        Ok(())
    }
}

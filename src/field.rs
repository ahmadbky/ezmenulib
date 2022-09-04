//! Module that defines several types about retrieving values from the user.

#[cfg(test)]
mod tests;

#[cfg(feature = "password")]
use rpassword::read_password;

use crate::{customs::MenuBool, menu::Handle, prelude::*, utils::*, DEFAULT_FMT};
use std::{
    borrow::Cow,
    env,
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    str::FromStr,
};

pub trait MenuDisplay {
    fn fmt_with<W: fmt::Write>(&self, f: W, fmt: &Format<'_>, opt: bool) -> fmt::Result;

    fn fmt<W: fmt::Write>(&self, f: W, opt: bool) -> fmt::Result
    where
        Self: UsesFormat,
    {
        self.fmt_with(f, self.get_format(), opt)
    }
}

pub trait UsesFormat {
    fn get_format(&self) -> &Format<'_>;
}

pub trait Promptable<T>: Sized + MenuDisplay + UsesFormat {
    type Middle;

    fn prompt_once<H: Handle>(
        &self,
        handle: H,
        fmt: &Format<'_>,
        opt: bool,
    ) -> MenuResult<Option<Self::Middle>>;

    fn convert(self, mid: Self::Middle) -> T;

    fn get(self) -> T {
        self.try_get().expect("error while prompting field")
    }

    fn get_with(self, fmt: &Format<'_>) -> T {
        self.try_get_with(fmt).expect("error while prompting field")
    }

    fn try_get(self) -> MenuResult<T> {
        self.prompt(MenuHandle::default())
    }

    fn try_get_with(self, fmt: &Format<'_>) -> MenuResult<T> {
        self.prompt_with(MenuHandle::default(), fmt)
    }

    fn prompt_with<H: Handle>(self, mut handle: H, fmt: &Format<'_>) -> MenuResult<T> {
        let fmt = self.get_format().merged(fmt);
        handle.show(&self, &fmt, false)?;
        loop {
            match self.prompt_once(&mut handle, &fmt, false)? {
                Some(out) => return Ok(self.convert(out)),
                None => (),
            }
        }
    }

    fn prompt<H: Handle>(self, mut handle: H) -> MenuResult<T> {
        handle.show(&self, self.get_format(), false)?;
        loop {
            match self.prompt_once(&mut handle, self.get_format(), false)? {
                Some(out) => return Ok(self.convert(out)),
                None => (),
            }
        }
    }

    fn optional_prompt_with<H: Handle>(
        self,
        mut handle: H,
        fmt: &Format<'_>,
    ) -> MenuResult<Option<T>> {
        let fmt = self.get_format().merged(fmt);
        handle.show(&self, &fmt, true)?;
        self.prompt_once(handle, &fmt, true)
            .map(|opt| opt.map(|m| self.convert(m)))
    }

    fn optional_prompt<H: Handle>(self, mut handle: H) -> MenuResult<Option<T>> {
        handle.show(&self, self.get_format(), true)?;
        self.prompt_once(handle, self.get_format(), true)
            .map(|opt| opt.map(|m| self.convert(m)))
    }

    fn prompt_or_default_with<H: Handle>(self, handle: H, fmt: &Format<'_>) -> T
    where
        T: Default,
    {
        self.optional_prompt_with(handle, fmt)
            .map(Option::unwrap_or_default)
            .unwrap_or_default()
    }

    fn prompt_or_default<H: Handle>(self, handle: H) -> T
    where
        T: Default,
    {
        self.optional_prompt(handle)
            .map(Option::unwrap_or_default)
            .unwrap_or_default()
    }
}

/// Builds the associated functions of the [`Format`] struct
/// according to its fields.
macro_rules! impl_fmt {
    ($(#[doc = $main_doc:expr])*
    $(
        $i:ident: $t:ty,
        $(#[doc = $doc:expr])*
    )*) => {
        $(#[doc = $main_doc])*
        #[derive(Debug, Clone)]
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
                    $i: if self.$i == DEFAULT_FMT.$i { r.$i } else { self.$i },
                )*}
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
    left_sur: &'a str,
    /// Defines the left "surrounding" of the index when displaying a list ("[" by default).
    ///
    /// It is displayed between at the beginning of the list field line, before the index.
    right_sur: &'a str,
    /// Defines the right "surrounding" of the index when displaying a list ("]" by default).
    ///
    /// It is displayed between the index and the chip.
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

#[derive(Debug, Clone)]
pub struct Bool<'a> {
    inner: Written<'a>,
}

impl<'a> From<&'a str> for Bool<'a> {
    fn from(msg: &'a str) -> Self {
        Self::new(msg)
    }
}

impl<'a> Bool<'a> {
    pub fn from_written(inner: Written<'a>) -> Self {
        Self { inner }
    }

    pub fn new(msg: &'a str) -> Self {
        Self::from_written(From::from(msg))
    }

    pub fn format(self, fmt: Format<'a>) -> Self {
        Self::from_written(self.inner.format(fmt))
    }

    pub fn example(self, example: &'a str) -> Self {
        Self::from_written(self.inner.example(example))
    }

    pub fn with_basic_example(self) -> Self {
        self.example("yes/no")
    }

    pub fn default_value(self, default: bool) -> Self {
        let val = if default { "yes" } else { "no" };
        Self::from_written(self.inner.default_value(val))
    }

    pub fn default_env(self, var: &'a str) -> MenuResult<Self> {
        Ok(Self::from_written(self.inner.default_env(var)?))
    }
}

impl UsesFormat for Bool<'_> {
    fn get_format(&self) -> &Format<'_> {
        self.inner.get_format()
    }
}

impl MenuDisplay for Bool<'_> {
    fn fmt_with<W: fmt::Write>(&self, f: W, fmt: &Format<'_>, opt: bool) -> fmt::Result {
        MenuDisplay::fmt_with(&self.inner, f, fmt, opt)
    }
}

impl Display for Bool<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Self as MenuDisplay>::fmt(self, f, false)
    }
}

impl Promptable<bool> for Bool<'_> {
    type Middle = bool;

    fn prompt_once<H: Handle>(
        &self,
        handle: H,
        fmt: &Format<'_>,
        opt: bool,
    ) -> MenuResult<Option<Self::Middle>> {
        let res: Option<MenuBool> = self.inner.prompt_once(handle, fmt, opt)?;
        Ok(res.map(Into::into))
    }

    fn convert(self, mid: Self::Middle) -> bool {
        mid
    }
}

#[derive(Debug, Clone)]
pub struct Separated<'a, I, T> {
    inner: Written<'a>,
    sep: &'a str,
    env_sep: Option<&'a str>,
    _marker: PhantomData<&'a (I, T)>,
}

impl<'a, I, T> Separated<'a, I, T> {
    pub fn from_written(inner: Written<'a>, sep: &'a str) -> Self {
        Self {
            inner,
            sep,
            env_sep: None,
            _marker: PhantomData,
        }
    }

    pub fn new(msg: &'a str, sep: &'a str) -> Self {
        Self::from_written(From::from(msg), sep)
    }

    pub fn format(self, fmt: Format<'a>) -> Self {
        let inner = self.inner.format(fmt);
        Self { inner, ..self }
    }

    pub fn example(self, example: &'a str) -> Self {
        let inner = self.inner.example(example);
        Self { inner, ..self }
    }

    pub fn default_value(self, default: &'a str) -> Self {
        let inner = self.inner.default_value(default);
        Self { inner, ..self }
    }

    pub fn default_env(self, var: &'a str) -> MenuResult<Self> {
        let inner = self.inner.default_env(var)?;
        Ok(Self { inner, ..self })
    }

    pub fn default_env_with(mut self, var: &'a str, sep: &'a str) -> MenuResult<Self> {
        self.env_sep = Some(sep);
        self.default_env(var)
    }
}

impl<I, T> UsesFormat for Separated<'_, I, T> {
    fn get_format(&self) -> &Format<'_> {
        self.inner.get_format()
    }
}

impl<I, T> MenuDisplay for Separated<'_, I, T> {
    fn fmt_with<W: fmt::Write>(&self, f: W, fmt: &Format<'_>, opt: bool) -> fmt::Result {
        MenuDisplay::fmt_with(&self.inner, f, fmt, opt)
    }
}

impl<I, T> Display for Separated<'_, I, T> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Self as MenuDisplay>::fmt(self, f, false)
    }
}

impl<I, T: FromStr> Promptable<I> for Separated<'_, I, T>
where
    I: FromIterator<T>,
{
    type Middle = I;

    fn prompt_once<H: Handle>(
        &self,
        handle: H,
        fmt: &Format<'_>,
        opt: bool,
    ) -> MenuResult<Option<Self::Middle>> {
        let s: String = match self.inner.prompt_once(handle, fmt, opt)? {
            Some(s) => s,
            None => return Ok(None),
        };
        let res: Result<I, T::Err> = s.split(self.sep).map(T::from_str).collect();
        let res = res.ok().or_else(|| {
            // Default value
            let d = self.inner.default.as_ref()?;
            let res: Result<I, T::Err> = d
                .split(match d {
                    // Default value provided directly
                    Cow::Borrowed(_) => self.sep,
                    // Default value provided from env var
                    Cow::Owned(_) => self.env_sep.unwrap_or(self.sep),
                })
                .map(T::from_str)
                .collect();
            Some(res.unwrap_or_else(|_| default_failed::<T>(d)))
        });

        Ok(res)
    }

    fn convert(self, mid: Self::Middle) -> I {
        mid
    }
}

pub type SeparatedUntil<'a, I, T, F> = Until<Separated<'a, I, T>, F>;

#[derive(Debug, Clone)]
pub struct Until<P, F> {
    inner: P,
    til: F,
}

impl<'a, P: From<&'a str> + 'a, F> Until<P, F> {
    pub fn new(msg: &'a str, til: F) -> Self {
        Self::from_promptable(From::from(msg), til)
    }
}

impl<P, F> Until<P, F> {
    pub fn from_promptable(inner: P, til: F) -> Self {
        Self { inner, til }
    }
}

impl<P: UsesFormat, F> UsesFormat for Until<P, F> {
    fn get_format(&self) -> &Format<'_> {
        self.inner.get_format()
    }
}

impl<P: MenuDisplay, F> MenuDisplay for Until<P, F> {
    fn fmt_with<W: fmt::Write>(&self, f: W, fmt: &Format<'_>, opt: bool) -> fmt::Result {
        MenuDisplay::fmt_with(&self.inner, f, fmt, opt)
    }
}

impl<P: MenuDisplay + UsesFormat, F> Display for Until<P, F> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Self as MenuDisplay>::fmt(self, f, false)
    }
}

impl<T, P, F> Promptable<T> for Until<P, F>
where
    P: Promptable<T>,
    F: Fn(&<P as Promptable<T>>::Middle) -> bool,
{
    type Middle = <P as Promptable<T>>::Middle;

    fn prompt_once<H: Handle>(
        &self,
        handle: H,
        fmt: &Format<'_>,
        opt: bool,
    ) -> MenuResult<Option<Self::Middle>> {
        self.inner
            .prompt_once(handle, fmt, opt)
            .map(|opt| opt.filter(&self.til))
    }

    fn convert(self, mid: Self::Middle) -> T {
        self.inner.convert(mid)
    }
}

#[cfg(feature = "password")]
#[cfg_attr(nightly, doc(cfg(feature = "password")))]
#[derive(Debug)]
pub struct Password<'a> {
    msg: &'a str,
    pub fmt: Format<'a>,
}

#[cfg(feature = "password")]
#[cfg_attr(nightly, doc(cfg(feature = "password")))]
impl<'a> From<&'a str> for Password<'a> {
    fn from(msg: &'a str) -> Self {
        Self::new(msg)
    }
}

#[cfg(feature = "password")]
#[cfg_attr(nightly, doc(cfg(feature = "password")))]
impl<'a> Password<'a> {
    fn fmt_with_<W: fmt::Write>(&self, mut f: W, fmt: &Format<'_>, opt: bool) -> fmt::Result {
        f.write_str(fmt.prefix)?;
        f.write_str(self.msg)?;

        if opt {
            f.write_str(" (optional)")?;
        }

        if fmt.line_brk {
            f.write_char('\n')?;
        }
        Ok(())
    }

    pub fn new(msg: &'a str) -> Self {
        Self {
            msg,
            fmt: Format::default(),
        }
    }

    pub fn format(self, fmt: Format<'a>) -> Self {
        Self { fmt, ..self }
    }
}

#[cfg(feature = "password")]
#[cfg_attr(nightly, doc(cfg(feature = "password")))]
impl UsesFormat for Password<'_> {
    fn get_format(&self) -> &Format<'_> {
        &self.fmt
    }
}

#[cfg(feature = "password")]
#[cfg_attr(nightly, doc(cfg(feature = "password")))]
impl MenuDisplay for Password<'_> {
    fn fmt_with<W: fmt::Write>(&self, f: W, fmt: &Format<'_>, opt: bool) -> fmt::Result {
        if !fmt.line_brk {
            return Ok(());
        }

        self.fmt_with_(f, fmt, opt)
    }
}

#[cfg(feature = "password")]
#[cfg_attr(nightly, doc(cfg(feature = "password")))]
impl Display for Password<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Self as MenuDisplay>::fmt(self, f, false)
    }
}

#[cfg(feature = "password")]
#[cfg_attr(nightly, doc(cfg(feature = "password")))]
impl Promptable<String> for Password<'_> {
    type Middle = String;

    fn prompt_once<H: Handle>(
        &self,
        mut handle: H,
        fmt: &Format<'_>,
        opt: bool,
    ) -> MenuResult<Option<Self::Middle>> {
        if fmt.line_brk {
            handle.write_all(fmt.suffix.as_bytes())?;
            handle.flush()?;
        } else {
            let mut s = String::new();
            self.fmt_with_(&mut s, fmt, opt)?;
            handle.write_all(s.as_bytes())?;
            handle.write_all(fmt.suffix.as_bytes())?;
            handle.flush()?;
        }

        let s = read_password()?;
        if s.is_empty() {
            return Ok(None);
        }

        Ok(s.parse().ok())
    }

    fn convert(self, mid: Self::Middle) -> String {
        mid
    }
}

#[cfg(feature = "password")]
#[cfg_attr(nightly, doc(cfg(feature = "password")))]
pub type PasswordUntil<'a, F> = Until<Password<'a>, F>;

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
/// use ezmenulib::prelude::*;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let author: Vec<String> = Written::from("Give the authors of the license")
///     .many_values(&mut MenuStream::default(), ", ")?;
/// # Ok(()) }
/// ```
#[derive(Debug, Clone)]
pub struct Written<'a> {
    msg: &'a str,
    /// The format of the written field value.
    pub fmt: Format<'a>,
    example: Option<&'a str>,
    default: Option<Cow<'a, str>>,
}

impl UsesFormat for Written<'_> {
    fn get_format(&self) -> &Format<'_> {
        &self.fmt
    }
}

impl MenuDisplay for Written<'_> {
    fn fmt_with<W: fmt::Write>(&self, f: W, fmt: &Format<'_>, opt: bool) -> fmt::Result {
        // We will write the prompt at the `Promptable::prompt_once` method call.
        if !fmt.line_brk {
            return Ok(());
        }

        self.fmt_with_(f, fmt, opt)
    }
}

impl<T: FromStr> Promptable<T> for Written<'_> {
    type Middle = T;

    fn prompt_once<H: Handle>(
        &self,
        mut handle: H,
        fmt: &Format<'_>,
        opt: bool,
    ) -> MenuResult<Option<Self::Middle>> {
        #[inline(never)]
        fn unwrap_default<T: FromStr>(s: &str) -> T {
            s.parse()
                .unwrap_or_else(|_| panic!("invalid default type for written value"))
        }

        if fmt.line_brk {
            // Write only suffix on next line
            handle.write_all(fmt.suffix.as_bytes())?;
            handle.flush()?;
        } else {
            // Write the whole prompt (message + suffix)
            let mut s = String::new();
            self.fmt_with_(&mut s, fmt, opt)?;
            handle.write_all(s.as_bytes())?;
            handle.write_all(fmt.suffix.as_bytes())?;
            handle.flush()?;
        }

        let s = handle.read_input()?;
        if s.is_empty() {
            return Ok(self.default.as_deref().map(unwrap_default));
        }
        let out = s
            .parse()
            .ok()
            .or_else(|| self.default.as_deref().map(unwrap_default));

        Ok(out)
    }

    fn convert(self, mid: Self::Middle) -> T {
        mid
    }
}

impl<'a> From<&'a str> for Written<'a> {
    fn from(msg: &'a str) -> Self {
        Self::new(msg)
    }
}

impl Display for Written<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Self as MenuDisplay>::fmt(self, f, false)
    }
}

/// Constructor methods defining how the field behaves
impl<'a> Written<'a> {
    fn fmt_with_<W: fmt::Write>(&self, mut f: W, fmt: &Format<'_>, opt: bool) -> fmt::Result {
        f.write_str(fmt.prefix)?;
        f.write_str(self.msg)?;

        // Field details
        if opt || self.example.is_some() || self.default.is_some() {
            f.write_str(" (")?;

            // - Example
            if let Some(e) = self.example {
                write!(f, "example: {}", e)?;
                if opt || self.fmt.show_default && self.default.is_some() {
                    f.write_str(", ")?;
                }
            }

            // - Default
            match &self.default {
                Some(d) if self.fmt.show_default => write!(f, "default: {}", d)?,
                _ => (),
            }

            // - Optional
            if opt && self.default.is_none() {
                f.write_str("optional")?;
            }

            f.write_str(")")?;
        }

        if fmt.line_brk {
            f.write_char('\n')?;
        }
        Ok(())
    }

    pub fn new(msg: &'a str) -> Self {
        Self {
            msg,
            fmt: Format::default(),
            example: None,
            default: None,
        }
    }

    /// Gives a custom formatting for the written field.
    ///
    /// # Example
    ///
    /// ```
    /// # use ezmenulib::prelude::*;
    /// let w = Written::from("hello").format(Format::prefix("==> "));
    /// ```
    pub fn format(self, fmt: Format<'a>) -> Self {
        Self { fmt, ..self }
    }

    /// Gives the default value accepted by the field.
    ///
    /// If the value type is incorrect, the [`Written::prompt`] function and its variations
    /// will panic at runtime.
    ///
    /// The default value and the example (see the [`example`](Written::example) method documentation)
    /// will be displayed inside parenthesis according to its formatting (see [`Format`]
    /// for more information).
    pub fn default_value(self, default: &'a str) -> Self {
        let default = Some(Cow::Borrowed(default));
        Self { default, ..self }
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
    pub fn default_env(self, var: &'a str) -> MenuResult<Self> {
        let default = env::var(var).map_err(|e| MenuError::EnvVar(var.to_owned(), e))?;
        let default = Some(Cow::Owned(default));
        Ok(Self { default, ..self })
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
    pub fn example(self, example: &'a str) -> Self {
        let example = Some(example);
        Self { example, ..self }
    }
}

pub type WrittenUntil<'a, F> = Until<Written<'a>, F>;

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
    fn select() -> Selected<'static, Self, N>;
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
// Clone is implemented on it because it is moved once the user selected the value.
#[derive(Debug, Clone)]
pub struct Selected<'a, T, const N: usize> {
    /// The format used by the selected field value.
    pub fmt: Format<'a>,
    msg: &'a str,
    fields: [(&'a str, T); N],
    default: Option<usize>,
}

impl<T, const N: usize> UsesFormat for Selected<'_, T, N> {
    fn get_format(&self) -> &Format<'_> {
        &self.fmt
    }
}

impl<T, const N: usize> MenuDisplay for Selected<'_, T, N> {
    fn fmt_with<W: fmt::Write>(&self, mut f: W, fmt: &Format<'_>, opt: bool) -> fmt::Result {
        write!(f, "{}{}", fmt.prefix, self.msg)?;
        if opt && self.default.is_none() || self.default.is_some() && !fmt.show_default {
            f.write_str(" (optional)")?;
        }
        f.write_str("\n")?;

        for (i, (msg, _)) in (1..=N).zip(self.fields.iter()) {
            write!(f, "{}{i}{}{}{msg}", fmt.left_sur, fmt.right_sur, fmt.chip)?;
            match self.default {
                Some(x) if x == i && fmt.show_default => f.write_str(" (default)")?,
                _ => (),
            }
            f.write_str("\n")?;
        }

        Ok(())
    }
}

impl<T, const N: usize> Promptable<T> for Selected<'_, T, N> {
    type Middle = usize;

    fn prompt_once<H: Handle>(
        &self,
        handle: H,
        fmt: &Format<'_>,
        _opt: bool,
    ) -> MenuResult<Option<Self::Middle>> {
        select(handle, fmt.suffix, N)
            .map(|o| o.or_else(|| self.default.map(|i| i.saturating_sub(1))))
    }

    fn convert(self, mid: Self::Middle) -> T {
        self.fields
            .into_iter()
            .nth(mid)
            .expect("index out of bound for selected prompt")
            .1
    }
}

impl<'a, T, const N: usize> Selected<'a, T, N> {
    fn new_(msg: &'a str, fields: [(&'a str, T); N], default: Option<usize>) -> Self {
        check_fields(fields.as_ref());

        Self {
            fmt: Default::default(),
            msg,
            fields,
            default,
        }
    }

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
        Self::new_(msg, fields, None)
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
}

impl<'a, T, const N: usize> Display for Selected<'a, T, N> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        <Self as MenuDisplay>::fmt(self, f, false)
    }
}

/// A menu field.
///
/// The string slice corresponds to the message displayed in the list,
/// and the kind corresponds to its behavior.
///
/// See [`Kind`] for more information.
pub type Field<'a, H = MenuHandle> = (&'a str, Kind<'a, H>);

/// The menu fields.
///
/// It simply corresponds to a slice of fields.
/// It is used for more convenience in the library.
pub type Fields<'a, H = MenuHandle> = Vec<Field<'a, H>>;

/// Corresponds to the function mapped to a field.
///
/// This function is called right after the user selected the corresponding field.
///
/// See [`Kind::Map`] for more information.
// pub type Binding<R = In, W = Out> = fn(&mut MenuStream<R, W>) -> MenuResult;
pub type Callback<H = MenuHandle> = Box<dyn FnMut(&mut H) -> MenuResult>;

/// Defines the behavior of a menu [field](Field).
pub enum Kind<'a, H = MenuHandle> {
    /// Maps a function to call right after the user selects the field.
    Map(Callback<H>),
    /// Defines the current field as a parent menu of a sub-menu defined by the given fields.
    Parent(Fields<'a, H>),
    /// Allows the user to go back to the given depth level from the current running prompt.
    ///
    /// The depth level of the current running prompt is at `0`, meaning it will stay at
    /// the current level if the index is at `0` when the user will select the field.
    Back(usize),
    /// Closes all the nested menus to the top when the user selects the field.
    Quit,
}

pub trait IntoFields<'a, H = MenuHandle> {
    fn into_fields(self) -> Fields<'a, H>;
}

impl<'a, H, T> IntoFields<'a, H> for T
where
    T: IntoIterator<Item = Field<'a, H>>,
{
    fn into_fields(self) -> Fields<'a, H> {
        Vec::from_iter(self)
    }
}

impl<'a, H> fmt::Debug for Kind<'a, H> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("Field::")?;
        match self {
            Self::Map(_) => f.write_str("Map"),
            Self::Parent(fields) => f.debug_tuple("Parent").field(fields).finish(),
            Self::Back(i) => f.debug_tuple("Back").field(i).finish(),
            Self::Quit => f.write_str("Quit"),
        }
    }
}

pub mod kinds {
    use super::*;

    #[macro_export]
    macro_rules! mapped {
        ($f:expr, $($s:expr),* $(,)?) => {{
            $crate::field::kinds::map(move |s| $f(s, $($s),*))
        }};
    }

    pub fn map<'a, F, R, H>(mut f: F) -> Kind<'a, H>
    where
        F: FnMut(&mut H) -> R + 'static,
        R: IntoResult,
    {
        Kind::Map(Box::new(move |d| f(d).into_result()))
    }

    #[inline(always)]
    pub fn parent<'a, I, H>(fields: I) -> Kind<'a, H>
    where
        I: IntoFields<'a, H>,
    {
        Kind::Parent(fields.into_fields())
    }

    #[inline(always)]
    pub fn back<'a, H>(i: usize) -> Kind<'a, H> {
        Kind::Back(i)
    }

    #[inline(always)]
    pub fn quit<'a, H>() -> Kind<'a, H> {
        Kind::Quit
    }
}

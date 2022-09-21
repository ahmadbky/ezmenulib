//! Module that defines several types for retrieving values from the user and interact with him.
//!
//! It contains the main aspects of the library:
//!
//! * The [`Promptable`] trait, to characterize the behavior of a prompt.
//! * The [`Format`] struct, to customize the format of a prompt.
//! * The [raw menu fields](Fields), to describe the type of a menu field.
//!
//! # The `Promptable` trait
//!
//! The `Promptable` trait serves to get an input value from the user, in any manner. The main way
//! is to loop the prompt to the user until he enters a correct value, which corresponds to the
//! [`Promptable::get`] method.
//!
//! You can also choose to prompt the user for an optional value,
//! with the [`Promptable::get_optional`] method. The latter returns an `Option<T>` where
//! `T` correponds to the output value. Thus, it will not loop the prompt to the user, but only
//! display to him once. If the user enters an incorrect input, or skips the prompt, it will return
//! `None`. Otherwise, it will return `Some(value)`.
//!
//! For syntactic sugar, there also exists the [`Promptable::get_or_default`] method,
//! which has the same behavior as the `get_optional`, but calls for the [`Default`]`::default`
//! method on the output type if `None` is returned.
//!
//! ```no_run
//! use ezmenulib::prelude::*;
//! let age: u8 = Written::new("how old are you?").get();
//! let age: Option<u8> = Written::new("how old are you?").get_optional();
//! // assuming the user entered a correct value:
//! assert!(age.is_some());
//! let age: u8 = Written::new("how old are you?").get_or_default();
//! // assuming the user hasn't entered a correct value or skipped the prompt:
//! assert_eq!(age, 0);
//! ```
//!
//! ## The promptable types
//!
//! There exists many promptable types that implement this trait. The most common one is [`Written`],
//! which correponds to a value written by the user. There is also the [`Selected`] one, which
//! asks the user to enter an index, to select a field among the provided selectable fields.
//!
//! Each type has its own behavior, defined by its name. You can retrieve [passwords](Password),
//! [boolean values](Bool), or a [list of values](Separated) from the user input.
//!
//! ## The `Values` struct
//!
//! The `Promptable` trait is also used by the [`Values`] struct. It is a simple container of a
//! [format](Format) and a [handle](Handle), and shares them to the promptable type that is
//! provided as argument to its associated functions. This helps to use a global format and handle
//! accross all the prompts.
//!
//! ```no_run
//! let mut vals = Values::from_format(Format::suffix("> "));
//! let age: u8 = vals.next(Written::new("how old are you?"));
//! ```
//!
//! # Customize the format of a prompt
//!
//! Every method of the `Promptable` trait is combined with its sibling, which accepts a provided
//! [format](Format) as argument to customize the prompt. For example, to provide a custom format
//! when calling the [`Promptable::prompt`] method, you may use the
//! [`prompt_with`](Promptable::prompt_with) method:
//!
//! ```no_run
//! let custom = Bool::new("should we use a custom format for the next prompt?").get();
//! let w = Written::new("how old are you?").format(Format::suffix("> "));
//! let age: u8 = if custom {
//!     w.get_with(&Format::prefix("-> "))
//! } else {
//!     w.get()
//! };
//! ```
//!
//! When calling the `*_with` methods, the provided format will be merged into the format used by
//! the promptable type, by saving its custom format specifications. So here, the suffix of the
//! [`Written`] prompt will be saved as `"> "`, but the prefix will de facto be `"-> "`:
//!
//! ```text
//! --> should we use a custom format for the next prompt?
//! >> yes
//! -> how old are you?
//! >
//! ```
//!
//! # Raw menu fields
//!
//! When constructing a [raw menu](RawMnu), you provide the fields of the menu. These fields
//! will be selected by the user, and thus will call predefined instructions bound on it.
//!
//! These instructions correponds to the [kind](Kind) of a [raw menu field](Field). Each kind has
//! its own behavior, to dynamize the menu. It allows you to make a field defined as a parent field
//! of a sub-menu, or to make it quit the menu.
//!
//! The most common kind is the [`Kind::Map`] variant. It is defined to provide a function or closure
//! to call right after the user selected the bound field. The function must take a
//! `<H> fn(&mut H) -> R` where `R` is the unit type `()`, or a [`Result<T, E>`] type where `E`
//! can be converted into a [`MenuError`]. You can use the [`bound`] attribute macro on your function
//! to transform its signature into an usable one for a raw menu.
//!
//! ```
//! use ezmenulib::prelude::*;
//!
//! #[bound]
//! fn greetings() {
//!     println!("hi!");
//! }
//!
//! let just_hi = RawMenu::from([("hello", kinds::map(greetings))]).run_once();
//! ```

#[cfg(test)]
mod tests;

use crate::{customs::MenuBool, menu::Handle, prelude::*, utils::*, DEFAULT_FMT};
use std::{
    borrow::Cow,
    env,
    fmt::{self, Display, Formatter},
    marker::PhantomData,
    str::FromStr,
};

/// The "ezmenulib" version of the [`Display`] trait.
///
/// It is used to print out a promptable to the buffer with some context.
/// The context corresponds to the [format](Format) and if the prompt is set as optional or not,
/// defined by the `opt` parameter.
pub trait MenuDisplay {
    /// Writes the promptable trait to the `W` buffer with the given format.
    ///
    /// # Arguments
    ///
    /// * f: The buffer to write to.
    /// * fmt: The prompt format used to print the promptable.
    /// * opt: Defines if the prompted is optional or not.
    fn fmt_with<W: fmt::Write>(&self, f: W, fmt: &Format<'_>, opt: bool) -> fmt::Result;

    /// Writes the promptable trait to the `W` buffer.
    ///
    /// # Arguments
    ///
    /// * f: The buffer to write to.
    /// * opt: Defines if the prompted is optional or not.
    fn fmt<W: fmt::Write>(&self, f: W, opt: bool) -> fmt::Result
    where
        Self: UsesFormat,
    {
        self.fmt_with(f, self.get_format(), opt)
    }
}

/// Defines a promptable that contains its own [`Format`].
pub trait UsesFormat {
    /// Returns the format used by the promptable.
    fn get_format(&self) -> &Format<'_>;
}

pub(crate) const PROMPT_ERR_MSG: &str = "error while prompting field";

/// Represents an object that returns a `T` value entered by the user.
pub trait Promptable<T>: Sized + MenuDisplay + UsesFormat {
    /// The type corresponding to the user text input.
    ///
    /// Usually, it corresponds to the `T` type.
    /// It is mainly declared for the [`Selected<T>`] promptable type, because the middle type
    /// is the `usize` index, while the final output type correponds to the `T` type pointed at this index.
    ///
    /// To convert from this type to the `T` type, the trait uses the
    /// [`convert`](Promptable::convert) method.
    type Middle;

    /// Prompts the struct once to the user, corresponding to a single try of parsing the entered value.
    ///
    /// If the parsing was a success, it returns `Some(output)`, otherwise it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `handle`: The [handle](Handle) used to write the promptable to and retrieve the input from.
    /// * `fmt`: The format used for the prompt.
    /// * `opt`: Defines if the prompt is optional or not.
    fn prompt_once<H: Handle>(
        &self,
        handle: H,
        fmt: &Format<'_>,
        opt: bool,
    ) -> MenuResult<Option<Self::Middle>>;

    /// Converts the middle type to the output `T` type.
    ///
    /// Usually, it immediately returns the `mid` value.
    /// For the [`Selected<T>`] promptable, it returns the value pointed at the middle index
    /// (which is an usize).
    fn convert(self, mid: Self::Middle) -> T;

    /// Prompts to the user and returns the value he entered.
    ///
    /// It loops the prompt until he enters a correct value.
    ///
    /// # Panic
    ///
    /// This method panics if an [error](MenuError) occurred.
    fn get(self) -> T {
        self.try_get().expect(PROMPT_ERR_MSG)
    }

    /// Prompts to the user with the given format and returns the value he entered.
    ///
    /// It loops the prompt until he enters a correct value.
    ///
    /// # Panic
    ///
    /// This method panics if an [error](MenuError) occurred.
    fn get_with(self, fmt: &Format<'_>) -> T {
        self.try_get_with(fmt).expect(PROMPT_ERR_MSG)
    }

    /// Prompts to the user and returns the value he entered safely.
    ///
    /// It loops the prompt until he enters a correct value.
    fn try_get(self) -> MenuResult<T> {
        self.prompt(MenuHandle::default())
    }

    /// Prompts to the user with the given format and returns the value he entered safely.
    ///
    /// It loops the prompt until he enters a correct value.
    fn try_get_with(self, fmt: &Format<'_>) -> MenuResult<T> {
        self.prompt_with(MenuHandle::default(), fmt)
    }

    /// Prompts to the user and returns the value he entered
    /// in an optional way.
    ///
    /// The optional way means that if the entered value is correct, then the function will
    /// return `Some(value)`, or `None` if he skipped the prompt or if he entered an incorrect input.
    ///
    /// # Panic
    ///
    /// This method panics if an [error](MenuError) occurred.
    fn get_optional(self) -> Option<T> {
        self.try_get_optional().expect(PROMPT_ERR_MSG)
    }

    /// Prompts to the user with the given format and returns the value he entered
    /// in an optional way.
    ///
    /// The optional way means that if the entered value is correct, then the function will
    /// return `Some(value)`, or `None` if he skipped the prompt or if he entered an incorrect input.
    ///
    /// # Panic
    ///
    /// This method panics if an [error](MenuError) occurred.
    fn get_optional_with(self, fmt: &Format<'_>) -> Option<T> {
        self.try_get_optional_with(fmt).expect(PROMPT_ERR_MSG)
    }

    /// Prompts safely to the user and returns the value he entered
    /// in an optional way.
    ///
    /// The optional way means that if the entered value is correct, then the function will
    /// return `Some(value)`, or `None` if he skipped the prompt or if he entered an incorrect input.
    fn try_get_optional(self) -> MenuResult<Option<T>> {
        self.optional_prompt(MenuHandle::default())
    }

    /// Prompts safely to the user with the given format and returns the value he entered
    /// in an optional way.
    ///
    /// The optional way means that if the entered value is correct, then the function will
    /// return `Some(value)`, or `None` if he skipped the prompt or if he entered an incorrect input.
    fn try_get_optional_with(self, fmt: &Format<'_>) -> MenuResult<Option<T>> {
        self.optional_prompt_with(MenuHandle::default(), fmt)
    }

    /// Prompts to the user and returns the value he entered
    /// or the default value of the type.
    ///
    /// If the entered value is correct, then the function will
    /// return it, otherwise if the user skipped the prompt or entered an incorrect value
    /// or if an [error](MenuError) occurred, it will return `Default::default()`.
    fn get_or_default(self) -> T
    where
        T: Default,
    {
        self.prompt_or_default(MenuHandle::default())
    }

    /// Prompts to the user and returns the value he entered
    /// or the default value of the type.
    ///
    /// If the entered value is correct, then the function will
    /// return it, otherwise if the user skipped the prompt or entered an incorrect value
    /// or if an [error](MenuError) occurred, it will return `Default::default()`.
    fn get_or_default_with(self, fmt: &Format<'_>) -> T
    where
        T: Default,
    {
        self.prompt_or_default_with(MenuHandle::default(), fmt)
    }

    /// Prompts to the user with the given format and returns the value he entered, by using the
    /// given handle.
    ///
    /// It loops the prompt until he enters a correct value.
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

    /// Prompts to the user and returns the value he entered, by using the
    /// given handle.
    ///
    /// It loops the prompt until he enters a correct value.
    fn prompt<H: Handle>(self, mut handle: H) -> MenuResult<T> {
        handle.show(&self, self.get_format(), false)?;
        loop {
            match self.prompt_once(&mut handle, self.get_format(), false)? {
                Some(out) => return Ok(self.convert(out)),
                None => (),
            }
        }
    }

    /// Prompts to the user with the given format and returns the value he entered
    /// in an optional way, by using the given handle.
    ///
    /// The optional way means that if the entered value is correct, then the function will
    /// return `Some(value)`, or `None` if he skipped the prompt or if he entered an incorrect input.
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

    /// Prompts to the user and returns the value he entered
    /// in an optional way, by using the given handle.
    ///
    /// The optional way means that if the entered value is correct, then the function will
    /// return `Some(value)`, or `None` if he skipped the prompt or if he entered an incorrect input.
    fn optional_prompt<H: Handle>(self, mut handle: H) -> MenuResult<Option<T>> {
        handle.show(&self, self.get_format(), true)?;
        self.prompt_once(handle, self.get_format(), true)
            .map(|opt| opt.map(|m| self.convert(m)))
    }

    /// Prompts to the user with the given format and returns the value he entered
    /// or the default value of the type, by using the given handle.
    ///
    /// If the entered value is correct, then the function will
    /// return it, otherwise if the user skipped the prompt or entered an incorrect value
    /// or if an [error](MenuError) occurred, it will return `Default::default()`.
    fn prompt_or_default_with<H: Handle>(self, handle: H, fmt: &Format<'_>) -> T
    where
        T: Default,
    {
        self.optional_prompt_with(handle, fmt)
            .map(Option::unwrap_or_default)
            .unwrap_or_default()
    }

    /// Prompts to the user and returns the value he entered
    /// or the default value of the type, by using the given handle.
    ///
    /// If the entered value is correct, then the function will
    /// return it, otherwise if the user skipped the prompt or entered an incorrect value
    /// or if an [error](MenuError) occurred, it will return `Default::default()`.
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
    /// Defines the prompt formatting specifications.
    ///
    /// To understand the role of each format specification, let's say that:
    ///
    /// * `[...]` means that the content is displayed if at least one of its elements is provided.
    /// * `{spec:...}` means that the content is displayed only if the spec `spec` is set at true.
    /// * `<...>` means a provided string slice (that might be missing, for example <default_value>).
    /// * Otherwise, everything is provided literally "as-is".
    ///
    /// For a written value, the format specifications can be summarized as:
    ///
    /// ```text
    /// <prefix><message>[ ({disp_default:default: <default_value>}, example: <example>)]{line_brk:\n}<suffix>
    /// ```
    ///
    /// If a default value is provided and the `show_default` format spec has been set to `false`,
    /// or the prompt is declared as optional, it will show `optional` in instead of `default: ...`.
    ///
    /// If the `line_brk` spec is set to `true`, each loop iteration to force him to enter a correct value
    /// will only show the suffix, because it will be on a separate line. Otherwise,
    /// if `line_brk` is set to false, it will reprint the whole line at each loop iteration.
    ///
    /// For a selected value and the [raw menu](crate::menu::RawMenu), the format specifications follows this pattern:
    ///
    /// ```text
    /// <prefix><message>
    /// <left_sur><X0><right_sur><chip><field0>[{show_default: (default)}]
    /// <left_sur><X1><right_sur><chip><field1>[{show_default: (default)}]
    /// ...
    /// <suffix>
    /// ```
    ///
    /// The `line_brk` of the selected/raw menu prompt cannot be turned to `false`.
    /// If so, it will use the default suffix spec (`">> "`).
    ///
    /// Same as the written values, if a default index is given to the selected promptable,
    /// but with the `show_default` spec set as `false`, or if the prompt is declared as optional,
    /// then the `" (default)"` next to the default field will be removed and an `" (optional)"`
    /// label will appear nex to the `<message>`.
    ///
    /// # Examples
    ///
    /// This written promptable:
    ///
    /// ```
    /// Written::new("hehe")
    ///     .format(Format {
    ///         suffix: ": ",
    ///         line_brk: false,
    ///         ..Default::default()
    ///     })
    ///     .default_value("hoho")
    ///     .example("huhu")
    /// ```
    ///
    /// will result to this output when prompted:
    ///
    /// ```text
    /// --> hehe (default: hoho, example: huhu):
    /// ```
    ///
    /// This selected promptable:
    ///
    /// ```
    /// // If a specification is provided alone, the Format struct can be constructed from it.
    /// let fmt = Format::show_default(false);
    /// Selected::new("hehe", [("hoho", 0), ("huhu", 1), ("haha", 2)])
    ///     .default(1)
    ///     .format(fmt)
    /// ```
    ///
    /// will result to this output when prompted:
    ///
    /// ```text
    /// --> hehe (optional)
    /// [1] - hoho
    /// [2] - huhu
    /// [3] - haha
    /// ```
    prefix: &'a str,
    /// Sets the prefix of the format (`"--> "` by default).
    ///
    /// It corresponds to the string slice displayed at the beginning of the prompt line.
    left_sur: &'a str,
    /// Defines the left "surrounding" of the index when displaying a list (`"["` by default).
    ///
    /// It is displayed between at the beginning of the list field line, before the index.
    right_sur: &'a str,
    /// Defines the right "surrounding" of the index when displaying a list (`"]"` by default).
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
    /// it will always be displayed.
    ///
    /// If a promptable has a default value but this spec is set as `false`, the prompt will be
    /// shown as `"(optional)"`.
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
    /// For selected prompt, if this specification is set as `false`,
    /// it will use the default suffix, and always use a line break, for more convenience.
);

/// The default format uses:
///
/// * `"--> "` as prefix
/// * `"["` as left surrounding
/// * `"]"` as right surrounding
/// * `" - "` as chip
/// * `true` to show the default value
/// * `">> "` as suffix
/// * `true` to put a line break
impl<'a> Default for Format<'a> {
    fn default() -> Self {
        DEFAULT_FMT
    }
}

/// Represents a boolean promptable.
///
/// The boolean is useful to ask the user a dichotomous question, so he can answer by yes or no.
/// In fact, this promptable is a shortcut for a written prompt that only accepts a boolean value.
/// It returns a `bool` when prompted.
///
/// The input parsing is flexible and accepts pretty much every form of boolean reply,
/// such as "yes", "Y", "yep", etc.
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
    /// Returns the boolean promptable from its inner written promptable.
    pub fn from_written(inner: Written<'a>) -> Self {
        Self { inner }
    }

    /// Creates the boolean promptable from the message displayed to the user.
    pub fn new(msg: &'a str) -> Self {
        Self::from_written(From::from(msg))
    }

    /// Sets the format of the prompt.
    pub fn format(self, fmt: Format<'a>) -> Self {
        Self::from_written(self.inner.format(fmt))
    }

    /// Sets the custom example of the boolean promptable.
    pub fn example(self, example: &'a str) -> Self {
        Self::from_written(self.inner.example(example))
    }

    /// Sets `"yes/no"` as example for the boolean promptable.
    pub fn with_basic_example(self) -> Self {
        self.example("yes/no")
    }

    /// Sets the default value for the boolean promptable.
    pub fn default_value(self, default: bool) -> Self {
        let val = if default { "yes" } else { "no" };
        Self::from_written(self.inner.default_value(val))
    }

    /// Sets the default value from the given environment variable
    /// for the boolean promptable.
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

/// Represents a list of values of type `T`, collected into an `I` type.
///
/// It uses a separator to parse the input string entered by the user.
/// Thus, it has the same behavior as a written prompt.
/// It can use a special separator for an environment variable that contains many values.
///
/// For example, the type `Separated<Vec<i32>, i32>` will return a `Vec<i32>` when prompted.
#[derive(Debug, Clone)]
pub struct Separated<'a, I, T> {
    inner: Written<'a>,
    sep: &'a str,
    env_sep: Option<&'a str>,
    _marker: PhantomData<(I, T)>,
}

impl<'a, I, T> Separated<'a, I, T> {
    /// Returns the separated promptable from its inner written promptable.
    pub fn from_written(inner: Written<'a>, sep: &'a str) -> Self {
        Self {
            inner,
            sep,
            env_sep: None,
            _marker: PhantomData,
        }
    }

    /// Creates the separated promptable from the message displayed to the user,
    /// and the separator for parsing his input.
    pub fn new(msg: &'a str, sep: &'a str) -> Self {
        Self::from_written(From::from(msg), sep)
    }

    /// Sets the format of the prompt.
    pub fn format(self, fmt: Format<'a>) -> Self {
        let inner = self.inner.format(fmt);
        Self { inner, ..self }
    }

    /// Sets the custom example of the separated promptable.
    pub fn example(self, example: &'a str) -> Self {
        let inner = self.inner.example(example);
        Self { inner, ..self }
    }

    /// Sets the default value for the separated promptable.
    pub fn default_value(self, default: &'a str) -> Self {
        let inner = self.inner.default_value(default);
        Self { inner, ..self }
    }

    /// Sets the default value for the separated promptable from the
    /// given environment variable.
    ///
    /// # Note
    ///
    /// It will parse its content to collect the separated values, by using
    /// the same separator of the user input. If you want to use a special separator
    /// for the environment variable, consider using the
    /// [`default_env_with`](Separated::default_env_with) method.
    pub fn default_env(self, var: &'a str) -> MenuResult<Self> {
        let inner = self.inner.default_env(var)?;
        Ok(Self { inner, ..self })
    }

    /// Sets the default value for the separated promptable from the
    /// environment variable, with its custom separator.
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

/// Corresponds to a promptable that checks that the given `F` predicate is applied to the
/// separated prompt.
///
/// The function must take a reference to the collection type and return a [`bool`].
/// The collection type correspond to the `I` generic type, that must implement [`FromIterator`] trait.
///
/// # Example
///
/// ```no_run
/// let date: Vec<i32> =
///     Until::from_promptable(Separated::new("enter a date", "/"), |v| v.len() == 3).get();
/// ```
pub type SeparatedUntil<'a, I, T, F> = Until<Separated<'a, I, T>, F>;

/// Represents the `P` promptable, filtered by the `F` predicate.
///
/// This promptable has many derived type definitions, such as [`SeparatedUntil`],
/// [`WrittenUntil`], etc.
///
/// At each request from the user, it maps the output value to the `F` function to keep or not
/// the input value.
///
/// To retrieve values from the user, the `P` parameter must implement the [`Promptable`] type,
/// and the `F` parameter must correspond to the `fn(&T)` function signature, where
/// `T` is the [middle output value type](Promptable::Middle) of the `P` implementation
/// (usually corresponding directly to the output value type).
#[derive(Debug, Clone)]
pub struct Until<P, F> {
    inner: P,
    til: F,
}

impl<'a, P: From<&'a str>, F> Until<P, F> {
    /// Creates the promptable from the message displayed to the user,
    /// and the predicate used to filter the output value.
    ///
    /// This function can be used only if the promptable implements [`From<&str>`], such as
    /// [`Written`] or [`Password`].
    pub fn new(msg: &'a str, til: F) -> Self {
        Self::from_promptable(From::from(msg), til)
    }
}

impl<P, F> Until<P, F> {
    /// Creates the promptable from its inner promptable and the predicate used to filter
    /// the output value.
    ///
    /// If the inner promptable type implements [`From<&str>`], such as [`Written`], or [`Password`],
    /// consider using the [`Until::new`](Until::new) function.
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

/// Represents a password prompt.
///
/// The password data is stored as a [`String`].
///
/// The prompt consists of hiding the text when the user enters his password.
#[cfg(feature = "password")]
#[cfg_attr(nightly, doc(cfg(feature = "password")))]
#[derive(Debug)]
pub struct Password<'a> {
    msg: &'a str,
    /// The format used by the prompt.
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

    /// Returns the password promptable from the message displayed to the user.
    pub fn new(msg: &'a str) -> Self {
        Self {
            msg,
            fmt: Format::default(),
        }
    }

    /// Sets the format of the prompt.
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

        let s = rpassword::read_password()?;
        if s.is_empty() {
            return Ok(None);
        }

        Ok(s.parse().ok())
    }

    fn convert(self, mid: Self::Middle) -> String {
        mid
    }
}

/// Correponds to a promptable that checks that the given `F` predicate is applied to the
/// password prompt.
///
/// The function must take a reference to a [`String`], or an `&`[`str`], and return a [`bool`].
///
/// # Example
///
/// ```no_run
/// let date: String = PasswordUntil::new("enter your password", |s| s.len() >= 5).get();
/// ```
#[cfg(feature = "password")]
#[cfg_attr(nightly, doc(cfg(feature = "password")))]
pub type PasswordUntil<'a, F> = Until<Password<'a>, F>;

/// Represents a written prompt.
///
/// It displays the message, with a given example and default value if it is provided
/// (see [`Written::example`] or [`Written::default_value`] functions).
///
/// # Example
///
/// ```no_run
/// let author: String = Written::new("What is your name?").get();
/// ```
#[derive(Debug, Clone)]
pub struct Written<'a> {
    msg: &'a str,
    /// The format used by the prompt.
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
            // Writes only suffix on next line
            handle.write_all(fmt.suffix.as_bytes())?;
            handle.flush()?;
        } else {
            // Writes the whole prompt (message + suffix)
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

    /// Returns the written promptable from the message displayed to the user.
    pub fn new(msg: &'a str) -> Self {
        Self {
            msg,
            fmt: Format::default(),
            example: None,
            default: None,
        }
    }

    /// Sets the format of the prompt.
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

    /// Sets the default value for the written promptable.
    ///
    /// If the string content is incorrect, the related call to the [`Promptable`] associated
    /// functions will panic at runtime.
    ///
    /// The default value and the example (see the [`example`](Written::example) method documentation)
    /// will be displayed inside parenthesis according to its formatting (see [`Format`]
    /// for more information).
    pub fn default_value(self, default: &'a str) -> Self {
        let default = Some(Cow::Borrowed(default));
        Self { default, ..self }
    }

    /// Sets the default value for the written promptable from the
    /// given environment variable.
    ///
    /// If the provided environment variable is incorrect, it will return an error
    /// (See [`MenuError::EnvVar`] variant).
    ///
    /// If the string content of the variable is incorrect, the related call to
    /// the [`Promptable`] associated functions will panic at runtime.
    ///
    /// # Example
    ///
    /// ```no_run
    /// # use ezmenulib::prelude::*;
    /// let user: String = Written::from("What is your name?").default_env("USERNAME").get();
    /// ```
    pub fn default_env(self, var: &'a str) -> MenuResult<Self> {
        let default = env::var(var).map_err(|e| MenuError::EnvVar(var.to_owned(), e))?;
        let default = Some(Cow::Owned(default));
        Ok(Self { default, ..self })
    }

    /// Sets the custom example of the written promptable.
    ///
    /// Obviously, it is better to give a correct value for the user as example,
    /// but if the value is incorrect, it will only mislead the user,
    /// and unlike the default value providing, the program will not panic at runtime
    /// to emphasize the problem.
    ///
    /// The example will be shown inside parenthesis according to its [format](Format).
    pub fn example(self, example: &'a str) -> Self {
        let example = Some(example);
        Self { example, ..self }
    }
}

/// Corresponds to a promptable that checks that the given `F` predicate is applied
/// to the written prompt.
///
/// The function must take a reference to the output type and return a [`bool`].
///
/// # Example
///
/// ```no_run
/// let name: u8 = WrittenUntil::new("How old are you?", |i| *i >= 10);
/// ```
pub type WrittenUntil<'a, F> = Until<Written<'a>, F>;

/// Defines a selectable type.
///
/// It returns the associated selected prompt of the type.
/// The `N` const generic parameter represents the amount of available selectable values.
///
/// Usually, consider using the `derive(`[`Prompted`]`)` macro to implement this trait on your type.
///
/// # Example
///
/// ```
/// # use ezmenulib::prelude::*;
/// enum Type {
///     MIT,
///     GPL,
///     BSD,
/// }
///
/// impl Selectable<3> for Type {
///     fn select() -> Selected<'static, Self, 3> {
///         Selected::new("License type", [
///             ("MIT", Self::MIT),
///             ("GPL", Self::GPL),
///             ("BSD", Self::BSD),
///         ])
///     }
/// }
///
/// #[derive(Prompted)]
/// enum Amount {
///     One,
///     #[prompted(default)]
///     Two,
///     Three,
///     #[prompted(("Four", 4), ("Five", 5))]
///     More(u8),
/// }
///
/// let mut values = Values::default();
/// let ty = values.next(Type::select());
/// let amount = values.next(Amount::select());
/// ```
///
/// This sample code will result to this output:
///
/// ```text
/// --> License type
/// [1] - MIT
/// [2] - GPL
/// [3] - BSD
/// >> 2
/// --> Amount
/// [1] - One
/// [2] - Two (default)
/// [3] - Three
/// [4] - Four
/// [5] - Five
/// >> 4
/// ```
///
/// Then, the variable `ty` will be bound to `Type::GPL`, and `amount` to `Amount::More(4)`.
pub trait Selectable<const N: usize>: Sized {
    /// Provides the fields, corresponding to a message and the return value.
    fn select() -> Selected<'static, Self, N>;
}

/// Represents a selected prompt.
///
/// It contains the selectable fields, with their associated `T` values.
///
/// It displays the message with the available fields to select, with the
/// default field marked as `(default)` if it is provided (see [`Selected::default`] function).
///
/// The `N` const generic parameter correponds to the amount of selectable values.
///
/// # Example
///
/// ```no_run
/// # use ezmenulib::prelude::*;
/// let adult: bool = Selected::new(
///     "how old are you?",
///     [
///         ("more than 18", true),
///         ("less than 18", false),
///     ],
/// ).get();
/// ```
///
/// For more complex types, consider using the [`Selectable`] trait or the
/// `derive(`[`Prompted`]`)` macro.
#[derive(Debug, Clone)]
pub struct Selected<'a, T, const N: usize> {
    /// The format used by the prompt.
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

    /// Returns the selected prompt from the message displayed to the user
    /// and its the selectable fields.
    ///
    /// # Panic
    ///
    /// If the fields array is empty, this function will panic. Indeed,
    /// when prompting the index to the user to select with an empty list, it will generate an
    /// infinite loop.
    pub fn new(msg: &'a str, fields: [(&'a str, T); N]) -> Self {
        Self::new_(msg, fields, None)
    }

    /// Sets the format of the prompt.
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

    /// Sets the default value for the selected prompt, from its index.
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
/// It simply corresponds to a vector of fields.
/// It is used for more convenience in the library.
///
/// Actually, the library uses the [`IntoFields`] trait to accept more collection types,
/// such as arrays.
pub type Fields<'a, H = MenuHandle> = Vec<Field<'a, H>>;

/// Corresponds to the function mapped to a field.
///
/// This function is called right after the user selected the corresponding field.
/// See [`Kind::Map`] for more information.
pub type Callback<H = MenuHandle> = Box<dyn FnMut(&mut H) -> MenuResult>;

/// Defines the behavior of a menu [field](Field).
///
/// When constructing a [raw menu](RawMenu), you need to bind to each field its behavior.
/// In general, you don't have to call directly the variants. Consider using the
/// [corresponding functions](kinds), for more convenience and syntax sugar.
pub enum Kind<'a, H = MenuHandle> {
    /// Binds a function to call right after the user selects the field.
    ///
    /// See [`kinds::map`] for more documentation.
    Map(Callback<H>),
    /// Defines the current field as a parent menu of a sub-menu defined by the given fields.
    ///
    /// The sub-menu will take the field message of the parent menu as title.
    ///
    /// See [`kinds::parent`] for more documentation.
    Parent(Fields<'a, H>),
    /// Allows the user to go back to the given depth level from the current running prompt.
    ///
    /// The depth level of the current running prompt is at `0`, meaning it will stay at
    /// the current level if the index is at `0` when the user selects the field.
    ///
    /// If the index is at 1, it will return to the previous parent menu, and so on.
    Back(usize),
    /// Closes all the nested menus to the top when the user selects the field, and finishes
    /// the execution of the menu.
    Quit,
}

/// Utility trait for converting a collection object into fields
/// for a [raw menu](crate::menu::RawMenu).
///
/// It is a shortcut to an implementation of the `IntoIterator<Item = `[`Field`]`<'a, H>>` trait.
pub trait IntoFields<'a, H = MenuHandle> {
    /// Converts the collection object into the fields.
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

/// Small module used to gather the utility functions to get the desired field kind for a [raw menu](RawMenu).
///
/// The functions are useful to generate menus without the `derive(Menu)` macro.
/// To distinguish them from the [tui fields kinds](crate::tui), they are placed
/// on a separate module. To use them, you need to import the prelude, and the kinds:
///
/// ```
/// use ezmenulib::{prelude::*, field::kinds::*};
/// ```
///
/// The function calls are shortcuts to the [`Kind`] variants call, for syntactic sugar.
pub mod kinds {
    use super::*;

    /// Utility macro for calling a mapped function for a [raw menu](crate::menu::RawMenu)
    /// by filling the corresponding parameters.
    ///
    /// # Example
    ///
    /// ```
    /// #[bound]
    /// fn play_with(name: &str) {
    ///     println!("Playing with {name}");
    /// }
    ///
    /// let game = RawMenu::from([
    ///     ("Play with Ahmad", mapped!(play_with, "Ahmad")),
    ///     ("Play with Jacques", mapped!(play_with, "Jacques")),
    /// ]);
    /// ```
    #[macro_export]
    macro_rules! mapped {
        ($f:expr, $($s:expr),* $(,)?) => {{
            $crate::field::kinds::map(move |s| $f(s, $($s),*))
        }};
    }

    /// Returns the [`Kind::Map`] variant.
    ///
    /// The mapped function must either return the unit `()` type, or a `Result<T, E>`
    /// where `E` can be converted into a [`MenuError`].
    ///
    /// It must take a single parameter. For other usage, consider using the [`mapped!`] macro.
    pub fn map<'a, F, R, H>(mut f: F) -> Kind<'a, H>
    where
        F: FnMut(&mut H) -> R + 'static,
        R: IntoResult,
    {
        Kind::Map(Box::new(move |d| f(d).into_result()))
    }

    /// Returns the [`Kind::Parent`] variant by converting the input collection into fields.
    pub fn parent<'a, I, H>(fields: I) -> Kind<'a, H>
    where
        I: IntoFields<'a, H>,
    {
        Kind::Parent(fields.into_fields())
    }

    /// Returns the [`Kind::Back`] variant.
    #[inline(always)]
    pub fn back<'a, H>(i: usize) -> Kind<'a, H> {
        Kind::Back(i)
    }

    /// Returns the [`Kind::Quit`] variant.
    #[inline(always)]
    pub fn quit<'a, H>() -> Kind<'a, H> {
        Kind::Quit
    }
}

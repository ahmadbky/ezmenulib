//! Building CLI menus in Rust becomes easy and fast.
//!
//! This repository contains the source code of the ezmenulib crate. This crate provides a library
//! that allows you to build console menus and other prompted resources such as passwords,
//! boolean values, written and selected values, and more. It includes type-checking
//! from the user input, and a prompt format customization.
//!
//! This crate is useful to put beside [clap](https://docs.rs/clap/) crate, to manage the case where
//! the user hasn't provided any command argument.
//!
//! For now, this crate only helps to draw menus in the console. It allows you to not spend time
//! on printing from scratch any form of prompt from the user. If you're making a console application,
//! you *must* use this crate.
//!
//! The built menus are useful for a game mode selection for example. It allows you to map functions
//! when the user selects the corresponding field. It also let you mutate your code depending
//! on the user input.
//!
//! # Provided Cargo features
//!
//! This crate comes with many features:
//!
//! * `derive` (enabled by default): provides derive and attribute macros.
//! * `password` (enabled by default): provides types to prompt a password from the user.
//! * `tui`: opens the `ezmenulib::tui` module letting you build interactive menus.
//! * `crossterm` and `termion`: backends to provide beside the `tui` feature.
//! * `extra-globals`: allows you to use [parking_lot](https://docs.rs/parking_lot/) types with the
//! `derive(Menu)` macro, thus needs to be provided with the `derive` feature.
//!
//! Without any feature enabled, the ezmenulib crate is very lightweight and has no dependency.
//! You still can build non-interactive menus and prompt values from the user with simple instructions.
//!
//! # Basic usage
//!
//! ### Asking values from the user
//!
//! First, for more convenience, you may want to import the ezmenulib prelude
//! when using the crate:
//!
//! ```rust
//! use ezmenulib::prelude::*;
//! ```
//!
//! Then, to ask the name of the user for example, you simply need to provide this instruction:
//!
//! ```rust
//! let name: String = Written::new("What is your name?").get();
//! ```
//!
//! This instruction, when executed, will result to this output:
//!
//! ```text
//! --> What is your name?
//! >>
//! ```
//!
//! `Written` is a struct that implements the `Promptable` trait. This trait is implemented
//! by all the types that prompts a message to the user, then returns the value he entered.
//! It includes type-checking, and loops until the provided value is correct and not empty.
//!
//! If you want to get an optional value, meaning an `Option<T>` as the output type, you can use
//! the `Promptable::get_optional` associated function, and it will return `Some(value)`
//! if the user provided a correct value, otherwise it will return `None`.
//! If you want to get the default value of the type if the user entered an incorrect value,
//! you can use the `Promptable::get_or_default` associated function.
//!
//! There exist many promptable types. You can ask the user for specific values by presenting
//! him available responses, so the user only needs to select the desired field:
//!
//! ```no_run
//! let amount: u8 = Selected::new("How many?", [("only one", 1), ("two", 2), ("none", 0)]).get();
//! ```
//!
//! When executed, this instruction will result to this output:
//!
//! ```text
//! --> How many?
//! [1] - only one
//! [2] - two
//! [3] - none
//! >>
//! ```
//!
//! #### Prompt formatting customization
//!
//! A promptable type, by default, uses its own format to display the text to the user.
//! This format may be changed in the construction of the type.
//! In general, the way to do this is to call the `format` method when constructing the promptable.
//! Then, you must provide the [`Format`] you want, for example:
//!
//! ```
//! Written::new("hey").format(Format {
//!     suffix: ": ",
//!     line_brk: false,
//!     ..Default::default()
//! })
//! ```
//!
//! To understand the role of each format specification, let's say that:
//!
//! * `[...]` means that the content is displayed if at least one of its elements is provided.
//! * `{spec:...}` means that the content is displayed only if the spec `spec` is set at true.
//! * `<...>` means a provided string slice (that might be missing, for example <default_value>).
//! * Otherwise, everything is provided literally "as-is".
//!
//! For a written value, the format specifications can be summarized as:
//!
//! ```text
//! <prefix><message>[ ({disp_default:default: <default_value>}, example: <example>)]{line_brk:\n}<suffix>
//! ```
//!
//! If a default value is provided and the `show_default` format spec has been set to `false`,
//! or the prompt is declared as optional, it will show `optional` in instead of `default: ...`.
//!
//! If the `line_brk` spec is set to `true`, each loop iteration to force him to enter a correct value
//! will only show the suffix, because it will be on a separate line. Otherwise,
//! if `line_brk` is set to false, it will reprint the whole line at each loop iteration.
//!
//! For a selected value and the [raw menu](crate::menu::RawMenu), the format specifications follows this pattern:
//!
//! ```text
//! <prefix><message>
//! <left_sur><X0><right_sur><chip><field0>[{show_default: (default)}]
//! <left_sur><X1><right_sur><chip><field1>[{show_default: (default)}]
//! ...
//! <suffix>
//! ```
//!
//! The `line_brk` of the selected/raw menu prompt cannot be turned to `false`.
//! If so, it will use the default suffix spec (`">> "`).
//!
//! Same as the written values, if a default index is given to the selected promptable,
//! but with the `show_default` spec set as `false`, or if the prompt is declared as optional,
//! then the `" (default)"` next to the default field will be removed and an `" (optional)"`
//! label will appear nex to the `<message>`.
//!
//! ##### Examples
//!
//! This written promptable:
//!
//! ```
//! Written::new("hehe")
//!     .format(Format {
//!         suffix: ": ",
//!         line_brk: false,
//!         ..Default::default()
//!     })
//!     .default_value("hoho")
//!     .example("huhu")
//! ```
//!
//! will result to this output when prompted:
//!
//! ```text
//! --> hehe (default: hoho, example: huhu):
//! ```
//!
//! This selected promptable:
//!
//! ```
//! // If a specification is provided alone, the Format struct can be constructed from it.
//! let fmt = Format::show_default(false);
//! Selected::new("hehe", [("hoho", 0), ("huhu", 1), ("haha", 2)])
//!     .default(1)
//!     .format(fmt)
//! ```
//!
//! will result to this output when prompted:
//!
//! ```text
//! --> hehe (optional)
//! [1] - hoho
//! [2] - huhu
//! [3] - haha
//! ```
//!
//! #### The `Prompted` trait
//!
//! The [`Prompted`](crate::menu::Prompted) trait is implemented on a type that results of a prompt
//! on the console. In general, it is implemented with the `derive(Prompted)` macro.
//! This derive macro can be placed on an enum or a struct to allow it to be built from
//! the user inputs in the console:
//!
//! ```no_run
//! #[derive(Prompted)]
//! enum LicenseType {
//!     #[prompted(default)]
//!     MIT,
//!     GPL,
//!     BSD,
//! }
//!
//! let ty = LicenseType::prompt();
//! ```
//!
//! This code will result to this output:
//!
//! ```text
//! --> License type
//! [1] - MIT (default)
//! [2] - GPL
//! [3] - BSD
//! >>
//! ```
//!
//! In this case, the `default` parameter in `#[prompted(default)]` means that if the user enters
//! an incorrect value, then it will return the `LicenseType::MIT` value.
//!
//! The `Prompted` may also be implemented on a `struct`. In this case, the prompt will follow the
//! struct fields declaration order. Each struct field will have its own behavior based on
//! its `prompted` attribute. The default behavior is a written field, so the field type must
//! implement the [`FromStr`](core::str::FromStr) trait.
//!
//! Here is an example of the `Prompted` implementation on a struct:
//!
//! ```
//! #[derive(Prompted)]
//! struct License {
//!     #[prompted(sep = ",")]
//!     authors: Vec<String>,
//!     project_name: String,
//! }
//! ```
//!
//! ### Menus
//!
//! A basic example of how to build a CLI menu in Rust is to modelize a game main menu, with a first
//! "Play" button, a button for the "Settings", and an other used to "Quit" the program.
//!
//! The "Play" button will call a function when selected to begin the game loop.
//! The "Settings" button will run a nested menu with a "Name" button used to edit the name
//! of the user, a "Main menu" button used to return back to the main menu, and a "Quit" button
//! used to quit the program.
//!
//! The "Name" nested field in the Settings menu will itself lead to an other nested menu,
//! with a "Firstname" button used to run the program that edits the firstname of the current player,
//! a "Lastname" button for his lastname, and a "Main menu" button to return to the main menu.
//!
//! With `ezmenulib`, this menu is very easy to build:
//!
//! ```no_run
//! use ezmenulib::{bound, menu::Menu};
//!
//! #[bound]
//! fn edit_name(span: &str) -> {
//!     println!("Editing {span}name");
//!     // edit the name...
//! }
//!
//! #[derive(Menu)]
//! enum Name {
//!     #[menu(mapped(edit_name, "first"))]
//!     Firstname,
//!     #[menu(mapped(edit_name, "last"))]
//!     Lastname,
//!     #[menu(back(2))]
//!     MainMenu,
//! }
//!
//! #[derive(Menu)]
//! enum Settings {
//!     #[menu(parent)]
//!     Name,
//!     // Elided index means it goes back to the previous depth
//!     // Equivalent to #[menu(back(1))]
//!     #[menu(back)]
//!     MainMenu,
//!     Quit,
//! }
//!
//! #[bound]
//! fn play() {
//!     println!("Now playing");
//!     // loop { ... }
//! }
//!
//! #[derive(Menu)]
//! enum MainMenu {
//!     #[menu(map(play))]
//!     Play,
//!     #[menu(parent)]
//!     Settings,
//!     Quit,
//! }
//!
//! MainMenu::run();
//! ```
//!
//! Then, a sample of the resulted output of this code would be:
//!
//! ```text
//! --> Main menu
//! [1] - Play
//! [2] - Settings
//! [3] - Quit
//! >> 1
//! Now playing
//! --> Main menu
//! [1] - Play
//! [2] - Settings
//! [3] - Quit
//! >> 2
//! --> Settings
//! [1] - Name
//! [2] - Main menu
//! [3] - Quit
//! >> 1
//! --> Name
//! [1] - Firstname
//! [2] - Lastname
//! [3] - Main menu
//! >> 1
//! Editing firstname
//! --> Name
//! [1] - Firstname
//! [2] - Lastname
//! [3] - Main menu
//! >> 3
//! --> Main menu
//! [1] - Play
//! [2] - Settings
//! [3] - Quit
//! >> 3
//! ```
//!
//! The code uses the `derive(Menu)` feature. If you want to keep the lightness of the library,
//! and disable the library, you still can build the menu like this:
//!
//! ```
//! RawMenu::from([
//!     ("Play", map(play)),
//!     ("Settings", parent([
//!         ("Name", parent([
//!             ("Firstname", mapped!(edit_name, "first")),
//!             ("Lastname", mapped!(edit_name, "last")),
//!             ("Main menu", back(2)),
//!         ])),
//!         ("Main menu", back(1)),
//!         ("Quit", quit()),
//!     ])),
//!     ("Quit", quit()),
//! ])
//! .title("Main menu")
//! ```
//!
//! However, this menu remains very simple and looks like
//! ["raw" menus](crate::menu::RawMenu).
//!
//! So, this raw menu may be transformed into an interactive menu that can be drawn by the
//! [`tui`](https://docs.rs/tui/0.19.0) library. The menu will then appear like so, depending on
//! your code context:
//!
//! ![A dynamic tui-menu built with ezmenulib](https://raw.githubusercontent.com/ahbalbk/ezmenulib/master/assets/dynamic_tuimenu.png)
//!
//! This output corresponds to the
//! ["dynamic tui-menu"](https://github.com/ahbalbk/ezmenulib/blob/master/examples/dynamic_tuimenu.rs)
//! example.

#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    unreachable_pub,
    unused_lifetimes,
    future_incompatible
)]
#![cfg_attr(nightly, feature(doc_cfg))]

#[cfg(feature = "tui")]
#[cfg_attr(nightly, doc(cfg(feature = "tui")))]
pub mod tui;

#[cfg(feature = "derive")]
#[doc(hidden)]
pub use ezmenu_macros::Menu;

#[cfg(feature = "derive")]
#[doc(hidden)]
pub use ezmenu_macros::Prompted;

#[cfg(feature = "derive")]
#[cfg_attr(nightly, doc(cfg(feature = "derive")))]
pub use ezmenu_macros::bound;

mod customs;
pub mod field;
pub mod menu;

pub(crate) mod utils;

/// Module used to import common structs, to build menus with their fields.
pub mod prelude {
    pub use super::*;
    #[doc(inline)]
    pub use crate::{field::*, menu::*};
}

use crate::field::Format;
use std::env::VarError;
use std::error::Error;
use std::fmt::{self, Debug, Display, Formatter};
use std::io;

pub(crate) const DEFAULT_FMT: Format<'static> = Format {
    prefix: "--> ",
    left_sur: "[",
    right_sur: "]",
    chip: " - ",
    show_default: true,
    suffix: ">> ",
    line_brk: true,
};

/// The error type used by the menu builder.
#[non_exhaustive]
pub enum MenuError {
    /// An IO error, when flushing, reading or writing values.
    IOError(io::Error),
    /// A parsing error for a value.
    Input,
    /// An environment variable error.
    EnvVar(String, VarError),
    /// An error occurred when formatting a field.
    Format(fmt::Error),
    /// A custom error.
    Other(Box<dyn Debug>),
}

#[cfg(test)]
impl PartialEq for MenuError {
    fn eq(&self, other: &Self) -> bool {
        // We are simply checking that the variants are the same.
        std::mem::discriminant(self) == std::mem::discriminant(other)
    }
}

impl Error for MenuError {}

impl Debug for MenuError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(self, f)
    }
}

impl Display for MenuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_fmt(format_args!(
            "{}",
            match self {
                Self::IOError(e) => format!("IO error: {}", e),
                Self::Input => "an incorrect input has been provided".to_owned(),
                Self::EnvVar(v, e) => format!(
                    "attempted to get a default value from the environment variable `{}`: {}",
                    v, e
                ),
                Self::Format(e) => format!("an error occurred while formatting a field: {:?}", e),
                Self::Other(d) => format!("{:?}", d),
            }
        ))
    }
}

impl From<io::Error> for MenuError {
    #[inline]
    fn from(e: io::Error) -> Self {
        Self::IOError(e)
    }
}

impl From<&'static str> for MenuError {
    #[inline]
    fn from(s: &'static str) -> Self {
        Self::Other(Box::new(s))
    }
}

impl From<String> for MenuError {
    #[inline]
    fn from(s: String) -> Self {
        Self::Other(Box::new(s))
    }
}

impl From<fmt::Error> for MenuError {
    #[inline]
    fn from(e: fmt::Error) -> Self {
        Self::Format(e)
    }
}

/// The main result type used in the EZMenu library.
pub type MenuResult<T = ()> = Result<T, MenuError>;

/// Utility trait that allows the library to accept many return types for the mapped functions
/// of the menus.
///
/// It converts the return type into a `MenuResult`.
///
/// See the [menu](crate::menu) module documentation for more information.
pub trait IntoResult {
    /// Converts the output type into a `MenuResult`.
    fn into_result(self) -> MenuResult;
}

impl<E: Into<MenuError>> IntoResult for Result<(), E> {
    fn into_result(self) -> MenuResult {
        self.map_err(E::into)
    }
}

impl IntoResult for () {
    fn into_result(self) -> MenuResult {
        Ok(())
    }
}

/// Not public API.
#[doc(hidden)]
pub mod __private {
    pub use core::default::Default;
    pub use core::option::Option;
    pub use core::ptr::addr_of;
    pub use core::result::Result;
    pub use core::str::FromStr;
    pub use std::string::String;
    use std::sync::{Arc, Mutex, RwLock};
    use std::thread::LocalKey;
    pub use std::vec;
    #[cfg(feature = "tui")]
    pub use tui;
    #[allow(non_camel_case_types)]
    pub type str = ty_export::Str;
    pub use std::io::Write;

    use std::cell::RefCell;
    use std::rc::Rc;

    /// Avoids cycle
    mod ty_export {
        pub(super) type Str = str;
    }

    /// Gathers most of the common types used as static variables, to mutate them.
    ///
    /// <T> type corresponds to the wrapped object that will actually be mutated.
    pub trait MutableStatic<T> {
        fn map<'hndl, H, R, F>(&'static self, h: &'hndl mut H, f: F) -> R
        where
            F: for<'obj> FnMut(&'hndl mut H, &'obj T) -> R;

        fn map_mut<'hndl, H, R, F>(&'static self, h: &'hndl mut H, f: F) -> R
        where
            F: for<'obj> FnMut(&'hndl mut H, &'obj mut T) -> R;
    }

    macro_rules! impl_static {
        (|$self:ident, $h:ident, $f:ident| <$($gens:ident $(: $bounds:tt $(+ $others:tt)*,)?),*> for $target:ty: $stmt:stmt, mut $mut_stmt:stmt) => {
            impl<$($gens $(: $bounds $(+ $others)*)?),*> $crate::__private::MutableStatic<T> for $target {
                fn map<'hndl, H, R, F>(&'static $self, $h: &'hndl mut H, mut $f: F) -> R
                where
                    F: for<'obj> FnMut(&'hndl mut H, &'obj T) -> R,
                {
                    $stmt
                }

                fn map_mut<'hndl, H, R, F>(&'static $self, $h: &'hndl mut H, mut $f: F) -> R
                where
                    F: for<'obj> FnMut(&'hndl mut H, &'obj mut T) -> R,
                {
                    $mut_stmt
                }
            }
        };

        (@with_borrow <$($gens:ident $(: $bounds:tt $(+ $others:tt)*,)?),*> for $target:ty) => {
            impl_static!{
                |self, h, f| <$($gens $(: $bounds $(+ $others)*)?),*> for $target:
                self.with(|p| f(h, &p.borrow())), mut self.with(|p| f(h, &mut p.borrow_mut()))
            }
        };

        (@with_read$(($unwrap:ident))? <$($gens:ident $(: $bounds:tt $(+ $others:tt)*,)?),*> for $target:ty) => {
            impl_static!{
                |self, h, f| <$($gens $(: $bounds $(+ $others)*)?),*> for $target:
                self.with(|p| f(h, &p.read()$(.$unwrap())?)), mut self.with(|p| f(h, &mut p.write()$(.$unwrap())?))
            }
        };

        (@read$(($unwrap:ident))? <$($gens:ident $(: $bounds:tt $(+ $others:tt)*,)?),*> for $target:ty) => {
            impl_static!{
                |self, h, f| <$($gens $(: $bounds $(+ $others)*)?),*> for $target:
                f(h, &self.read()$(.$unwrap())?), mut f(h, &mut self.write()$(.$unwrap())?)
            }
        };

        (@with_lock$(($unwrap:ident))? <$($gens:ident $(: $bounds:tt $(+ $others:tt)*,)?),*> for $target:ty) => {
            impl_static!{
                |self, h, f| <$($gens $(: $bounds $(+ $others)*)?),*> for $target:
                self.with(|p| f(h, &p.lock()$(.$unwrap())?)), mut self.with(|p| f(h, &mut p.lock()$(.$unwrap())?))
            }
        };

        (@lock$(($unwrap:ident))? <$($gens:ident $(: $bounds:tt $(+ $others:tt)*,)?),*> for $target:ty) => {
            impl_static!{
                |self, h, f| <$($gens $(: $bounds $(+ $others)*)?),*> for $target:
                f(h, &self.lock()$(.$unwrap())?), mut f(h, &mut self.lock()$(.$unwrap())?)
            }
        };
    }

    impl_static!(@with_borrow <T> for LocalKey<RefCell<T>>);
    impl_static!(@with_borrow <T> for LocalKey<Rc<RefCell<T>>>);
    impl_static!(@with_read(unwrap) <T> for LocalKey<Arc<RwLock<T>>>);
    impl_static!(@with_read(unwrap) <T> for LocalKey<RwLock<T>>);
    impl_static!(@read(unwrap) <T> for RwLock<T>);
    impl_static!(@with_lock(unwrap) <T> for LocalKey<Arc<Mutex<T>>>);
    impl_static!(@with_lock(unwrap) <T> for LocalKey<Mutex<T>>);
    impl_static!(@lock(unwrap) <T> for Mutex<T>);

    /// Implementation of MutableStatic trait for external crates types.
    ///
    /// For now, the "extra-globals" feature only adds parking_lot types.
    // FIXME: Add once_cell types to "extra-globals" feature
    #[cfg(feature = "extra-globals")]
    mod custom_impl {
        use super::{Arc, LocalKey};
        use parking_lot::{Mutex, RwLock};

        impl_static!(@with_read <T> for LocalKey<Arc<RwLock<T>>>);
        impl_static!(@with_read <T> for LocalKey<RwLock<T>>);
        impl_static!(@read <T> for RwLock<T>);
        impl_static!(@with_lock <T> for LocalKey<Arc<Mutex<T>>>);
        impl_static!(@with_lock <T> for LocalKey<Mutex<T>>);
        impl_static!(@lock <T> for Mutex<T>);
    }
}

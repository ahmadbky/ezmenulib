//! Ezmenu attribute and derive macros.
//! This crate shouldn't be used alone. Consider using [ezmenulib](https://docs.rs/ezmenulib).

#![warn(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    unreachable_pub,
    unused_lifetimes,
    future_incompatible
)]

extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use syn::{parse_macro_input, DeriveInput};

mod bound;
mod format;
mod generics;
mod kw;
mod menu;
mod pretend;
mod prompted;
mod utils;

use self::{
    bound::{build_bound, BoundArgs},
    menu::build_menu,
    prompted::build_prompted,
};

/// The `Prompted` derive macro.
///
/// This derive macro is used on struct and enum items to implement the [`Prompted`] trait.
/// It allows to construct the item from a prompt asked to the user.
///
/// For an enum item, the prompt will be selective: it will present the variants of the enum
/// as selection options. The variants may have fields, to allow many bound options
/// on the same variant. Generics are not allowed for a prompted enum.
///
/// For a struct item, the prompt will follow the fields declaration order.
/// Each struct field will have its own behavior based on its `prompted` attribute.
/// The default behavior is a written field, so the field type
/// must implement the [`FromStr`] trait.
///
/// If a struct field has a generic type, the expansion will require the generic type parameter
/// to implement a certain trait depending on its behavior, such as the [`FromStr`] trait
/// for a written field, or the [`Prompted`] trait for a flattened field.
///
/// See more documentation on the [`Prompted`] trait documentation page.
///
/// [`Prompted`]: https://docs.rs/ezmenulib/1.0.0/ezmenulib/menu/trait.Prompted.html
/// [`FromStr`]: core::str::FromStr
#[proc_macro_error]
#[proc_macro_derive(Prompted, attributes(prompt))]
pub fn derive_prompted(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    build_prompted(input).into()
}

/// The `Menu` derive macro.
///
/// This derive macro must be implemented only on unit enums.
/// It expands to the implementation of the [`menu::Menu`], or the [`tui::Menu`] trait,
/// depending on the `tui` menu attribute parameter, placed on the enum:
///
/// ```
/// #[derive(Menu)]
/// // #[menu(tui)]
/// enum MyMenu { /* ... */ }
/// ```
///
/// > Note: to be able to use the `tui` parameter, you must provide the `tui` feature.
///
/// This implementation allows to construct the [`RawMenu`] or the [`TuiMenu`] from the enum.
///
/// The menu can now then be ran. For a raw menu, the instruction is a simple `run()` method call.
/// However, for a tui menu, you must update the state of the tui menu on each loop iteration
/// of your code context, by calling the `TuiMenu::handle_*_event` or `TuiMenu::handle_*_event_with`,
/// with `*` as the event library (current library supported are [`crossterm`] and [`termion`]).
///
/// [`menu::Menu`]: https://docs.rs/ezmenulib/1.0.0/ezmenulib/menu/trait.Menu.html
/// [`tui::Menu`]: https://docs.rs/ezmenulib/1.0.0/ezmenulib/tui/trait.Menu.html
/// [`RawMenu`]: https://docs.rs/ezmenulib/1.0.0/ezmenulib/menu/struct.RawMenu.html
/// [`TuiMenu`]: https://docs.rs/ezmenulib/1.0.0/ezmenulib/tui/struct.TuiMenu.html
/// [`crossterm`]: https://docs.rs/crossterm/0.25.0/
/// [`termion`]: https://docs.rs/termion/1.5.6/
#[proc_macro_error]
#[proc_macro_derive(Menu, attributes(menu))]
pub fn derive_menu(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    build_menu(input).into()
}

/// The `bound` attribute macro.
///
/// This attribute macro is useful if used with the [`Menu`] derive macro.
/// It must be placed on an `fn` item, to transform it to a mapped function to a menu.
///
/// You may provide the `tui` parameter to this attribute, to consider the mapped function to
/// be used by a tui menu.
///
/// > Note: to be able to use the `tui` parameter, you must provide the `tui` feature.
///
/// The `bound` attribute simply insert at first position a new arguments to the bound,
/// so that its signature becomes valid to be called by the corresponding menu.
///
/// So if the function already takes an `<H> &mut H` parameter for a raw menu,
/// or a `<B: Backend> &mut Terminal<B>` parameter for a tui menu, it already represents
/// a bound function, so there is no need to provide the `bound` attribute on it.
///
/// However, the inserted parameter from the `bound` attribute will thus be unusable
/// in the body of the function.
///
/// # Example
///
/// For a raw menu:
///
/// ```
/// #[ezmenulib::bound]
/// fn play() { /* ... */ }
/// ```
/// 
/// For a tui-menu:
/// 
/// ```
/// #[ezmenulib::bound(tui)]
/// fn play() { /* ... */ }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn bound(attr: TokenStream, item: TokenStream) -> TokenStream {
    let tui = parse_macro_input!(attr as BoundArgs);
    let input = parse_macro_input!(item as syn::ItemFn);
    build_bound(tui, input).into()
}

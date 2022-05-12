# Changelog

## 0.3.0 (WIP)

* Removed `T: FromStr` and `<T as FromStr>::Err: 'static + Debug` restrictions for `SelectMenu`.
  * Now the output type `T` has to be `'static`.
  * The user has to enter the index of the selection.
  * To bind the output value to its selection field, you use the `SelectField::new` function.
  * Retrieving a selectable menu output from a value-menu is no more done with
`<ValueMenu as MenuBuilder>::next_output` function, because it requests output to
implement `FromStr`. Instead, it is done with `ValueMenu::next_select` function, to bypass this restriction.
* Introducing `Query<T>` type, used for control flow.
* Removed common `MenuBuilder` trait.
* Removed `new_line` field on `ValueFieldFormatting`.
  * Now always on `true`, meaning there will always be a line break between prompt and prefix.
provided, but only reprints the prefix.
* Renamed `GetStream` trait to `Streamable`.
* Fixing IO error never returned in loop.
* [`chrono`](https://docs.rs/chrono/0.4.19) crate is now re-exported and available from the `ezmenulib` crate.
* Introducing `ValueMenu::next_value_until` and `ValueField::build_until` methods.
* Added example showing for `ValueField`.
  * Separating by comma between default value and example at the prompt, according to the formatting rules.
* Added math expression as value type, using [`meval`](https://docs.rs/meval/0.2.0) crate.

## 0.2.10 (migrated from 0.2.9)

* Added new custom value type: `MenuOption<T>`.
* Added generic parameters for menus and fields for readers and writers.
* Removing the generic const parameter N for `ValueMenu`.
* Introducing `MenuStream<R, W>` to gather the reader and writer.
  * Added methods and function arguments to inherit a stream.
  * Added methods to retrieve a menu stream, and the reader and writer.
* Added default value from an environment variable.
* Added `chrono` optional dependency crate for date-time values providing.
# Changelog

## 0.3.0

* Removed `T: FromStr` and `<T as FromStr>::Err: 'static + Debug` restrictions for `SelectMenu`.
  * Now the output type `T` has to be `'static`.
  * The user has to enter the index of the selection.
  * To bind the output value to its selection field, you use the `SelectField::new` function.
  * Retrieving a selectable menu output from a value-menu is no more done with
`<ValueMenu as MenuBuilder>::next_output` function, because it requests output to
implement `FromStr`. Instead, it is done with `ValueMenu::next_select` function, to bypass this restriction.
* Fixing IO error never returned in loop.
* Introducing `ValueMenu::next_output_until` and `ValueField::build_until` methods.

## 0.2.10 (migrated from 0.2.9)

* Added new custom value type: `MenuOption<T>`.
* Added generic parameters for menus and fields for readers and writers.
* Removing the generic const parameter N for `ValueMenu`.
* Introducing `MenuStream<R, W>` to gather the reader and writer.
  * Added methods and function arguments to inherit a stream.
  * Added methods to retrieve a menu stream, and the reader and writer.
* Added default value from an environment variable.
* Added `chrono` optional dependency crate for date-time values providing.
# Changelog

## 0.3.0 (WIP)

### Breaking changes

* Moving out the field types from the value-menus.
  * `ValueMenu` has been renamed to `Values`.
  * `Field` enum has been removed and replaced by structs `Written` and `Selected`.
  * `Values` does not contain any fields but uses the given field to retrieve a value from the user at a given instruction.
  * `Values` only requires the output `T` type to implement `FromStr`.
  * `Selected` does not require anymore any implementation from the output `T` selected type.
* Real menus.
  * New struct: `RawMenu` for building basic CLI menus.

### Other changes

* `MenuError` does not handle parsing error anymore.
* New custom value type: `MenuNumber`, with `expr` feature using `meval` crate.
  * `MenuOption` and `MenuVec` custom value types have been removed.
* New trait: `Selectable`.
* `MenuBuilder` trait has been removed.
* `ValueFieldFormatting` has been renamed to `Format`.
  * `Format`'s fields have been renamed for more convenience.
  * New fields: `left_sur` and `right_sur`.
* `SelectTitle` struct has been removed.
  * And so `TitlePos` too.
* `GetStream` trait has been renamed to `UsesStream`.
  * The output type is not only `MenuStream` anymore, but a generic `T` type according to the implementation on the structs.

## 0.2.10 (migrated from 0.2.9)

* Added new custom value type: `MenuOption<T>`.
* Added generic parameters for menus and fields for readers and writers.
* Removing the generic const parameter N for `ValueMenu`.
* Introducing `MenuStream<R, W>` to gather the reader and writer.
  * Added methods and function arguments to inherit a stream.
  * Added methods to retrieve a menu stream, and the reader and writer.
* Added default value from an environment variable.
* Added `chrono` optional dependency crate for date-time values providing.
# Changelog of Ezmenu library

## 1.0.0

### Breaking changes

#### Retrieving values

* Removed `MenuBuilder` trait.
* `ValueMenu` renamed to `Values`.
  * It does not contain any field anymore.
  * It acts as a container that gives its format and stream to each field passed to retrieve a value.
* `SelectMenu` renamed to `Selected`.
  * `Selected` does not require the output type to implement `FromStr`.
  * New trait: `Selectable`.
  * `Selected` does not have an optional title anymore but a direct prompt message.
* `ValueField` renamed to `Written`.
  * `Written` only requires the output type to implement `FromStr`.
* Removed `Field` enum.
* Removed the `ezmenulib::customs` module.

#### Format

* Renamed `ValueFieldFormatting` to `Format`.
* Format can now be merged, and will save the custom format specifications of the merged format.
* Reordered fields with new ones:
  * `prefix`.
  * `left_sur`.
  * `right_sur`.
  * `chip`.
  * `show_default`.
  * `suffix`.
  * `line_brk`.

#### Real menus

##### Raw menus

* New struct: `RawMenu`.
* New field types.
  * `Field` with `Fields`.
  * `Kind`.
  * `Callback`.

##### `tui-rs` menus

* Enabled with new `"tui"` feature.
* New struct: `TuiMenu`.
* New type definition: `FieldStyle`.
* New field types.
  * `TuiField` with `TuiFields`.
  * `TuiKind`.
  * `TuiCallback`.

#### Macros

* New derive macros in line with their trait: `ezmenulib::[menu/tui]::Menu` and `ezmenulib::menu::Prompted`.
* New attribute macro: `bound`.

### Other changes

* Removed `GetStream` trait.
* Removed `SelectTitle` and `TitlePos` types.
* Changed `MenuError` variants:
  * Replaced `Parse` variant with `Input` unit variant.
  * Removed `Select` variant.
  * New variant: `Format`.
* Given `()` as default `Ok` type for `MenuResult` type definition.
* New trait: `IntoResult`.

---

## 0.2.10 (migrated from 0.2.9)

* Added new custom value type: `MenuOption<T>`.
* Added generic parameters for menus and fields for readers and writers.
* Removing the generic const parameter N for `ValueMenu`.
* Introducing `MenuStream<R, W>` to gather the reader and writer.
  * Added methods and function arguments to inherit a stream.
  * Added methods to retrieve a menu stream, and the reader and writer.
* Added default value from an environment variable.
* Added `chrono` optional dependency crate for date-time values providing.
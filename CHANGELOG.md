# Changelog of `ezmenulib`.

## 1.0.0 (WIP)

### Breaking changes

#### Retrieving values

* Removed `MenuBuilder` trait.
* `ValueMenu` renamed to `Values`.
  * It does not contain any field anymore.
  * It acts as a container that gives its format and stream to each field passed to retrieve a value.
* `SelectMenu` renamed to `Selected`.
  * `Selected` does not require the output type to implement `FromStr`.
  * New associated function: `optional_select`.
  * New trait: `Selectable`.
  * `Selected` does not have an optional title anymore but a
* `ValueField` renamed to `Written`.
  * New associated function: `many_values`.
  * New associated function: `many_values_until`.
  * New associated function: `many_values_until_with`.
  * New associated function: `many_values_with`.
  * New associated function: `optional_value`.
  * New associated function: `optional_value_with`.
  * New associated function: `prompt_or_default_with`.
  * New associated function: `prompt_until`.
  * New associated function: `prompt_until_with`.
  * New associated function: `prompt_with`.
  * `Written` only requires the output type to implement `FromStr`.
* Removed `Field` enum.
* Removed `MenuOption` and `MenuVec` custom value types.
* New custom value type: `MenuNumber`.
  * Enabled with new `"expr"` feature.

#### Format

* Renamed `ValueFieldFormatting` to `Format`.
* Format can now be merged, and will save the custom format specifications.
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
  * New associated function: `format`.
  * New associated function: `title`.
  * New associated function: `run_once`.
  * New associated function: `run`.
* New field types.
  * `Field` with `Fields`.
  * `Kind`.
  * `Binding`.

##### `tui-rs` menus

* Enabled with new `"tui"` feature.
* New struct: `TuiMenu`.
* New util functions with new `"crossterm"` and `"termion"` features.
  * `new_terminal`.
  * `read`.
  * `restore_terminal`.
  * `setup_terminal`.
* New type definitions for backend types: `Termion` and `Crossterm`.
* New type definition: `FieldStyle`.
* New field types.
  * `TuiField` with `TuiFields`.
  * `TuiKind`.
  * `TuiBinding`.
* New module: `event`, merged from `termion` and `crossterm` event modules.
  * New enum: `Event`.
  * New enum: `KeyEvent`.
  * New enum: `MouseButton`.
  * New enum: `MouseEvent`.

### Other changes

* `GetStream` trait renamed to `UsesMutable`.
  * `MenuStream` output type replaced to generic `S` type.
* New trait: `FromMutable`.
* Removed `SelectTitle` and `TitlePos` types.
* Changed `MenuError` variants:
  * Replaced `Parse` variant with `Input` unit variant.
  * Removed `Select` variant.
  * New variant: `Format`.
* Given `()` as default `Ok` type for `MenuResult` type definition.

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
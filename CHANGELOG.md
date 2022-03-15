# Changelog

## 0.2.10 (migrated from 0.2.9)

* Added new custom value type: `MenuOption<T>`.
* Added generic parameters for menus and fields for readers and writers.
* Removing the generic const parameter N for `ValueMenu`.
* Introducing `MenuStream<R, W>` to gather the reader and writer.
  * Added methods and function arguments to inherit a stream.
  * Added methods to retrieve a menu stream, and the reader and writer.
* Added default value from an environment variable.
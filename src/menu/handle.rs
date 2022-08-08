use std::{
    fmt::{self, Arguments},
    io::{
        self, empty, sink, stdin, stdout, BufRead, BufReader, Empty, IoSlice, IoSliceMut, Read,
        Sink, Write,
    },
};

use crate::{
    field::{Format, MenuDisplay},
    MenuResult,
};

pub trait Handle: Read + Write {
    fn show<T: ?Sized + MenuDisplay>(
        &mut self,
        text: &T,
        fmt: &Format<'_>,
        opt: bool,
    ) -> MenuResult {
        {
            let mut s = String::new();
            MenuDisplay::fmt_with(text, &mut s, fmt, opt)?;
            self.write_all(s.as_bytes())?;
        }
        self.flush()?;
        Ok(())
    }

    fn read_input(&mut self) -> MenuResult<String> {
        let mut out = String::new();
        let mut buf = BufReader::new(self);
        buf.read_line(&mut out)?;
        Ok(out.trim().to_owned())
    }

    fn prompt<T: ?Sized + MenuDisplay>(
        &mut self,
        text: &T,
        fmt: &Format<'_>,
        opt: bool,
    ) -> MenuResult<String> {
        self.show(text, fmt, opt)?;
        self.read_input()
    }
}

impl<H: Read + Write> Handle for H {}

macro_rules! map_impl {
    (
        $target:tt,
        $($name:ident($($arg:ident: $ty:ty),*)$( -> $ret:ty)?),*
        $(,)?
    ) => {$(
        #[inline]
        fn $name(&mut self, $($arg: $ty),*) $(-> $ret)? {
            self.$target.$name($($arg),*)
        }
    )*};
}

/// Represents the stream used to process input and output values from a menu.
///
/// This struct is used to inherit the stream from a parent menu to its fields or a submenu.
///
/// It uses by default the standard input and output streams to manage input and output operations.
///
/// However, the reader must implement [`BufRead`] trait, and the writer must implement [`Write`] trait.
/// Therefore, because [`Stdin`](std::io::Stdin) does not implement `BufRead`, we need to wrap it with the [`BufReader`]
/// struct, thus we get: `BufReader<Stdin>`.
/// So for example if you need to get the values from a file, you need to wrap the [`File`](std::fs::File) type
/// with `BufReader<File>`.
///
/// ## Example
///
/// ```no_run
/// use std::io::{Read, stdin, stdout};
/// use ezmenulib::menu::MenuStream;
///
/// let my_stdin = stdin();
/// let my_stdout = stdout();
/// let mut stream = MenuStream::new(my_stdin, my_stdout);
///
/// let mut buf = [0; 10];
/// // you can still use `Read` associated trait functions if `R` implements `Read`:
/// stream.read(&mut buf).unwrap();
/// // however you cannot use `BufRead` associated trait functions:
/// // let mut s = String::new();
/// // stream.read_line(&mut s).unwrap();
/// ```
///
/// If you want to wrap the reader with `BufReader` struct, you can wrap it on your side with
/// `BufReader::new(...)`, or use the [`MenuStream::wrap_reader`] method, to directly
/// instantiate the stream:
///
/// ```no_run
/// # use std::io::{stdin, stdout};
/// # use ezmenulib::menu::MenuStream;
/// # let my_stdin = stdin();
/// # let my_stdout = stdout();
/// let mut stream = MenuStream::wrap_reader(my_stdin, my_stdout);
/// ```
///
/// Although, you cannot wrap and instantiate if the stream does not own the reader,
/// because `BufReader` needs to own it.
///
/// ## Inheritance
///
/// You may give a mutable reference to the streams instead of giving the ownership.
/// In the latter case, you can still get the ownership of the menu stream with the
/// [`MenuStream::retrieve`] method:
/// ```no_run
/// # use std::io::{stdin, stdout};
/// use ezmenulib::menu::MenuStream;
/// # let input = stdin();
/// # let output = stdout();
/// let mut stream = MenuStream::new(input, output);
/// // ...
/// let (input, output) = stream.retrieve();
/// ```
#[derive(Debug)]
pub struct MenuHandle<R = super::In, W = super::Out>(pub R, pub W);

impl Default for MenuHandle {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}

impl<R, W> From<(R, W)> for MenuHandle<R, W> {
    fn from((reader, writer): (R, W)) -> Self {
        Self::with(reader, writer)
    }
}

impl<R, W> From<MenuHandle<R, W>> for (R, W) {
    fn from(handle: MenuHandle<R, W>) -> Self {
        handle.retrieve()
    }
}

impl MenuHandle {
    pub fn new() -> Self {
        Self::from_reader(stdin())
    }
}

impl<W> MenuHandle<super::In, W> {
    pub fn from_writer(writer: W) -> Self {
        Self(stdin(), writer)
    }
}

impl<R> MenuHandle<R, super::Out> {
    pub fn from_reader(reader: R) -> Self {
        Self(reader, stdout())
    }
}

impl MenuHandle<Empty, super::Out> {
    pub fn empty_reader() -> Self {
        Self::empty_reader_with(stdout())
    }
}

impl<W> MenuHandle<Empty, W> {
    pub fn empty_reader_with(writer: W) -> Self {
        Self(empty(), writer)
    }
}

impl MenuHandle<super::In, Sink> {
    pub fn empty_writer() -> Self {
        Self::empty_writer_with(stdin())
    }
}

impl<R> MenuHandle<R, Sink> {
    pub fn empty_writer_with(reader: R) -> Self {
        Self(reader, sink())
    }
}

impl<R, W> MenuHandle<R, W> {
    /// Instantiates the stream with a given reader and writer.
    pub fn with(reader: R, writer: W) -> Self {
        Self(reader, writer)
    }

    pub fn get(&self) -> (&R, &W) {
        (&self.0, &self.1)
    }

    pub fn get_mut(&mut self) -> (&mut R, &mut W) {
        (&mut self.0, &mut self.1)
    }

    pub fn retrieve(self) -> (R, W) {
        (self.0, self.1)
    }
}

impl<R: Read, W> Read for MenuHandle<R, W> {
    map_impl!(
        0,
        read(buf: &mut [u8]) -> io::Result<usize>,
        read_vectored(bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize>,
        read_to_end(buf: &mut Vec<u8>) -> io::Result<usize>,
        read_to_string(buf: &mut String) -> io::Result<usize>,
        read_exact(buf: &mut [u8]) -> io::Result<()>,
    );
}

impl<R: BufRead, W> BufRead for MenuHandle<R, W> {
    map_impl!(
        0,
        fill_buf() -> io::Result<&[u8]>,
        consume(amt: usize),
        read_until(byte: u8, buf: &mut Vec<u8>) -> io::Result<usize>,
        read_line(buf: &mut String) -> io::Result<usize>,
    );
}

impl<R, W: Write> Write for MenuHandle<R, W> {
    map_impl!(
        1,
        write(buf: &[u8]) -> io::Result<usize>,
        write_vectored(bufs: &[IoSlice<'_>]) -> io::Result<usize>,
        flush() -> io::Result<()>,
        write_all(buf: &[u8]) -> io::Result<()>,
        write_fmt(fmt: Arguments<'_>) -> io::Result<()>,
    );
}

impl<R, W: Write> fmt::Write for MenuHandle<R, W> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.1
            .write_all(s.as_bytes())
            .and(self.1.flush())
            .map_err(|_| fmt::Error)
    }
}

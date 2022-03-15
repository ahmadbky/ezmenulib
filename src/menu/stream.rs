use std::fmt::Arguments;
use std::io::{self, stdin, stdout, BufRead, BufReader, IoSlice, IoSliceMut, Read, Write};
use std::ops::{Deref, DerefMut};

macro_rules! map_impl {
    (
        $target:ident,
        $($name:ident($($arg:ident: $ty:ty),*)$( -> $ret:ty)?),*
        $(,)?
    ) => {$(
        #[inline]
        fn $name(&mut self, $($arg: $ty),*) $(-> $ret)? {
            self.$target.$name($($arg),*)
        }
    )*};
}

pub(super) enum Stream<'a, T> {
    Owned(T),
    Borrowed(&'a mut T),
}

impl<T> Stream<'_, T> {
    pub fn retrieve(self) -> T {
        if let Self::Owned(t) = self {
            t
        } else {
            unreachable!("the stream must own the type to retrieve it")
        }
    }
}

impl<T> Deref for Stream<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Owned(t) => t,
            Self::Borrowed(t) => *t as &T,
        }
    }
}

impl<T> DerefMut for Stream<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            Self::Owned(t) => t,
            Self::Borrowed(t) => *t,
        }
    }
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
/// ```
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
/// ```
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
/// ```
/// # use std::io::{stdin, stdout};
/// use ezmenulib::menu::MenuStream;
/// # let input = stdin();
/// # let output = stdout();
/// let mut stream = MenuStream::new(input, output);
/// // ...
/// let (input, output) = stream.retrieve();
/// ```
pub struct MenuStream<'a, R = super::In, W = super::Out> {
    reader: Stream<'a, R>,
    writer: Stream<'a, W>,
}

impl Default for MenuStream<'_> {
    #[inline]
    fn default() -> Self {
        Self::wrap_reader(stdin(), stdout())
    }
}

impl<R: Read, W> MenuStream<'_, BufReader<R>, W> {
    /// Instantiate the stream by wrapping the reader with a [`BufReader`].
    #[inline]
    pub fn wrap_reader(reader: R, writer: W) -> Self {
        Self::new(BufReader::new(reader), writer)
    }
}

impl<'a, R, W> MenuStream<'a, R, W> {
    /// Instantiates the stream with a given reader and writer.
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader: Stream::Owned(reader),
            writer: Stream::Owned(writer),
        }
    }

    /// Instantiates the stream with a borrowed reader and a borrowed writer.
    pub fn with(reader: &'a mut R, writer: &'a mut W) -> Self {
        Self {
            reader: Stream::Borrowed(reader),
            writer: Stream::Borrowed(writer),
        }
    }

    /// Retrieves the reader and writer of the stream.
    ///
    /// ## Panics
    ///
    /// If it hasn't been given a reader and a writer, this method will panic, because it needs
    /// to own the reader and writer to retrieve it at the end.
    #[inline]
    pub fn retrieve(self) -> (R, W) {
        (self.reader.retrieve(), self.writer.retrieve())
    }
}

impl<R: Read, W> Read for MenuStream<'_, R, W> {
    map_impl!(
        reader,
        read(buf: &mut [u8]) -> io::Result<usize>,
        read_vectored(bufs: &mut [IoSliceMut<'_>]) -> io::Result<usize>,
        read_to_end(buf: &mut Vec<u8>) -> io::Result<usize>,
        read_to_string(buf: &mut String) -> io::Result<usize>,
        read_exact(buf: &mut [u8]) -> io::Result<()>,
    );
}

impl<R: BufRead, W> BufRead for MenuStream<'_, R, W> {
    map_impl!(
        reader,
        fill_buf() -> io::Result<&[u8]>,
        consume(amt: usize),
        read_until(byte: u8, buf: &mut Vec<u8>) -> io::Result<usize>,
        read_line(buf: &mut String) -> io::Result<usize>,
    );
}

impl<R, W: Write> Write for MenuStream<'_, R, W> {
    map_impl!(
        writer,
        write(buf: &[u8]) -> io::Result<usize>,
        write_vectored(bufs: &[IoSlice<'_>]) -> io::Result<usize>,
        flush() -> io::Result<()>,
        write_all(buf: &[u8]) -> io::Result<()>,
        write_fmt(fmt: Arguments<'_>) -> io::Result<()>,
    );
}

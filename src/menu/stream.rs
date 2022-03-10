use std::fmt::Arguments;
use std::io::{
    self, stdin, stdout, BufRead, BufReader, IoSlice, IoSliceMut, Read, Stdin, Stdout, Write,
};

enum Stream<'a, T> {
    Owned(T),
    Borrowed(&'a mut T),
}

pub struct MenuStream<'a, R = BufReader<Stdin>, W = Stdout> {
    reader: Stream<'a, R>,
    writer: Stream<'a, W>,
}

impl Default for MenuStream<'_> {
    #[inline]
    fn default() -> Self {
        Self::new(BufReader::new(stdin()), stdout())
    }
}

impl<'a, R, W> MenuStream<'a, R, W> {
    pub fn new(reader: R, writer: W) -> Self {
        Self {
            reader: Stream::Owned(reader),
            writer: Stream::Owned(writer),
        }
    }

    pub fn with(reader: &'a mut R, writer: &'a mut W) -> Self {
        Self {
            reader: Stream::Borrowed(reader),
            writer: Stream::Borrowed(writer),
        }
    }

    pub fn retrieve(self) -> Option<(R, W)> {
        if let (Stream::Owned(reader), Stream::Owned(writer)) = (self.reader, self.writer) {
            Some((reader, writer))
        } else {
            None
        }
    }
}

macro_rules! map_impl {
    ($target:ident, $($name:ident($($arg:ident: $ty:ty),*)$( -> $ret:ty)?),* $(,)?) => {$(
        #[inline]
        fn $name(&mut self, $($arg: $ty),*)$(-> $ret)? {
            match &mut self.$target {
                Stream::Owned($target) => $target.$name($($arg),*),
                Stream::Borrowed($target) => $target.$name($($arg),*),
            }
        }
    )*}
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

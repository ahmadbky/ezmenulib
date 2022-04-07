use ezmenulib::menu::MenuStream;
use std::error::Error;
use std::io::{BufRead, Write};

#[test]
fn basic() -> Result<(), Box<dyn Error>> {
    let input = "hello\n".as_bytes();
    let output = Vec::<u8>::new();
    let mut stream = MenuStream::new(input, output);
    let mut s = String::new();
    stream.read_line(&mut s)?;
    stream.write_all(s.as_bytes())?;
    let (_, output) = stream.retrieve();
    let output = String::from_utf8(output)?;
    assert_eq!(output, "hello\n");
    Ok(())
}

#[test]
fn borrow_params() -> Result<(), Box<dyn Error>> {
    let mut input = "hey\n".as_bytes();
    let mut output = Vec::<u8>::new();
    let mut stream = MenuStream::with(&mut input, &mut output);
    let mut s = String::new();
    stream.read_line(&mut s)?;
    stream.write_all(s.as_bytes())?;
    let output = String::from_utf8(output)?;
    assert_eq!(output, "hey\n");
    Ok(())
}

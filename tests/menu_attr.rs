use ezmenu::Menu;
use std::io::stdout;

#[derive(Debug)]
#[ezmenu::parsed]
enum Type {
    MIT,
    GPL,
    BSD,
}

#[derive(Menu, Debug)]
#[menu(title = "bonsoir")]
struct License {
    ty: Type,
    author: String,
    date: u16,
}

#[test]
fn base_impl() {
    let input = "no\nmit\nahmad\n2004\n".as_bytes();
    let license = License::from_io(input, stdout());
    println!("{:?}", license);
}

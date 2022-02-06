use ezmenu::{Menu, StructField, StructMenu};
use std::io::Write;

// used as testing for now
fn main() {
    let mut menu = StructMenu::default()
        .title("bonsoir")
        .with_field(StructField::from("oui").default("43"))
        .with_field(StructField::from("allo"));

    let _first: u8 = menu.next();
    let _second: String = menu.next_with(|_, w| {
        w.write(b"nice name").expect("let me pat u");
    });
}

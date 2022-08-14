#![allow(dead_code)]

use ezmenulib::prelude::*;
use std::io;

#[bound]
fn edit_name<H: Handle>(s: &mut H, span: &str) -> io::Result<()> {
    writeln!(s, "Editing {span}stname")
}

#[derive(Menu)]
enum Name {
    #[menu(mapped(editname, "fir"))]
    Firstname,
    #[menu(mapped(editname, "la"))]
    Lastname,
    #[menu(back(2))]
    MainMenu,
}

#[derive(Menu)]
enum Settings {
    #[menu(flatten)]
    Name,
}

#[derive(Menu)]
enum Bonjour {
    #[menu(bind(play))]
    Play,
    #[menu(flatten)]
    Settings,
    #[menu()]
    Quit,
}

#[derive(Prompted)]
struct Testing;

#[derive(Prompted, Debug)]
#[prompt(case = up)]
struct Foo {
    value: i32,
    is_sure: bool,
    #[prompt(sep = ", ")]
    names: Vec<String>,
    proj_name: Option<String>,
    #[prompt(flatten)]
    amount: Amount,
}

/// HEYYYYYYY
#[derive(Prompted, Debug)]
struct HeheHello(
    /// Bonsoir
    #[prompt(sep = ", ")]
    Vec<u8>,
);

/// How many?!
#[derive(Prompted, Debug)]
#[prompt(case = up, fmt(suf: "> ", no_default))]
enum Amount {
    #[prompt(("ZERO", 0), ("ONE", 1), default("TWO", 2))]
    N(u8),
    /// The users selects more than `2`.
    #[prompt(nodoc)]
    MoreThanTwo,
}

#[derive(Prompted, Debug)]
struct WithGenerics<T> {
    prompt: T,
}

fn main() {
    // let amount = Amount::prompt();
    // println!("Amount = {amount:?}");

    // let hehe = HeheHello::prompt();
    // println!("HeheHello = {hehe:?}");

    // let gens = WithGenerics::<i32>::prompt();
    // println!("WithGenerics = {gens:?}");

    // let foo = Foo::prompt();
    // println!("Foo = {foo:#?}");
}

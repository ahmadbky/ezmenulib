#![allow(dead_code)]

use ezmenulib::prelude::*;
use std::io;
#[allow(unused_imports)]
use std::str::FromStr;

#[bound]
fn edit_name<H: Handle>(s: &mut H, span: &str) -> io::Result<()> {
    writeln!(s, "Editing {span}stname")
}

#[derive(Menu)]
enum Name {
    #[menu(mapped(edit_name, "fir"))]
    Firstname,
    #[menu(mapped(edit_name, "la"))]
    Lastname,
    #[menu(back(2))]
    MainMenu,
}

#[derive(Menu)]
enum Settings {
    #[menu(parent)]
    Name,
}

#[bound]
fn play() {
    println!("salut");
}

#[derive(Menu)]
enum Bonjour {
    #[menu(map(play))]
    Play,
    #[menu(parent)]
    Settings,
    #[menu()]
    Quit,
}

#[derive(Prompted)]
struct Testing;

#[derive(Prompted, Debug)]
#[prompt(case = upper)]
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
#[prompt(case = upper, fmt(suffix = "> ", show_default = false))]
enum Amount {
    #[prompt(("ZERO", 0), ("ONE", 1), default("TWO", 2))]
    N(u8),
    /// The users selects more than `2`.
    #[prompt(nodoc)]
    MoreThanTwo,
}

#[derive(Prompted, Debug)]
struct WithGenerics<T /*: FromStr */> {
    prompt: T,
}

fn main() {
    let amount = Amount::prompt();
    println!("Amount = {amount:?}");

    // let hehe = HeheHello::prompt();
    // println!("HeheHello = {hehe:?}");

    // let gens = WithGenerics::<i32>::prompt();
    // println!("WithGenerics = {gens:?}");

    // let foo = Foo::prompt();
    // println!("Foo = {foo:#?}");
}

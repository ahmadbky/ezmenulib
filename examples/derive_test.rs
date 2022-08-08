#![allow(dead_code)]

use ezmenulib::prelude::*;

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

fn main() {
    // let amount = Amount::prompt();
    // println!("Amount = {amount:?}");

    // let hehe = HeheHello::prompt();
    // println!("HeheHello = {hehe:?}");

    let foo = Foo::prompt();
    println!("Foo = {foo:#?}");
}

#![allow(dead_code)]

use ezmenulib::{prelude::*, Select};

/// How many?!
#[derive(Select, Debug)]
#[select(case = up, fmt(suf: "> ", no_default))]
enum Amount {
    #[select(("ZERO", 0), ("ONE", 1), default("TWO", 2))]
    N(u8),
    /// The users selects more than `2`.
    #[select(nodoc)]
    MoreThanTwo,
}

fn main() {
    let amount = Amount::select().get();
    println!("{amount:?}");
}

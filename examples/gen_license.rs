use ezmenulib::prelude::*;

/// Select a license type
#[derive(Prompted, Debug)]
enum Type {
    #[prompt(default)]
    MIT,
    GPL,
    BSD,
}

#[allow(dead_code)]
#[derive(Prompted, Debug)]
#[prompt(no_title)]
struct License {
    #[prompt(sep = ", ")]
    authors: Vec<String>,
    name: Option<String>,
    #[prompt(until(|e| *e > 0), or_val("2022"))]
    date: u16,
    #[prompt(flatten)]
    ty: Type,
}

/// Describe your project
#[derive(Prompted)]
#[prompt(fmt(prefix = "==> ", chip = " = "))]
struct Opt {
    #[prompt(flatten)]
    license: License,
    #[prompt(msg = "Are you sure?", basic_example, or_val("no"))]
    is_sure: bool,
}

fn main() {
    let opt = Opt::prompt();
    if opt.is_sure {
        println!("{:?}", opt.license);
    }
}

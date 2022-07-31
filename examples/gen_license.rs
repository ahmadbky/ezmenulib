use ezmenulib::prelude::*;
use std::error::Error;
use std::io::Write;

#[derive(Debug)]
enum Type {
    MIT,
    GPL,
    BSD,
}

impl Default for Type {
    fn default() -> Self {
        Self::MIT
    }
}

impl Selectable<3> for Type {
    fn select() -> Selected<'static, Self, 3> {
        use Type::*;
        Selected::new(
            "Select a license type",
            [("MIT", MIT), ("GPL", GPL), ("BSD", BSD)],
        )
        .default(0)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = MenuHandle::default();
    writeln!(stream, "Describe your project")?;

    let mut lic = Values::from(stream).format(Format {
        prefix: "==> ",
        chip: " = ",
        ..Default::default()
    });

    let authors: Vec<String> = lic.next(Separated::new("Authors", ", "))?;
    let name: Option<String> = lic.next_optional(Written::from("Project name"))?;
    let date: u16 = lic.next(Written::from("License date").default_value("2022"))?;
    let ty: Type = lic.next(Type::select())?;
    println!(
        "{ty:?} License, Copyright (C) {date} {}\n{}",
        authors.join("; "),
        if let Some(n) = name { n } else { "".to_owned() }
    );
    Ok(())
}

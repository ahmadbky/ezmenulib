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
    fn values() -> [(&'static str, Self); 3] {
        use Type::*;
        [("MIT", MIT), ("GPL", GPL), ("BSD", BSD)]
    }

    fn default() -> Option<usize> {
        Some(0)
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let mut stream = MenuStream::default();
    writeln!(stream, "Describe your project")?;

    let mut lic = Values::from(stream).format(Format {
        prefix: "==> ",
        chip: " = ",
        ..Default::default()
    });

    let authors: Vec<String> =
        lic.many_written(&Written::from("Authors").example("Ahmad, ..."), ", ")?;
    let name: Option<String> = lic.optional_written(&Written::from("Project name"))?;
    let date: u16 = lic.written(&Written::from("License date").default_value("2022"))?;
    let ty: Type = lic.selected(Selected::from("Select a license type"))?;
    println!(
        "{ty:?} License, Copyright (C) {date} {}\n{}",
        authors.join(", "),
        if let Some(n) = name { n } else { "".to_owned() }
    );
    Ok(())
}

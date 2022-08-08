use chrono::Datelike;
use ezmenulib::field::Bool;
use ezmenulib::prelude::*;
use std::env;
use std::error::Error;
use std::io::Write;
use std::path::Path;

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
    let mut handle = MenuHandle::default();
    writeln!(handle, "Describe your project")?;

    let mut lic = Values::from(&mut handle).format(Format {
        prefix: "==> ",
        chip: " = ",
        ..Default::default()
    });

    let authors =
        match lic.next_optional(Separated::new("Authors", ", ").example("Ahmad, Julien..."))? {
            Some(out) => out,
            None => {
                let home = env::var("HOME")?;
                let home = Path::new(&home)
                    .into_iter()
                    .last()
                    .unwrap()
                    .to_os_string()
                    .into_string()
                    .unwrap();
                vec![home]
            }
        };
    let name: Option<String> = lic.next_optional(Written::from("Project name"))?;
    let current_year = chrono::Utc::now().year().to_string();
    let date: u16 = lic.next(Written::from("License date").default_value(current_year.as_str()))?;
    let ty: Type = lic.next(Type::select())?;

    if lic.next(
        Bool::new("Are you sure?")
            .with_basic_example()
            .default_value(false),
    )? {
        writeln!(
            handle,
            "{ty:?} License, Copyright (C) {date} {}\n{}",
            authors.join("; "),
            if let Some(n) = name { n } else { "".to_owned() }
        )?;
    }

    Ok(())
}

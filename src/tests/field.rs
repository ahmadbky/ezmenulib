use crate::field::StructFieldFormatting;
use crate::StructField;

// uses &[u8] instead of Stdin
macro_rules! test_field {
    ($ident:ident: $ty:ty, $input:literal) => {{
        use std::io::stdout;
        let mut input = $input.as_bytes();
        let _: $ty = $ident.build_with(&mut input, &mut stdout()).unwrap();
    }};
}

#[test]
fn basic_field() {
    let age = StructField::from("how old are you");
    test_field!(age: u8, "aaa\n34\n");
}

#[test]
fn custom_fmt() {
    // this should become the expanded version of a field
    let lastname = StructField::from("what is your last name?").fmt(StructFieldFormatting {
        chip: "Now, ",
        prefix: ">> ",
        new_line: true,
        ..Default::default()
    });

    test_field!(lastname: String, "baAlBaKy\n");
}

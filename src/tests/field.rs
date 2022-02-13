use crate::field::StructFieldFormatting;
use crate::StructField;

// uses &[u8] instead of Stdin and Vec<u8> instead of Stdout
macro_rules! test_field {
    ($ident:ident: $ty:ty, $input:literal) => {{
        let mut input = $input.as_bytes();
        let mut output = Vec::new();
        let _: $ty = $ident.build(&mut input, &mut output).unwrap();

        String::from_utf8(output).unwrap()
    }};
}

#[test]
fn basic_field() {
    let age = StructField::from("how old are you");
    let output = test_field!(age: u8, "aaa\n34");
    println!("{}", output);
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

    let output = test_field!(lastname: String, "baAlBaKy");
    println!("{}", output);
}

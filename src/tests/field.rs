use crate::Field;

macro_rules! test_field {
    ($ident:ident: $ty:ty, $input:literal) => {{
        let mut input = $input.as_bytes();
        let mut output = Vec::new();
        let _: $ty = $ident.build(&mut input, &mut output);

        String::from_utf8(output).unwrap()
    }};
}

#[test]
fn test_fields() {
    // this should become the expanded version of a field
    let lastname = Field::from("what is your last name?")
        .then(|s: &String, w| {
            if s.to_lowercase() == "baalbaky" {
                writeln!(w, "are you his brother or sister?")
                    .expect("Unable to write the message after providing the last name");
            }
        })
        .chip(Some("Now, "))
        .prefix(Some(">> "))
        .new_line(true);

    let output = test_field!(lastname: String, "baAlBaKy");
    println!("{}", output);
}

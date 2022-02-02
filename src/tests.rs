use crate::Field;

#[test]
fn test_fields() {
    let lastname = Field::from("what is your last name?")
        .then(|s: &String| {
            if s.to_lowercase() == "baalbaky" {
                println!("are you his brother or sister?");
            }
        })
        .display_default(false)
        .chip(Some("Now, "))
        .prefix(Some(">> "))
        .new_line(true);

    let mut input = b"baalbaky" as &[u8];
    let mut output = Vec::new();
    let value: String = lastname.build(&mut input, &mut output);

    let output = String::from_utf8_lossy(output.as_slice());
    println!("{}", output);
    assert_eq!(value, "baalbaky");
}

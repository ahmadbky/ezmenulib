use ezmenulib::prelude::*;
use std::io::stdout;

macro_rules! test_menu {
    ($input:expr, $fields:expr, $var:ident: $ty:ty, => $output:ident $(,)?) => {
        let mut input = $input.as_bytes();
        let mut output = Vec::<u8>::new();
        let mut my_menu = SelectMenu::new_ref(&mut input, &mut output, $fields);
        let $var: $ty = my_menu.next_output().expect("invalid next output");
        let $output = String::from_utf8(output).expect("unexpected invalid utf8 output");
    };
}

#[test]
#[should_panic]
fn no_field() {
    let input = "zmlerkjg".as_bytes();
    let mut my_menu = SelectMenu::new(input, stdout(), vec![]);
    let _: bool = my_menu.next_output().unwrap();
}

enum Type {
    MIT,
    GPL,
    BSD,
}

#[test]
fn one_field() {
    test_menu! {
        "3\n-4\n340\n1\n",
        vec![SelectField::new("MIT", Type::MIT)],
        _name: Type,
        => output,
    };

    assert_eq!(
        output,
        "1 - MIT
>> 
>> 
>> 
>> \n"
    );
}

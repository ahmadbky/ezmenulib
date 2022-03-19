use ezmenulib::{chrono::prelude::*, prelude::*};
use std::io::stdout;

macro_rules! test_menu {
    ($input:expr, $fields:expr, $($name:ident: $ty:ty),* $(,)? => $output:ident $(,)?) => {
        test_menu! {
            my_menu,
            $input,
            $fields,
            $(let $name: $ty = my_menu.next_output().expect("invalid next output")),*
            => $output
        };
    };

    ($name:ident, $input:expr, $fields:expr, $($st:stmt),* => $output:ident $(,)?) => {
        let mut input = $input.as_bytes();
        let mut output = Vec::<u8>::new();
        let mut $name = ValueMenu::with_ref(
            &mut input,
            &mut output,
            $fields,
        );
        $($st)*
        let $output = String::from_utf8(output).expect("unexpected invalid utf8 output");
    };
}

#[test]
#[should_panic]
fn no_field() {
    let input = "".as_bytes();
    let mut my_menu = ValueMenu::new(input, stdout(), Vec::new());
    let _: bool = my_menu.next_output().unwrap();
}

#[test]
fn one_field() {
    test_menu! {
        "Ahmad\n",
        vec![Field::Value(ValueField::from("your name please"))],
        _name: String,
        => output,
    };
    assert_eq!(
        output,
        "--> your name please
>> \n"
    );
}

#[test]
fn retrieve_value() {
    test_menu! {
        "Ahmad\n19\n",
        vec![
            Field::Value(ValueField::from("your name please")),
            Field::Value(ValueField::from("how old are you")),
        ],
        name: String,
        age: u8,
        => output,
    };

    assert_eq!(
        output,
        "--> your name please
>> 
--> how old are you
>> \n"
    );

    assert_eq!(name, "Ahmad");
    assert_eq!(age, 19u8);
}

#[test]
fn loop_ask() {
    test_menu! {
        "zmelkfjz\n86\n",
        vec![Field::Value(ValueField::from("your age please"))],
        age: u8,
        => output,
    };

    assert_eq!(age, 86u8);
    assert_eq!(
        output,
        "--> your age please
>> 
--> your age please
>> \n"
    );
}

#[test]
fn default_value() {
    test_menu! {
        my_menu,
        "mlzigujz\n",
        vec![Field::Value(ValueField::from("your age please"))],
        let age: u8 = my_menu.next_or_default()
        => output,
    };

    assert_eq!(age, 0u8);
    assert_eq!(
        output,
        "--> your age please
>> \n"
    );
}

#[test]
fn date_value() {
    test_menu! {
        "lol\n",
        vec![Field::Value(ValueField::from("date").default_value("2015-04-29"))],
        date: NaiveDate,
        => output,
    };

    assert_eq!(date, NaiveDate::from_ymd(2015, 04, 29));
    assert_eq!(
        output,
        "--> date (default: 2015-04-29)
>> \n"
    );
}

#[test]
#[should_panic]
fn incorrect_default_value() {
    test_menu! {
        "Ahmad\nno",
        vec![
            Field::Value(ValueField::from("name")),
            Field::Value(ValueField::from("age").default_value("yep")),
        ],
        _name: String,
        _age: u8,
        => _output,
    };
}

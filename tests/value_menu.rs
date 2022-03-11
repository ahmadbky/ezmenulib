use chrono::prelude::*;
use ezmenulib::prelude::*;
use std::io::stdout;

macro_rules! test_menu {
    ($input:expr, $fields:expr, $($name:ident: $ty:ty),* $(,)? => $output:ident $(,)?) => {
        test_menu!(my_menu,
            $input,
            $fields,
            $(let $name: $ty = my_menu.next_output().expect("invalid next output")),*
            => $output);
    };

    ($name:ident, $input:expr, $fields:expr, $($st:stmt),* => $output:ident $(,)?) => {
        let input = $input.as_bytes();
        let output = Vec::<u8>::new();
        let mut $name = ValueMenu::new(
            input,
            output,
            $fields,
        );
        $($st)*
        let $output = String::from_utf8($name.get_stream().unwrap().1).expect("unexpected invalid utf8 output");
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
>> Ahmad\n"
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
>> Ahmad
--> how old are you
>> 19\n"
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
>> zmelkfjz
--> your age please
>> 86\n"
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
>> mlzigujz\n"
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
>> lol\n"
    );
}

use std::error::Error;

#[cfg(feature = "date")]
use crate::chrono::prelude::*;
use crate::prelude::*;

type Res = Result<(), Box<dyn Error>>;

macro_rules! test_menu {
    ($name:ident, $input:expr, $($st:stmt),* $(,)?) => {{
        let mut input = $input.as_bytes();
        let mut output = Vec::<u8>::new();
        let mut $name = Values::from(MenuStream::with(&mut input, &mut output));
        {$($st)*}
        String::from_utf8(output)
    }};
}

#[test]
fn one_field() -> Res {
    let output = test_menu! {
        menu,
        "Ahmad\n",
        let name: String = menu.written(&Written::from("your name please"))?,
        assert_eq!(name, "Ahmad"),
    }?;

    Ok(assert_eq!(output, "--> your name please\n>> "))
}

#[test]
fn retrieve_value() -> Res {
    let output = test_menu! {
        menu,
        "Ahmad\n19\n",
        let name: String = menu.written(&Written::from("your name please"))?,
        assert_eq!(name, "Ahmad"),
        let age: u8 = menu.written(&Written::from("how old are you"))?,
        assert_eq!(age, 19),
    }?;

    Ok(assert_eq!(
        output,
        "--> your name please\n>> \
--> how old are you\n>> "
    ))
}

#[test]
fn loop_ask() -> Res {
    let output = test_menu! {
        menu,
        "zmelkfjz\n86\n",
        let age: u8 = menu.written(&Written::from("your age please"))?,
        assert_eq!(age, 86),
    }?;

    Ok(assert_eq!(output, "--> your age please\n>> >> "))
}

#[test]
fn field_example_value() -> Res {
    // with both example and default value
    let output = test_menu! {
        menu,
        "mlzigujz\n",
        let age: u8 = menu
            .written_or_default(&Written::from("your age please").example("19").default_value("18")),
        assert_eq!(age, 18),
    }?;

    assert_eq!(
        output,
        "--> your age please (example: 19, default: 18)\n>> "
    );

    // with only example
    let output = test_menu! {
        menu,
        "mlzigujz\n",
        let age: u8 = menu
            .written_or_default(&Written::from("your age please").example("19")),
        assert_eq!(age, 0),
    }?;

    assert_eq!(output, "--> your age please (example: 19, optional)\n>> ");

    // with only default value
    let output = test_menu! {
        menu,
        "mlzigujz\n",
        let age: u8 = menu
            .written_or_default(&Written::from("your age please").default_value("19")),
        assert_eq!(age, 19),
    }?;

    Ok(assert_eq!(output, "--> your age please (default: 19)\n>> "))
}

#[cfg(feature = "date")]
#[test]
fn date_value() -> Res {
    let output = test_menu! {
        menu,
        "lol\n",
        let date: NaiveDate = menu.written(&Written::from("date").default_value("2015-04-29"))?,
        assert_eq!(date, NaiveDate::from_ymd(2015, 04, 29)),
    }?;

    Ok(assert_eq!(output, "--> date (default: 2015-04-29)\n>> "))
}

#[test]
#[should_panic]
fn incorrect_default_value() {
    let _output = test_menu! {
        menu,
        "Ahmad\nno",
        let _name: MenuResult<String> = menu.written(&Written::from("name")),
        let _age: MenuResult<u8> = menu.written(&Written::from("age").default_value("yep")),
    };
}

#[test]
fn ask_until() -> Res {
    let output = test_menu! {
        menu,
        "402385\nAhmad\n",
        let name = menu.written_until(&Written::from("Author name"), |s: &String| !s.parse::<i32>().is_ok())?,
        assert_eq!(name, "Ahmad"),
    }?;

    assert_eq!(output, "--> Author name\n>> >> ");

    let output = test_menu! {
        menu,
        "-54\n-34\n0\n23\n",
        let age = menu.written_until(&Written::from("age"), |n: &i32| *n > 0)?,
        assert_eq!(age, 23),
    }?;

    Ok(assert_eq!(output, "--> age\n>> >> >> >> "))
}

#[test]
fn optional_written() -> Res {
    let written = Written::from("age");

    let output = test_menu! {
        menu,
        "38\n",
        let age: Option<u8> = menu.optional_written(&written)?,
        assert_eq!(age, Some(38)),
    }?;

    assert_eq!(output, "--> age (optional)\n>> ");

    let output = test_menu! {
        menu,
        "\n",
        let age: Option<u8> = menu.optional_written(&written)?,
        assert_eq!(age, None),
    }?;

    Ok(assert_eq!(output, "--> age (optional)\n>> "))
}

#[test]
fn optional_select() -> Res {
    let sel = Selected::new("amount", [("one", 1), ("two", 2), ("three", 3)]);
    let res = "--> amount (optional)
1 - one
2 - two
3 - three
>> ";

    let output = test_menu! {
        menu,
        "2",
        let amount: Option<u8> = menu.optional_selected(sel.clone())?,
        assert_eq!(amount, Some(2)),
    }?;

    assert_eq!(output, res);

    let output = test_menu! {
        menu,
        "zemklfj\n",
        let amount: Option<u8> = menu.optional_selected(sel)?,
        assert_eq!(amount, None),
    }?;

    Ok(assert_eq!(output, res))
}

#[test]
#[should_panic]
fn select_no_field() {
    let _output = test_menu! {
        menu,
        "hello",
        let _msg: MenuResult = menu.selected(Selected::new("hey", [])),
    };
}

#[derive(Debug, PartialEq)]
enum Type1 {
    MIT,
}

#[test]
fn select_one_field() -> Res {
    let output = test_menu! {
        menu,
        "3\n-4\n340\n1\n",
        let name = menu.selected(Selected::new("select the type", [("mit", Type1::MIT)]))?,
        assert_eq!(name, Type1::MIT),
    }?;

    Ok(assert_eq!(
        output,
        "--> select the type\n1 - mit\n>> >> >> >> "
    ))
}

#[derive(Debug, PartialEq)]
enum Type2 {
    MIT,
    GPL,
    BSD,
}

impl Default for Type2 {
    fn default() -> Self {
        Self::MIT
    }
}

impl Selectable<3> for Type2 {
    fn values() -> [(&'static str, Self); 3] {
        [("MIT", Self::MIT), ("GPL", Self::GPL), ("BSD", Self::BSD)]
    }
}

#[test]
fn selectable() -> Res {
    let output = test_menu! {
        menu,
        "2",
        let name: Type2 = menu.selected(Selected::from("select the type"))?,
        assert_eq!(name, Type2::GPL),
    }?;

    Ok(assert_eq!(
        output,
        "--> select the type
1 - MIT
2 - GPL
3 - BSD
>> "
    ))
}

#[test]
fn select_default() -> Res {
    let output = test_menu! {
        menu,
        "zmrlkgjzmklj\n",
        let name: Type2 = menu.selected_or_default(Selected::from("select the type")),
        assert_eq!(name, Type2::MIT),
    }?;

    Ok(assert_eq!(
        output,
        "--> select the type (optional)
1 - MIT
2 - GPL
3 - BSD
>> "
    ))
}

use crate::customs::*;
use crate::field::Promptable;
use crate::prelude::*;

#[test]
fn bool_parse() {
    let input = "yeppppp".parse::<MenuBool>();
    assert!(input.is_err());

    let input: MenuResult<Vec<MenuBool>> = Separated::new("", " ").prompt(
        MenuHandle::empty_writer_with("yes yep y no ye nop nan nah\n".as_bytes()),
    );

    assert_eq!(
        input.map(|v| v.into_iter().map(bool::from).collect()),
        Ok(vec![true, true, true, false, true, false, false, false])
    );
}

#[cfg(feature = "expr")]
#[test]
fn math_expr() {
    let opt: MenuNumber = "5+3-6*3".parse().unwrap();
    assert_eq!(*opt, -10.);
}

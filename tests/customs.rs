use ezmenulib::customs::*;

#[test]
fn bool_parse() {
    let input = "yeppppp".parse::<MenuBool>();
    assert!(input.is_err());

    let input = "yes yep y no ye nop nan nah"
        .parse::<MenuVec<MenuBool>>()
        .map(|v| v.iter().map(|b| b.0).collect::<Vec<bool>>())
        .map_err(|_| ());
    assert_eq!(
        input,
        Ok(vec![true, true, true, false, true, false, false, false])
    );
}

#[test]
fn empty_vec_parse() {
    let vec = "".parse::<MenuVec<i32>>();
    assert_eq!(vec, Err(MenuVecParseError::Empty));
}

#[test]
fn nonempty_vec_parse() {
    let vec = "43 56 34".parse::<MenuVec<i32>>();
    assert_eq!(vec, Ok(MenuVec(vec![43, 56, 34])));

    let vec = "Ahmad, Hugo, Oui".parse::<MenuVec<i32>>();
    assert!(vec.is_err());

    let vec = "Ahmad, Hugo, Oui".parse::<MenuVec<String>>();
    assert_eq!(
        vec,
        Ok(MenuVec(vec![
            "Ahmad,".to_owned(),
            "Hugo,".to_owned(),
            "Oui".to_owned()
        ]))
    )
}

#[test]
fn empty_menu_option_parse() {
    let opt = "".parse::<MenuOption<String>>();
    assert_eq!(opt, Ok(MenuOption(None)));
}

#[test]
fn empty_menu_option_display() {
    let opt = MenuOption(None::<i32>);
    assert_eq!(format!("{}", opt), "");
}

#[test]
fn present_menu_option_parse() {
    let opt = "345".parse::<MenuOption<i32>>();
    assert_eq!(opt, Ok(MenuOption(Some(345))));
}

#[test]
fn present_menu_option_display() {
    let opt = MenuOption(Some(345));
    assert_eq!(format!("{}", opt), "345");
}

use crate::Format;

#[test]
fn fmt_merge() {
    let fmt = Format::suffix("--> ");
    let new = fmt.merged(&Format::suffix("> "));
    assert_eq!(new.suffix, "--> ");
}

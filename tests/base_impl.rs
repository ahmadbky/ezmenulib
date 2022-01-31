use ezmenu::Menu;

// TODO: testings
/// FIXME: find a way to test input and output without using stdin and stdout
/// the idea is to do something like:
/// ```rust
/// #[derive(Menu)]
/// struct Person {
///     #[field::test(b"Ahmad")]
///     name: String,
///     #[field::test(b"19")]
///     age: u8,
/// }
///
/// let output = Person::from_menu();
/// assert_eq!(output, "- name: Ahmad\n- age: 19\n");
/// ```
#[test]
fn init_struct() {
    // ...
}

#[derive(Menu)]
struct Person {
    name: String,
    age: u8,
}

// used as testing for now
fn main() {
    let Person { name, age } = Person::from_menu();
    println!("name={}, age={}", name, age);
}

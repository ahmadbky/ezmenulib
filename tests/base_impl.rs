use ezmenu::Menu;
use std::io::stdout;

mod test {
    use ezmenu::MenuResult;
    use std::io::Write;

    pub fn check_age(age: u8, writer: &mut impl Write) -> MenuResult<u8> {
        if age == 19 {
            writer.write(b"omg just like me\n")?;
        }
        Ok(age)
    }
}

#[derive(Menu, Debug)]
#[menu(title = "bonjour", chip("- "), prefix(">> "), new_line(true))]
#[allow(dead_code)]
struct MyMenu {
    #[menu(msg = "your first name plz", prefix = ": ")]
    firstname: String,
    lastname: String,
    #[menu(msg("how old are u"), default = 4, then(test::check_age))]
    age: u8,
}

#[test]
fn base_impl() {
    let input = "Ahmad\nBaalbaky\n598\n".as_bytes();
    let menu = MyMenu::from_io(input, stdout()).unwrap();
    println!("{:?}", menu);
}

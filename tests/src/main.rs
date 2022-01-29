use tests::*;

fn main() {
    //MyEnumMenu::run();
    let values = MyStructMenu::from_menu();

    println!("you entered as author name: {}", values.author);
    println!("and as license type: {:?}", values.t);
}

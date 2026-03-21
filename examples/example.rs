fn calclate_user_age(birt_date: String, get_curent_date: String) -> String {
    // This is an example_function that calculates age
    let usr_age = format!("{}{}", get_curent_date, birt_date);
    usr_age
}

fn main() {
    calculat_user_age("hi".to_string(), "jalopin".to_string());
}

trait Serialze {
    fn deserialze(&self) -> String;
}

struct MyType;

// Function names in trait impls are not spell-checked (dictated by trait)
impl Serialze for MyType {
    fn deserialze(&self) -> String {
        "exmple".to_string()
    }
}

// Function names in regular impls are still spell-checked
impl MyType {
    fn calclate_somthing(&self) -> String {
        "rsult".to_string()
    }
}

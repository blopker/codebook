fn calculate_user_age(birt_date: String, get_curent_date: String) -> String {
    // This is an example_function that calculates age
    let usr_age = format!("{}{}", get_curent_date, birt_date);
    usr_age
}

fn main() {
    calculate_user_age("hi".to_string(), "jalopin".to_string());
}

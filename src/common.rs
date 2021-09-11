pub fn remove_last_char(input: &str) -> &str {
    let mut chars = input.chars();
    chars.next_back();
    chars.as_str()
}

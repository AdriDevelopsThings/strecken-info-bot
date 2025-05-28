pub fn normalize_spaces(input: &str) -> String {
    input.split_whitespace().collect::<Vec<&str>>().join(" ")
}

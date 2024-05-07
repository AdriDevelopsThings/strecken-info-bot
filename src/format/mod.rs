use regex::Regex;

pub mod hash;
pub mod partial_format;

pub fn format_text(text: &str) -> String {
    let text_regex = Regex::new("<br */?>").unwrap();
    text_regex.replace_all(text, "\n").to_string()
}

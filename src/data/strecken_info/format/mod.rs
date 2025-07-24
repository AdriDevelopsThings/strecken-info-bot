use regex::Regex;

mod html;
mod partial_format;
mod text;

pub use html::format as format_to_html;
pub use text::format as format_to_text;

fn format_text(text: &str) -> String {
    let text_regex = Regex::new("<br */?>").unwrap();
    text_regex.replace_all(text, "\n").to_string()
}

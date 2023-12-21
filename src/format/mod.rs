use regex::Regex;

mod hash;
#[cfg(feature = "mastodon")]
mod mastodon;
mod partial_format;
mod telegram;

fn format_text(text: &str) -> String {
    let text_regex = Regex::new("<br */?>").unwrap();
    text_regex.replace_all(text, "\n").to_string()
}

pub use hash::format as format_hash;
#[cfg(feature = "mastodon")]
pub use mastodon::format as format_mastodon;
pub use telegram::format as format_telegram;

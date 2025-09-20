use log::error;
use regex::Regex;

pub struct TriggerWarningRegex<'a> {
    pub name: &'a str,
    pub regex: &'a str,
}

impl<'a> TriggerWarningRegex<'a> {
    pub fn new(name: &'a str, regex: &'a str) -> TriggerWarningRegex<'a> {
        TriggerWarningRegex { name, regex }
    }

    fn get_regex(&self) -> Result<Regex, regex::Error> {
        Regex::new(self.regex)
    }
}

/// search for trigger warnings in a message
/// if a triggering word was found the word gets returned
/// if no word was found `None` gets returned
/// strings in `trigger_warnings_words` must occur as own words (not as parts of other words)
pub fn get_message_tw_word(message: &str, trigger_warning_words: &[&str]) -> Option<String> {
    if message.is_empty() {
        return None;
    }

    // first we checn for triggering words
    let chars = message.chars().collect::<Vec<char>>();
    let mut words: Vec<String> = Vec::new();
    // split message into words

    let mut start_of_word: Option<usize> = None;
    for (i, c) in chars.iter().enumerate() {
        if !c.is_ascii_alphanumeric() {
            // character is not alphanumeric: end of word reached
            if let Some(start_of_word) = start_of_word {
                // word is from start_of_word to i-1
                words.push(
                    chars[start_of_word..i]
                        .iter()
                        .collect::<String>()
                        .to_lowercase(),
                );
            }
            start_of_word = None;
        } else if start_of_word.is_none() {
            // i is start of word
            start_of_word = Some(i);
        }
    }

    // loop ends -> end of word reached
    if let Some(start_of_word) = start_of_word {
        // word is from start_of_word to i-1
        words.push(
            chars[start_of_word..chars.len()]
                .iter()
                .collect::<String>()
                .to_lowercase(),
        );
    }

    for tw in trigger_warning_words {
        if words.contains(&tw.to_string().to_lowercase()) {
            return Some(tw.to_string());
        }
    }

    None
}

/// search for trigger warnings in a message
/// if a regex applies on the message the name will be returned,
/// otherwise None
pub fn get_message_tw_regex(
    message: &str,
    trigger_warning_regex: &[TriggerWarningRegex],
) -> Option<String> {
    for regex in trigger_warning_regex {
        let r = match regex.get_regex() {
            Ok(r) => r,
            Err(e) => {
                error!(
                    "Error while compiling regex name {} with regex {}: {e:?}",
                    regex.name, regex.regex
                );
                continue;
            }
        };

        if r.is_match(message) {
            return Some(regex.name.to_string());
        }
    }

    None
}

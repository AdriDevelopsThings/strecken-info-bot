/// search for trigger warnings in a mesage
/// if a triggering word was found the word gets returned
/// if no word was found `None` gets returned
pub fn get_message_tw_word(message: &str, trigger_warnings: &[&str]) -> Option<String> {
    if message.is_empty() {
        return None;
    }

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

    for tw in trigger_warnings {
        if words.contains(&tw.to_string().to_lowercase()) {
            return Some(tw.to_string());
        }
    }

    None
}

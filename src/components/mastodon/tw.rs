use log::error;
use std::env;

use crate::{
    data::DataDisruptionInformation,
    tw::{get_message_tw_regex, get_message_tw_word, TriggerWarningRegex},
};

pub(super) fn get_trigger_word(disruption: &DataDisruptionInformation) -> Option<String> {
    let tws = env::var("MASTODON_TRIGGER_WARNINGS")
        .map(|s| s.split(',').map(|k| k.to_owned()).collect::<Vec<String>>())
        .unwrap_or_default();
    let tws_str = tws.iter().map(|s| s.as_str()).collect::<Vec<_>>();

    let tw_regex = env::var("MASTODON_TRIGGER_WARNING_REGEX")
        .map(|s| s.split(',').map(|k| k.to_owned()).collect::<Vec<String>>())
        .unwrap_or_default();
    let tw_regex_twr = tw_regex
        .iter()
        .filter_map(|s| {
            let l = s.split_once(':');
            if let Some(l) = l {
                Some(TriggerWarningRegex::new(l.0, l.1))
            } else {
                error!("Regex environment variable {s} is invalid, it does not contain a ':'");
                None
            }
        })
        .collect::<Vec<TriggerWarningRegex>>();

    let message = disruption.disruption.format_tws();

    get_message_tw_word(&message, &tws_str)
        .or_else(|| get_message_tw_regex(&message, &tw_regex_twr))
}

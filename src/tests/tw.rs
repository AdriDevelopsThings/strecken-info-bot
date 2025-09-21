use crate::tw::{get_message_tw_regex, get_message_tw_word, TriggerWarningRegex};

#[test]
fn test_tw_words() {
    let tws = &["PU", "Personenunfall", "Suizid"];
    assert_eq!(
        get_message_tw_word(
            "Das ist ein einfacher Disruption Text ohne Wörter die triggern könnten.",
            tws
        ),
        None
    );
    assert_eq!(
        get_message_tw_word(
            "Das ist ein Disruption Text der triggern könnte, denn es geht um einen PErsonenunfall.",
            tws
        ),
         Some("Personenunfall".to_string())
        );
    assert_eq!(
        get_message_tw_word(
            "Leider kann Suizid auch in einem Disruption Text vorkommen...",
            tws
        ),
        Some("Suizid".to_string())
    );

    assert_eq!(
        get_message_tw_word(
            "Außerdem ist zu testen, wenn das letzte Wort das TW Wort ist: Personenunfall",
            &["Personenunfall"]
        ),
        Some("Personenunfall".to_string())
    );
}

#[test]
fn test_tw_regex() {
    let regex = &[TriggerWarningRegex::new("Suizid", ".*(?i)suizid.*")];

    assert_eq!(
        get_message_tw_regex(
            "Hallo das ist ein Test, der das Wort mit S nicht enthält.",
            regex
        ),
        None
    );

    assert_eq!(
        get_message_tw_regex(
            "Im Haltepunkt Leipzig-Sellerhausen liegt eine Person mit Suizidabsichten im Gleis. Behörden und Rettungsdienst sind in der Anfahrt.",
            regex
        ),
        Some("Suizid".to_string())
    );
}

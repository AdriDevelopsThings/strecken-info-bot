use crate::tw::get_message_tw_word;

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
    assert_eq!(get_message_tw_word("Das ist ein Disruption Text der triggern könnte, denn es geht um einen PErsonenunfall.", tws), Some("Personenunfall".to_string()));
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

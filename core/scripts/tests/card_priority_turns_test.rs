#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use scripts::models::card_priority_per_wave::CardPriorityPerWave;

const ENTRY0: &str = "WB, WA, WQ, B, A, Q, RB, RA, RQ";
const ENTRY1: &str = "Q, WQ, RQ, A, WA, RA, B, WB, RB";
const ENTRY2: &str = "RA, WA, A, B, WB, RB, Q, WQ, RQ";

#[test]
fn ten_entries_parse_and_are_stored_unchanged() {
    let text: Vec<&str> = (0..10)
        .map(|index| {
            if index == 9 {
                ENTRY2
            } else if index % 2 == 0 {
                ENTRY0
            } else {
                ENTRY1
            }
        })
        .collect();
    let text = text.join("\n");

    let parsed = CardPriorityPerWave::of(&text).unwrap();

    assert_eq!(parsed.len(), 10);
    assert_eq!(
        CardPriorityPerWave::of(&parsed.to_string())
            .unwrap()
            .to_string(),
        parsed.to_string()
    );
    assert_eq!(parsed.to_string().lines().count(), 10);
    let expected = CardPriorityPerWave::of(ENTRY2).unwrap().at_wave(0).unwrap();
    let last: Vec<String> = parsed
        .at_wave(9)
        .unwrap()
        .iter()
        .take(9)
        .map(ToString::to_string)
        .collect();
    let expected: Vec<String> = expected.iter().take(9).map(ToString::to_string).collect();
    assert_eq!(last.join(", "), expected.join(", "));
}

#[test]
fn the_last_entry_repeats_past_the_end_of_the_list() {
    let three = [ENTRY0, ENTRY1, ENTRY2].join("\n");
    let parsed = CardPriorityPerWave::of(&three).unwrap();

    assert_eq!(parsed.at_wave(2).unwrap(), parsed.at_wave(3).unwrap());
    assert_eq!(parsed.at_wave(2).unwrap(), parsed.at_wave(50).unwrap());
}

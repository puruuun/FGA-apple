#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use scripts::enums::card_type_enum::CardTypeEnum;
use scripts::models::model_error::ModelError;
use scripts::models::servant_card_key::ServantCardKey;
use scripts::models::servant_card_priority_per_wave::ServantCardPriorityPerWave;

const INTERLEAVED: &str = "1B,2B,1A,3Q,2A,1Q,3B,3A,2Q";

fn servant_priority(entries: &[&str]) -> ServantCardPriorityPerWave {
    ServantCardPriorityPerWave::of(&entries.join("\n")).unwrap()
}

fn join(keys: &[ServantCardKey]) -> String {
    let parts: Vec<String> = keys.iter().map(ToString::to_string).collect();
    parts.join(",")
}

#[test]
fn lists_round_trip_through_their_stored_text() {
    let list = servant_priority(&[INTERLEAVED, "3Q,3B,3A,2Q,2B,2A,1Q,1B,1A"]);

    let restored = ServantCardPriorityPerWave::of(&list.to_string()).unwrap();

    assert_eq!(restored.to_string(), list.to_string());
    assert_eq!(restored.len(), 2);
    assert_eq!(join(restored.at_wave(0)), INTERLEAVED);
}

#[test]
fn ten_entries_are_stored() {
    let all = ServantCardKey::all();
    let entries: Vec<String> = (0..10)
        .map(|rotation| {
            let mut rotated = all.clone();
            rotated.rotate_left(rotation);
            join(&rotated)
        })
        .collect();

    let list = ServantCardPriorityPerWave::of(&entries.join("\n")).unwrap();

    assert_eq!(list.len(), 10);
    assert_eq!(
        ServantCardPriorityPerWave::of(&list.to_string())
            .unwrap()
            .to_string(),
        list.to_string()
    );
}

#[test]
fn blank_settings_give_the_default_order() {
    assert_eq!(
        ServantCardPriorityPerWave::of("").unwrap().to_string(),
        "1B,1A,1Q,2B,2A,2Q,3B,3A,3Q"
    );
    assert_eq!(ServantCardPriorityPerWave::default().len(), 1);
}

#[test]
fn old_affinity_text_is_rejected_so_settings_fall_back_to_the_default() {
    for text in [
        "WB, WA, WQ, B, A, Q, RB, RA, RQ",
        "1B,1A,1Q,2B,2A,2Q,3B,3A",
        "1B,1B,1Q,2B,2A,2Q,3B,3A,3Q",
        "4B,1A,1Q,2B,2A,2Q,3B,3A,3Q",
    ] {
        assert!(
            matches!(
                ServantCardPriorityPerWave::of(text),
                Err(ModelError::IllegalArgument(_))
            ),
            "{text}"
        );
    }
}

#[test]
fn keys_parse_and_print() {
    assert_eq!(
        ServantCardKey::parse("2a"),
        Some(ServantCardKey::new(2, CardTypeEnum::Arts).unwrap())
    );
    assert_eq!(ServantCardKey::parse("3X"), None);
    assert_eq!(ServantCardKey::parse("0B"), None);
    assert_eq!(
        ServantCardKey::new(3, CardTypeEnum::Quick)
            .unwrap()
            .to_string(),
        "3Q"
    );
    assert_eq!(ServantCardKey::all().len(), 9);
}

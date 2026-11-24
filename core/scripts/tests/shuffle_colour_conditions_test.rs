#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use scripts::enums::card_type_enum::CardTypeEnum;
use scripts::models::auto_skill_action::{AutoSkillAction, ShuffleCondition, ShuffleUntil};
use scripts::models::field_slot::FieldSlot;

fn parse_shuffle(token: &str) -> ShuffleUntil {
    let mut parsed = actions(token, 0, 0);
    assert_eq!(parsed.len(), 1, "{token}");
    match parsed.remove(0) {
        AutoSkillAction::ShuffleUntil(shuffle) => shuffle,
        other => panic!("{token}: not a shuffle: {other:?}"),
    }
}

#[test]
fn old_conditions_retain_all_colours_and_their_command_form() {
    for token in ["aH{10-30:13}", "dH{8:22|31!}"] {
        let action = parse_shuffle(token);
        assert!(action.conditions.iter().all(|c| c.r#type.is_none()));
        assert_eq!(action.to_command_string(), token);
        assert_eq!(parse_shuffle(&action.to_command_string()), action);
    }
}

#[test]
fn coloured_mixed_and_uppercase_tokens_round_trip_in_lowercase_with_stop_suffix() {
    for (token, expected) in [
        ("aH{10-30:1b2|1q2}", "aH{10-30:1b2|1q2}"),
        ("dH{8:13|2a2}", "dH{8:13|2a2}"),
        ("gH{3-9:1B2|2A3|3Q1!}", "gH{3-9:1b2|2a3|3q1!}"),
    ] {
        let action = parse_shuffle(token);
        assert_eq!(action.to_command_string(), expected);
        assert_eq!(parse_shuffle(expected), action);
    }
    assert_eq!(
        parse_shuffle("aH{10-30:1b2|1q2}").conditions,
        vec![
            ShuffleCondition::new(FieldSlot::A, 2, Some(CardTypeEnum::Buster)).unwrap(),
            ShuffleCondition::new(FieldSlot::A, 2, Some(CardTypeEnum::Quick)).unwrap()
        ]
    );
}

#[test]
fn invalid_colours_and_counts_fail_through_the_existing_parse_error() {
    for condition in [
        "1x2", "1n2", "1bb2", "1b0", "1b6", "0b2", "4a2", "1?2", "1_2", "1b", "1b22",
    ] {
        assert!(
            is_parse_error(&format!("aH{{8:{condition}}}")),
            "{condition}"
        );
    }
    assert!(ShuffleCondition::new(FieldSlot::A, 2, Some(CardTypeEnum::Unknown)).is_err());
}

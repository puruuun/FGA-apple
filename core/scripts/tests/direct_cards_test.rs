#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use scripts::enums::card_affinity_enum::CardAffinityEnum;
use scripts::enums::card_type_enum::CardTypeEnum;
use scripts::kotlin::LinkedSet;
use scripts::models::auto_skill_action::AutoSkillAction;
use scripts::models::command_card::{CommandCard, Face, NP};
use scripts::models::direct_card_sequence::DirectCardSequence;
use scripts::models::parsed_card::ParsedCard;
use scripts::models::skill::Servant;

fn seq(code: &str) -> DirectCardSequence {
    DirectCardSequence::parse(code).unwrap()
}

fn atk_of(action: &AutoSkillAction) -> &scripts::models::auto_skill_action::Atk {
    match action {
        AutoSkillAction::Atk(atk) => atk,
        other => panic!("not an attack: {other:?}"),
    }
}

fn by_index(remaining: Vec<ParsedCard>) -> Vec<ParsedCard> {
    let mut sorted = remaining;
    sorted.sort_by_key(|card| card.card.index());
    sorted
}

fn by_index_desc(remaining: Vec<ParsedCard>) -> Vec<ParsedCard> {
    let mut sorted = remaining;
    sorted.sort_by_key(|card| std::cmp::Reverse(card.card.index()));
    sorted
}

fn arts_and_quicks() -> Vec<ParsedCard> {
    hand()
        .into_iter()
        .enumerate()
        .map(|(index, card)| ParsedCard {
            r#type: if index == 0 {
                CardTypeEnum::Arts
            } else {
                CardTypeEnum::Quick
            },
            affinity: CardAffinityEnum::Normal,
            ..card
        })
        .collect()
}

fn stunned(card: ParsedCard) -> ParsedCard {
    ParsedCard {
        is_stunned: true,
        r#type: CardTypeEnum::Unknown,
        ..card
    }
}

#[test]
fn ampersand_commands_retain_skills_turns_waves_and_np_identities() {
    let command = parse_ok("ab&ba4,&bba,&b5a,#,&4ba");
    let first = command.get(0, 0).unwrap();
    assert_eq!(first.len(), 3);
    assert_eq!(
        atk_of(first.last().unwrap())
            .direct_cards
            .as_ref()
            .map(|d| d.code()),
        Some("ba4")
    );
    let second = command.get(0, 1).unwrap();
    assert_eq!(second.len(), 1);
    assert_eq!(
        atk_of(&second[0]).direct_cards.as_ref().map(|d| d.code()),
        Some("bba")
    );
    let third = command.get(0, 2).unwrap();
    assert_eq!(third.len(), 1);
    assert_eq!(
        atk_of(&third[0]).nps,
        [NP::B].into_iter().collect::<LinkedSet<_>>()
    );
    let wave = command.get(1, 0).unwrap();
    assert_eq!(wave.len(), 1);
    assert_eq!(atk_of(&wave[0]).to_command_string(), "&4ba");
}

#[test]
fn weak_normal_resist_busters_are_selected_in_affinity_order_regardless_of_input_order() {
    let sequence = seq("bbb");
    let cards: Vec<ParsedCard> = hand()
        .into_iter()
        .enumerate()
        .map(|(index, card)| {
            if index < 3 {
                ParsedCard {
                    r#type: CardTypeEnum::Buster,
                    affinity: CardAffinityEnum::ENTRIES[index],
                    ..card
                }
            } else {
                card
            }
        })
        .collect();
    let reversed: Vec<ParsedCard> = cards.into_iter().rev().collect();
    assert_eq!(
        sequence.resolve(&reversed),
        Some(vec![FACE_B, FACE_A, FACE_C])
    );
}

#[test]
fn every_requested_colour_prefers_weak_then_normal_then_resist() {
    for (token, colour) in [
        ('b', CardTypeEnum::Buster),
        ('a', CardTypeEnum::Arts),
        ('q', CardTypeEnum::Quick),
    ] {
        let cards: Vec<ParsedCard> = hand()
            .into_iter()
            .enumerate()
            .map(|(index, card)| ParsedCard {
                r#type: colour,
                affinity: match index {
                    2 => CardAffinityEnum::Weak,
                    1 => CardAffinityEnum::Normal,
                    _ => CardAffinityEnum::Resist,
                },
                ..card
            })
            .collect();
        let reversed: Vec<ParsedCard> = cards.iter().rev().copied().collect();
        let sequence = seq(&format!("{token}45"));
        assert_eq!(sequence.resolve(&reversed).unwrap().first(), Some(&FACE_C));
        let with_stunned: Vec<ParsedCard> = cards
            .iter()
            .map(|card| {
                if card.card == Face::C {
                    ParsedCard {
                        is_stunned: true,
                        ..*card
                    }
                } else {
                    *card
                }
            })
            .collect();
        assert_eq!(
            sequence.resolve(&with_stunned).unwrap().first(),
            Some(&FACE_B)
        );
    }
}

#[test]
fn affinity_ties_stay_leftmost_even_with_shuffled_input() {
    let cards: Vec<ParsedCard> = hand()
        .into_iter()
        .map(|card| ParsedCard {
            r#type: CardTypeEnum::Buster,
            affinity: CardAffinityEnum::Weak,
            ..card
        })
        .collect();
    let reversed: Vec<ParsedCard> = cards.into_iter().rev().collect();
    assert_eq!(
        seq("bbb").resolve(&reversed),
        Some(vec![FACE_A, FACE_B, FACE_C])
    );
}

#[test]
fn bba_and_bbb_reserve_the_best_buster_for_the_earliest_slot() {
    let cards: Vec<ParsedCard> = hand()
        .into_iter()
        .enumerate()
        .map(|(index, card)| ParsedCard {
            r#type: if index < 3 {
                CardTypeEnum::Buster
            } else {
                CardTypeEnum::Arts
            },
            affinity: match index {
                0 => CardAffinityEnum::Resist,
                2 => CardAffinityEnum::Weak,
                _ => CardAffinityEnum::Normal,
            },
            ..card
        })
        .collect();
    assert_eq!(
        seq("bba").resolve(&cards),
        Some(vec![FACE_C, FACE_B, FACE_D])
    );
    assert_eq!(
        seq("bbb").resolve(&cards),
        Some(vec![FACE_C, FACE_B, FACE_A])
    );
}

#[test]
fn fallback_still_uses_supplied_priority_for_gaps_after_affinity_reservation() {
    let cards: Vec<ParsedCard> = hand()
        .into_iter()
        .map(|card| {
            if card.r#type == CardTypeEnum::Quick {
                ParsedCard {
                    r#type: CardTypeEnum::Arts,
                    ..card
                }
            } else {
                card
            }
        })
        .collect();
    assert_eq!(
        seq("?q4b").resolve_with(&cards, by_index),
        Some(vec![FACE_A, NP_A, FACE_C])
    );
}

#[test]
fn stunned_cards_are_excluded_and_cannot_be_reused_to_satisfy_two_slots() {
    let cards: Vec<ParsedCard> = hand()
        .into_iter()
        .map(|card| {
            if card.card == Face::A {
                stunned(card)
            } else {
                card
            }
        })
        .collect();
    assert_eq!(seq("b4a").resolve(&cards), Some(vec![FACE_C, NP_A, FACE_B]));
    assert_eq!(seq("bba").resolve(&cards), None);
}

#[test]
fn invalid_lengths_choices_duplicate_nps_and_mixing_automatic_attacks_are_rejected() {
    for code in [
        "&ba", "&bb44", "&wb4", "&4a4", "4&ba4", "n1&ba4", "&ba4a", "&ba4&bba", "&bq7", "&ba0",
    ] {
        assert!(is_parse_error(code), "{code}");
    }
}

#[test]
fn normal_commands_preserve_their_old_syntax_and_uppercase_fixed_choices_normalize() {
    for code in ["4", "n14", "n24", "54", "0"] {
        let actions = actions(code, 0, 0);
        assert_eq!(actions.len(), 1, "{code}");
        assert_eq!(atk_of(&actions[0]).to_command_string(), code);
    }
    let normalized = actions("&BA4", 0, 0);
    assert_eq!(atk_of(&normalized[0]).to_command_string(), "&ba4");
}

#[test]
fn legacy_no_op_before_a_fixed_attack_is_harmless_and_does_not_remove_preceding_skills() {
    let actions = actions("0a&bba", 0, 0);
    assert_eq!(actions.len(), 2);
    assert_eq!(atk_of(actions.last().unwrap()).to_command_string(), "&bba");
    match &actions[0] {
        AutoSkillAction::ServantSkill(skill) => assert_eq!(skill.skill, Servant::A1),
        other => panic!("not a servant skill: {other:?}"),
    }
}

#[test]
fn multiple_gaps_repeated_colors_and_all_np_positions_reserve_globally_without_reuse() {
    let cards = arts_and_quicks();
    let expected = [
        ("?bba", vec![FACE_B, FACE_C, FACE_A]),
        ("?aaa", vec![FACE_A, FACE_B, FACE_C]),
        ("?4ba", vec![NP_A, FACE_B, FACE_A]),
        ("?b4a", vec![FACE_B, NP_A, FACE_A]),
        ("?ba4", vec![FACE_B, FACE_A, NP_A]),
        ("?bb4", vec![FACE_A, FACE_B, NP_A]),
    ];
    for (code, wanted) in expected {
        assert_eq!(seq(code).resolve(&cards), Some(wanted), "{code}");
    }
    assert_eq!(
        seq("?aaa").resolve_with(&cards, |remaining| remaining.into_iter().rev().collect()),
        Some(vec![FACE_A, FACE_E, FACE_D])
    );
}

#[test]
fn malformed_duplicate_priorities_never_reuse_faces_and_fail_atomically_when_exhausted() {
    assert_eq!(
        seq("?bb4").resolve_with(&arts_and_quicks(), |it| vec![it[0], it[0], it[1]]),
        Some(vec![FACE_A, FACE_B, NP_A])
    );
}

#[test]
fn fallback_excludes_stunned_cards_and_rejects_bad_hands_insufficient_cards_and_missing_np_before_taps()
 {
    let stunned_hand: Vec<ParsedCard> = arts_and_quicks()
        .into_iter()
        .map(|card| {
            if card.card == Face::B {
                stunned(card)
            } else {
                card
            }
        })
        .collect();
    assert_eq!(
        seq("?b4a").resolve(&stunned_hand),
        Some(vec![FACE_C, NP_A, FACE_A])
    );
}

#[test]
fn fallback_syntax_normalizes_and_retains_skills_separators_no_ops_and_per_turn_flag() {
    let parsed = parse_ok("0a&?B4A,&b4a,#,&?456");
    let first = parsed.get(0, 0).unwrap();
    assert_eq!(first.len(), 2);
    assert_eq!(atk_of(first.last().unwrap()).to_command_string(), "&?b4a");
    let second = parsed.get(0, 1).unwrap();
    assert_eq!(second.len(), 1);
    assert_eq!(
        atk_of(&second[0])
            .direct_cards
            .as_ref()
            .map(|d| d.fallback()),
        Some(false)
    );
    let third = parsed.get(1, 0).unwrap();
    assert_eq!(third.len(), 1);
    assert_eq!(atk_of(&third[0]).to_command_string(), "&?456");
    assert!(!seq("b4a").fallback());
}

#[test]
fn fallback_marker_is_allowed_only_once_immediately_after_ampersand_with_three_choices() {
    for code in [
        "&?",
        "&?ba",
        "&?bb44",
        "&??ba4",
        "&b?4a",
        "?&ba4",
        "&?4a4",
        "&?ba4a",
        "&?ba4&bba",
        "4&?ba4",
        "n1&?ba4",
        "&?bq7",
        "&?ba0",
        "&?ba4?",
    ] {
        assert!(is_parse_error(code), "{code}");
    }
}

#[test]
fn unselected_slots_at_front_middle_and_back_are_filled_by_priority_after_fixed_slots() {
    let cards = hand();
    assert_eq!(
        seq("?pb4").resolve_with(&cards, by_index),
        Some(vec![FACE_A, FACE_C, NP_A])
    );
    assert_eq!(
        seq("?bp4").resolve_with(&cards, by_index),
        Some(vec![FACE_C, FACE_A, NP_A])
    );
    assert_eq!(
        seq("?b4p").resolve_with(&cards, by_index),
        Some(vec![FACE_C, NP_A, FACE_A])
    );
    assert_eq!(
        seq("?pp4").resolve_with(&cards, by_index),
        Some(vec![FACE_A, FACE_B, NP_A])
    );
    assert_eq!(
        seq("?ppp").resolve_with(&cards, by_index),
        Some(vec![FACE_A, FACE_B, FACE_C])
    );
    assert_eq!(seq("?pp4").cards_before_np(), 2);
    assert_eq!(seq("?ppp").nps(), LinkedSet::new());
}

#[test]
fn colour_reservation_beats_an_earlier_unselected_slot_and_cards_are_never_picked_twice() {
    assert_eq!(
        seq("?pa4").resolve_with(&arts_and_quicks(), by_index),
        Some(vec![FACE_B, FACE_A, NP_A])
    );
    for code in ["?ppp", "?pap", "?qpp", "?pp4", "?pqa"] {
        let result = seq(code)
            .resolve_with(&arts_and_quicks(), by_index)
            .unwrap();
        let distinct: std::collections::HashSet<CommandCard> = result.iter().copied().collect();
        assert_eq!(distinct.len(), 3, "{code}");
    }
}

#[test]
fn colour_shortage_and_unselected_slots_share_one_priority_sort_in_slot_order() {
    let no_quick: Vec<ParsedCard> = hand()
        .into_iter()
        .map(|card| {
            if card.r#type == CardTypeEnum::Quick {
                ParsedCard {
                    r#type: CardTypeEnum::Arts,
                    ..card
                }
            } else {
                card
            }
        })
        .collect();
    assert_eq!(
        seq("?qp4").resolve_with(&no_quick, by_index),
        Some(vec![FACE_A, FACE_B, NP_A])
    );
    assert_eq!(
        seq("?qp4").resolve_with(&no_quick, by_index_desc),
        Some(vec![FACE_E, FACE_D, NP_A])
    );
    let detailed = seq("?bp4")
        .resolve_detailed_with(&hand(), by_index)
        .unwrap();
    assert!(detailed.used_priority);
    assert!(
        !seq("?bq4")
            .resolve_detailed_with(&hand(), by_index)
            .unwrap()
            .used_priority
    );
}

#[test]
fn priority_changes_only_affect_the_unselected_slots() {
    let cards = hand();
    let forward = seq("?bp4").resolve_with(&cards, by_index).unwrap();
    let backward = seq("?bp4").resolve_with(&cards, by_index_desc).unwrap();
    assert_eq!(forward[0], backward[0]);
    assert_eq!(forward[2], backward[2]);
    assert_eq!(forward[1], FACE_A);
    assert_eq!(backward[1], FACE_E);
}

#[test]
fn unselected_skips_stunned_cards_and_stops_on_too_few_usable_or_unknown_cards() {
    let stun_b: Vec<ParsedCard> = hand()
        .into_iter()
        .map(|card| {
            if card.card == Face::B {
                stunned(card)
            } else {
                card
            }
        })
        .collect();
    assert_eq!(
        seq("?ppp").resolve_with(&stun_b, by_index),
        Some(vec![FACE_A, FACE_C, FACE_D])
    );
    let two_usable: Vec<ParsedCard> = hand()
        .into_iter()
        .map(|card| {
            if matches!(card.card, Face::A | Face::B | Face::C) {
                stunned(card)
            } else {
                card
            }
        })
        .collect();
    assert_eq!(seq("?ppp").resolve_with(&two_usable, by_index), None);
    assert_eq!(
        seq("?pp4").resolve_with(&two_usable, by_index),
        Some(vec![FACE_D, FACE_E, NP_A])
    );
    let unknown: Vec<ParsedCard> = hand()
        .into_iter()
        .map(|card| {
            if card.card == Face::E {
                ParsedCard {
                    r#type: CardTypeEnum::Unknown,
                    ..card
                }
            } else {
                card
            }
        })
        .collect();
    assert_eq!(seq("?ppp").resolve_with(&unknown, by_index), None);
    let mut short = hand();
    short.pop();
    assert_eq!(seq("?ppp").resolve_with(&short, by_index), None);
}

#[test]
fn unselected_requires_the_fallback_flag_and_uppercase_normalizes_and_round_trips() {
    assert!(DirectCardSequence::parse("bp4").is_err());
    for code in ["&bp4", "&ppp", "&4pp"] {
        assert!(is_parse_error(code), "{code}");
    }
    for code in ["&?bp4", "&?pp4", "&?ppp", "&?p4q"] {
        let parsed = actions(code, 0, 0);
        assert_eq!(parsed.len(), 1, "{code}");
        assert_eq!(atk_of(&parsed[0]).to_command_string(), code);
    }
    assert_eq!(
        atk_of(&actions("&?BP4", 0, 0)[0]).to_command_string(),
        "&?bp4"
    );
    assert_eq!(seq("?bp4").to_string(), "&?bp4");
    for code in ["&b4a", "&?b4a"] {
        assert_eq!(atk_of(&actions(code, 0, 0)[0]).to_command_string(), code);
    }
}

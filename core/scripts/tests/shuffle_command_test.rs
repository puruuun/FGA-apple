#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use scripts::kotlin::LinkedSet;
use scripts::models::auto_skill_action::{AutoSkillAction, ShuffleCondition, ShuffleUntil};
use scripts::models::command_card::NP;
use scripts::models::field_slot::FieldSlot;
use scripts::models::servant_target::{ServantTarget, SpecialTarget};
use scripts::models::skill::Servant;

fn single_shuffle(token: &str) -> ShuffleUntil {
    let parsed = actions(token, 0, 0);
    assert_eq!(parsed.len(), 1, "{token}");
    match parsed.into_iter().next() {
        Some(AutoSkillAction::ShuffleUntil(shuffle)) => shuffle,
        other => panic!("{token}: not a shuffle: {other:?}"),
    }
}

#[test]
fn uppercase_h_parses_each_servant_first_skill_with_optional_min_colours_and_stop() {
    for skill in Servant::SKILL1 {
        for min in ["", "0-", "2-"] {
            for conditions in ["13|22", "1b2|2a3|3q1"] {
                for stop in ["", "!"] {
                    let token = format!(
                        "{}H{{{}8:{}{}}}",
                        skill.auto_skill_code(),
                        min,
                        conditions,
                        stop
                    );
                    let action = single_shuffle(&token);
                    assert_eq!(action.skill, skill, "{token}");
                    assert_eq!(action.min, if min == "2-" { 2 } else { 0 }, "{token}");
                    assert_eq!(action.max, 8, "{token}");
                    let expected: Vec<ShuffleCondition> = conditions
                        .split('|')
                        .map(|code| ShuffleCondition::from_code(code).unwrap())
                        .collect();
                    assert_eq!(action.conditions, expected, "{token}");
                    assert_eq!(action.stop_if_unmet, stop == "!", "{token}");
                    assert_eq!(
                        action.to_command_string(),
                        token.replace("H{0-", "H{"),
                        "{token}"
                    );
                    assert_eq!(
                        AutoSkillAction::ShuffleUntil(action),
                        actions(&token.replace("H{0-", "H{"), 0, 0).remove(0),
                        "{token}"
                    );
                }
            }
        }
    }
}

#[test]
fn shuffle_rejects_every_servant_second_and_third_skill() {
    for skill in Servant::SKILL2.iter().chain(Servant::SKILL3.iter()) {
        let token = format!("{}H{{8:13}}", skill.auto_skill_code());
        assert!(is_parse_error(&token), "{token}");
        let condition = ShuffleCondition::new(FieldSlot::A, 3, None).unwrap();
        assert!(
            ShuffleUntil::new(*skill, 0, 8, vec![condition], false).is_err(),
            "{token}"
        );
    }
}

#[test]
fn old_shuffle_form_and_lowercase_h_typo_are_rejected_for_every_servant_skill() {
    for skill in Servant::LIST {
        for marker in ["", "h"] {
            let token = format!("{}{}{{8:13}}", skill.auto_skill_code(), marker);
            assert!(is_parse_error(&token), "{token}");
        }
    }
}

#[test]
fn uppercase_h_requires_a_preceding_servant_skill_and_an_opening_brace() {
    for token in [
        "H",
        "H{8:13}",
        "aH",
        "aH8:13}",
        "aH[Ch2A]",
        "aHH{8:13}",
        "a1H{8:13}",
        "jH{8:13}",
    ] {
        assert!(is_parse_error(token), "{token}");
    }
}

#[test]
fn lowercase_h_and_targeted_servant_skills_without_braces_retain_their_meaning() {
    let parsed = actions("aha1c[Ch2A]", 0, 0);
    let skills: Vec<Servant> = parsed
        .iter()
        .map(|action| match action {
            AutoSkillAction::ServantSkill(skill) => skill.skill,
            other => panic!("not a servant skill: {other:?}"),
        })
        .collect();
    assert_eq!(
        skills,
        vec![Servant::A1, Servant::C2, Servant::A1, Servant::A3]
    );
    let targets: Vec<Vec<ServantTarget>> = parsed
        .iter()
        .map(|action| match action {
            AutoSkillAction::ServantSkill(skill) => skill.targets.clone(),
            other => panic!("not a servant skill: {other:?}"),
        })
        .collect();
    assert_eq!(
        targets,
        vec![
            vec![],
            vec![],
            vec![ServantTarget::A],
            vec![ServantTarget::SpecialTarget(SpecialTarget::Choice2OptionA)]
        ]
    );
}

#[test]
fn tokens_parse_and_round_trip_as_equal_actions() {
    for token in [
        "aH{8:13|22}",
        "aH{2-8:13|22!}",
        "dH{30:23}",
        "gH{0-1:15}",
        "gH{30-30:11|25|32!}",
    ] {
        let action = single_shuffle(token);
        assert_eq!(
            AutoSkillAction::ShuffleUntil(action.clone()),
            actions(&action.to_command_string(), 0, 0).remove(0),
            "{token}"
        );
    }
}

#[test]
fn malformed_tokens_use_the_existing_parse_error() {
    for token in [
        "aH{8:13",
        "aH{0:13}",
        "aH{31:13}",
        "aH{3-2:13}",
        "aH{-1-8:13}",
        "aH{8:}",
        "aH{8:10}",
        "aH{8:16}",
        "aH{8:43}",
        "aH{8:123}",
        "aH{8:13|}",
        "aH{8:13!!}",
        "aH{8:13,22}",
        "aH{8:13#22}",
        "aH{99999999999:13}",
    ] {
        assert!(is_parse_error(token), "{token}");
    }
}

#[test]
fn legacy_targets_commands_turns_and_attacks_retain_their_actions() {
    let command = parse_ok("a1b[Ch2A]c(1[Ch3B])x13t3n14,0,#,d&ba4");
    let first = command.get(0, 0).unwrap();
    let targets = |index: usize| match &first[index] {
        AutoSkillAction::ServantSkill(skill) => skill.targets.clone(),
        other => panic!("not a servant skill: {other:?}"),
    };
    assert_eq!(targets(0), vec![ServantTarget::A]);
    assert_eq!(
        targets(1),
        vec![ServantTarget::SpecialTarget(SpecialTarget::Choice2OptionA)]
    );
    assert_eq!(
        targets(2),
        vec![
            ServantTarget::A,
            ServantTarget::SpecialTarget(SpecialTarget::Choice3OptionB)
        ]
    );
    assert!(matches!(first[3], AutoSkillAction::OrderChange(_)));
    assert!(matches!(first[4], AutoSkillAction::TargetEnemy(_)));
    match &first[5] {
        AutoSkillAction::Atk(atk) => {
            assert_eq!(atk.cards_before_np, 1);
            assert_eq!(atk.nps, [NP::A].into_iter().collect::<LinkedSet<_>>());
        }
        other => panic!("not an attack: {other:?}"),
    }
    let second = command.get(0, 1).unwrap();
    assert_eq!(second.len(), 1);
    assert_eq!(
        second[0],
        AutoSkillAction::Atk(scripts::models::auto_skill_action::Atk::no_op())
    );
    match command.get(1, 0).unwrap().last() {
        Some(AutoSkillAction::Atk(atk)) => assert_eq!(atk.to_command_string(), "&ba4"),
        other => panic!("not an attack: {other:?}"),
    }
}

#[test]
fn shuffle_can_precede_direct_cards_and_np_actions_in_the_same_turn() {
    for suffix in ["&ba4", "456"] {
        let parsed = actions(&format!("aH{{8:13}}{suffix}"), 0, 0);
        assert!(matches!(
            parsed.first(),
            Some(AutoSkillAction::ShuffleUntil(_))
        ));
        assert!(matches!(parsed.last(), Some(AutoSkillAction::Atk(_))));
    }
}

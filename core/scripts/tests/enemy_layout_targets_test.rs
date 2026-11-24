#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

mod common;

use common::*;
use scripts::models::auto_skill_action::AutoSkillAction;
use scripts::models::command_card::NP;
use scripts::models::enemy_target::{EnemyTarget, Normal};
use scripts::models::skill::Servant;

fn enemy(action: &AutoSkillAction) -> EnemyTarget {
    match action {
        AutoSkillAction::TargetEnemy(target) => target.enemy,
        other => panic!("not an enemy target: {other:?}"),
    }
}

fn servant_skill(action: &AutoSkillAction) -> Servant {
    match action {
        AutoSkillAction::ServantSkill(skill) => skill.skill,
        other => panic!("not a servant skill: {other:?}"),
    }
}

#[test]
fn every_enemy_code_parses_to_its_target() {
    let expected = [
        ("t1", EnemyTarget::A),
        ("t2", EnemyTarget::B),
        ("t3", EnemyTarget::C),
        ("t4", EnemyTarget::SixTopLeft),
        ("t5", EnemyTarget::SixTopMiddle),
        ("t6", EnemyTarget::SixTopRight),
        ("t7", EnemyTarget::SixBottomLeft),
        ("t8", EnemyTarget::SixBottomMiddle),
        ("t9", EnemyTarget::SixBottomRight),
        ("tR", EnemyTarget::RaidBoss),
        ("tA", EnemyTarget::RaidLeft),
        ("tB", EnemyTarget::RaidMiddle),
        ("tC", EnemyTarget::RaidRight),
        ("tD", EnemyTarget::RaidLowerLeft),
        ("tE", EnemyTarget::RaidLowerMiddle),
        ("tF", EnemyTarget::RaidLowerRight),
        ("t10", EnemyTarget::RaidLeft),
        ("t11", EnemyTarget::RaidMiddle),
        ("t12", EnemyTarget::RaidRight),
    ];
    for (code, target) in expected {
        let parsed = actions(code, 0, 0);
        assert_eq!(parsed.len(), 1, "{code}");
        assert_eq!(enemy(&parsed[0]), target, "{code}");
    }
}

#[test]
fn raid_codes_sit_next_to_other_commands() {
    let a = actions("tCa", 0, 0);
    assert_eq!(enemy(&a[0]), EnemyTarget::RaidRight);

    let a2 = actions("t12a", 0, 0);
    assert_eq!(enemy(&a2[0]), EnemyTarget::RaidRight);
    assert!(matches!(a2[1], AutoSkillAction::ServantSkill(_)));
    assert_eq!(servant_skill(&a[1]), Servant::A1);

    let b = actions("tAb4", 0, 0);
    assert_eq!(enemy(&b[0]), EnemyTarget::RaidLeft);
    assert_eq!(servant_skill(&b[1]), Servant::A2);
    assert!(matches!(b[2], AutoSkillAction::Atk(_)));

    let c = actions("t9c", 0, 0);
    assert_eq!(enemy(&c[0]), EnemyTarget::SixBottomRight);
    assert_eq!(servant_skill(&c[1]), Servant::A3);

    let r = actions("tRa", 0, 0);
    assert_eq!(enemy(&r[0]), EnemyTarget::RaidBoss);
    assert!(matches!(r[1], AutoSkillAction::ServantSkill(_)));
}

#[test]
fn t1_followed_by_an_np_is_still_t1_then_the_np() {
    let parsed = actions("t14", 0, 0);
    assert_eq!(parsed.len(), 2);
    assert_eq!(enemy(&parsed[0]), EnemyTarget::A);
    match &parsed[1] {
        AutoSkillAction::Atk(atk) => assert!(atk.nps.contains(&NP::A)),
        other => panic!("not an attack: {other:?}"),
    }
}

#[test]
fn t13_and_t15_keep_their_old_meaning() {
    assert!(is_parse_error("t13"));
    let parsed = actions("t15", 0, 0);
    assert_eq!(parsed.len(), 2);
    assert_eq!(enemy(&parsed[0]), EnemyTarget::A);
}

#[test]
fn autoskillcode_of_raid_targets_is_a_letter() {
    let codes: Vec<&str> = EnemyTarget::RAID_LIST
        .iter()
        .map(|t| t.auto_skill_code())
        .collect();
    assert_eq!(codes, vec!["R", "A", "B", "C", "D", "E", "F"]);
}

#[test]
fn unknown_enemy_codes_fail_to_parse() {
    for code in ["t0", "tX", "tG", "t"] {
        assert!(is_parse_error(code), "{code}");
    }
}

#[test]
fn the_normal_list_is_still_exactly_the_three_normal_targets() {
    assert_eq!(EnemyTarget::LIST, [Normal::A, Normal::B, Normal::C]);
    assert_eq!(EnemyTarget::SIX_LIST.len(), 6);
    assert_eq!(EnemyTarget::RAID_LIST.len(), 7);
    assert_eq!(EnemyTarget::ALL.len(), 16);
    let codes: std::collections::HashSet<&str> = EnemyTarget::ALL
        .iter()
        .map(|t| t.auto_skill_code())
        .collect();
    assert_eq!(codes.len(), 16);
}

#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use scripts::enums::brave_chain_enum::BraveChainEnum;
use scripts::enums::card_priority_split::CardPrioritySplit;
use scripts::enums::card_type_enum::CardTypeEnum;
use scripts::enums::shuffle_cards_enum::ShuffleCardsEnum;
use scripts::enums::spam_enum::SpamEnum;
use scripts::kotlin::to_int;
use scripts::models::auto_skill_action::ShuffleCondition;
use scripts::models::bond_result::{BondSlotSummary, BondSummary, Gain, LevelUp};
use scripts::models::boost_item::{BoostItem, Enabled};
use scripts::models::card_colour_counts::CardColourCounts;
use scripts::models::field_slot::FieldSlot;
use scripts::models::model_error::ModelError;
use scripts::models::servant_target::ServantTarget;
use scripts::models::team_slot::TeamSlot;

#[test]
fn to_int_matches_kotlin_to_int() {
    let ok = [
        ("\u{663}", 3),
        ("+5", 5),
        ("-0", 0),
        ("\u{663}\u{663}", 33),
        ("\u{ff12}", 2),
        ("\u{966}\u{967}", 1),
        ("\u{ff10}\u{ff11}", 1),
        ("\u{6f1}", 1),
        ("\u{e51}", 1),
        ("\u{d66}\u{d67}", 1),
        ("\u{a9d1}", 1),
        ("\u{17e1}", 1),
        ("\u{1bb1}", 1),
        ("\u{1a81}", 1),
        ("\u{1a91}", 1),
        ("\u{abf1}", 1),
        ("2147483647", 2147483647),
        ("-2147483648", -2147483648),
        ("0000000000000000000000001", 1),
    ];
    for (text, expected) in ok {
        assert_eq!(to_int(text), Ok(expected), "{text:?}");
    }
    for text in [
        "",
        "+",
        "-",
        "1_0",
        "\u{b2}",
        "\u{2460}",
        "\u{bd}",
        "2147483648",
        "-2147483649",
        "+-1",
        "--1",
        "- 1",
        "1-",
        "1 ",
        " 1",
        "x",
    ] {
        assert_eq!(
            to_int(text),
            Err(ModelError::NumberFormat(format!(
                "For input string: \"{text}\""
            ))),
            "{text:?}"
        );
    }
}

#[test]
fn shuffle_condition_from_code_matches_kotlin() {
    let described = |code: &str| {
        ShuffleCondition::from_code(code).map(|c| {
            format!(
                "slot={} count={} type={} str={}",
                c.slot.position(),
                c.count,
                c.r#type.map_or("null".to_string(), |t| t.to_string()),
                c.to_command_string()
            )
        })
    };
    let ok = [
        ("13", "slot=1 count=3 type=null str=13"),
        ("1b3", "slot=1 count=3 type=Buster str=1b3"),
        ("1B3", "slot=1 count=3 type=Buster str=1b3"),
        ("3q5", "slot=3 count=5 type=Quick str=3q5"),
        ("14", "slot=1 count=4 type=null str=14"),
        ("15", "slot=1 count=5 type=null str=15"),
        ("11", "slot=1 count=1 type=null str=11"),
        ("35", "slot=3 count=5 type=null str=35"),
        ("2A4", "slot=2 count=4 type=Arts str=2a4"),
    ];
    for (code, expected) in ok {
        assert_eq!(described(code), Ok(expected.to_string()), "{code:?}");
    }
    for code in [
        "16", "01", "4a2", "1x2", "1bb2", "1b", "1", "", "1b22", "1B", "1b6", "1b0",
    ] {
        assert!(
            matches!(
                ShuffleCondition::from_code(code),
                Err(ModelError::IllegalArgument(_))
            ),
            "{code:?}"
        );
    }
}

#[test]
fn enum_names_follow_kotlin_declaration_order() {
    let names = |entries: &[&str]| entries.iter().map(ToString::to_string).collect::<Vec<_>>();
    let of = |list: Vec<String>| list;
    assert_eq!(
        of(BraveChainEnum::ENTRIES
            .iter()
            .map(|e| e.name().to_string())
            .collect()),
        names(&["None", "WithNP", "Avoid"])
    );
    assert_eq!(
        of(CardPrioritySplit::ENTRIES
            .iter()
            .map(|e| e.name().to_string())
            .collect()),
        names(&["Wave", "Turn"])
    );
    assert_eq!(
        of(CardTypeEnum::ENTRIES
            .iter()
            .map(|e| e.name().to_string())
            .collect()),
        names(&["Buster", "Arts", "Quick", "Unknown"])
    );
    assert_eq!(
        of(ShuffleCardsEnum::ENTRIES
            .iter()
            .map(|e| e.name().to_string())
            .collect()),
        names(&[
            "None",
            "NoEffective",
            "NoNPMatching",
            "FirstTurnStartCondition"
        ])
    );
    assert_eq!(
        of(SpamEnum::ENTRIES
            .iter()
            .map(|e| e.name().to_string())
            .collect()),
        names(&["None", "Spam", "Danger"])
    );
    assert_eq!(
        ShuffleCardsEnum::from_name("NoEffective"),
        Some(ShuffleCardsEnum::NoEffective)
    );
    assert_eq!(ShuffleCardsEnum::from_name("noeffective"), None);
    assert_eq!(
        serde_json::to_string(&ShuffleCardsEnum::FirstTurnStartCondition).unwrap(),
        "\"FirstTurnStartCondition\""
    );
    assert_eq!(
        serde_json::from_str::<BraveChainEnum>("\"WithNP\"").unwrap(),
        BraveChainEnum::WithNP
    );
}

#[test]
fn special_targets_have_prefix_free_codes() {
    let codes: Vec<&str> = ServantTarget::LIST
        .iter()
        .map(|t| t.special_target())
        .filter(|code| !code.is_empty())
        .collect();
    assert_eq!(codes, vec!["Ch2A", "Ch2B", "Ch3A", "Ch3B", "Ch3C"]);
    for (i, a) in codes.iter().enumerate() {
        for (j, b) in codes.iter().enumerate() {
            if i != j {
                assert!(!a.starts_with(b), "{a} starts with {b}");
            }
        }
    }
    let codes_of_list: Vec<char> = ServantTarget::LIST
        .iter()
        .map(|t| t.auto_skill_code())
        .collect();
    assert_eq!(
        codes_of_list,
        vec![
            '1', '2', '3', '7', '8', 'K', 'U', 'M', '[', '[', '[', '[', '['
        ]
    );
}

#[test]
fn small_models_match_kotlin() {
    assert_eq!(BoostItem::of(0), BoostItem::Enabled(Enabled::Skip));
    assert_eq!(BoostItem::of(3), BoostItem::Enabled(Enabled::BoostItem3));
    assert_eq!(BoostItem::of(4), BoostItem::Disabled);
    assert_eq!(BoostItem::of(-1), BoostItem::Disabled);
    assert_eq!(
        CardColourCounts {
            buster: 1,
            arts: 2,
            quick: 3
        }
        .to_string(),
        "B1 A2 Q3"
    );
    assert_eq!(CardColourCounts::default().to_string(), "B0 A0 Q0");
    assert_eq!(TeamSlot::Unknown.to_string(), "[0]");
    assert_eq!(TeamSlot::F.to_string(), "[6]");
    assert_eq!(FieldSlot::C.to_string(), "[3]");
    assert_eq!(
        FieldSlot::B.skills(),
        [
            scripts::models::skill::Servant::B1,
            scripts::models::skill::Servant::B2,
            scripts::models::skill::Servant::B3
        ]
    );
    assert_eq!(
        Gain {
            base: i32::MAX,
            bonus: i32::MAX
        }
        .total(),
        4_294_967_294
    );
    assert_eq!(LevelUp::new(Some(10), Some(0)).at_least_ten, Some(true));
    assert_eq!(LevelUp::new(Some(9), None).at_least_ten, Some(false));
    assert_eq!(LevelUp::new(None, None).at_least_ten, None);
    let summary = BondSummary::new(
        vec![BondSlotSummary {
            position: 1,
            total: 5,
            read_runs: 1,
            missed_runs: 0,
            max_observed: false,
        }],
        2,
    );
    assert_eq!(summary.level_up_missed_runs, 0);
}

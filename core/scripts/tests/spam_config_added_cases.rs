#![allow(clippy::panic, clippy::unwrap_used, clippy::expect_used)]

use scripts::enums::spam_enum::SpamEnum;
use scripts::kotlin::LinkedSet;
use scripts::models::spam_config::{
    NpSpamConfig, ServantSpamConfig, SkillSpamConfig, SkillSpamTarget, SpamConfigPerTeamSlot,
    parse_servant_spam_configs, serialize_servant_spam_configs,
};
use scripts::models::team_slot::TeamSlot;

const DEFAULT_SKILL: &str = r#"{"waves":[1,2,3],"spam":"None","target":"None"}"#;
const DEFAULT_NP: &str = r#"{"waves":[1,2,3],"spam":"None"}"#;

fn default_servant() -> String {
    format!(r#"{{"skills":[{DEFAULT_SKILL},{DEFAULT_SKILL},{DEFAULT_SKILL}],"np":{DEFAULT_NP}}}"#)
}

fn round_trip(text: &str) -> Option<String> {
    parse_servant_spam_configs(text)
        .ok()
        .and_then(|configs| serialize_servant_spam_configs(&configs).ok())
}

#[test]
fn default_configuration_encodes_like_kotlin() {
    let defaults: Vec<ServantSpamConfig> = (0..6).map(|_| ServantSpamConfig::default()).collect();
    let expected = format!("[{}]", vec![default_servant(); 6].join(","));
    assert_eq!(serialize_servant_spam_configs(&defaults).unwrap(), expected);
    assert_eq!(parse_servant_spam_configs(&expected).unwrap(), defaults);
}

#[test]
fn decoding_fills_defaults_ignores_unknown_keys_and_dedupes_waves_like_kotlin() {
    assert_eq!(round_trip("[]"), Some("[]".to_string()));
    assert_eq!(round_trip("[{}]"), Some(format!("[{}]", default_servant())));
    assert_eq!(round_trip(" [ ] "), Some("[]".to_string()));
    assert_eq!(
        round_trip(r#"[{"x":1}]"#),
        Some(format!("[{}]", default_servant()))
    );
    assert_eq!(
        round_trip(r#"[{"spam":"Spam"}]"#),
        Some(format!("[{}]", default_servant()))
    );
    assert_eq!(
        round_trip(r#"[{"NP":{}}]"#),
        Some(format!("[{}]", default_servant()))
    );
    assert_eq!(
        round_trip(r#"[{"np":{"extra":[1,2,{"a":1}]}}]"#),
        Some(format!("[{}]", default_servant()))
    );
    assert_eq!(
        round_trip(r#"[{"skills":[{"waves":[3,1,1,2],"spam":"Spam","target":"Self"}]}]"#),
        Some(format!(
            r#"[{{"skills":[{{"waves":[3,1,2],"spam":"Spam","target":"Self"}}],"np":{DEFAULT_NP}}}]"#
        ))
    );
    assert_eq!(
        round_trip(r#"[{"skills":[{"spam":"Danger"}],"np":{"waves":[]}}]"#),
        Some(r#"[{"skills":[{"waves":[1,2,3],"spam":"Danger","target":"None"}],"np":{"waves":[],"spam":"None"}}]"#.to_string())
    );
    assert_eq!(
        round_trip(r#"[{"np":{"spam":"Spam"}}]"#),
        Some(format!(
            r#"[{{"skills":[{DEFAULT_SKILL},{DEFAULT_SKILL},{DEFAULT_SKILL}],"np":{{"waves":[1,2,3],"spam":"Spam"}}}}]"#
        ))
    );
    assert_eq!(
        round_trip(r#"[{"np":{"waves":[1,1,2]}}]"#),
        Some(format!(
            r#"[{{"skills":[{DEFAULT_SKILL},{DEFAULT_SKILL},{DEFAULT_SKILL}],"np":{{"waves":[1,2],"spam":"None"}}}}]"#
        ))
    );
    assert_eq!(
        round_trip(r#"[{"skills":[{"waves":["1"]}]}]"#),
        Some(format!(
            r#"[{{"skills":[{{"waves":[1],"spam":"None","target":"None"}}],"np":{DEFAULT_NP}}}]"#
        ))
    );
    assert_eq!(
        round_trip(r#"[{"skills":[{"waves":[1,2],"waves":[3]}]}]"#),
        Some(format!(
            r#"[{{"skills":[{{"waves":[3],"spam":"None","target":"None"}}],"np":{DEFAULT_NP}}}]"#
        ))
    );
    assert_eq!(
        round_trip(r#"[{"np":{"spam":"Spam"},"np":{"spam":"Danger"}}]"#),
        Some(format!(
            r#"[{{"skills":[{DEFAULT_SKILL},{DEFAULT_SKILL},{DEFAULT_SKILL}],"np":{{"waves":[1,2,3],"spam":"Danger"}}}}]"#
        ))
    );
    assert_eq!(
        round_trip(r#"[{"np":{"spam":"Spam","spam":"Danger"}}]"#),
        Some(format!(
            r#"[{{"skills":[{DEFAULT_SKILL},{DEFAULT_SKILL},{DEFAULT_SKILL}],"np":{{"waves":[1,2,3],"spam":"Danger"}}}}]"#
        ))
    );
    assert_eq!(
        round_trip(r#"[{"skills":[],"np":{}}]"#),
        Some(format!(r#"[{{"skills":[],"np":{DEFAULT_NP}}}]"#))
    );
    assert_eq!(
        round_trip(r#"[{"skills":[{"target":"Slot3","waves":[-1,0]}]}]"#),
        Some(format!(
            r#"[{{"skills":[{{"waves":[-1,0],"spam":"None","target":"Slot3"}}],"np":{DEFAULT_NP}}}]"#
        ))
    );
    assert_eq!(
        round_trip(
            r#"[{"skills":[{"waves":[1,2,3],"spam":"None","target":"Left"},{"target":"Right"}],"np":{"spam":"Danger","waves":[2]}}]"#
        ),
        Some(r#"[{"skills":[{"waves":[1,2,3],"spam":"None","target":"Left"},{"waves":[1,2,3],"spam":"None","target":"Right"}],"np":{"waves":[2],"spam":"Danger"}}]"#.to_string())
    );
}

#[test]
fn decoding_rejects_what_kotlin_rejects() {
    for text in [
        r#"[{"np":{"spam":"Nope"}}]"#,
        "invalid",
        r#"[{"np":null}]"#,
        r#"[{"skills":[{"waves":[1.0]}]}]"#,
        r#"[{"skills":[{"waves":[1,2147483648]}]}]"#,
        "[{},]",
        r#"[{"np":{}}] x"#,
        r#"{"a":1}"#,
        "null",
        "[{'np':{}}]",
        r#"[{"np":{"spam":Spam}}]"#,
        r#"[{"skills":[{"target":"self"}]}]"#,
        r#"[{"skills":null}]"#,
        r#"[{"skills":[{"waves":[" 1"]}]}]"#,
        r#"[{"skills":[{"waves":["+1"]}]}]"#,
        r#"[{"skills":[{"waves":[+1]}]}]"#,
        r#"[{"skills":[{"waves":[true]}]}]"#,
        r#"[{"skills":[{"waves":[null]}]}]"#,
        r#"[{"skills":[{"spam":null}]}]"#,
        r#"[{"skills":[{"spam":1}]}]"#,
        r#"[{"np":{"waves":"1"}}]"#,
        r#"[{"np":{"waves":{}}}]"#,
        "[null]",
        "[1]",
        "[[]]",
    ] {
        assert!(parse_servant_spam_configs(text).is_err(), "{text}");
    }
}

#[test]
fn wave_sets_compare_without_order_but_keep_their_insertion_order_when_encoded() {
    let a: LinkedSet<i32> = [3, 1, 2].into_iter().collect();
    let b: LinkedSet<i32> = [1, 2, 3].into_iter().collect();
    assert_eq!(a, b);
    let config = SkillSpamConfig {
        waves: a,
        spam: SpamEnum::Spam,
        target: SkillSpamTarget::SelfTarget,
    };
    assert_eq!(
        serde_json::to_string(&config).unwrap(),
        r#"{"waves":[3,1,2],"spam":"Spam","target":"Self"}"#
    );
    assert_eq!(SkillSpamTarget::SelfTarget.name(), "Self");
}

#[test]
fn per_team_slot_lookup_falls_back_to_the_default_like_kotlin_get_or_else() {
    let custom = ServantSpamConfig {
        skills: Vec::new(),
        np: NpSpamConfig {
            waves: [2].into_iter().collect(),
            spam: SpamEnum::Danger,
        },
    };
    let per_slot = SpamConfigPerTeamSlot::new(vec![custom.clone()]);
    assert_eq!(per_slot.get(TeamSlot::A), custom);
    assert_eq!(per_slot.get(TeamSlot::B), ServantSpamConfig::default());
    assert_eq!(
        per_slot.get(TeamSlot::Unknown),
        ServantSpamConfig::default()
    );
}

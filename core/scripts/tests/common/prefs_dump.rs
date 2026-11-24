#![allow(dead_code)]

use scripts::models::model_error::ModelError;
use scripts::models::spam_config::{NpSpamConfig, ServantSpamConfig, SkillSpamConfig};
use scripts::prefs::battle_config::BattleConfig;
use scripts::prefs::execution_profile::ExecutionProfile;
use scripts::prefs::support_preferences::SupportPreferences;
use serde_json::{Map, Value, json};

const BATTLE_EXIT_EXCEPTION: &str =
    "io.github.fate_grand_automata.scripts.entrypoints.AutoBattle$BattleExitException";

fn exception_name(error: &ModelError) -> &'static str {
    match error {
        ModelError::CardPriorityParse(_) => BATTLE_EXIT_EXCEPTION,
        ModelError::SkillCommandParse(_) => BATTLE_EXIT_EXCEPTION,
        ModelError::IllegalArgument(_) => "java.lang.IllegalArgumentException",
        ModelError::IllegalState(_) => "java.lang.IllegalStateException",
        ModelError::NumberFormat(_) => "java.lang.NumberFormatException",
        ModelError::IndexOutOfBounds(_) => "java.lang.IndexOutOfBoundsException",
        ModelError::NoSuchElement(_) => "java.util.NoSuchElementException",
    }
}

fn sorted_set(mut values: Vec<Value>) -> Value {
    values.sort_by_key(Value::to_string);
    Value::Array(values)
}

fn skill_spam(skill: &SkillSpamConfig) -> Value {
    json!({
        "spam": skill.spam.name(),
        "target": skill.target.name(),
        "waves": sorted_set(skill.waves.iter().map(|w| json!(w)).collect()),
    })
}

fn np_spam(np: &NpSpamConfig) -> Value {
    json!({
        "spam": np.spam.name(),
        "waves": sorted_set(np.waves.iter().map(|w| json!(w)).collect()),
    })
}

fn servant_spam(servant: &ServantSpamConfig) -> Value {
    json!({
        "np": np_spam(&servant.np),
        "skills": servant.skills.iter().map(skill_spam).collect::<Vec<_>>(),
    })
}

fn execution(profile: &ExecutionProfile, into: &mut Map<String, Value>) {
    into.insert("skillCommand".into(), json!(profile.skill_command));
    into.insert(
        "cardPriority".into(),
        match &profile.card_priority {
            Ok(per_wave) => Value::Array(
                per_wave
                    .iter()
                    .map(|priority| {
                        Value::Array(
                            priority
                                .iter()
                                .map(|score| {
                                    json!({
                                        "affinity": score.affinity.name(),
                                        "type": score.r#type.name(),
                                    })
                                })
                                .collect(),
                        )
                    })
                    .collect(),
            ),
            Err(error) => json!({ "error": exception_name(error) }),
        },
    );
    into.insert(
        "cardPrioritySplit".into(),
        json!(profile.card_priority_split.name()),
    );
    into.insert(
        "cardPriorityStyle".into(),
        json!(profile.card_priority_style.name()),
    );
    into.insert(
        "cardPriorityStyles".into(),
        Value::Array(
            profile
                .card_priority_styles
                .iter()
                .map(|style| json!(style.name()))
                .collect(),
        ),
    );
    into.insert(
        "servantCardPriority".into(),
        Value::Array(
            profile
                .servant_card_priority
                .iter()
                .map(|entry| {
                    Value::Array(
                        entry
                            .iter()
                            .map(|key| {
                                json!({
                                    "fieldPosition": key.field_position,
                                    "type": key.r#type.name(),
                                })
                            })
                            .collect(),
                    )
                })
                .collect(),
        ),
    );
    into.insert(
        "useServantPriority".into(),
        json!(profile.use_servant_priority),
    );
    into.insert(
        "useServantPriorities".into(),
        json!(profile.use_servant_priorities),
    );
    into.insert(
        "servantPriority".into(),
        Value::Array(
            profile
                .servant_priority
                .iter()
                .map(|wave| {
                    Value::Array(
                        wave.iter()
                            .map(|slot| json!({ "position": slot.position() }))
                            .collect(),
                    )
                })
                .collect(),
        ),
    );
    into.insert("rearrangeCards".into(), json!(profile.rearrange_cards));
    into.insert(
        "braveChains".into(),
        Value::Array(
            profile
                .brave_chains
                .iter()
                .map(|chain| json!(chain.name()))
                .collect(),
        ),
    );
}

pub fn execution_object(profile: &ExecutionProfile) -> Value {
    let mut map = Map::new();
    execution(profile, &mut map);
    Value::Object(map)
}

fn support(support: &SupportPreferences) -> Value {
    json!({
        "friendNames": support.friend_names,
        "preferredServants": support.preferred_servants,
        "mlb": support.mlb,
        "preferredCEs": support.preferred_ces,
        "preferredNoblePhantasms": support.preferred_noble_phantasms,
        "friendsOnly": support.friends_only,
        "autoFriendRequest": support.auto_friend_request,
        "selectionMode": support.selection_mode.name(),
        "fallbackTo": support.fallback_to.name(),
        "supportClass": support.support_class.name(),
        "alsoCheckAll": support.also_check_all,
        "maxAscended": support.max_ascended,
        "skill1Max": support.skill1_max,
        "skill2Max": support.skill2_max,
        "skill3Max": support.skill3_max,
        "appendSkillCheck": support.append_skill_check,
        "appendSkillsMax": support.append_skills_max,
        "score62Check": support.score62_check,
        "grandScore16Check": support.grand_score16_check,
        "npLevelCheck": support.np_level_check,
        "minimumNpLevel": support.minimum_np_level,
        "grandServant": support.grand_servant,
        "bondCEEffect": support.bond_ce_effect.name(),
        "requireBothNormalAndRewardMatch": support.require_both_normal_and_reward_match,
    })
}

pub fn dump(config: &BattleConfig) -> Value {
    let mut map = Map::new();
    execution(&config.execution, &mut map);
    let mut put = |key: &str, value: Value| {
        map.insert(key.to_string(), value);
    };
    put("id", json!(config.id));
    put("name", json!(config.name));
    put("commandStartMode", json!(config.command_start_mode.name()));
    put("commandStartServant", json!(config.command_start_servant));
    put(
        "commandStartCardCount",
        json!(config.command_start_card_count),
    );
    put("commandStartBuster", json!(config.command_start_buster));
    put("commandStartArts", json!(config.command_start_arts));
    put("commandStartQuick", json!(config.command_start_quick));
    put(
        "commandStartCombineMode",
        json!(config.command_start_combine_mode.name()),
    );
    put(
        "commandStartMaxTurns",
        json!(config.command_start_max_turns),
    );
    put(
        "secondaryStartEnabled",
        json!(config.secondary_start_enabled),
    );
    put(
        "secondaryStartServant",
        json!(config.secondary_start_servant),
    );
    put(
        "secondaryStartCardCount",
        json!(config.secondary_start_card_count),
    );
    put("secondaryStartBuster", json!(config.secondary_start_buster));
    put("secondaryStartArts", json!(config.secondary_start_arts));
    put("secondaryStartQuick", json!(config.secondary_start_quick));
    put(
        "secondaryStartCombineMode",
        json!(config.secondary_start_combine_mode.name()),
    );
    put(
        "secondaryExecution",
        execution_object(&config.secondary_execution),
    );
    put("tertiaryStartEnabled", json!(config.tertiary_start_enabled));
    put("tertiaryStartServant", json!(config.tertiary_start_servant));
    put(
        "tertiaryStartCardCount",
        json!(config.tertiary_start_card_count),
    );
    put("tertiaryStartBuster", json!(config.tertiary_start_buster));
    put("tertiaryStartArts", json!(config.tertiary_start_arts));
    put("tertiaryStartQuick", json!(config.tertiary_start_quick));
    put(
        "tertiaryStartCombineMode",
        json!(config.tertiary_start_combine_mode.name()),
    );
    put(
        "tertiaryExecution",
        execution_object(&config.tertiary_execution),
    );
    put(
        "spam",
        Value::Array(config.spam.iter().map(servant_spam).collect()),
    );
    put("trackBond", json!(config.track_bond));
    put("stopOnBondMax", json!(config.stop_on_bond_max));
    put(
        "bondMaxIgnoredCharacters",
        sorted_set(
            config
                .bond_max_ignored_characters
                .iter()
                .map(|n| json!(n))
                .collect(),
        ),
    );
    put("shuffleCards", json!(config.shuffle_cards.name()));
    put("shuffleCardsWave", json!(config.shuffle_cards_wave));
    put("cardSwapDetection", json!(config.card_swap_detection));
    put("party", json!(config.party));
    put(
        "materials",
        sorted_set(
            config
                .materials
                .iter()
                .map(|material| json!(material.name()))
                .collect(),
        ),
    );
    put("support", support(&config.support));
    put("autoChooseTarget", json!(config.auto_choose_target));
    put(
        "server",
        match config.server {
            None => Value::Null,
            Some(server) => json!({
                "betterFgo": server.better_fgo(),
                "simple": server.simple(),
            }),
        },
    );
    put("addRaidTurnDelay", json!(config.add_raid_turn_delay));
    put(
        "raidTurnDelaySeconds",
        json!(config.raid_turn_delay_seconds),
    );
    put(
        "turnLimitRetreatEnabled",
        json!(config.turn_limit_retreat_enabled),
    );
    put(
        "turnLimitRetreatTurns",
        json!(config.turn_limit_retreat_turns),
    );
    put(
        "turnLimitRetreatStopAfterStreak",
        json!(config.turn_limit_retreat_stop_after_streak),
    );
    put(
        "turnLimitRetreatStreakCount",
        json!(config.turn_limit_retreat_streak_count),
    );
    Value::Object(map)
}

pub fn key_count(value: &Value) -> usize {
    match value {
        Value::Object(map) => map.values().map(|v| 1 + key_count(v)).sum(),
        Value::Array(items) => items.iter().map(key_count).sum(),
        _ => 0,
    }
}

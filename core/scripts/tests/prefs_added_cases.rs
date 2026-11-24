#[path = "common/prefs_dump.rs"]
mod prefs_dump;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use scripts::enums::bond_ce_effect_enum::BondCEEffectEnum;
use scripts::enums::command_start_combine_mode::CommandStartCombineMode;
use scripts::enums::command_start_mode::CommandStartMode;
use scripts::enums::material_enum::MaterialEnum;
use scripts::enums::refill_resource_enum::RefillResourceEnum;
use scripts::enums::script_mode_enum::ScriptModeEnum;
use scripts::enums::support_class::SupportClass;
use scripts::enums::support_selection_mode_enum::SupportSelectionModeEnum;
use scripts::models::model_error::ModelError;
use scripts::prefs::battle_config::BattleConfig;
use scripts::prefs::battle_config_file::BattleConfigFile;
use scripts::prefs::per_server_config_prefs::PerServerConfigPrefs;
use scripts::prefs::pref_maker::PrefStore;
use scripts::prefs::prefs_error::PrefsError;
use serde_json::{Map, Value};

const STRING_SET_PATHS: [&str; 4] = [
    "friendNames",
    "preferredServants",
    "preferredCEs",
    "preferredNoblePhantasms",
];

fn cases_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata/prefs_cases")
}

fn testdata() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata")
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))
}

fn read_json(path: &Path) -> Result<Value, String> {
    serde_json::from_str(&read(path)?).map_err(|e| format!("{}: {e}", path.display()))
}

struct Case {
    name: String,
    input: String,
    expected: Value,
}

fn cases() -> Result<Vec<Case>, String> {
    let dir = cases_dir();
    let entries = fs::read_dir(&dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    let mut cases = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|e| format!("cannot list {}: {e}", dir.display()))?
            .path();
        let Some(name) = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_suffix(".fga.json"))
        else {
            continue;
        };
        let expected = read_json(&dir.join(format!("{name}.expected.json")))?;
        cases.push(Case {
            name: name.to_string(),
            input: read(&path)?,
            expected,
        });
    }
    cases.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(cases)
}

fn merge(base: &Value, overlay: &Value) -> Value {
    match (base, overlay) {
        (Value::Object(b), Value::Object(o)) => {
            let mut merged: Map<String, Value> = b.clone();
            for (key, value) in o {
                let next = match b.get(key) {
                    Some(existing) => merge(existing, value),
                    None => value.clone(),
                };
                merged.insert(key.clone(), next);
            }
            Value::Object(merged)
        }
        (_, replacement) => replacement.clone(),
    }
}

fn sort_string_sets(value: &mut Value) {
    if let Some(support) = value.get_mut("support").and_then(Value::as_object_mut) {
        for path in STRING_SET_PATHS {
            if let Some(Value::Array(items)) = support.get_mut(path) {
                items.sort_by_key(Value::to_string);
            }
        }
    }
}

fn empty_dump() -> Result<Value, String> {
    let mut empty = read_json(&testdata().join("golden/prefs/empty.json"))?;
    if let Some(map) = empty.as_object_mut() {
        map.insert("folder".to_string(), Value::from(""));
        map.insert("notes".to_string(), Value::from(""));
    }
    Ok(empty)
}

fn class_matches(error: &PrefsError, class: &str) -> bool {
    match error {
        PrefsError::Serialization(_) => class.ends_with("JsonDecodingException"),
        PrefsError::IllegalArgument(_) => class == "java.lang.IllegalArgumentException",
    }
}

fn expected_config(case: &Case, which: &str, empty: &Value) -> Option<Value> {
    let overlay = case.expected.get(which)?.get("config")?;
    let mut merged = merge(empty, overlay);
    sort_string_sets(&mut merged);
    Some(merged)
}

fn rust_config(store: &PrefStore) -> Value {
    let config = BattleConfig::from_store("host", store);
    let mut dump = prefs_dump::dump(&config);
    if let Some(map) = dump.as_object_mut() {
        map.insert("folder".to_string(), Value::from(config.folder.clone()));
        map.insert("notes".to_string(), Value::from(config.notes.clone()));
    }
    sort_string_sets(&mut dump);
    dump
}

const PARSER_DIFFERENCES: [&str; 6] = [
    "number_exponent",
    "number_leading_zero",
    "number_leading_zero_unknown_key",
    "number_negative_zero",
    "number_plus_sign",
    "number_trailing_dot",
];

#[test]
fn every_case_reads_like_the_android_reader() -> Result<(), String> {
    let empty = empty_dump()?;
    let cases = cases()?;
    let mut compared = 0;
    let mut errors = 0;
    for case in &cases {
        if PARSER_DIFFERENCES.contains(&case.name.as_str()) {
            continue;
        }
        let android = case
            .expected
            .get("android")
            .ok_or_else(|| format!("{}: no android expectation", case.name))?;
        match (
            android.get("decodeError"),
            BattleConfigFile::decode(&case.input),
        ) {
            (Some(error), Err(actual)) => {
                let class = error.get("class").and_then(Value::as_str).unwrap_or("");
                assert!(
                    class_matches(&actual, class),
                    "{}: kotlin {class}, rust {actual:?}",
                    case.name
                );
                if class == "java.lang.IllegalArgumentException" {
                    let message = error.get("message").and_then(Value::as_str).unwrap_or("");
                    assert_eq!(actual.to_string(), message, "{}", case.name);
                }
                errors += 1;
            }
            (Some(error), Ok(_)) => {
                return Err(format!(
                    "{}: kotlin failed with {error}, rust read it",
                    case.name
                ));
            }
            (None, Err(actual)) => {
                return Err(format!(
                    "{}: kotlin read it, rust failed with {actual}",
                    case.name
                ));
            }
            (None, Ok(store)) => {
                let expected = expected_config(case, "android", &empty)
                    .ok_or_else(|| format!("{}: no config expectation", case.name))?;
                assert_eq!(rust_config(&store), expected, "{}", case.name);
                compared += 1;
            }
        }
    }
    println!(
        "{} cases: {compared} configs and {errors} decode errors compared, {} parser differences skipped",
        cases.len(),
        PARSER_DIFFERENCES.len()
    );
    assert!(compared >= 40, "{compared}");
    assert!(errors >= 25, "{errors}");
    Ok(())
}

#[test]
fn the_spellings_the_android_parser_accepts_but_serde_json_rejects_are_the_known_gap()
-> Result<(), String> {
    let by_name: BTreeMap<String, Case> =
        cases()?.into_iter().map(|c| (c.name.clone(), c)).collect();
    let mut kotlin_accepts_rust_rejects = Vec::new();
    for name in [
        "number_exponent",
        "number_leading_zero",
        "number_leading_zero_unknown_key",
        "number_negative_zero",
    ] {
        let case = by_name
            .get(name)
            .ok_or_else(|| format!("missing case {name}"))?;
        assert!(
            case.expected["android"].get("config").is_some(),
            "{name}: kotlin should have read it"
        );
        assert!(
            BattleConfigFile::decode(&case.input).is_err(),
            "{name}: rust unexpectedly read it"
        );
        kotlin_accepts_rust_rejects.push(name);
    }
    assert_eq!(kotlin_accepts_rust_rejects.len(), 4);
    for name in ["number_plus_sign", "number_trailing_dot"] {
        let case = by_name
            .get(name)
            .ok_or_else(|| format!("missing case {name}"))?;
        assert!(
            case.expected["android"].get("decodeError").is_some(),
            "{name}"
        );
        assert!(BattleConfigFile::decode(&case.input).is_err(), "{name}");
    }
    Ok(())
}

#[test]
fn typed_copies_of_the_mistyped_golden_inputs_read_to_the_golden_values() -> Result<(), String> {
    let by_name: BTreeMap<String, Case> =
        cases()?.into_iter().map(|c| (c.name.clone(), c)).collect();
    for (typed, golden) in [("skills_typed", "skills"), ("strings_typed", "strings")] {
        let case = by_name
            .get(typed)
            .ok_or_else(|| format!("missing case {typed}"))?;
        let store = BattleConfigFile::decode(&case.input).map_err(|e| format!("{typed}: {e}"))?;
        let expected = read_json(&testdata().join(format!("golden/prefs/{golden}.json")))?;
        assert_eq!(
            prefs_dump::dump(&BattleConfig::from_store("host", &store)),
            expected,
            "{typed}"
        );
    }
    Ok(())
}

#[test]
fn declared_key_types_equal_the_android_schema() -> Result<(), String> {
    let expected = read_json(&cases_dir().join("schema.expected.json"))?;
    let expected: BTreeMap<String, String> = expected
        .as_object()
        .ok_or("schema is not an object")?
        .iter()
        .map(|(k, v)| (k.clone(), v.as_str().unwrap_or_default().to_string()))
        .collect();
    let actual: BTreeMap<String, String> = BattleConfig::pref_types()
        .iter()
        .map(|(k, v)| (k.clone(), format!("{v:?}")))
        .collect();
    assert_eq!(actual, expected);
    println!("{} declared keys", actual.len());
    Ok(())
}

#[test]
fn where_the_host_reader_reads_a_string_number_differently_rust_follows_android()
-> Result<(), String> {
    let by_name: BTreeMap<String, Case> =
        cases()?.into_iter().map(|c| (c.name.clone(), c)).collect();
    let read_party_and_raid = |name: &str| -> Result<(i64, i64), String> {
        let case = by_name
            .get(name)
            .ok_or_else(|| format!("missing case {name}"))?;
        let config = BattleConfig::from_store(
            "host",
            &BattleConfigFile::decode(&case.input).map_err(|e| e.to_string())?,
        );
        Ok((
            i64::from(config.party),
            i64::from(config.raid_turn_delay_seconds),
        ))
    };
    let host_value = |name: &str, key: &str, default: i64| -> Result<i64, String> {
        let case = by_name
            .get(name)
            .ok_or_else(|| format!("missing case {name}"))?;
        Ok(case.expected["host"]["config"]
            .get(key)
            .and_then(Value::as_i64)
            .unwrap_or(default))
    };

    assert_eq!(read_party_and_raid("string_as_int_malformed")?, (-1, 3));
    assert_eq!(host_value("string_as_int_malformed", "party", -1)?, 5);

    assert_eq!(read_party_and_raid("string_as_int_edges")?, (-1, 3));
    assert_eq!(host_value("string_as_int_edges", "party", -1)?, 3);
    assert_eq!(
        host_value("string_as_int_edges", "raidTurnDelaySeconds", 3)?,
        2_147_483_647
    );
    Ok(())
}

fn entry_names<T: Copy>(entries: &[T], name: fn(T) -> &'static str) -> Vec<String> {
    entries
        .iter()
        .map(|entry| name(*entry).to_string())
        .collect()
}

#[test]
fn enum_entries_keep_the_kotlin_declaration_order() -> Result<(), String> {
    let expected = read_json(&cases_dir().join("enum_entries.expected.json"))?;
    let actual: BTreeMap<&str, Vec<String>> = BTreeMap::from([
        (
            "BondCEEffectEnum",
            entry_names(BondCEEffectEnum::ENTRIES, BondCEEffectEnum::name),
        ),
        (
            "CommandStartCombineMode",
            entry_names(
                CommandStartCombineMode::ENTRIES,
                CommandStartCombineMode::name,
            ),
        ),
        (
            "CommandStartMode",
            entry_names(CommandStartMode::ENTRIES, CommandStartMode::name),
        ),
        (
            "MaterialEnum",
            entry_names(MaterialEnum::ENTRIES, MaterialEnum::name),
        ),
        (
            "RefillResourceEnum",
            entry_names(RefillResourceEnum::ENTRIES, RefillResourceEnum::name),
        ),
        (
            "ScriptModeEnum",
            entry_names(ScriptModeEnum::ENTRIES, ScriptModeEnum::name),
        ),
        (
            "SupportClass",
            entry_names(SupportClass::ENTRIES, SupportClass::name),
        ),
        (
            "SupportSelectionModeEnum",
            entry_names(
                SupportSelectionModeEnum::ENTRIES,
                SupportSelectionModeEnum::name,
            ),
        ),
    ]);
    let expected_map = expected.as_object().ok_or("not an object")?;
    assert_eq!(expected_map.len(), actual.len());
    for (name, entries) in &actual {
        let kotlin: Vec<String> = expected_map[*name]
            .as_array()
            .ok_or("not an array")?
            .iter()
            .map(|v| v.as_str().unwrap_or_default().to_string())
            .collect();
        assert_eq!(&kotlin, entries, "{name}");
    }
    Ok(())
}

#[test]
fn execution_profiles_are_picked_by_index_like_kotlin() -> Result<(), String> {
    let mut checked = 0;
    for case in cases()? {
        let Some(profiles) = case.expected["android"].get("profiles") else {
            continue;
        };
        let store = BattleConfigFile::decode(&case.input).map_err(|e| e.to_string())?;
        let config = BattleConfig::from_store("host", &store);
        let profiles = profiles.as_array().ok_or("profiles is not an array")?;
        assert_eq!(profiles.len(), 3);
        for (index, expected) in profiles.iter().enumerate() {
            let profile = config
                .execution_profile(i32::try_from(index).map_err(|e| e.to_string())?)
                .map_err(|e| e.to_string())?;
            assert_eq!(
                &prefs_dump::execution_object(profile),
                expected,
                "{} #{index}",
                case.name
            );
        }
        let invalid = &case.expected["android"]["invalidProfile"];
        assert_eq!(invalid["class"], "java.lang.IllegalStateException");
        assert_eq!(
            config.execution_profile(3),
            Err(ModelError::IllegalState(
                invalid["message"].as_str().unwrap_or_default().to_string()
            ))
        );
        assert!(config.execution_profile(-1).is_err());
        checked += 1;
    }
    assert_eq!(checked, 2);
    Ok(())
}

#[test]
fn refill_shape_matches_the_android_per_server_prefs() -> Result<(), String> {
    let expected = read_json(&cases_dir().join("per_server.expected.json"))?;
    let defaults = PerServerConfigPrefs::default();
    let rendered = serde_json::json!({
        "blueApple": defaults.blue_apple,
        "copperApple": defaults.copper_apple,
        "goldApple": defaults.gold_apple,
        "limitCEs": defaults.limit_ces,
        "limitMats": defaults.limit_mats,
        "limitRuns": defaults.limit_runs,
        "rainbowApple": defaults.rainbow_apple,
        "resources": defaults.resources.iter().map(|r| r.name()).collect::<Vec<_>>(),
        "selectedApple": defaults.selected_apple.name(),
        "shouldLimitCEs": defaults.should_limit_ces,
        "shouldLimitMats": defaults.should_limit_mats,
        "shouldLimitRuns": defaults.should_limit_runs,
        "silverApple": defaults.silver_apple,
        "useStormPod": defaults.use_storm_pod,
        "useTeapot": defaults.use_teapot,
        "waitForAPRegen": defaults.wait_for_ap_regen,
    });
    let mut kotlin = expected["defaults"].clone();
    if let Some(map) = kotlin.as_object_mut() {
        for key in ["currentAppleCount", "selectedAutoSkillKey", "server"] {
            map.remove(key);
        }
    }
    assert_eq!(rendered, kotlin);
    assert_eq!(
        i64::from(defaults.current_apple_count()),
        expected["defaults"]["currentAppleCount"]
            .as_i64()
            .ok_or("no default count")?
    );

    let stored = &expected["storedCounts"];
    let count = |key: &str| -> Result<i32, String> {
        stored[key]
            .as_str()
            .and_then(|text| text.parse().ok())
            .ok_or_else(|| format!("bad stored count {key}"))
    };
    let configured = PerServerConfigPrefs {
        rainbow_apple: count("rainbowApple")?,
        gold_apple: count("goldApple")?,
        silver_apple: count("silverApple")?,
        blue_apple: count("blueApple")?,
        copper_apple: count("copperApple")?,
        ..PerServerConfigPrefs::default()
    };
    let by_selection = expected["currentAppleCount"]
        .as_object()
        .ok_or("currentAppleCount is not an object")?;
    assert_eq!(by_selection.len(), RefillResourceEnum::ENTRIES.len());
    for selected in RefillResourceEnum::ENTRIES {
        let prefs = PerServerConfigPrefs {
            selected_apple: *selected,
            ..configured.clone()
        };
        assert_eq!(
            Some(i64::from(prefs.current_apple_count())),
            by_selection[selected.name()].as_i64(),
            "{}",
            selected.name()
        );
    }
    Ok(())
}

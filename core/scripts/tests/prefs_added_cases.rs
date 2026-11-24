#[path = "common/prefs_dump.rs"]
mod prefs_dump;

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use scripts::prefs::battle_config::BattleConfig;
use scripts::prefs::battle_config_file::BattleConfigFile;
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
    read_json(&testdata().join("golden/prefs/empty.json"))
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
    let mut dump = prefs_dump::dump(&BattleConfig::from_store("host", store));
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
fn lenient_reader_matches_the_host_reader_on_mistyped_values() -> Result<(), String> {
    let empty = empty_dump()?;
    let mut compared = 0;
    for case in cases()? {
        if !case.name.starts_with("lenient_") {
            continue;
        }
        assert!(
            BattleConfigFile::decode(&case.input).is_err(),
            "{}: the android reader must refuse this file",
            case.name
        );
        let expected = expected_config(&case, "host", &empty)
            .ok_or_else(|| format!("{}: no host expectation", case.name))?;
        let store = BattleConfigFile::decode_lenient(&case.input).map_err(|e| e.to_string())?;
        assert_eq!(rust_config(&store), expected, "{}", case.name);
        compared += 1;
    }
    assert_eq!(compared, 4);
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

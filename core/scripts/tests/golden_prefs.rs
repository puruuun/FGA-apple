#[path = "common/prefs_dump.rs"]
mod prefs_dump;

use std::fs;
use std::path::{Path, PathBuf};

use scripts::prefs::battle_config::BattleConfig;
use scripts::prefs::battle_config_file::BattleConfigFile;
use scripts::prefs::pref_maker::PrefType;
use serde_json::Value;

fn testdata() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../testdata")
}

fn read(path: &Path) -> Result<String, String> {
    fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))
}

fn inputs() -> Result<Vec<PathBuf>, String> {
    let dir = testdata().join("fga");
    let entries = fs::read_dir(&dir).map_err(|e| format!("cannot read {}: {e}", dir.display()))?;
    let mut files = Vec::new();
    for entry in entries {
        let path = entry
            .map_err(|e| format!("cannot list {}: {e}", dir.display()))?
            .path();
        if path
            .file_name()
            .and_then(|n| n.to_str())
            .is_some_and(|n| n.ends_with(".fga.json"))
        {
            files.push(path);
        }
    }
    files.sort();
    Ok(files)
}

fn retype(text: &str) -> Result<String, String> {
    let mut root: Value = serde_json::from_str(text).map_err(|e| e.to_string())?;
    let object = root.as_object_mut().ok_or("not an object")?;
    for (key, value) in object.iter_mut() {
        let Some(stored) = BattleConfig::pref_types().get(key.as_str()) else {
            continue;
        };
        let replacement = match (stored, &*value) {
            (PrefType::String, Value::Number(number)) => Some(Value::String(number.to_string())),
            (PrefType::String, Value::Bool(flag)) => Some(Value::String(flag.to_string())),
            (PrefType::Boolean, Value::String(text)) => match text.as_str() {
                "true" => Some(Value::Bool(true)),
                "false" => Some(Value::Bool(false)),
                _ => None,
            },
            (PrefType::Int, Value::String(text)) => {
                text.trim().parse::<i32>().ok().map(Value::from)
            }
            _ => None,
        };
        if let Some(replacement) = replacement {
            *value = replacement;
        }
    }
    serde_json::to_string(&root).map_err(|e| e.to_string())
}

fn first_difference(path: &str, expected: &Value, actual: &Value) -> Option<String> {
    match (expected, actual) {
        (Value::Object(e), Value::Object(a)) => {
            for (key, value) in e {
                let child = format!("{path}.{key}");
                match a.get(key) {
                    None => return Some(format!("{child}: missing in the Rust output")),
                    Some(other) => {
                        if let Some(found) = first_difference(&child, value, other) {
                            return Some(found);
                        }
                    }
                }
            }
            a.keys()
                .find(|key| !e.contains_key(*key))
                .map(|key| format!("{path}.{key}: not in the golden file"))
        }
        (Value::Array(e), Value::Array(a)) => {
            if e.len() != a.len() {
                return Some(format!("{path}: length {} != {}", e.len(), a.len()));
            }
            e.iter()
                .zip(a)
                .enumerate()
                .find_map(|(i, (x, y))| first_difference(&format!("{path}[{i}]"), x, y))
        }
        _ if expected == actual => None,
        _ => Some(format!("{path}: golden {expected} != rust {actual}")),
    }
}

#[test]
fn every_golden_key_of_every_input_matches_in_both_readers() -> Result<(), String> {
    let files = inputs()?;
    assert_eq!(files.len(), 6, "{files:?}");
    let mut compared_files = 0;
    let mut total_keys = 0;
    for path in &files {
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .and_then(|n| n.strip_suffix(".fga.json"))
            .ok_or_else(|| format!("unexpected file name {}", path.display()))?;
        let text = read(path)?;
        let golden: Value = serde_json::from_str(&read(
            &testdata().join("golden/prefs").join(format!("{name}.json")),
        )?)
        .map_err(|e| format!("{name}: golden is not JSON: {e}"))?;

        let store =
            BattleConfigFile::decode(&retype(&text)?).map_err(|e| format!("{name}: {e}"))?;
        let actual = prefs_dump::dump(&BattleConfig::from_store("host", &store));
        if let Some(difference) = first_difference(name, &golden, &actual) {
            return Err(difference);
        }
        let keys = prefs_dump::key_count(&golden);
        assert_eq!(keys, prefs_dump::key_count(&actual), "{name}");
        println!("{name}: {keys} golden keys compared");
        total_keys += keys;
        compared_files += 1;
    }
    assert_eq!(compared_files, 6);
    println!("total: {total_keys} golden keys in {compared_files} files");
    Ok(())
}

#[test]
fn the_files_the_android_app_would_refuse_are_the_two_with_mistyped_values() -> Result<(), String> {
    let mut refused = Vec::new();
    for path in inputs()? {
        let text = read(&path)?;
        if BattleConfigFile::decode(&text).is_err() {
            refused.push(
                path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or_default()
                    .to_string(),
            );
        }
    }
    assert_eq!(refused, ["skills.fga.json", "strings.fga.json"]);
    Ok(())
}

use serde_json::{Map, Number, Value};

use super::battle_config::BattleConfig;
use super::pref_maker::{PrefStore, PrefType, PrefValue};
use super::prefs_error::PrefsError;
use crate::kotlin::{to_int, trim};

enum Decoded {
    Text(String),
    Int(i32),
    Long(i64),
    Double,
    Flag(bool),
    List(Vec<String>),
}

impl Decoded {
    fn type_name(&self) -> &'static str {
        match self {
            Decoded::Text(_) => "String",
            Decoded::Int(_) => "Int",
            Decoded::Long(_) => "Long",
            Decoded::Double => "Double",
            Decoded::Flag(_) => "Boolean",
            Decoded::List(_) => "ArrayList",
        }
    }
}

pub struct BattleConfigFile;

impl BattleConfigFile {
    pub fn decode(text: &str) -> Result<PrefStore, PrefsError> {
        let root = parse_object(text)?;
        let schema = BattleConfig::pref_types();
        let mut decoded: Vec<(&str, Decoded)> = Vec::new();

        for (key, element) in &root {
            if schema.get(key.as_str()) == Some(&PrefType::StringSet)
                && let Value::Array(items) = element
                && !items.iter().all(Value::is_string)
            {
                return Err(PrefsError::IllegalArgument(format!(
                    "Expected array of strings for '{key}'"
                )));
            }
            if let Some(value) = to_pref_value(element) {
                decoded.push((key, value));
            }
        }

        let mut store = PrefStore::new();
        for (key, value) in decoded {
            if let Some(expected) = schema.get(key) {
                store.insert(key.to_string(), normalize_for_type(key, value, *expected)?);
            }
        }
        Ok(store)
    }

    pub fn decode_lenient(text: &str) -> Result<PrefStore, PrefsError> {
        let root = parse_object(text)?;
        let mut store = PrefStore::new();
        for (key, expected) in BattleConfig::pref_types() {
            if let Some(value) = root.get(key).and_then(|element| coerce(element, *expected)) {
                store.insert(key.clone(), value);
            }
        }
        Ok(store)
    }
}

fn parse_object(text: &str) -> Result<Map<String, Value>, PrefsError> {
    match serde_json::from_str::<Value>(text) {
        Ok(Value::Object(map)) => Ok(map),
        Ok(other) => Err(PrefsError::IllegalArgument(format!(
            "Element class kotlinx.serialization.json.{} is not a JsonObject",
            element_kind(&other)
        ))),
        Err(error) => Err(PrefsError::Serialization(error.to_string())),
    }
}

fn element_kind(value: &Value) -> &'static str {
    match value {
        Value::Array(_) => "JsonArray",
        Value::Null => "JsonNull",
        _ => "JsonLiteral",
    }
}

fn to_pref_value(element: &Value) -> Option<Decoded> {
    match element {
        Value::Array(items) => Some(Decoded::List(
            items.iter().filter_map(primitive_content).collect(),
        )),
        Value::Null | Value::Object(_) => None,
        Value::String(text) => Some(Decoded::Text(text.clone())),
        Value::Bool(flag) => Some(Decoded::Flag(*flag)),
        Value::Number(number) => Some(decode_number(number)),
    }
}

fn decode_number(number: &Number) -> Decoded {
    match number.as_i64() {
        Some(value) => i32::try_from(value).map_or(Decoded::Long(value), Decoded::Int),
        None => Decoded::Double,
    }
}

fn primitive_content(element: &Value) -> Option<String> {
    match element {
        Value::String(text) => Some(text.clone()),
        Value::Number(number) => Some(number.to_string()),
        Value::Bool(flag) => Some(flag.to_string()),
        Value::Null => Some("null".to_string()),
        Value::Array(_) | Value::Object(_) => None,
    }
}

fn normalize_for_type(
    key: &str,
    value: Decoded,
    expected: PrefType,
) -> Result<PrefValue, PrefsError> {
    let mismatch = |name: &str, value: &Decoded| {
        PrefsError::IllegalArgument(format!(
            "Expected {name} for '{key}', got {}",
            value.type_name()
        ))
    };
    match (expected, value) {
        (PrefType::String, Decoded::Text(text)) => Ok(PrefValue::String(text)),
        (PrefType::String, other) => Err(mismatch("String", &other)),
        (PrefType::Boolean, Decoded::Flag(flag)) => Ok(PrefValue::Boolean(flag)),
        (PrefType::Boolean, other) => Err(mismatch("Boolean", &other)),
        (PrefType::Int, Decoded::Int(number)) => Ok(PrefValue::Int(number)),
        (PrefType::Int, Decoded::Long(number)) => match i32::try_from(number) {
            Ok(number) => Ok(PrefValue::Int(number)),
            Err(_) => Err(PrefsError::IllegalArgument(format!(
                "Expected Int in range for '{key}', got Long {number}"
            ))),
        },
        (PrefType::Int, other) => Err(mismatch("Int", &other)),
        (PrefType::StringSet, Decoded::List(items)) => {
            let mut unique: Vec<String> = Vec::with_capacity(items.len());
            for item in items {
                if !unique.contains(&item) {
                    unique.push(item);
                }
            }
            Ok(PrefValue::StringSet(unique))
        }
        (PrefType::StringSet, other) => Err(mismatch("StringSet", &other)),
    }
}

fn scalar_text(element: &Value) -> Option<String> {
    match element {
        Value::Null => None,
        other => primitive_content(other),
    }
}

fn coerce(element: &Value, expected: PrefType) -> Option<PrefValue> {
    match expected {
        PrefType::String => scalar_text(element).map(PrefValue::String),
        PrefType::Boolean => {
            let text = scalar_text(element)?.to_lowercase();
            match text.as_str() {
                "true" => Some(PrefValue::Boolean(true)),
                "false" => Some(PrefValue::Boolean(false)),
                _ => None,
            }
        }
        PrefType::Int => {
            let text = scalar_text(element)?;
            let text = trim(&text);
            match to_int(text) {
                Ok(number) => Some(PrefValue::Int(number)),
                Err(_) => text
                    .parse::<f64>()
                    .ok()
                    .map(|number| PrefValue::Int(number as i32)),
            }
        }
        PrefType::StringSet => match element {
            Value::Array(items) => {
                let mut unique: Vec<String> = Vec::new();
                for item in items.iter().filter_map(primitive_content) {
                    if !unique.contains(&item) {
                        unique.push(item);
                    }
                }
                Some(PrefValue::StringSet(unique))
            }
            _ => None,
        },
    }
}

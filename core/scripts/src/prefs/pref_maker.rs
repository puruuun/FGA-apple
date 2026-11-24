use std::collections::BTreeMap;

use crate::kotlin::to_int;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PrefType {
    String,
    Int,
    Boolean,
    StringSet,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrefValue {
    String(String),
    Int(i32),
    Boolean(bool),
    StringSet(Vec<String>),
}

impl PrefValue {
    pub fn pref_type(&self) -> PrefType {
        match self {
            PrefValue::String(_) => PrefType::String,
            PrefValue::Int(_) => PrefType::Int,
            PrefValue::Boolean(_) => PrefType::Boolean,
            PrefValue::StringSet(_) => PrefType::StringSet,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PrefStore {
    values: BTreeMap<String, PrefValue>,
}

impl PrefStore {
    pub fn new() -> PrefStore {
        PrefStore::default()
    }

    pub(super) fn insert(&mut self, key: String, value: PrefValue) {
        self.values.insert(key, value);
    }

    pub fn get(&self, key: &str) -> Option<&PrefValue> {
        self.values.get(key)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.values.keys()
    }
}

pub struct PrefMaker<'a> {
    store: Option<&'a PrefStore>,
    types: BTreeMap<String, PrefType>,
}

impl<'a> PrefMaker<'a> {
    pub fn new(store: &'a PrefStore) -> PrefMaker<'a> {
        PrefMaker {
            store: Some(store),
            types: BTreeMap::new(),
        }
    }

    pub fn schema_only() -> PrefMaker<'static> {
        PrefMaker {
            store: None,
            types: BTreeMap::new(),
        }
    }

    pub fn types(&self) -> &BTreeMap<String, PrefType> {
        &self.types
    }

    pub fn into_types(self) -> BTreeMap<String, PrefType> {
        self.types
    }

    fn stored(&self, key: &str) -> Option<&PrefValue> {
        self.store.and_then(|store| store.get(key))
    }

    pub fn int(&mut self, key: &str, default: i32) -> i32 {
        self.types.insert(key.to_string(), PrefType::Int);
        match self.stored(key) {
            Some(PrefValue::Int(value)) => *value,
            _ => default,
        }
    }

    pub fn bool(&mut self, key: &str, default: bool) -> bool {
        self.types.insert(key.to_string(), PrefType::Boolean);
        match self.stored(key) {
            Some(PrefValue::Boolean(value)) => *value,
            _ => default,
        }
    }

    pub fn string(&mut self, key: &str, default: &str) -> String {
        self.types.insert(key.to_string(), PrefType::String);
        match self.stored(key) {
            Some(PrefValue::String(value)) => value.clone(),
            _ => default.to_string(),
        }
    }

    pub fn serialized<T>(
        &mut self,
        key: &str,
        default: T,
        deserialize: impl FnOnce(&str) -> T,
    ) -> T {
        self.types.insert(key.to_string(), PrefType::String);
        match self.stored(key) {
            Some(PrefValue::String(value)) => deserialize(value),
            _ => default,
        }
    }

    pub fn string_as_int(&mut self, key: &str, default: i32) -> i32 {
        self.serialized(key, default, |text| to_int(text).unwrap_or(default))
    }

    pub fn enum_pref<E: Copy>(
        &mut self,
        key: &str,
        default: E,
        from_name: fn(&str) -> Option<E>,
    ) -> E {
        self.serialized(key, default, |name| from_name(name).unwrap_or(default))
    }

    pub fn string_set(&mut self, key: &str) -> Vec<String> {
        self.types.insert(key.to_string(), PrefType::StringSet);
        match self.stored(key) {
            Some(PrefValue::StringSet(values)) => values.clone(),
            _ => Vec::new(),
        }
    }
}

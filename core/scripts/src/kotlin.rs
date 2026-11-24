use serde::de::{Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use crate::models::model_error::ModelError;

const DECIMAL_DIGIT_ZEROS: [u32; 37] = [
    0x0030, 0x0660, 0x06F0, 0x07C0, 0x0966, 0x09E6, 0x0A66, 0x0AE6, 0x0B66, 0x0BE6, 0x0C66, 0x0CE6,
    0x0D66, 0x0DE6, 0x0E50, 0x0ED0, 0x0F20, 0x1040, 0x1090, 0x17E0, 0x1810, 0x1946, 0x19D0, 0x1A80,
    0x1A90, 0x1B50, 0x1BB0, 0x1C40, 0x1C50, 0xA620, 0xA8D0, 0xA900, 0xA9D0, 0xA9F0, 0xAA50, 0xABF0,
    0xFF10,
];

pub fn is_whitespace(c: char) -> bool {
    matches!(
        c as u32,
        0x09..=0x0D
            | 0x1C..=0x20
            | 0xA0
            | 0x1680
            | 0x2000..=0x200A
            | 0x2028
            | 0x2029
            | 0x202F
            | 0x205F
            | 0x3000
    )
}

pub fn trim(text: &str) -> &str {
    text.trim_matches(is_whitespace)
}

pub fn is_blank(text: &str) -> bool {
    text.chars().all(is_whitespace)
}

pub fn digit_of(c: char) -> Option<u32> {
    let code = c as u32;
    DECIMAL_DIGIT_ZEROS
        .iter()
        .find(|zero| code >= **zero && code <= **zero + 9)
        .map(|zero| code - zero)
}

pub fn to_int(text: &str) -> Result<i32, ModelError> {
    let fail = || ModelError::NumberFormat(format!("For input string: \"{text}\""));
    let mut chars = text.chars().peekable();
    let negative = match chars.peek() {
        None => return Err(fail()),
        Some('-') => {
            chars.next();
            true
        }
        Some('+') => {
            chars.next();
            false
        }
        Some(_) => false,
    };
    let mut value: i64 = 0;
    let mut any = false;
    for c in chars {
        let digit = digit_of(c).ok_or_else(fail)?;
        value = value * 10 + i64::from(digit);
        if value > i64::from(i32::MAX) + 1 {
            return Err(fail());
        }
        any = true;
    }
    if !any {
        return Err(fail());
    }
    let signed = if negative { -value } else { value };
    i32::try_from(signed).map_err(|_| fail())
}

pub fn coerce_index(value: i32, len: usize) -> Option<usize> {
    let last = len.checked_sub(1)?;
    Some(usize::try_from(value).map_or(0, |v| v.min(last)))
}

#[derive(Debug, Clone)]
pub struct LinkedSet<T> {
    items: Vec<T>,
}

impl<T> Default for LinkedSet<T> {
    fn default() -> Self {
        Self { items: Vec::new() }
    }
}

impl<T: PartialEq> LinkedSet<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, item: T) -> bool {
        if self.items.contains(&item) {
            false
        } else {
            self.items.push(item);
            true
        }
    }

    pub fn contains(&self, item: &T) -> bool {
        self.items.contains(item)
    }

    pub fn iter(&self) -> std::slice::Iter<'_, T> {
        self.items.iter()
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }
}

impl<T: PartialEq + Clone> LinkedSet<T> {
    pub fn plus(&self, other: &LinkedSet<T>) -> LinkedSet<T> {
        let mut result = self.clone();
        for item in other.iter() {
            result.insert(item.clone());
        }
        result
    }
}

impl<T: PartialEq> FromIterator<T> for LinkedSet<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        let mut set = Self::new();
        for item in iter {
            set.insert(item);
        }
        set
    }
}

impl<T: PartialEq> PartialEq for LinkedSet<T> {
    fn eq(&self, other: &Self) -> bool {
        self.items.len() == other.items.len() && self.items.iter().all(|i| other.contains(i))
    }
}

impl<T: Eq> Eq for LinkedSet<T> {}

impl<'a, T> IntoIterator for &'a LinkedSet<T> {
    type Item = &'a T;
    type IntoIter = std::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.iter()
    }
}

impl<T: Serialize> Serialize for LinkedSet<T> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.items.serialize(serializer)
    }
}

impl<'de, T: Deserialize<'de> + PartialEq> Deserialize<'de> for LinkedSet<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Ok(Vec::<T>::deserialize(deserializer)?.into_iter().collect())
    }
}

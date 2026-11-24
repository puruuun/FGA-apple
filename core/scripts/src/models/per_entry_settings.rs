use crate::enums::card_priority_style::CardPriorityStyle;
use crate::kotlin::{coerce_index, is_blank, trim};

const SEPARATOR: &str = ",";

pub fn at<T: Clone>(values: &[T], index: i32, default: T) -> T {
    match coerce_index(index, values.len()).and_then(|i| values.get(i)) {
        Some(value) => value.clone(),
        None => default,
    }
}

pub fn with_legacy<T: Clone>(values: &[T], legacy: T) -> Vec<T> {
    if values.is_empty() {
        vec![legacy]
    } else {
        values.to_vec()
    }
}

pub fn parse_styles(text: &str) -> Vec<CardPriorityStyle> {
    if is_blank(text) {
        Vec::new()
    } else {
        text.split(SEPARATOR)
            .map(|part| {
                CardPriorityStyle::from_name(trim(part)).unwrap_or(CardPriorityStyle::Affinity)
            })
            .collect()
    }
}

pub fn serialize_styles(styles: &[CardPriorityStyle]) -> String {
    let names: Vec<&str> = styles.iter().map(|style| style.name()).collect();
    names.join(SEPARATOR)
}

pub fn parse_flags(text: &str) -> Vec<bool> {
    if is_blank(text) {
        Vec::new()
    } else {
        text.split(SEPARATOR)
            .map(|part| trim(part).eq_ignore_ascii_case("true"))
            .collect()
    }
}

pub fn serialize_flags(flags: &[bool]) -> String {
    let parts: Vec<String> = flags.iter().map(ToString::to_string).collect();
    parts.join(SEPARATOR)
}

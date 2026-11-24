use std::collections::HashSet;
use std::fmt;
use std::ops::Deref;

use super::model_error::ModelError;
use super::servant_card_key::ServantCardKey;
use crate::kotlin::{coerce_index, is_blank};

const ENTRY_SEPARATOR: &str = "\n";
const SEPARATOR: &str = ",";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServantCardPriorityPerWave {
    per_entry: Vec<Vec<ServantCardKey>>,
}

impl Deref for ServantCardPriorityPerWave {
    type Target = [Vec<ServantCardKey>];

    fn deref(&self) -> &[Vec<ServantCardKey>] {
        &self.per_entry
    }
}

impl fmt::Display for ServantCardPriorityPerWave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let entries: Vec<String> = self
            .per_entry
            .iter()
            .map(|entry| {
                let keys: Vec<String> = entry.iter().map(ToString::to_string).collect();
                keys.join(SEPARATOR)
            })
            .collect();
        f.write_str(&entries.join(ENTRY_SEPARATOR))
    }
}

impl Default for ServantCardPriorityPerWave {
    fn default() -> ServantCardPriorityPerWave {
        ServantCardPriorityPerWave {
            per_entry: vec![ServantCardKey::all()],
        }
    }
}

impl ServantCardPriorityPerWave {
    pub fn at_wave(&self, wave: i32) -> &[ServantCardKey] {
        coerce_index(wave, self.per_entry.len())
            .and_then(|index| self.per_entry.get(index))
            .map_or(&[], Vec::as_slice)
    }

    pub fn from(
        per_entry: Vec<Vec<ServantCardKey>>,
    ) -> Result<ServantCardPriorityPerWave, ModelError> {
        if per_entry.is_empty() {
            return Err(ModelError::illegal_argument("At least one entry is needed"));
        }
        for entry in &per_entry {
            let distinct: HashSet<&ServantCardKey> = entry.iter().collect();
            if entry.len() != 9 || distinct.len() != 9 {
                return Err(ModelError::illegal_argument(
                    "Each entry needs the nine distinct cards",
                ));
            }
        }
        Ok(ServantCardPriorityPerWave { per_entry })
    }

    pub fn of(text: &str) -> Result<ServantCardPriorityPerWave, ModelError> {
        if is_blank(text) {
            return Ok(ServantCardPriorityPerWave::default());
        }
        let mut per_entry = Vec::new();
        for line in text.split(ENTRY_SEPARATOR) {
            let mut keys = Vec::new();
            for part in line.split(SEPARATOR) {
                let key = ServantCardKey::parse(part).ok_or_else(|| {
                    ModelError::illegal_argument(format!("Invalid card '{part}'"))
                })?;
                keys.push(key);
            }
            per_entry.push(keys);
        }
        ServantCardPriorityPerWave::from(per_entry)
    }
}

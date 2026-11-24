use std::fmt;
use std::ops::Deref;

use super::model_error::ModelError;
use super::team_slot::TeamSlot;
use crate::kotlin::{coerce_index, is_blank, to_int};

const STAGE_SEPARATOR: &str = "\n";
const SEPARATOR: &str = ",";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServantPriorityPerWave {
    scores_per_wave: Vec<Vec<TeamSlot>>,
}

impl Deref for ServantPriorityPerWave {
    type Target = [Vec<TeamSlot>];

    fn deref(&self) -> &[Vec<TeamSlot>] {
        &self.scores_per_wave
    }
}

impl fmt::Display for ServantPriorityPerWave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stages: Vec<String> = self
            .scores_per_wave
            .iter()
            .map(|stage| {
                let positions: Vec<String> = stage
                    .iter()
                    .map(|slot| slot.position().to_string())
                    .collect();
                positions.join(SEPARATOR)
            })
            .collect();
        f.write_str(&stages.join(STAGE_SEPARATOR))
    }
}

impl Default for ServantPriorityPerWave {
    fn default() -> ServantPriorityPerWave {
        ServantPriorityPerWave {
            scores_per_wave: vec![TeamSlot::LIST.to_vec()],
        }
    }
}

impl ServantPriorityPerWave {
    pub fn from(scores_per_wave: Vec<Vec<TeamSlot>>) -> ServantPriorityPerWave {
        ServantPriorityPerWave { scores_per_wave }
    }

    pub fn at_wave(&self, wave: i32) -> Result<&[TeamSlot], ModelError> {
        coerce_index(wave, self.scores_per_wave.len())
            .and_then(|index| self.scores_per_wave.get(index))
            .map(Vec::as_slice)
            .ok_or_else(|| {
                ModelError::illegal_argument("Cannot coerce value to an empty range: 0..-1.")
            })
    }

    pub fn of(priority: &str) -> Result<ServantPriorityPerWave, ModelError> {
        if is_blank(priority) {
            return Ok(ServantPriorityPerWave::default());
        }
        let mut scores_per_wave = Vec::new();
        for priority_on_a_wave in priority.split(STAGE_SEPARATOR) {
            let mut slots = Vec::new();
            for part in priority_on_a_wave.split(SEPARATOR) {
                let index = to_int(part)?.wrapping_sub(1);
                let slot = usize::try_from(index)
                    .ok()
                    .and_then(|i| TeamSlot::LIST.get(i))
                    .ok_or_else(|| {
                        ModelError::IndexOutOfBounds(format!(
                            "Index {index} out of bounds for length {}",
                            TeamSlot::LIST.len()
                        ))
                    })?;
                slots.push(*slot);
            }
            scores_per_wave.push(slots);
        }
        Ok(ServantPriorityPerWave { scores_per_wave })
    }
}

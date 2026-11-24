use std::fmt;
use std::ops::Deref;

use super::card_priority::CardPriority;
use super::card_score::CardScore;
use super::model_error::ModelError;
use crate::enums::card_affinity_enum::CardAffinityEnum;
use crate::enums::card_type_enum::CardTypeEnum;
use crate::kotlin::{coerce_index, is_blank};

const CARD_PRIORITY_STAGE_SEPARATOR: &str = "\n";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardPriorityPerWave {
    scores_per_wave: Vec<CardPriority>,
}

impl Deref for CardPriorityPerWave {
    type Target = [CardPriority];

    fn deref(&self) -> &[CardPriority] {
        &self.scores_per_wave
    }
}

impl fmt::Display for CardPriorityPerWave {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self
            .scores_per_wave
            .iter()
            .map(ToString::to_string)
            .collect();
        f.write_str(&parts.join(CARD_PRIORITY_STAGE_SEPARATOR))
    }
}

impl Default for CardPriorityPerWave {
    fn default() -> CardPriorityPerWave {
        use CardAffinityEnum::{Normal, Resist, Weak};
        use CardTypeEnum::{Arts, Buster, Quick};
        let scores = [
            (Buster, Weak),
            (Arts, Weak),
            (Quick, Weak),
            (Buster, Normal),
            (Arts, Normal),
            (Quick, Normal),
            (Buster, Resist),
            (Arts, Resist),
            (Quick, Resist),
        ];
        CardPriorityPerWave {
            scores_per_wave: vec![CardPriority::from(
                scores
                    .iter()
                    .map(|(card_type, affinity)| CardScore::new(*card_type, *affinity))
                    .collect(),
            )],
        }
    }
}

impl CardPriorityPerWave {
    pub fn from(scores_per_wave: Vec<CardPriority>) -> CardPriorityPerWave {
        CardPriorityPerWave { scores_per_wave }
    }

    pub fn of(priority: &str) -> Result<CardPriorityPerWave, ModelError> {
        if is_blank(priority) {
            Ok(CardPriorityPerWave::default())
        } else {
            let scores_per_wave = priority
                .split(CARD_PRIORITY_STAGE_SEPARATOR)
                .map(CardPriority::of)
                .collect::<Result<Vec<_>, _>>()?;
            Ok(CardPriorityPerWave { scores_per_wave })
        }
    }

    pub fn at_wave(&self, wave: i32) -> Result<Vec<CardScore>, ModelError> {
        let index = coerce_index(wave, self.scores_per_wave.len()).ok_or_else(|| {
            ModelError::illegal_argument("Cannot coerce value to an empty range: 0..-1.")
        })?;
        let mut scores: Vec<CardScore> = self
            .scores_per_wave
            .get(index)
            .map(|priority| priority.to_vec())
            .unwrap_or_default();
        scores.push(CardScore::new(
            CardTypeEnum::Unknown,
            CardAffinityEnum::Normal,
        ));
        Ok(scores)
    }
}

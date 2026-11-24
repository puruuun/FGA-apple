use std::fmt;
use std::ops::Deref;

use super::card_score::CardScore;
use super::model_error::ModelError;
use crate::enums::card_affinity_enum::CardAffinityEnum;
use crate::enums::card_type_enum::CardTypeEnum;
use crate::kotlin::trim;

const DUMMY_NORMAL_AFFINITY_CHAR: u16 = 'X' as u16;
const BUSTER_CHAR: u16 = 'B' as u16;
const ARTS_CHAR: u16 = 'A' as u16;
const QUICK_CHAR: u16 = 'Q' as u16;
const WEAK_CHAR: u16 = 'W' as u16;
const RESIST_CHAR: u16 = 'R' as u16;
const CARD_PRIORITY_ERROR_STRING: &str = "Battle_CardPriority Error at '";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardPriority(Vec<CardScore>);

impl Deref for CardPriority {
    type Target = [CardScore];

    fn deref(&self) -> &[CardScore] {
        &self.0
    }
}

impl fmt::Display for CardPriority {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let parts: Vec<String> = self.0.iter().map(ToString::to_string).collect();
        f.write_str(&parts.join(", "))
    }
}

fn raise_parse_error(message: String) -> ModelError {
    ModelError::CardPriorityParse(message)
}

fn unit_string(units: &[u16]) -> String {
    String::from_utf16_lossy(units)
}

impl CardPriority {
    pub fn from(scores: Vec<CardScore>) -> CardPriority {
        CardPriority(scores)
    }

    pub fn of(priority: &str) -> Result<CardPriority, ModelError> {
        let mut scores = Vec::new();
        for part in priority.split(',') {
            let text = trim(part).to_uppercase();
            let units: Vec<u16> = text.encode_utf16().collect();
            let card = match units.as_slice() {
                [only] => [DUMMY_NORMAL_AFFINITY_CHAR, *only],
                [first, second] => [*first, *second],
                _ => {
                    return Err(raise_parse_error(format!(
                        "{CARD_PRIORITY_ERROR_STRING}{text}': Invalid card length."
                    )));
                }
            };
            let card_type = match card[1] {
                BUSTER_CHAR => CardTypeEnum::Buster,
                ARTS_CHAR => CardTypeEnum::Arts,
                QUICK_CHAR => CardTypeEnum::Quick,
                other => {
                    return Err(raise_parse_error(format!(
                        "{CARD_PRIORITY_ERROR_STRING}{}': Only 'B', 'A' and 'Q' are valid card types.",
                        unit_string(&[other])
                    )));
                }
            };
            let card_affinity = match card[0] {
                WEAK_CHAR => CardAffinityEnum::Weak,
                RESIST_CHAR => CardAffinityEnum::Resist,
                DUMMY_NORMAL_AFFINITY_CHAR => CardAffinityEnum::Normal,
                other => {
                    return Err(raise_parse_error(format!(
                        "{CARD_PRIORITY_ERROR_STRING}{}': Only 'W', and 'R' are valid card affinities.",
                        unit_string(&[other])
                    )));
                }
            };
            scores.push(CardScore::new(card_type, card_affinity));
        }
        if scores.len() != 9 {
            return Err(raise_parse_error(format!(
                "{CARD_PRIORITY_ERROR_STRING}': Expected 9 cards, but {} found.",
                scores.len()
            )));
        }
        Ok(CardPriority(scores))
    }
}

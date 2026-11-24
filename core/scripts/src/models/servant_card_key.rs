use std::fmt;

use super::model_error::ModelError;
use crate::enums::card_type_enum::CardTypeEnum;
use crate::kotlin::{digit_of, trim};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ServantCardKey {
    pub field_position: i32,
    pub r#type: CardTypeEnum,
}

impl ServantCardKey {
    pub fn new(field_position: i32, r#type: CardTypeEnum) -> Result<ServantCardKey, ModelError> {
        if !(1..=3).contains(&field_position) {
            return Err(ModelError::illegal_argument("Field position must be 1..3"));
        }
        if r#type == CardTypeEnum::Unknown {
            return Err(ModelError::illegal_argument("Card type must be known"));
        }
        Ok(ServantCardKey {
            field_position,
            r#type,
        })
    }

    pub fn all() -> Vec<ServantCardKey> {
        let mut keys = Vec::with_capacity(9);
        for field_position in 1..=3 {
            for r#type in [
                CardTypeEnum::Buster,
                CardTypeEnum::Arts,
                CardTypeEnum::Quick,
            ] {
                keys.push(ServantCardKey {
                    field_position,
                    r#type,
                });
            }
        }
        keys
    }

    pub fn parse(text: &str) -> Option<ServantCardKey> {
        let upper = trim(text).to_uppercase();
        let units: Vec<u16> = upper.encode_utf16().collect();
        let [first, second] = units.as_slice() else {
            return None;
        };
        let position = digit_of(char::from_u32(u32::from(*first))?)?;
        let field_position = i32::try_from(position)
            .ok()
            .filter(|p| (1..=3).contains(p))?;
        let r#type = match char::from_u32(u32::from(*second))? {
            'B' => CardTypeEnum::Buster,
            'A' => CardTypeEnum::Arts,
            'Q' => CardTypeEnum::Quick,
            _ => return None,
        };
        Some(ServantCardKey {
            field_position,
            r#type,
        })
    }
}

impl fmt::Display for ServantCardKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let initial = self.r#type.name().chars().next().unwrap_or_default();
        write!(f, "{}{}", self.field_position, initial)
    }
}

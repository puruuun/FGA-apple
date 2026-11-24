use std::fmt;

use crate::enums::card_affinity_enum::CardAffinityEnum;
use crate::enums::card_type_enum::CardTypeEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CardScore {
    pub r#type: CardTypeEnum,
    pub affinity: CardAffinityEnum,
}

impl CardScore {
    pub fn new(r#type: CardTypeEnum, affinity: CardAffinityEnum) -> Self {
        Self { r#type, affinity }
    }
}

impl fmt::Display for CardScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut result = String::new();
        if self.affinity != CardAffinityEnum::Normal {
            result.push_str(&format!("{} ", self.affinity));
        }
        result.push_str(&self.r#type.to_string());
        let capitals: String = result.chars().filter(|c| c.is_uppercase()).collect();
        f.write_str(&capitals)
    }
}

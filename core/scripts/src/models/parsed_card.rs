use std::hash::{Hash, Hasher};

use super::command_card::Face;
use super::field_slot::FieldSlot;
use super::team_slot::TeamSlot;
use crate::enums::card_affinity_enum::CardAffinityEnum;
use crate::enums::card_type_enum::CardTypeEnum;

#[derive(Debug, Clone, Copy)]
pub struct ParsedCard {
    pub card: Face,
    pub servant: TeamSlot,
    pub field_slot: Option<FieldSlot>,
    pub r#type: CardTypeEnum,
    pub affinity: CardAffinityEnum,
    pub is_stunned: bool,
    pub swapped_from: Option<CardTypeEnum>,
}

impl ParsedCard {
    pub fn new(
        card: Face,
        servant: TeamSlot,
        field_slot: Option<FieldSlot>,
        r#type: CardTypeEnum,
    ) -> ParsedCard {
        ParsedCard {
            card,
            servant,
            field_slot,
            r#type,
            affinity: CardAffinityEnum::Normal,
            is_stunned: false,
            swapped_from: None,
        }
    }
}

impl PartialEq for ParsedCard {
    fn eq(&self, other: &Self) -> bool {
        self.card == other.card
    }
}

impl Eq for ParsedCard {}

impl Hash for ParsedCard {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.card.hash(state);
    }
}

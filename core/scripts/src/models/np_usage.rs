use super::command_card::NP;
use super::direct_card_sequence::DirectCardSequence;
use crate::kotlin::LinkedSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NPUsage {
    pub nps: LinkedSet<NP>,
    pub cards_before_np: i32,
    pub direct_cards: Option<DirectCardSequence>,
}

impl NPUsage {
    pub fn new(nps: LinkedSet<NP>, cards_before_np: i32) -> NPUsage {
        NPUsage {
            nps,
            cards_before_np,
            direct_cards: None,
        }
    }

    pub fn none() -> NPUsage {
        NPUsage::new(LinkedSet::new(), 0)
    }
}

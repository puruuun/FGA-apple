use std::collections::HashSet;
use std::fmt;

use super::command_card::{CommandCard, Face, NP};
use super::model_error::ModelError;
use super::parsed_card::ParsedCard;
use super::special_command::SpecialCommand;
use crate::enums::card_affinity_enum::CardAffinityEnum;
use crate::enums::card_type_enum::CardTypeEnum;
use crate::kotlin::LinkedSet;

const ALLOWED_TOKENS: &str = "baq456p";

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DirectCardSequence {
    code: String,
    fallback: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Resolution {
    pub cards: Vec<CommandCard>,
    pub used_priority: bool,
}

fn np(token: char) -> Option<NP> {
    NP::LIST
        .iter()
        .copied()
        .find(|np| np.auto_skill_code() == token)
}

fn card_type(token: char) -> Result<CardTypeEnum, ModelError> {
    match token {
        'b' => Ok(CardTypeEnum::Buster),
        'a' => Ok(CardTypeEnum::Arts),
        'q' => Ok(CardTypeEnum::Quick),
        _ => Err(ModelError::IllegalState(
            "Invalid direct face card".to_string(),
        )),
    }
}

fn affinity_rank(card: &ParsedCard) -> i32 {
    match card.affinity {
        CardAffinityEnum::Weak => 0,
        CardAffinityEnum::Normal => 1,
        CardAffinityEnum::Resist => 2,
    }
}

impl DirectCardSequence {
    fn new(code: String, fallback: bool) -> Result<DirectCardSequence, ModelError> {
        if code.encode_utf16().count() != 3 || !code.chars().all(|c| ALLOWED_TOKENS.contains(c)) {
            return Err(ModelError::illegal_argument(
                "Direct attack requires exactly three choices: b, a, q, p, 4, 5 or 6",
            ));
        }
        if !fallback && code.contains('p') {
            return Err(ModelError::illegal_argument(
                "Unselected cards require the priority fallback",
            ));
        }
        let np_codes: Vec<char> = code.chars().filter(|c| np(*c).is_some()).collect();
        let distinct: HashSet<&char> = np_codes.iter().collect();
        if distinct.len() != np_codes.len() {
            return Err(ModelError::illegal_argument(
                "The same NP cannot be selected twice",
            ));
        }
        Ok(DirectCardSequence { code, fallback })
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn fallback(&self) -> bool {
        self.fallback
    }

    pub fn nps(&self) -> LinkedSet<NP> {
        self.code.chars().filter_map(np).collect()
    }

    pub fn cards_before_np(&self) -> i32 {
        self.code
            .chars()
            .position(|c| np(c).is_some())
            .and_then(|index| i32::try_from(index).ok())
            .unwrap_or(0)
    }

    pub fn resolve(&self, cards: &[ParsedCard]) -> Option<Vec<CommandCard>> {
        self.resolve_with(cards, |remaining| remaining)
    }

    pub fn resolve_with(
        &self,
        cards: &[ParsedCard],
        prioritize: impl Fn(Vec<ParsedCard>) -> Vec<ParsedCard>,
    ) -> Option<Vec<CommandCard>> {
        self.resolve_detailed_with(cards, prioritize)
            .map(|resolution| resolution.cards)
    }

    pub fn resolve_detailed(&self, cards: &[ParsedCard]) -> Option<Resolution> {
        self.resolve_detailed_with(cards, |remaining| remaining)
    }

    pub fn resolve_detailed_with(
        &self,
        cards: &[ParsedCard],
        prioritize: impl Fn(Vec<ParsedCard>) -> Vec<ParsedCard>,
    ) -> Option<Resolution> {
        let faces: HashSet<Face> = cards.iter().map(|card| card.card).collect();
        let all_faces: HashSet<Face> = Face::LIST.iter().copied().collect();
        if cards.len() != 5
            || faces != all_faces
            || cards
                .iter()
                .any(|card| !card.is_stunned && card.r#type == CardTypeEnum::Unknown)
        {
            return None;
        }
        let mut remaining: Vec<ParsedCard> = cards
            .iter()
            .filter(|card| !card.is_stunned)
            .copied()
            .collect();
        remaining.sort_by_key(|card| (affinity_rank(card), card.card.index()));

        let mut result: Vec<Option<CommandCard>> = Vec::with_capacity(3);
        for token in self.code.chars() {
            if token == 'p' {
                result.push(None);
            } else if let Some(np) = np(token) {
                result.push(Some(CommandCard::NP(np)));
            } else {
                let wanted = card_type(token).ok()?;
                let position = remaining.iter().position(|card| card.r#type == wanted);
                result.push(position.map(|index| CommandCard::Face(remaining.remove(index).card)));
            }
        }

        let mut used_priority = false;
        if result.iter().any(Option::is_none) {
            if !self.fallback {
                return None;
            }
            let mut seen: HashSet<Face> = HashSet::new();
            let mut replacements = prioritize(remaining.clone())
                .into_iter()
                .filter(|card| remaining.contains(card))
                .filter(|card| seen.insert(card.card))
                .collect::<Vec<_>>()
                .into_iter();
            for slot in result.iter_mut() {
                if slot.is_none() {
                    let replacement = replacements.next()?;
                    *slot = Some(CommandCard::Face(replacement.card));
                    used_priority = true;
                }
            }
        }
        Some(Resolution {
            cards: result.into_iter().flatten().collect(),
            used_priority,
        })
    }

    pub fn parse(value: &str) -> Result<DirectCardSequence, ModelError> {
        let code = value.strip_prefix('?').unwrap_or(value).to_lowercase();
        DirectCardSequence::new(code, value.starts_with('?'))
    }
}

impl fmt::Display for DirectCardSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}{}{}",
            SpecialCommand::DirectCards.auto_skill_code(),
            if self.fallback { "?" } else { "" },
            self.code
        )
    }
}

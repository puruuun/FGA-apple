use super::command_card::NP;
use super::direct_card_sequence::DirectCardSequence;
use super::enemy_target::EnemyTarget;
use super::field_slot::FieldSlot;
use super::model_error::ModelError;
use super::np_usage::NPUsage;
use super::order_change_member::{Starting, Sub};
use super::servant_target::ServantTarget;
use super::skill::{CommandSpell as CommandSpellSkill, Master, Servant};
use super::special_command::SpecialCommand;
use crate::enums::card_type_enum::CardTypeEnum;
use crate::kotlin::LinkedSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AutoSkillAction {
    ShuffleUntil(ShuffleUntil),
    Atk(Atk),
    ServantSkill(ServantSkill),
    MasterSkill(MasterSkill),
    TargetEnemy(TargetEnemy),
    CommandSpell(CommandSpell),
    OrderChange(OrderChange),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShuffleUntil {
    pub skill: Servant,
    pub min: i32,
    pub max: i32,
    pub conditions: Vec<ShuffleCondition>,
    pub stop_if_unmet: bool,
}

impl ShuffleUntil {
    pub fn new(
        skill: Servant,
        min: i32,
        max: i32,
        conditions: Vec<ShuffleCondition>,
        stop_if_unmet: bool,
    ) -> Result<ShuffleUntil, ModelError> {
        if !Servant::SKILL1.contains(&skill) {
            return Err(ModelError::illegal_argument(format!(
                "Shuffle requires a first servant skill, got '{}'",
                skill.auto_skill_code()
            )));
        }
        if !(1..=30).contains(&max) || !(0..=max).contains(&min) {
            return Err(ModelError::illegal_argument(format!(
                "Shuffle limits must satisfy 0 <= min <= max in 1..=30, got min {min} and max {max}"
            )));
        }
        if conditions.is_empty() {
            return Err(ModelError::illegal_argument(
                "Shuffle requires at least one condition",
            ));
        }
        Ok(ShuffleUntil {
            skill,
            min,
            max,
            conditions,
            stop_if_unmet,
        })
    }

    pub fn to_command_string(&self) -> String {
        let limits = if self.min == 0 {
            self.max.to_string()
        } else {
            format!("{}-{}", self.min, self.max)
        };
        let conditions: Vec<String> = self
            .conditions
            .iter()
            .map(ShuffleCondition::to_command_string)
            .collect();
        format!(
            "{}H{{{}:{}{}}}",
            self.skill.auto_skill_code(),
            limits,
            conditions.join("|"),
            if self.stop_if_unmet { "!" } else { "" }
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShuffleCondition {
    pub slot: FieldSlot,
    pub count: i32,
    pub r#type: Option<CardTypeEnum>,
}

impl ShuffleCondition {
    pub fn new(
        slot: FieldSlot,
        count: i32,
        r#type: Option<CardTypeEnum>,
    ) -> Result<ShuffleCondition, ModelError> {
        if !(1..=5).contains(&count) {
            return Err(ModelError::illegal_argument(format!(
                "Shuffle condition count must be in 1..=5, got {count}"
            )));
        }
        if let Some(card_type) = r#type
            && !matches!(
                card_type,
                CardTypeEnum::Buster | CardTypeEnum::Arts | CardTypeEnum::Quick
            )
        {
            return Err(ModelError::illegal_argument(format!(
                "Shuffle condition colour must be Buster, Arts or Quick, got {card_type}"
            )));
        }
        Ok(ShuffleCondition {
            slot,
            count,
            r#type,
        })
    }

    pub fn key(&self) -> Result<String, ModelError> {
        let colour = match self.r#type {
            None => "",
            Some(CardTypeEnum::Buster) => "b",
            Some(CardTypeEnum::Arts) => "a",
            Some(CardTypeEnum::Quick) => "q",
            Some(CardTypeEnum::Unknown) => {
                return Err(ModelError::IllegalState(
                    "Unknown is not a shuffle condition colour".to_string(),
                ));
            }
        };
        Ok(format!("{}{}", self.slot.position(), colour))
    }

    pub fn to_command_string(&self) -> String {
        let key = match self.key() {
            Ok(key) => key,
            Err(_) => format!("{}", self.slot.position()),
        };
        format!("{key}{}", self.count)
    }

    pub fn from_code(code: &str) -> Result<ShuffleCondition, ModelError> {
        let chars: Vec<char> = code.chars().collect();
        let units = code.encode_utf16().count();
        let well_formed = (2..=3).contains(&units)
            && chars.first().is_some_and(|c| ('1'..='3').contains(c))
            && chars.last().is_some_and(|c| ('1'..='5').contains(c));
        if !well_formed {
            return Err(ModelError::illegal_argument(format!(
                "Invalid shuffle condition: {code}"
            )));
        }
        let r#type = if chars.len() == 2 {
            None
        } else {
            match chars.get(1).map(|c| c.to_ascii_lowercase()) {
                Some('b') => Some(CardTypeEnum::Buster),
                Some('a') => Some(CardTypeEnum::Arts),
                Some('q') => Some(CardTypeEnum::Quick),
                _ => {
                    return Err(ModelError::illegal_argument(format!(
                        "Invalid shuffle condition colour: {code}"
                    )));
                }
            }
        };
        let slot_index = chars
            .first()
            .and_then(|c| c.to_digit(10))
            .and_then(|d| usize::try_from(d).ok())
            .and_then(|d| d.checked_sub(1))
            .and_then(|index| FieldSlot::LIST.get(index))
            .ok_or_else(|| {
                ModelError::illegal_argument(format!("Invalid shuffle condition: {code}"))
            })?;
        let count = chars
            .last()
            .and_then(|c| c.to_digit(10))
            .and_then(|d| i32::try_from(d).ok())
            .ok_or_else(|| {
                ModelError::illegal_argument(format!("Invalid shuffle condition: {code}"))
            })?;
        ShuffleCondition::new(*slot_index, count, r#type)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Atk {
    pub nps: LinkedSet<NP>,
    pub cards_before_np: i32,
    pub direct_cards: Option<DirectCardSequence>,
}

impl Atk {
    pub fn new(
        nps: LinkedSet<NP>,
        cards_before_np: i32,
        direct_cards: Option<DirectCardSequence>,
    ) -> Result<Atk, ModelError> {
        if !(0..=2).contains(&cards_before_np) {
            return Err(ModelError::illegal_argument(
                "Only 0, 1 or 2 cards can be used before NP",
            ));
        }
        if let Some(direct) = &direct_cards
            && (nps != direct.nps() || cards_before_np != direct.cards_before_np())
        {
            return Err(ModelError::illegal_argument(
                "Direct cards must match the NPs and cards before NP",
            ));
        }
        Ok(Atk {
            nps,
            cards_before_np,
            direct_cards,
        })
    }

    pub fn plus(&self, other: &Atk) -> Result<Atk, ModelError> {
        if self.direct_cards.is_some() || other.direct_cards.is_some() {
            return Err(ModelError::illegal_argument(
                "Direct and automatic attacks cannot be combined",
            ));
        }
        Atk::new(
            self.nps.plus(&other.nps),
            self.cards_before_np.wrapping_add(other.cards_before_np),
            None,
        )
    }

    pub fn to_np_usage(&self) -> NPUsage {
        NPUsage {
            nps: self.nps.clone(),
            cards_before_np: self.cards_before_np,
            direct_cards: self.direct_cards.clone(),
        }
    }

    pub fn to_command_string(&self) -> String {
        if let Some(direct) = &self.direct_cards {
            return direct.to_string();
        }
        if *self == Atk::no_op() {
            return "0".to_string();
        }
        let mut result = String::new();
        if self.cards_before_np > 0 {
            result.push(SpecialCommand::CardsBeforeNP.auto_skill_code());
            result.push_str(&self.cards_before_np.to_string());
        }
        for np in &self.nps {
            result.push(np.auto_skill_code());
        }
        result
    }

    pub fn no_op() -> Atk {
        Atk {
            nps: LinkedSet::new(),
            cards_before_np: 0,
            direct_cards: None,
        }
    }

    pub fn np(np: NP) -> Atk {
        Atk {
            nps: std::iter::once(np).collect(),
            cards_before_np: 0,
            direct_cards: None,
        }
    }

    pub fn cards_before_np(cards: i32) -> Result<Atk, ModelError> {
        Atk::new(LinkedSet::new(), cards, None)
    }

    pub fn direct(sequence: DirectCardSequence) -> Result<Atk, ModelError> {
        Atk::new(sequence.nps(), sequence.cards_before_np(), Some(sequence))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ServantSkill {
    pub skill: Servant,
    pub targets: Vec<ServantTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MasterSkill {
    pub skill: Master,
    pub target: Option<ServantTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TargetEnemy {
    pub enemy: EnemyTarget,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpell {
    pub skill: CommandSpellSkill,
    pub target: Option<ServantTarget>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderChange {
    pub starting: Starting,
    pub sub: Sub,
}

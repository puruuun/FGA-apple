use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::team_slot::TeamSlot;
use crate::enums::spam_enum::SpamEnum;
use crate::kotlin::LinkedSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SkillSpamTarget {
    None,
    #[serde(rename = "Self")]
    SelfTarget,
    Slot1,
    Slot2,
    Slot3,
    Left,
    Right,
}

impl SkillSpamTarget {
    pub const ENTRIES: [SkillSpamTarget; 7] = [
        SkillSpamTarget::None,
        SkillSpamTarget::SelfTarget,
        SkillSpamTarget::Slot1,
        SkillSpamTarget::Slot2,
        SkillSpamTarget::Slot3,
        SkillSpamTarget::Left,
        SkillSpamTarget::Right,
    ];

    pub fn name(self) -> &'static str {
        match self {
            SkillSpamTarget::None => "None",
            SkillSpamTarget::SelfTarget => "Self",
            SkillSpamTarget::Slot1 => "Slot1",
            SkillSpamTarget::Slot2 => "Slot2",
            SkillSpamTarget::Slot3 => "Slot3",
            SkillSpamTarget::Left => "Left",
            SkillSpamTarget::Right => "Right",
        }
    }
}

fn default_waves() -> LinkedSet<i32> {
    (1..=3).collect()
}

fn default_spam() -> SpamEnum {
    SpamEnum::None
}

fn default_target() -> SkillSpamTarget {
    SkillSpamTarget::None
}

fn default_skills() -> Vec<SkillSpamConfig> {
    (1..=3).map(|_| SkillSpamConfig::default()).collect()
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SkillSpamConfig {
    #[serde(default = "default_waves")]
    pub waves: LinkedSet<i32>,
    #[serde(default = "default_spam")]
    pub spam: SpamEnum,
    #[serde(default = "default_target")]
    pub target: SkillSpamTarget,
}

impl Default for SkillSpamConfig {
    fn default() -> Self {
        SkillSpamConfig {
            waves: default_waves(),
            spam: default_spam(),
            target: default_target(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NpSpamConfig {
    #[serde(default = "default_waves")]
    pub waves: LinkedSet<i32>,
    #[serde(default = "default_spam")]
    pub spam: SpamEnum,
}

impl Default for NpSpamConfig {
    fn default() -> Self {
        NpSpamConfig {
            waves: default_waves(),
            spam: default_spam(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServantSpamConfig {
    #[serde(default = "default_skills")]
    pub skills: Vec<SkillSpamConfig>,
    #[serde(default)]
    pub np: NpSpamConfig,
}

impl Default for ServantSpamConfig {
    fn default() -> Self {
        ServantSpamConfig {
            skills: default_skills(),
            np: NpSpamConfig::default(),
        }
    }
}

fn shape_error(message: &str) -> serde_json::Error {
    <serde_json::Error as serde::de::Error>::custom(message)
}

fn as_object(value: &mut Value) -> Result<&mut serde_json::Map<String, Value>, serde_json::Error> {
    value
        .as_object_mut()
        .ok_or_else(|| shape_error("expected an object"))
}

fn normalize_waves(owner: &mut serde_json::Map<String, Value>) -> Result<(), serde_json::Error> {
    if let Some(Value::Array(waves)) = owner.get_mut("waves") {
        for wave in waves.iter_mut() {
            if let Value::String(text) = wave {
                let digits = text.strip_prefix('-').unwrap_or(text);
                if digits.is_empty() || !digits.chars().all(|c| c.is_ascii_digit()) {
                    return Err(shape_error("expected a numeric literal"));
                }
                let number: i32 = text
                    .parse()
                    .map_err(|_| shape_error("numeric literal out of range"))?;
                *wave = Value::from(number);
            }
        }
    }
    Ok(())
}

pub fn parse_servant_spam_configs(text: &str) -> Result<Vec<ServantSpamConfig>, serde_json::Error> {
    let mut root: Value = serde_json::from_str(text)?;
    let servants = root
        .as_array_mut()
        .ok_or_else(|| shape_error("expected an array"))?;
    for servant in servants.iter_mut() {
        let servant = as_object(servant)?;
        if let Some(skills) = servant.get_mut("skills") {
            let skills = skills
                .as_array_mut()
                .ok_or_else(|| shape_error("expected an array"))?;
            for skill in skills.iter_mut() {
                normalize_waves(as_object(skill)?)?;
            }
        }
        if let Some(np) = servant.get_mut("np") {
            normalize_waves(as_object(np)?)?;
        }
    }
    serde_json::from_value(root)
}

pub fn serialize_servant_spam_configs(
    configs: &[ServantSpamConfig],
) -> Result<String, serde_json::Error> {
    serde_json::to_string(configs)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpamConfigPerTeamSlot {
    config: Vec<ServantSpamConfig>,
}

impl SpamConfigPerTeamSlot {
    pub fn new(config: Vec<ServantSpamConfig>) -> Self {
        SpamConfigPerTeamSlot { config }
    }

    pub fn get(&self, team_slot: TeamSlot) -> ServantSpamConfig {
        usize::try_from(team_slot.position() - 1)
            .ok()
            .and_then(|index| self.config.get(index))
            .cloned()
            .unwrap_or_default()
    }
}

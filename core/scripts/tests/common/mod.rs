#![allow(dead_code)]

use scripts::enums::card_affinity_enum::CardAffinityEnum;
use scripts::enums::card_type_enum::CardTypeEnum;
use scripts::models::auto_skill_action::AutoSkillAction;
use scripts::models::auto_skill_command::AutoSkillCommand;
use scripts::models::command_card::{CommandCard, Face, NP};
use scripts::models::field_slot::FieldSlot;
use scripts::models::model_error::ModelError;
use scripts::models::parsed_card::ParsedCard;
use scripts::models::servant_target::ServantTarget;
use scripts::models::team_slot::TeamSlot;

pub fn parse(command: &str) -> Result<AutoSkillCommand, ModelError> {
    AutoSkillCommand::parse(command)
}

pub fn parse_ok(command: &str) -> AutoSkillCommand {
    match AutoSkillCommand::parse(command) {
        Ok(parsed) => parsed,
        Err(error) => panic!("{command:?} failed to parse: {error}"),
    }
}

pub fn actions(command: &str, stage: i32, turn: i32) -> Vec<AutoSkillAction> {
    match parse_ok(command).get(stage, turn) {
        Ok(actions) => actions.to_vec(),
        Err(error) => panic!("{command:?} has no action list at {stage},{turn}: {error}"),
    }
}

pub fn is_parse_error(command: &str) -> bool {
    matches!(
        AutoSkillCommand::parse(command),
        Err(ModelError::SkillCommandParse(_))
    )
}

fn target(target: Option<&ServantTarget>) -> String {
    match target {
        None => "null".to_string(),
        Some(target) if !target.special_target().is_empty() => target.special_target().to_string(),
        Some(target) => target.auto_skill_code().to_string(),
    }
}

pub fn render_action(action: &AutoSkillAction) -> String {
    match action {
        AutoSkillAction::ShuffleUntil(shuffle) => {
            let conditions: Vec<String> = shuffle
                .conditions
                .iter()
                .map(|c| c.to_command_string())
                .collect();
            format!(
                "SU({},{},{},[{}],{})",
                shuffle.skill.auto_skill_code(),
                shuffle.min,
                shuffle.max,
                conditions.join(","),
                shuffle.stop_if_unmet
            )
        }
        AutoSkillAction::Atk(atk) => {
            let nps: Vec<String> = atk
                .nps
                .iter()
                .map(|np| np.auto_skill_code().to_string())
                .collect();
            let direct = atk
                .direct_cards
                .as_ref()
                .map_or("null".to_string(), ToString::to_string);
            format!(
                "Atk([{}],{},{},{})",
                nps.join(","),
                atk.cards_before_np,
                direct,
                atk.to_command_string()
            )
        }
        AutoSkillAction::ServantSkill(skill) => {
            let targets: Vec<String> = skill.targets.iter().map(|t| target(Some(t))).collect();
            format!(
                "S({},[{}])",
                skill.skill.auto_skill_code(),
                targets.join(",")
            )
        }
        AutoSkillAction::MasterSkill(skill) => {
            format!(
                "M({},{})",
                skill.skill.auto_skill_code(),
                target(skill.target.as_ref())
            )
        }
        AutoSkillAction::TargetEnemy(enemy) => format!("T({})", enemy.enemy.auto_skill_code()),
        AutoSkillAction::CommandSpell(spell) => {
            format!(
                "C({},{})",
                spell.skill.auto_skill_code(),
                target(spell.target.as_ref())
            )
        }
        AutoSkillAction::OrderChange(change) => format!(
            "O({},{})",
            change.starting.auto_skill_code(),
            change.sub.auto_skill_code()
        ),
    }
}

pub fn render(command: &AutoSkillCommand) -> String {
    let stages: Vec<String> = command
        .stages()
        .iter()
        .map(|turns| {
            let turns: Vec<String> = turns
                .iter()
                .map(|actions| {
                    let actions: Vec<String> = actions.iter().map(render_action).collect();
                    actions.join(";")
                })
                .collect();
            turns.join("|")
        })
        .collect();
    stages.join("/")
}

pub fn card(
    face: Face,
    servant: TeamSlot,
    slot: FieldSlot,
    card_type: CardTypeEnum,
    affinity: CardAffinityEnum,
) -> ParsedCard {
    ParsedCard {
        affinity,
        ..ParsedCard::new(face, servant, Some(slot), card_type)
    }
}

pub fn hand() -> Vec<ParsedCard> {
    vec![
        card(
            Face::A,
            TeamSlot::A,
            FieldSlot::A,
            CardTypeEnum::Buster,
            CardAffinityEnum::Resist,
        ),
        card(
            Face::B,
            TeamSlot::B,
            FieldSlot::B,
            CardTypeEnum::Arts,
            CardAffinityEnum::Weak,
        ),
        card(
            Face::C,
            TeamSlot::B,
            FieldSlot::B,
            CardTypeEnum::Buster,
            CardAffinityEnum::Normal,
        ),
        card(
            Face::D,
            TeamSlot::A,
            FieldSlot::A,
            CardTypeEnum::Arts,
            CardAffinityEnum::Resist,
        ),
        card(
            Face::E,
            TeamSlot::C,
            FieldSlot::C,
            CardTypeEnum::Quick,
            CardAffinityEnum::Weak,
        ),
    ]
}

pub fn faces(cards: &[CommandCard]) -> Vec<CommandCard> {
    cards.to_vec()
}

pub const FACE_A: CommandCard = CommandCard::Face(Face::A);
pub const FACE_B: CommandCard = CommandCard::Face(Face::B);
pub const FACE_C: CommandCard = CommandCard::Face(Face::C);
pub const FACE_D: CommandCard = CommandCard::Face(Face::D);
pub const FACE_E: CommandCard = CommandCard::Face(Face::E);
pub const NP_A: CommandCard = CommandCard::NP(NP::A);
pub const NP_B: CommandCard = CommandCard::NP(NP::B);
pub const NP_C: CommandCard = CommandCard::NP(NP::C);

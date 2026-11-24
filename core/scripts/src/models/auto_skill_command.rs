use std::collections::VecDeque;

use super::auto_skill_action::{
    Atk, AutoSkillAction, CommandSpell, MasterSkill, OrderChange, ServantSkill, ShuffleCondition,
    ShuffleUntil, TargetEnemy,
};
use super::command_card::NP;
use super::direct_card_sequence::DirectCardSequence;
use super::enemy_target::EnemyTarget;
use super::model_error::ModelError;
use super::order_change_member::{Starting, Sub};
use super::servant_target::ServantTarget;
use super::skill::{CommandSpell as CommandSpellSkill, Master, Servant};
use super::special_command::SpecialCommand;
use super::wave_turn::WaveTurn;
use crate::kotlin::to_int;

type Queue = VecDeque<char>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutoSkillCommand {
    stages: Vec<Vec<Vec<AutoSkillAction>>>,
}

fn remove(queue: &mut Queue) -> Result<char, ModelError> {
    queue
        .pop_front()
        .ok_or_else(|| ModelError::NoSuchElement("Skill command ended unexpectedly".to_string()))
}

fn special_target_named(name: &str) -> Option<ServantTarget> {
    ServantTarget::LIST
        .iter()
        .copied()
        .filter(|target| !target.special_target().is_empty())
        .find(|target| target.special_target() == name)
}

fn target_with_code(code: char) -> Option<ServantTarget> {
    ServantTarget::LIST
        .iter()
        .copied()
        .find(|target| target.auto_skill_code() == code)
}

fn get_target(queue: &mut Queue) -> Result<Option<ServantTarget>, ModelError> {
    let peek_target = queue.front().copied();
    if peek_target == Some(SpecialCommand::StartSpecialTarget.auto_skill_code()) {
        queue.pop_front();
        let mut special = String::new();
        let mut last: Option<char> = None;
        let mut target: Option<ServantTarget> = None;
        while let Some(c) = queue.pop_front() {
            last = Some(c);
            if c == SpecialCommand::EndSpecialTarget.auto_skill_code() {
                break;
            }
            if c != SpecialCommand::StartSpecialTarget.auto_skill_code() {
                special.push(c);
            }
            target = special_target_named(&special);
        }
        if last != Some(SpecialCommand::EndSpecialTarget.auto_skill_code()) {
            return Err(ModelError::illegal_argument(
                "Found [ but no matching ] in Skill Command",
            ));
        }
        if special.is_empty() {
            return Err(ModelError::illegal_argument("Command Can't be empty"));
        }
        match target {
            Some(found) => Ok(Some(found)),
            None => Err(ModelError::illegal_argument(format!(
                "Special target \"{special}\" not found"
            ))),
        }
    } else {
        let target = peek_target.and_then(target_with_code);
        if target.is_some() {
            queue.pop_front();
        }
        Ok(target)
    }
}

fn get_targets(queue: &mut Queue) -> Result<Vec<ServantTarget>, ModelError> {
    let mut targets = Vec::new();
    if queue.front().copied() == Some(SpecialCommand::StartMultiTarget.auto_skill_code()) {
        queue.pop_front();
        let mut last: Option<char> = None;
        let mut special_found = false;
        let mut special = String::new();
        while let Some(c) = queue.pop_front() {
            last = Some(c);
            if c == SpecialCommand::EndMultiTarget.auto_skill_code() {
                break;
            }
            if c == SpecialCommand::StartSpecialTarget.auto_skill_code() {
                special_found = true;
            } else if c == SpecialCommand::EndSpecialTarget.auto_skill_code() {
                match special_target_named(&special) {
                    Some(target) => {
                        targets.push(target);
                        special.clear();
                    }
                    None if special.is_empty() => {
                        return Err(ModelError::illegal_argument("Command Can't be empty"));
                    }
                    None => {
                        return Err(ModelError::illegal_argument(format!(
                            "Special target \"{special}\" not found"
                        )));
                    }
                }
                special_found = false;
            }
            if special_found {
                if c != SpecialCommand::StartSpecialTarget.auto_skill_code() {
                    special.push(c);
                }
            } else if let Some(target) = target_with_code(c) {
                targets.push(target);
            }
        }
        if last != Some(SpecialCommand::EndMultiTarget.auto_skill_code()) {
            return Err(ModelError::illegal_argument(
                "Found ( but no matching ) in Skill Command",
            ));
        }
        if special_found {
            return Err(ModelError::illegal_argument(
                "Found [ but no matching ] in Skill Command",
            ));
        }
    } else if let Some(target) = get_target(queue)? {
        targets.push(target);
    }
    Ok(targets)
}

fn read_enemy_target_code(queue: &mut Queue) -> Result<String, ModelError> {
    let first = remove(queue)?;
    if first == '1' && matches!(queue.front(), Some('0' | '1' | '2')) {
        let second = remove(queue)?;
        return Ok(match second {
            '0' => "A",
            '1' => "B",
            _ => "C",
        }
        .to_string());
    }
    Ok(first.to_string())
}

struct ShuffleBody {
    min: Option<String>,
    max: String,
    conditions: String,
    stop_if_unmet: bool,
}

fn take_digits(chars: &[char], index: &mut usize) -> String {
    let start = *index;
    while chars.get(*index).is_some_and(char::is_ascii_digit) {
        *index += 1;
    }
    chars[start..*index].iter().collect()
}

fn take_condition(chars: &[char], index: &mut usize) -> bool {
    if !chars.get(*index).is_some_and(|c| ('1'..='3').contains(c)) {
        return false;
    }
    *index += 1;
    if chars
        .get(*index)
        .is_some_and(|c| matches!(c.to_ascii_lowercase(), 'b' | 'a' | 'q'))
    {
        *index += 1;
    }
    if !chars.get(*index).is_some_and(|c| ('1'..='5').contains(c)) {
        return false;
    }
    *index += 1;
    true
}

fn match_shuffle_body(body: &str) -> Option<ShuffleBody> {
    let chars: Vec<char> = body.chars().collect();
    let mut index = 0;
    let first = take_digits(&chars, &mut index);
    if first.is_empty() {
        return None;
    }
    let (min, max) = if chars.get(index) == Some(&'-') {
        index += 1;
        let second = take_digits(&chars, &mut index);
        if second.is_empty() {
            return None;
        }
        (Some(first), second)
    } else {
        (None, first)
    };
    if chars.get(index) != Some(&':') {
        return None;
    }
    index += 1;
    let conditions_start = index;
    if !take_condition(&chars, &mut index) {
        return None;
    }
    while chars.get(index) == Some(&'|') {
        index += 1;
        if !take_condition(&chars, &mut index) {
            return None;
        }
    }
    let conditions: String = chars[conditions_start..index].iter().collect();
    let stop_if_unmet = chars.get(index) == Some(&'!');
    if stop_if_unmet {
        index += 1;
    }
    if index != chars.len() {
        return None;
    }
    Some(ShuffleBody {
        min,
        max,
        conditions,
        stop_if_unmet,
    })
}

fn parse_shuffle(skill: Servant, queue: &mut Queue) -> Result<AutoSkillAction, ModelError> {
    remove(queue)?;
    if queue.pop_front() != Some('{') {
        return Err(ModelError::illegal_argument("Expected { after shuffle H"));
    }
    let mut body = String::new();
    while let Some(c) = queue.front().copied() {
        if c == '}' {
            break;
        }
        body.push(c);
        queue.pop_front();
    }
    if queue.pop_front() != Some('}') {
        return Err(ModelError::illegal_argument("Unclosed shuffle token"));
    }
    let matched = match_shuffle_body(&body)
        .ok_or_else(|| ModelError::illegal_argument(format!("Invalid shuffle token: {body}")))?;
    let min = match &matched.min {
        Some(text) => to_int(text)?,
        None => 0,
    };
    let max = to_int(&matched.max)?;
    let conditions = matched
        .conditions
        .split('|')
        .map(ShuffleCondition::from_code)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(AutoSkillAction::ShuffleUntil(ShuffleUntil::new(
        skill,
        min,
        max,
        conditions,
        matched.stop_if_unmet,
    )?))
}

fn no_matching_element() -> ModelError {
    ModelError::NoSuchElement("Collection contains no element matching the predicate.".to_string())
}

fn parse_action_inner(queue: &mut Queue) -> Result<AutoSkillAction, ModelError> {
    let c = remove(queue)?;
    if let Some(skill) = Servant::LIST
        .iter()
        .copied()
        .find(|s| s.auto_skill_code() == c)
    {
        if queue.front() == Some(&'{') {
            return Err(ModelError::illegal_argument("Shuffle requires H before {"));
        }
        if queue.front() == Some(&'H') {
            return parse_shuffle(skill, queue);
        }
        let targets = get_targets(queue)?;
        return Ok(AutoSkillAction::ServantSkill(ServantSkill {
            skill,
            targets,
        }));
    }
    if let Some(skill) = Master::LIST
        .iter()
        .copied()
        .find(|s| s.auto_skill_code() == c)
    {
        let target = get_target(queue)?;
        return Ok(AutoSkillAction::MasterSkill(MasterSkill { skill, target }));
    }
    if let Some(skill) = CommandSpellSkill::LIST
        .iter()
        .copied()
        .find(|s| s.auto_skill_code() == c)
    {
        let target = get_target(queue)?;
        return Ok(AutoSkillAction::CommandSpell(CommandSpell {
            skill,
            target,
        }));
    }
    if let Some(np) = NP::LIST.iter().copied().find(|n| n.auto_skill_code() == c) {
        return Ok(AutoSkillAction::Atk(Atk::np(np)));
    }
    if c == SpecialCommand::EnemyTarget.auto_skill_code() {
        let code = read_enemy_target_code(queue)?;
        let enemy = EnemyTarget::from_code(&code).ok_or_else(|| {
            ModelError::illegal_argument(format!("Unknown enemy target: t{code}"))
        })?;
        return Ok(AutoSkillAction::TargetEnemy(TargetEnemy { enemy }));
    }
    if c == SpecialCommand::CardsBeforeNP.auto_skill_code() {
        let code = remove(queue)?;
        let count = to_int(&code.to_string())?;
        return Ok(AutoSkillAction::Atk(Atk::cards_before_np(count)?));
    }
    if c == SpecialCommand::DirectCards.auto_skill_code() {
        let mut sequence = String::new();
        if queue.front() == Some(&'?') {
            sequence.push(remove(queue)?);
        }
        for _ in 0..3 {
            sequence.push(remove(queue)?);
        }
        if !queue.is_empty() {
            return Err(ModelError::illegal_argument(
                "Direct attack must be the last command in the turn",
            ));
        }
        return Ok(AutoSkillAction::Atk(Atk::direct(
            DirectCardSequence::parse(&sequence)?,
        )?));
    }
    if c == SpecialCommand::OrderChange.auto_skill_code() {
        let starting_code = remove(queue)?;
        let starting = Starting::LIST
            .iter()
            .copied()
            .find(|member| member.auto_skill_code() == starting_code)
            .ok_or_else(no_matching_element)?;
        let sub_code = remove(queue)?;
        let sub = Sub::LIST
            .iter()
            .copied()
            .find(|member| member.auto_skill_code() == sub_code)
            .ok_or_else(no_matching_element)?;
        return Ok(AutoSkillAction::OrderChange(OrderChange { starting, sub }));
    }
    if c == SpecialCommand::NoOp.auto_skill_code() {
        return Ok(AutoSkillAction::Atk(Atk::no_op()));
    }
    Err(ModelError::illegal_argument(format!(
        "Unknown character: {c}"
    )))
}

fn parse_action(queue: &mut Queue) -> Result<AutoSkillAction, ModelError> {
    parse_action_inner(queue).map_err(|error| ModelError::SkillCommandParse(error.to_string()))
}

fn index_error(index: i32, len: usize) -> ModelError {
    ModelError::IndexOutOfBounds(format!("Index {index} out of bounds for length {len}"))
}

impl AutoSkillCommand {
    pub fn stages(&self) -> &[Vec<Vec<AutoSkillAction>>] {
        &self.stages
    }

    pub fn get(&self, stage: i32, turn: i32) -> Result<&[AutoSkillAction], ModelError> {
        if usize::try_from(stage).map_or(true, |s| s < self.stages.len()) {
            let turns = usize::try_from(stage)
                .ok()
                .and_then(|s| self.stages.get(s))
                .ok_or_else(|| index_error(stage, self.stages.len()))?;
            if usize::try_from(turn).map_or(true, |t| t < turns.len()) {
                return usize::try_from(turn)
                    .ok()
                    .and_then(|t| turns.get(t))
                    .map(Vec::as_slice)
                    .ok_or_else(|| index_error(turn, turns.len()));
            }
        }
        Ok(&[])
    }

    pub fn parse(command: &str) -> Result<AutoSkillCommand, ModelError> {
        let mut command_table = Vec::new();
        for wave in command.split(WaveTurn::Wave.code()) {
            let mut turns = Vec::new();
            for cmd in wave.split(WaveTurn::Turn.code()) {
                let mut queue: Queue = cmd.chars().collect();
                let mut actions: Vec<AutoSkillAction> = Vec::new();
                while !queue.is_empty() {
                    let action = parse_action(&mut queue)?;
                    let no_op = Atk::no_op();

                    if let AutoSkillAction::Atk(atk) = &action
                        && atk.direct_cards.is_some()
                    {
                        if actions
                            .iter()
                            .any(|a| matches!(a, AutoSkillAction::Atk(other) if *other != no_op))
                        {
                            return Err(ModelError::SkillCommandParse(
                                "Direct and automatic attacks cannot share a turn".to_string(),
                            ));
                        }
                        actions.retain(
                            |a| !matches!(a, AutoSkillAction::Atk(other) if *other == no_op),
                        );
                    }

                    if let AutoSkillAction::Atk(atk) = &action
                        && let Some(AutoSkillAction::Atk(last)) = actions.last()
                    {
                        let merged = last.plus(atk)?;
                        let last_index = actions.len() - 1;
                        actions[last_index] = AutoSkillAction::Atk(merged);
                        continue;
                    }

                    actions.push(action);
                }
                turns.push(actions);
            }
            command_table.push(turns);
        }
        Ok(AutoSkillCommand {
            stages: command_table,
        })
    }
}

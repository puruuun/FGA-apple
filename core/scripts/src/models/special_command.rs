#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecialCommand {
    EnemyTarget,
    OrderChange,
    CardsBeforeNP,
    DirectCards,
    NoOp,
    StartMultiTarget,
    EndMultiTarget,
    StartSpecialTarget,
    EndSpecialTarget,
}

impl SpecialCommand {
    pub fn auto_skill_code(self) -> char {
        match self {
            SpecialCommand::EnemyTarget => 't',
            SpecialCommand::OrderChange => 'x',
            SpecialCommand::CardsBeforeNP => 'n',
            SpecialCommand::DirectCards => '&',
            SpecialCommand::NoOp => '0',
            SpecialCommand::StartMultiTarget => '(',
            SpecialCommand::EndMultiTarget => ')',
            SpecialCommand::StartSpecialTarget => '[',
            SpecialCommand::EndSpecialTarget => ']',
        }
    }
}

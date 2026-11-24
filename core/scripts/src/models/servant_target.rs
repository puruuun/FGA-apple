use super::special_command::SpecialCommand;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ServantTarget {
    A,
    B,
    C,
    Left,
    Right,
    Option1,
    Option2,
    Transform,
    SpecialTarget(SpecialTarget),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SpecialTarget {
    Choice2OptionA,
    Choice2OptionB,
    Choice3OptionA,
    Choice3OptionB,
    Choice3OptionC,
}

impl SpecialTarget {
    pub const LIST: [SpecialTarget; 5] = [
        SpecialTarget::Choice2OptionA,
        SpecialTarget::Choice2OptionB,
        SpecialTarget::Choice3OptionA,
        SpecialTarget::Choice3OptionB,
        SpecialTarget::Choice3OptionC,
    ];

    pub fn special_target(self) -> &'static str {
        match self {
            SpecialTarget::Choice2OptionA => "Ch2A",
            SpecialTarget::Choice2OptionB => "Ch2B",
            SpecialTarget::Choice3OptionA => "Ch3A",
            SpecialTarget::Choice3OptionB => "Ch3B",
            SpecialTarget::Choice3OptionC => "Ch3C",
        }
    }
}

impl ServantTarget {
    pub const LIST: [ServantTarget; 13] = [
        ServantTarget::A,
        ServantTarget::B,
        ServantTarget::C,
        ServantTarget::Left,
        ServantTarget::Right,
        ServantTarget::Option1,
        ServantTarget::Option2,
        ServantTarget::Transform,
        ServantTarget::SpecialTarget(SpecialTarget::Choice2OptionA),
        ServantTarget::SpecialTarget(SpecialTarget::Choice2OptionB),
        ServantTarget::SpecialTarget(SpecialTarget::Choice3OptionA),
        ServantTarget::SpecialTarget(SpecialTarget::Choice3OptionB),
        ServantTarget::SpecialTarget(SpecialTarget::Choice3OptionC),
    ];

    pub fn auto_skill_code(self) -> char {
        match self {
            ServantTarget::A => '1',
            ServantTarget::B => '2',
            ServantTarget::C => '3',
            ServantTarget::Left => '7',
            ServantTarget::Right => '8',
            ServantTarget::Option1 => 'K',
            ServantTarget::Option2 => 'U',
            ServantTarget::Transform => 'M',
            ServantTarget::SpecialTarget(_) => SpecialCommand::StartSpecialTarget.auto_skill_code(),
        }
    }

    pub fn special_target(self) -> &'static str {
        match self {
            ServantTarget::SpecialTarget(target) => target.special_target(),
            _ => "",
        }
    }
}

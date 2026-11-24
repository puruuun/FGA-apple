#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Skill {
    Servant(Servant),
    Master(Master),
    CommandSpell(CommandSpell),
}

impl Skill {
    pub fn auto_skill_code(self) -> char {
        match self {
            Skill::Servant(skill) => skill.auto_skill_code(),
            Skill::Master(skill) => skill.auto_skill_code(),
            Skill::CommandSpell(skill) => skill.auto_skill_code(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Servant {
    A1,
    A2,
    A3,
    B1,
    B2,
    B3,
    C1,
    C2,
    C3,
}

impl Servant {
    pub const LIST: [Servant; 9] = [
        Servant::A1,
        Servant::A2,
        Servant::A3,
        Servant::B1,
        Servant::B2,
        Servant::B3,
        Servant::C1,
        Servant::C2,
        Servant::C3,
    ];

    pub const SKILL1: [Servant; 3] = [Servant::A1, Servant::B1, Servant::C1];
    pub const SKILL2: [Servant; 3] = [Servant::A2, Servant::B2, Servant::C2];
    pub const SKILL3: [Servant; 3] = [Servant::A3, Servant::B3, Servant::C3];

    pub fn auto_skill_code(self) -> char {
        match self {
            Servant::A1 => 'a',
            Servant::A2 => 'b',
            Servant::A3 => 'c',
            Servant::B1 => 'd',
            Servant::B2 => 'e',
            Servant::B3 => 'f',
            Servant::C1 => 'g',
            Servant::C2 => 'h',
            Servant::C3 => 'i',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Master {
    A,
    B,
    C,
}

impl Master {
    pub const LIST: [Master; 3] = [Master::A, Master::B, Master::C];

    pub fn auto_skill_code(self) -> char {
        match self {
            Master::A => 'j',
            Master::B => 'k',
            Master::C => 'l',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandSpell {
    NpCharge,
}

impl CommandSpell {
    pub const LIST: [CommandSpell; 1] = [CommandSpell::NpCharge];

    pub fn auto_skill_code(self) -> char {
        match self {
            CommandSpell::NpCharge => 'o',
        }
    }
}

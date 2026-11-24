#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrderChangeMember {
    Starting(Starting),
    Sub(Sub),
}

impl OrderChangeMember {
    pub fn auto_skill_code(self) -> char {
        match self {
            OrderChangeMember::Starting(member) => member.auto_skill_code(),
            OrderChangeMember::Sub(member) => member.auto_skill_code(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Starting {
    A,
    B,
    C,
}

impl Starting {
    pub const LIST: [Starting; 3] = [Starting::A, Starting::B, Starting::C];

    pub fn auto_skill_code(self) -> char {
        match self {
            Starting::A => '1',
            Starting::B => '2',
            Starting::C => '3',
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sub {
    A,
    B,
    C,
}

impl Sub {
    pub const LIST: [Sub; 3] = [Sub::A, Sub::B, Sub::C];

    pub fn auto_skill_code(self) -> char {
        match self {
            Sub::A => '1',
            Sub::B => '2',
            Sub::C => '3',
        }
    }
}

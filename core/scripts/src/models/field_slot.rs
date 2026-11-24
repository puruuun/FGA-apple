use std::fmt;

use super::skill::Servant;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FieldSlot {
    A,
    B,
    C,
}

impl FieldSlot {
    pub const LIST: [FieldSlot; 3] = [FieldSlot::A, FieldSlot::B, FieldSlot::C];

    pub fn position(self) -> i32 {
        match self {
            FieldSlot::A => 1,
            FieldSlot::B => 2,
            FieldSlot::C => 3,
        }
    }

    pub fn skill1(self) -> Servant {
        match self {
            FieldSlot::A => Servant::A1,
            FieldSlot::B => Servant::B1,
            FieldSlot::C => Servant::C1,
        }
    }

    pub fn skill2(self) -> Servant {
        match self {
            FieldSlot::A => Servant::A2,
            FieldSlot::B => Servant::B2,
            FieldSlot::C => Servant::C2,
        }
    }

    pub fn skill3(self) -> Servant {
        match self {
            FieldSlot::A => Servant::A3,
            FieldSlot::B => Servant::B3,
            FieldSlot::C => Servant::C3,
        }
    }

    pub fn skills(self) -> [Servant; 3] {
        [self.skill1(), self.skill2(), self.skill3()]
    }
}

impl fmt::Display for FieldSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.position())
    }
}

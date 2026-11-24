use std::fmt;

use super::field_slot::FieldSlot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CommandCard {
    Face(Face),
    NP(NP),
}

impl fmt::Display for CommandCard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandCard::Face(face) => face.fmt(f),
            CommandCard::NP(np) => np.fmt(f),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Face {
    A,
    B,
    C,
    D,
    E,
}

impl Face {
    pub const LIST: [Face; 5] = [Face::A, Face::B, Face::C, Face::D, Face::E];

    pub fn index(self) -> i32 {
        match self {
            Face::A => 1,
            Face::B => 2,
            Face::C => 3,
            Face::D => 4,
            Face::E => 5,
        }
    }
}

impl fmt::Display for Face {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.index())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NP {
    A,
    B,
    C,
}

impl NP {
    pub const LIST: [NP; 3] = [NP::A, NP::B, NP::C];

    pub fn auto_skill_code(self) -> char {
        match self {
            NP::A => '4',
            NP::B => '5',
            NP::C => '6',
        }
    }

    pub fn to_field_slot(self) -> FieldSlot {
        match self {
            NP::A => FieldSlot::A,
            NP::B => FieldSlot::B,
            NP::C => FieldSlot::C,
        }
    }
}

impl fmt::Display for NP {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.auto_skill_code())
    }
}

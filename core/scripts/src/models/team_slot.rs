use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TeamSlot {
    A,
    B,
    C,
    D,
    E,
    F,
    Unknown,
}

impl TeamSlot {
    pub const LIST: [TeamSlot; 6] = [
        TeamSlot::A,
        TeamSlot::B,
        TeamSlot::C,
        TeamSlot::D,
        TeamSlot::E,
        TeamSlot::F,
    ];

    pub fn position(self) -> i32 {
        match self {
            TeamSlot::A => 1,
            TeamSlot::B => 2,
            TeamSlot::C => 3,
            TeamSlot::D => 4,
            TeamSlot::E => 5,
            TeamSlot::F => 6,
            TeamSlot::Unknown => 0,
        }
    }
}

impl fmt::Display for TeamSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}]", self.position())
    }
}

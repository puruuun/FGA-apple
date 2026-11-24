#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CommandPosition {
    pub stage: i32,
    pub turn: i32,
    pub total_turn: i32,
}

impl CommandPosition {
    pub fn new(stage: i32, turn: i32, total_turn: i32) -> Self {
        Self {
            stage,
            turn,
            total_turn,
        }
    }

    pub fn at(stage: i32, turn: i32) -> Self {
        Self::new(stage, turn, 0)
    }
}

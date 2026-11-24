#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BondValue {
    Gain(Gain),
    Max,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Gain {
    pub base: i32,
    pub bonus: i32,
}

impl Gain {
    pub fn total(self) -> i64 {
        i64::from(self.base) + i64::from(self.bonus)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum BondFrame {
    Normal(Normal),
    LevelUp(LevelUp),
    Other,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Normal {
    pub values: Vec<BondValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct LevelUp {
    pub reached_level: Option<i32>,
    pub result_index: Option<i32>,
    pub at_least_ten: Option<bool>,
}

impl LevelUp {
    pub fn new(reached_level: Option<i32>, result_index: Option<i32>) -> Self {
        Self {
            reached_level,
            result_index,
            at_least_ten: reached_level.map(|level| level >= 10),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BondSlotSummary {
    pub position: i32,
    pub total: i64,
    pub read_runs: i32,
    pub missed_runs: i32,
    pub max_observed: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BondSummary {
    pub slots: Vec<BondSlotSummary>,
    pub unmapped_level_up_runs: i32,
    pub level_up_missed_runs: i32,
}

impl BondSummary {
    pub fn new(slots: Vec<BondSlotSummary>, unmapped_level_up_runs: i32) -> Self {
        Self {
            slots,
            unmapped_level_up_runs,
            level_up_missed_runs: 0,
        }
    }
}

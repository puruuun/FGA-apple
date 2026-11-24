#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SkillLevelMode {
    Skill,
    Append,
}

impl SkillLevelMode {
    pub fn slots(self) -> usize {
        match self {
            SkillLevelMode::Skill => 3,
            SkillLevelMode::Append => 5,
        }
    }
}

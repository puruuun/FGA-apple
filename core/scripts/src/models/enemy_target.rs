#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EnemyTarget {
    A,
    B,
    C,
    SixTopLeft,
    SixTopMiddle,
    SixTopRight,
    SixBottomLeft,
    SixBottomMiddle,
    SixBottomRight,
    RaidBoss,
    RaidLeft,
    RaidMiddle,
    RaidRight,
    RaidLowerLeft,
    RaidLowerMiddle,
    RaidLowerRight,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Normal {
    A,
    B,
    C,
}

impl From<Normal> for EnemyTarget {
    fn from(normal: Normal) -> EnemyTarget {
        match normal {
            Normal::A => EnemyTarget::A,
            Normal::B => EnemyTarget::B,
            Normal::C => EnemyTarget::C,
        }
    }
}

impl EnemyTarget {
    pub const LIST: [Normal; 3] = [Normal::A, Normal::B, Normal::C];

    pub const SIX_LIST: [EnemyTarget; 6] = [
        EnemyTarget::SixTopLeft,
        EnemyTarget::SixTopMiddle,
        EnemyTarget::SixTopRight,
        EnemyTarget::SixBottomLeft,
        EnemyTarget::SixBottomMiddle,
        EnemyTarget::SixBottomRight,
    ];

    pub const RAID_LIST: [EnemyTarget; 7] = [
        EnemyTarget::RaidBoss,
        EnemyTarget::RaidLeft,
        EnemyTarget::RaidMiddle,
        EnemyTarget::RaidRight,
        EnemyTarget::RaidLowerLeft,
        EnemyTarget::RaidLowerMiddle,
        EnemyTarget::RaidLowerRight,
    ];

    pub const ALL: [EnemyTarget; 16] = [
        EnemyTarget::A,
        EnemyTarget::B,
        EnemyTarget::C,
        EnemyTarget::SixTopLeft,
        EnemyTarget::SixTopMiddle,
        EnemyTarget::SixTopRight,
        EnemyTarget::SixBottomLeft,
        EnemyTarget::SixBottomMiddle,
        EnemyTarget::SixBottomRight,
        EnemyTarget::RaidBoss,
        EnemyTarget::RaidLeft,
        EnemyTarget::RaidMiddle,
        EnemyTarget::RaidRight,
        EnemyTarget::RaidLowerLeft,
        EnemyTarget::RaidLowerMiddle,
        EnemyTarget::RaidLowerRight,
    ];

    pub fn auto_skill_code(self) -> &'static str {
        match self {
            EnemyTarget::A => "1",
            EnemyTarget::B => "2",
            EnemyTarget::C => "3",
            EnemyTarget::SixTopLeft => "4",
            EnemyTarget::SixTopMiddle => "5",
            EnemyTarget::SixTopRight => "6",
            EnemyTarget::SixBottomLeft => "7",
            EnemyTarget::SixBottomMiddle => "8",
            EnemyTarget::SixBottomRight => "9",
            EnemyTarget::RaidBoss => "R",
            EnemyTarget::RaidLeft => "A",
            EnemyTarget::RaidMiddle => "B",
            EnemyTarget::RaidRight => "C",
            EnemyTarget::RaidLowerLeft => "D",
            EnemyTarget::RaidLowerMiddle => "E",
            EnemyTarget::RaidLowerRight => "F",
        }
    }

    pub fn is_normal(self) -> bool {
        matches!(self, EnemyTarget::A | EnemyTarget::B | EnemyTarget::C)
    }

    pub fn from_code(code: &str) -> Option<EnemyTarget> {
        EnemyTarget::ALL
            .iter()
            .copied()
            .find(|target| target.auto_skill_code() == code)
    }
}

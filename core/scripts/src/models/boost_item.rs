#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BoostItem {
    Disabled,
    Enabled(Enabled),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Enabled {
    Skip,
    BoostItem1,
    BoostItem2,
    BoostItem3,
}

impl BoostItem {
    pub fn of(value: i32) -> BoostItem {
        match value {
            0 => BoostItem::Enabled(Enabled::Skip),
            1 => BoostItem::Enabled(Enabled::BoostItem1),
            2 => BoostItem::Enabled(Enabled::BoostItem2),
            3 => BoostItem::Enabled(Enabled::BoostItem3),
            _ => BoostItem::Disabled,
        }
    }
}

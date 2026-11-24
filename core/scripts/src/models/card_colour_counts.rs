use std::fmt;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct CardColourCounts {
    pub buster: i32,
    pub arts: i32,
    pub quick: i32,
}

impl fmt::Display for CardColourCounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "B{} A{} Q{}", self.buster, self.arts, self.quick)
    }
}

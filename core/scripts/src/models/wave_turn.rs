#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WaveTurn {
    Wave,
    Turn,
}

impl WaveTurn {
    pub fn code(self) -> &'static str {
        match self {
            WaveTurn::Wave => ",#,",
            WaveTurn::Turn => ",",
        }
    }
}

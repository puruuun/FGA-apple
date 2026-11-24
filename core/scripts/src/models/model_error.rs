#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ModelError {
    #[error("skill command parse error: {0}")]
    SkillCommandParse(String),
    #[error("card priority parse error: {0}")]
    CardPriorityParse(String),
    #[error("{0}")]
    IllegalArgument(String),
    #[error("{0}")]
    IllegalState(String),
    #[error("{0}")]
    NumberFormat(String),
    #[error("{0}")]
    IndexOutOfBounds(String),
    #[error("{0}")]
    NoSuchElement(String),
}

impl ModelError {
    pub fn illegal_argument(message: impl Into<String>) -> ModelError {
        ModelError::IllegalArgument(message.into())
    }
}

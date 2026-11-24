#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum PrefsError {
    #[error("{0}")]
    Serialization(String),
    #[error("{0}")]
    IllegalArgument(String),
}

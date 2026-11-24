use automata::GeometryError;

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum LocationError {
    #[error("{0}")]
    Geometry(#[from] GeometryError),
    #[error("{0}")]
    IllegalArgument(String),
    #[error("{0}")]
    IndexOutOfBounds(String),
    #[error("{0}")]
    Arithmetic(String),
}

pub type LocationResult<T> = Result<T, LocationError>;

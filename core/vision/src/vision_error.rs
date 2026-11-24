use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VisionError {
    PixelCountMismatch { expected: usize, actual: usize },
    SideTooLarge,
    WidthNotPositive,
    HeightNotPositive,
    EmptyRange,
    MaskSizeMismatch,
    MaskedNotSupported,
    EmptyTemplate,
}

impl fmt::Display for VisionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VisionError::PixelCountMismatch { expected, actual } => {
                write!(f, "expected {expected} pixels, got {actual}")
            }
            VisionError::SideTooLarge => f.write_str("image side does not fit in an Int"),
            VisionError::WidthNotPositive => f.write_str("width must be positive"),
            VisionError::HeightNotPositive => f.write_str("height must be positive"),
            VisionError::EmptyRange => f.write_str("cannot coerce value to an empty range"),
            VisionError::MaskSizeMismatch => f.write_str("mask must have the template's size"),
            VisionError::MaskedNotSupported => {
                f.write_str("masked template: operation not supported")
            }
            VisionError::EmptyTemplate => f.write_str("template is empty"),
        }
    }
}

impl std::error::Error for VisionError {}

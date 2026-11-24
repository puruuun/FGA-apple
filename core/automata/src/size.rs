use std::ops::Mul;

use crate::kotlin_math::{GeometryError, round_to_int};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Size {
    pub(crate) width: i32,
    pub(crate) height: i32,
}

impl Size {
    pub fn new(width: i32, height: i32) -> Result<Self, GeometryError> {
        if width <= 0 {
            return Err(GeometryError::WidthNotPositive);
        }
        if height <= 0 {
            return Err(GeometryError::HeightNotPositive);
        }
        Ok(Self { width, height })
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }
}

impl Mul<f64> for Size {
    type Output = Result<Size, GeometryError>;

    fn mul(self, scale: f64) -> Self::Output {
        Size::new(
            round_to_int(f64::from(self.width) * scale)?,
            round_to_int(f64::from(self.height) * scale)?,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_positive_sides() {
        assert_eq!(Size::new(0, 1), Err(GeometryError::WidthNotPositive));
        assert_eq!(Size::new(1, -1), Err(GeometryError::HeightNotPositive));
        assert!(Size::new(1, 1).is_ok());
    }

    #[test]
    fn times_rounds_like_kotlin() {
        assert_eq!(Size::new(1280, 720).unwrap() * 0.3, Size::new(384, 216));
        assert_eq!(Size::new(5, 3).unwrap() * 0.5, Size::new(3, 2));
    }

    #[test]
    fn times_rejects_zero_and_nan() {
        assert_eq!(
            Size::new(1, 1).unwrap() * 0.2,
            Err(GeometryError::WidthNotPositive)
        );
        assert_eq!(
            Size::new(2, 2).unwrap() * f64::NAN,
            Err(GeometryError::NotANumber)
        );
    }
}

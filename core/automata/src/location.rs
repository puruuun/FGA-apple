use std::cmp::Ordering;
use std::ops::{Add, Mul, Sub};

use crate::kotlin_math::{GeometryError, round_to_int};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Location {
    pub x: i32,
    pub y: i32,
}

impl Location {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn compare_to(&self, other: &Location) -> Ordering {
        if self.y > other.y {
            Ordering::Greater
        } else if self.y < other.y {
            Ordering::Less
        } else if self.x > other.x {
            Ordering::Greater
        } else if self.x < other.x {
            Ordering::Less
        } else {
            Ordering::Equal
        }
    }
}

impl PartialOrd for Location {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Location {
    fn cmp(&self, other: &Self) -> Ordering {
        self.compare_to(other)
    }
}

impl Mul<f64> for Location {
    type Output = Result<Location, GeometryError>;

    fn mul(self, scale: f64) -> Self::Output {
        Ok(Location::new(
            round_to_int(f64::from(self.x) * scale)?,
            round_to_int(f64::from(self.y) * scale)?,
        ))
    }
}

impl Add for Location {
    type Output = Location;

    fn add(self, other: Location) -> Location {
        Location::new(self.x.wrapping_add(other.x), self.y.wrapping_add(other.y))
    }
}

impl Sub for Location {
    type Output = Location;

    fn sub(self, other: Location) -> Location {
        Location::new(self.x.wrapping_sub(other.x), self.y.wrapping_sub(other.y))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn times(x: i32, y: i32, scale: f64) -> Result<Location, GeometryError> {
        Location::new(x, y) * scale
    }

    #[test]
    fn times_rounds_like_kotlin() {
        assert_eq!(times(5, 5, 0.5), Ok(Location::new(3, 3)));
        assert_eq!(times(-5, -5, 0.5), Ok(Location::new(-2, -2)));
        assert_eq!(times(1, -1, 1.5), Ok(Location::new(2, -1)));
        assert_eq!(times(-1, -3, 1.5), Ok(Location::new(-1, -4)));
        assert_eq!(times(1, 1, 0.49999999999999994), Ok(Location::new(0, 0)));
        assert_eq!(times(3, 7, 0.25), Ok(Location::new(1, 2)));
        assert_eq!(times(2, -3, 1.0 / 3.0), Ok(Location::new(1, -1)));
        assert_eq!(times(1, -1, 0.0), Ok(Location::new(0, 0)));
    }

    #[test]
    fn times_saturates_and_rejects_nan() {
        assert_eq!(
            times(i32::MAX, i32::MIN, 2.0),
            Ok(Location::new(i32::MAX, i32::MIN))
        );
        assert_eq!(times(1, 1, f64::NAN), Err(GeometryError::NotANumber));
    }

    #[test]
    fn plus_and_minus() {
        assert_eq!(
            Location::new(1, 2) + Location::new(-4, 9),
            Location::new(-3, 11)
        );
        assert_eq!(
            Location::new(1, 2) - Location::new(-4, 9),
            Location::new(5, -7)
        );
        assert_eq!(
            Location::new(i32::MAX, 0) + Location::new(1, 0),
            Location::new(i32::MIN, 0)
        );
        assert_eq!(Location::default(), Location::new(0, 0));
    }

    #[test]
    fn compares_by_y_then_x() {
        assert_eq!(
            Location::new(0, 2).compare_to(&Location::new(5, 1)),
            Ordering::Greater
        );
        assert_eq!(
            Location::new(9, 1).compare_to(&Location::new(0, 2)),
            Ordering::Less
        );
        assert_eq!(
            Location::new(3, 1).compare_to(&Location::new(2, 1)),
            Ordering::Greater
        );
        assert_eq!(
            Location::new(2, 1).compare_to(&Location::new(3, 1)),
            Ordering::Less
        );
        assert_eq!(
            Location::new(2, 1).compare_to(&Location::new(2, 1)),
            Ordering::Equal
        );
        assert!(Location::new(0, 2) > Location::new(5, 1));
    }
}

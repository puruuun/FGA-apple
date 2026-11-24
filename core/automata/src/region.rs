use std::cmp::Ordering;
use std::ops::{Add, Mul, Sub};

use crate::kotlin_math::{GeometryError, coerce_in, round_to_int};
use crate::location::Location;
use crate::size::Size;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Region {
    x: i32,
    y: i32,
    width: i32,
    height: i32,
}

impl Region {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Result<Self, GeometryError> {
        if width <= 0 {
            return Err(GeometryError::WidthNotPositive);
        }
        if height <= 0 {
            return Err(GeometryError::HeightNotPositive);
        }
        Ok(Self {
            x,
            y,
            width,
            height,
        })
    }

    pub fn from_location_size(location: Location, size: Size) -> Self {
        Self {
            x: location.x,
            y: location.y,
            width: size.width(),
            height: size.height(),
        }
    }

    pub fn x(&self) -> i32 {
        self.x
    }

    pub fn y(&self) -> i32 {
        self.y
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn location(&self) -> Location {
        Location::new(self.x, self.y)
    }

    pub fn size(&self) -> Size {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    pub fn center(&self) -> Location {
        Location::new(
            self.x.wrapping_add(self.width / 2),
            self.y.wrapping_add(self.height / 2),
        )
    }

    pub fn right(&self) -> i32 {
        self.x.wrapping_add(self.width)
    }

    pub fn bottom(&self) -> i32 {
        self.y.wrapping_add(self.height)
    }

    pub fn clip(&self, region: &Region) -> Result<Region, GeometryError> {
        let left = coerce_in(region.x, self.x, self.right().wrapping_sub(1))?;
        let right = coerce_in(region.right(), self.x.wrapping_add(1), self.right())?;
        let top = coerce_in(region.y, self.y, self.bottom().wrapping_sub(1))?;
        let bottom = coerce_in(region.bottom(), self.y.wrapping_add(1), self.bottom())?;

        Region::new(
            left,
            top,
            right.wrapping_sub(left),
            bottom.wrapping_sub(top),
        )
    }

    pub fn contains(&self, region: &Region) -> bool {
        self.x <= region.x
            && self.y <= region.y
            && self.right() >= region.right()
            && self.bottom() >= region.bottom()
    }

    pub fn compare_to(&self, other: &Region) -> Ordering {
        self.location().compare_to(&other.location())
    }
}

impl Mul<f64> for Region {
    type Output = Result<Region, GeometryError>;

    fn mul(self, scale: f64) -> Self::Output {
        Region::new(
            round_to_int(f64::from(self.x) * scale)?,
            round_to_int(f64::from(self.y) * scale)?,
            round_to_int(f64::from(self.width) * scale)?,
            round_to_int(f64::from(self.height) * scale)?,
        )
    }
}

impl Add<Location> for Region {
    type Output = Region;

    fn add(self, location: Location) -> Region {
        Region::from_location_size(self.location() + location, self.size())
    }
}

impl Sub<Location> for Region {
    type Output = Region;

    fn sub(self, location: Location) -> Region {
        Region::from_location_size(self.location() - location, self.size())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(x: i32, y: i32, width: i32, height: i32) -> Region {
        Region::new(x, y, width, height).unwrap()
    }

    fn base() -> Region {
        region(100, 100, 200, 100)
    }

    #[test]
    fn rejects_non_positive_sides() {
        assert_eq!(
            Region::new(0, 0, 0, 1),
            Err(GeometryError::WidthNotPositive)
        );
        assert_eq!(
            Region::new(0, 0, 1, -5),
            Err(GeometryError::HeightNotPositive)
        );
    }

    #[test]
    fn times_rounds_like_kotlin() {
        assert_eq!(region(3, 5, 7, 9) * 0.5, Ok(region(2, 3, 4, 5)));
        assert_eq!(region(-3, -5, 7, 9) * 0.5, Ok(region(-1, -2, 4, 5)));
        assert_eq!(
            region(1, 1, 1, 1) * 0.2,
            Err(GeometryError::WidthNotPositive)
        );
    }

    #[test]
    fn plus_and_minus_move_the_location() {
        assert_eq!(
            region(1, 2, 3, 4) + Location::new(10, -20),
            region(11, -18, 3, 4)
        );
        assert_eq!(
            region(1, 2, 3, 4) - Location::new(10, -20),
            region(-9, 22, 3, 4)
        );
    }

    #[test]
    fn location_and_size_round_trip() {
        let built = Region::from_location_size(Location::new(1, 2), Size::new(3, 4).unwrap());
        assert_eq!(built, region(1, 2, 3, 4));
        assert_eq!(built.location(), Location::new(1, 2));
        assert_eq!(built.size(), Size::new(3, 4).unwrap());
    }

    #[test]
    fn center_uses_integer_division() {
        assert_eq!(region(10, 20, 100, 50).center(), Location::new(60, 45));
        assert_eq!(region(10, 20, 101, 51).center(), Location::new(60, 45));
        assert_eq!(region(-10, -20, 101, 51).center(), Location::new(40, 5));
    }

    #[test]
    fn right_and_bottom() {
        let r = region(10, 20, 100, 50);
        assert_eq!((r.right(), r.bottom()), (110, 70));
    }

    #[test]
    fn clip_keeps_the_intersection() {
        let b = base();
        assert_eq!(
            b.clip(&region(150, 120, 20, 20)),
            Ok(region(150, 120, 20, 20))
        );
        assert_eq!(
            b.clip(&region(50, 50, 100, 100)),
            Ok(region(100, 100, 50, 50))
        );
        assert_eq!(
            b.clip(&region(250, 150, 200, 200)),
            Ok(region(250, 150, 50, 50))
        );
        assert_eq!(
            b.clip(&region(0, 0, 1000, 1000)),
            Ok(region(100, 100, 200, 100))
        );
    }

    #[test]
    fn clip_without_intersection_is_one_pixel_at_the_nearest_edge() {
        let b = base();
        assert_eq!(b.clip(&region(0, 120, 50, 10)), Ok(region(100, 120, 1, 10)));
        assert_eq!(
            b.clip(&region(400, 120, 50, 10)),
            Ok(region(299, 120, 1, 10))
        );
        assert_eq!(b.clip(&region(120, 0, 10, 50)), Ok(region(120, 100, 10, 1)));
        assert_eq!(
            b.clip(&region(120, 500, 10, 50)),
            Ok(region(120, 199, 10, 1))
        );
        assert_eq!(
            b.clip(&region(50, 120, 50, 10)),
            Ok(region(100, 120, 1, 10))
        );
        assert_eq!(
            b.clip(&region(300, 120, 50, 10)),
            Ok(region(299, 120, 1, 10))
        );
        assert_eq!(
            b.clip(&region(-500, -500, 10, 10)),
            Ok(region(100, 100, 1, 1))
        );
        assert_eq!(
            region(5, 5, 1, 1).clip(&region(0, 0, 100, 100)),
            Ok(region(5, 5, 1, 1))
        );
    }

    #[test]
    fn clip_fails_when_the_right_edge_overflows() {
        assert_eq!(
            region(i32::MAX - 1, 0, 5, 5).clip(&region(0, 0, 1, 1)),
            Err(GeometryError::EmptyRange)
        );
    }

    #[test]
    fn contains_needs_every_edge_inside() {
        let b = base();
        assert!(b.contains(&region(150, 120, 20, 20)));
        assert!(b.contains(&b));
        assert!(!b.contains(&region(99, 120, 20, 20)));
        assert!(!b.contains(&region(290, 120, 20, 20)));
        assert!(!b.contains(&region(150, 99, 20, 20)));
        assert!(!b.contains(&region(150, 190, 20, 20)));
        assert!(!b.contains(&region(0, 0, 1000, 1000)));
    }

    #[test]
    fn compares_by_location_only() {
        assert_eq!(
            region(0, 2, 1, 1).compare_to(&region(5, 1, 9, 9)),
            Ordering::Greater
        );
        assert_eq!(
            region(1, 1, 1, 1).compare_to(&region(2, 1, 9, 9)),
            Ordering::Less
        );
        assert_eq!(
            region(1, 1, 1, 1).compare_to(&region(1, 1, 9, 9)),
            Ordering::Equal
        );
    }
}

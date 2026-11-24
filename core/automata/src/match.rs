use std::cmp::Ordering;

use crate::region::Region;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Match {
    pub region: Region,
    pub score: f64,
}

impl Match {
    pub fn new(region: Region, score: f64) -> Self {
        Self { region, score }
    }

    pub fn compare_to(&self, other: &Match) -> Ordering {
        self.region.compare_to(&other.region)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn region(x: i32, y: i32, width: i32, height: i32) -> Region {
        Region::new(x, y, width, height).unwrap()
    }

    #[test]
    fn compares_by_region_location_only() {
        assert_eq!(
            Match::new(region(0, 2, 1, 1), 0.9).compare_to(&Match::new(region(0, 1, 1, 1), 0.1)),
            Ordering::Greater
        );
        assert_eq!(
            Match::new(region(1, 1, 1, 1), 0.9).compare_to(&Match::new(region(1, 1, 5, 5), 0.1)),
            Ordering::Equal
        );
    }
}

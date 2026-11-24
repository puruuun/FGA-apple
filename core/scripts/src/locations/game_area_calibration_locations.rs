use automata::{Location, Region};

use super::location_error::LocationResult;
use super::script_area_transforms::WIDE_EDGE_INSET;

pub struct GameAreaCalibrationLocations;

impl GameAreaCalibrationLocations {
    pub fn notice720(wide: bool) -> (f64, f64) {
        let it = Location::new(20, 24);
        let inset = if wide {
            f64::from(WIDE_EDGE_INSET) / 2.0
        } else {
            0.0
        };
        (f64::from(it.x) + inset, f64::from(it.y))
    }

    pub fn menu720(wide: bool) -> (f64, f64) {
        let inset = if wide {
            f64::from(WIDE_EDGE_INSET) / 2.0
        } else {
            0.0
        };
        (175.0 + inset, 622.0)
    }

    pub fn menu_search(script_width: i32, wide: bool) -> LocationResult<Region> {
        Ok(if wide {
            Region::new(script_width.wrapping_sub(600), 1200, 600, 240)?
        } else {
            Region::new(script_width.wrapping_sub(460), 1200, 460, 240)?
        })
    }
}

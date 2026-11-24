use automata::{Location, Region};

use super::location_error::LocationResult;
use super::script_area_transforms::{Transforms, delegate_script_area_transforms};

pub struct LotteryLocations {
    transforms: Transforms,
    pub finished_region: Region,
    pub check_region: Region,
    pub spin_click: Location,
    pub full_present_box_region: Region,
    pub done_region: Region,
    pub transition_region: Region,
}

delegate_script_area_transforms!(LotteryLocations, transforms);

impl LotteryLocations {
    pub fn new(transforms: Transforms) -> LocationResult<Self> {
        let t = &*transforms;
        let finished_region = t.x_from_center_region(Region::new(-510, 700, 55, 100)?);
        let check_region = t.x_from_center_region(Region::new(-1130, 700, 675, 330)?);
        let spin_click = t.x_from_center_location(Location::new(-646, 860));
        let full_present_box_region = t.x_from_center_region(Region::new(20, 860, 1000, 500)?);
        let done_region = t.x_from_center_region(Region::new(-510, 700, 60, 90)?);
        let transition_region = t.x_from_center_region(Region::new(1180, 30, 65, 70)?);
        Ok(Self {
            transforms,
            finished_region,
            check_region,
            spin_click,
            full_present_box_region,
            done_region,
            transition_region,
        })
    }
}

use automata::{Location, Region};

use super::location_error::LocationResult;
use super::script_area_transforms::{Transforms, delegate_script_area_transforms};

pub struct CEEnhanceLocations {
    transforms: Transforms,
    pub execute: Region,
    pub auto_fill: Region,
    pub reselect: Region,
    pub filter_warning_confirm_region: Region,
    pub filter_warning_confirm: Location,
}

delegate_script_area_transforms!(CEEnhanceLocations, transforms);

impl CEEnhanceLocations {
    pub fn new(transforms: Transforms) -> LocationResult<Self> {
        let t = &*transforms;
        let execute = t.x_from_center_region(Region::new(675, 1210, 300, 120)?);
        let auto_fill = t.x_from_center_region(Region::new(265, 1013, 100, 80)?);
        let reselect = t.x_from_left_edge_region(Region::new(340, 0, 340, 180)?);
        let filter_warning_confirm_region =
            t.x_from_center_region(Region::new(300, 1085, 260, 90)?);
        let filter_warning_confirm = t.x_from_center_location(Location::new(400, 1130));
        Ok(Self {
            transforms,
            execute,
            auto_fill,
            reselect,
            filter_warning_confirm_region,
            filter_warning_confirm,
        })
    }
}

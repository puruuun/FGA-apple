use automata::{Location, Region};

use super::location_error::LocationResult;
use super::script_area_transforms::{Transforms, delegate_script_area_transforms};

pub struct FPLocations {
    transforms: Transforms,
    pub summon_check: Region,
    pub initial_summon_check: Region,
    pub initial_summon_click: Location,
    pub initial_summon_continue_click: Region,
    pub continue_summon_region: Region,
    pub first10_summon_click: Location,
    pub ok_click: Location,
    pub continue_summon_click: Location,
    pub skip_rapid_click: Location,
}

delegate_script_area_transforms!(FPLocations, transforms);

impl FPLocations {
    pub fn new(transforms: Transforms) -> LocationResult<Self> {
        let t = &*transforms;
        let summon_check = t.x_from_center_region(Region::new(250, 1121, 100, 143)?);
        let initial_summon_check = t.x_from_center_region(Region::new(-305, 1121, 240, 183)?);
        let initial_summon_click = t.x_from_center_location(Location::new(2, 977));
        let initial_summon_continue_click =
            t.x_from_center_region(Region::new(341, 1145, 120, 60)?);
        let continue_summon_region = t.x_from_center_region(Region::new(-36, 1264, 580, 170)?);
        let first10_summon_click = t.x_from_center_location(Location::new(400, 1062));
        let ok_click = t.x_from_center_location(Location::new(320, 1120));
        let continue_summon_click = t.x_from_center_location(Location::new(320, 1325));
        let skip_rapid_click = t.x_from_center_location(Location::new(1240, 1400));
        Ok(Self {
            transforms,
            summon_check,
            initial_summon_check,
            initial_summon_click,
            initial_summon_continue_click,
            continue_summon_region,
            first10_summon_click,
            ok_click,
            continue_summon_click,
            skip_rapid_click,
        })
    }
}

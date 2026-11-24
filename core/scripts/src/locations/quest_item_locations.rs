use automata::Region;

use super::location_error::LocationResult;
use super::script_area_transforms::{Transforms, delegate_script_area_transforms};

pub struct QuestItemLocations {
    transforms: Transforms,
    pub main_control: Region,
    pub dialog_heading: Region,
    pub storm_pod_control: Region,
    pub teapot_control: Region,
    pub dialog_close: Region,
}

delegate_script_area_transforms!(QuestItemLocations, transforms);

impl QuestItemLocations {
    pub fn new(transforms: Transforms) -> LocationResult<Self> {
        let t = &*transforms;
        let main_control = t.y_from_bottom_region(t.x_from_right_region(if t.is_wide() {
            Region::new(-1330, -242, 520, 158)?
        } else {
            Region::new(-1140, -172, 520, 158)?
        }));
        let dialog_heading = t.x_from_center_region(Region::new(-520, 160, 1040, 160)?);
        let storm_pod_control = t.x_from_center_region(Region::new(-250, 730, 200, 164)?);
        let teapot_control = t.x_from_center_region(Region::new(250, 730, 200, 164)?);
        let dialog_close = t.x_from_center_region(Region::new(-310, 1040, 620, 174)?);
        Ok(Self {
            transforms,
            main_control,
            dialog_heading,
            storm_pod_control,
            teapot_control,
            dialog_close,
        })
    }
}

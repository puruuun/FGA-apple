use automata::{Location, Region};

use super::location_error::LocationResult;
use super::script_area_transforms::{
    IScriptAreaTransforms, Transforms, delegate_script_area_transforms,
};

pub struct QuestPanelLocations {
    transforms: Transforms,
    pub enlarged_spot_name: Region,
    pub free_label: Region,
    pub ap_digits: Region,
    pub name: Region,
}

delegate_script_area_transforms!(QuestPanelLocations, transforms);

impl QuestPanelLocations {
    pub const SLOT_PITCH: i32 = 376;
    pub const SLOTS: i32 = 3;

    pub fn new(transforms: Transforms) -> LocationResult<Self> {
        let t = &*transforms;
        let enlarged_spot_name = t.x_from_center_region(Region::new(-880, 936, 700, 64)?);
        let free_label = t.x_from_right_edge_region(Region::new(-1330, 270, 330, 220)?);
        let ap_digits = t.x_from_right_edge_region(Region::new(-922, 410, 100, 76)?);
        let name = t.x_from_right_edge_region(Region::new(-1004, 272, 694, 60)?);
        Ok(Self {
            transforms,
            enlarged_spot_name,
            free_label,
            ap_digits,
            name,
        })
    }

    pub fn free_search_band(&self) -> LocationResult<Region> {
        Ok(Region::new(
            24,
            160,
            self.script_area().width().wrapping_sub(48),
            1160,
        )?)
    }

    fn panel(&self, region: Region) -> Region {
        self.x_from_right_edge_region(region)
    }

    pub fn slot(&self, region: Region, slot: i32) -> Region {
        region + Location::new(0, slot.wrapping_mul(Self::SLOT_PITCH))
    }

    pub fn tap(&self, slot: i32) -> LocationResult<Location> {
        Ok(self
            .panel(Region::new(
                -800,
                384i32.wrapping_add(slot.wrapping_mul(Self::SLOT_PITCH)),
                1,
                1,
            )?)
            .center())
    }
}

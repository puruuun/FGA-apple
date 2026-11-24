use automata::Region;

use super::location_error::LocationResult;
use super::script_area_transforms::{
    IScriptAreaTransforms, Transforms, delegate_script_area_transforms,
};

pub struct BondLocations {
    transforms: Transforms,
    pub result_title: Region,
    pub popup_title: Region,
    pub popup_arrow: Region,
}

delegate_script_area_transforms!(BondLocations, transforms);

impl BondLocations {
    pub fn new(transforms: Transforms) -> LocationResult<Self> {
        let t = &*transforms;
        let result_title = t.x_from_center_region(Region::new(-1200, 280, 1000, 180)?);
        let popup_title = t.x_from_center_region(Region::new(0, 560, 1220, 220)?);
        let popup_arrow = t.x_from_center_region(Region::new(680, 700, 400, 240)?);
        Ok(Self {
            transforms,
            result_title,
            popup_title,
            popup_arrow,
        })
    }

    pub fn reached_level(&self, arrow: Region) -> LocationResult<Region> {
        Ok(Region::new(
            arrow.right().wrapping_add(22),
            arrow.y().wrapping_sub(24),
            180,
            110,
        )?)
    }

    pub fn gain(&self, index: i32) -> LocationResult<Region> {
        Ok(self.x_from_center_region(Region::new(
            (-1115i32).wrapping_add(index.wrapping_mul(450)),
            976,
            430,
            92,
        )?))
    }

    pub fn gain_text(&self, index: i32) -> LocationResult<Region> {
        Ok(self.x_from_center_region(Region::new(
            (-1115i32).wrapping_add(index.wrapping_mul(450)),
            976,
            430,
            76,
        )?))
    }

    pub fn level_up_marker(&self, index: i32) -> LocationResult<Region> {
        Ok(self.x_from_center_region(Region::new(
            (-1125i32).wrapping_add(index.wrapping_mul(450)),
            660,
            450,
            190,
        )?))
    }
}

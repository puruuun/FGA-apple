use automata::{Location, Region};

use super::location_error::{LocationError, LocationResult};
use super::script_area_transforms::{
    IScriptAreaTransforms, Transforms, WIDE_ACTION_INSET, delegate_script_area_transforms,
};
use crate::modules::skill_level_flow::SkillLevelMode;

pub struct SkillLevelLocations {
    transforms: Transforms,
    slot_rise: i32,
    pub label: Region,
    pub reselect: Region,
    pub button_label: Region,
    pub dialog_watermark: Region,
    pub skip_click: Location,
    pub result: Region,
}

delegate_script_area_transforms!(SkillLevelLocations, transforms);

const SKILL_SLOT_X: [i32; 3] = [473, 759, 1047];
const APPEND_SLOT_X: [i32; 5] = [473, 650, 828, 1005, 1183];

impl SkillLevelLocations {
    pub fn new(transforms: Transforms) -> LocationResult<Self> {
        let t = &*transforms;
        let wide = t.is_wide();
        let slot_rise = if wide { 44 } else { 0 };
        let label_rise = if wide { 50 } else { 0 };

        let label = t.x_from_center_region(Region::new(-560, 730 - label_rise, 1240, 140)?);
        let reselect = t.x_from_left_edge_region(Region::new(340, 0, 340, 180)?);
        let button_label = t.x_from_right_region(if wide {
            Region::new(-500 - WIDE_ACTION_INSET, 1200, 500, 180)?
        } else {
            Region::new(-500, 1260, 500, 180)?
        });
        let dialog_watermark = t.x_from_center_region(Region::new(-240, 1180, 500, 120)?);
        let skip_click = t.x_from_center_location(Location::new(720, 72));
        let result = t.x_from_center_region(Region::new(-520, 1280, 1120, 140)?);

        Ok(Self {
            transforms,
            slot_rise,
            label,
            reselect,
            button_label,
            dialog_watermark,
            skip_click,
            result,
        })
    }

    fn slot_x(mode: SkillLevelMode, slot: i32) -> LocationResult<i32> {
        let table: &[i32] = match mode {
            SkillLevelMode::Skill => &SKILL_SLOT_X,
            SkillLevelMode::Append => &APPEND_SLOT_X,
        };
        usize::try_from(slot)
            .ok()
            .and_then(|index| table.get(index).copied())
            .ok_or_else(|| {
                LocationError::IndexOutOfBounds(format!(
                    "Index {slot} out of bounds for length {}",
                    table.len()
                ))
            })
    }

    pub fn slot_click(&self, mode: SkillLevelMode, slot: i32) -> LocationResult<Location> {
        Ok(self.x_from_center_location(Location::new(
            (Self::slot_x(mode, slot)? - 640) * 2,
            522 - self.slot_rise,
        )))
    }

    pub fn slot_glow_sides(&self, mode: SkillLevelMode, slot: i32) -> LocationResult<Vec<Region>> {
        let cx = (Self::slot_x(mode, slot)? - 640) * 2;
        let cy = 522 - self.slot_rise;
        Ok(vec![
            self.x_from_center_region(Region::new(cx - 72, cy - 120, 146, 2)?),
            self.x_from_center_region(Region::new(cx - 72, cy + 120, 146, 2)?),
            self.x_from_center_region(Region::new(cx - 120, cy - 72, 2, 146)?),
            self.x_from_center_region(Region::new(cx + 120, cy - 72, 2, 146)?),
        ])
    }

    pub fn slot_lock(&self, mode: SkillLevelMode, slot: i32) -> LocationResult<Region> {
        Ok(self.x_from_center_region(Region::new(
            (Self::slot_x(mode, slot)? - 40 - 640) * 2,
            410 - self.slot_rise,
            160,
            224,
        )?))
    }

    pub fn panel_level(&self, mode: SkillLevelMode, slot: i32) -> LocationResult<Region> {
        let in_range = usize::try_from(slot).is_ok_and(|index| index < mode.slots());
        if !in_range {
            return Err(LocationError::IllegalArgument(
                "Failed requirement.".to_owned(),
            ));
        }
        let x = Self::slot_x(mode, slot)? - 640;
        let region = match mode {
            SkillLevelMode::Skill => Region::new((x + 65) * 2, 584 - self.slot_rise, 290, 68)?,
            SkillLevelMode::Append => Region::new(
                (x - 5) * 2,
                if self.is_wide() {
                    590 - self.slot_rise
                } else {
                    588
                },
                66,
                42,
            )?,
        };
        Ok(self.x_from_center_region(region))
    }

    pub fn dialog_title(&self, mode: SkillLevelMode) -> LocationResult<Region> {
        let region = match mode {
            SkillLevelMode::Skill => Region::new(-80, 268, 320, 112)?,
            SkillLevelMode::Append => Region::new(60, 216, 320, 112)?,
        };
        Ok(self.x_from_center_region(region))
    }

    fn level_top(mode: SkillLevelMode) -> i32 {
        match mode {
            SkillLevelMode::Skill => 624,
            SkillLevelMode::Append => 524,
        }
    }

    pub fn current_level(&self, mode: SkillLevelMode) -> LocationResult<Region> {
        let region = match mode {
            SkillLevelMode::Append => Region::new(-206, Self::level_top(mode), 64, 120)?,
            SkillLevelMode::Skill => Region::new(-208, Self::level_top(mode), 104, 120)?,
        };
        Ok(self.x_from_center_region(region))
    }

    pub fn next_level(&self, mode: SkillLevelMode) -> LocationResult<Region> {
        let region = match mode {
            SkillLevelMode::Append => Region::new(216, 508, 192, 136)?,
            SkillLevelMode::Skill => Region::new(216, Self::level_top(mode), 164, 120)?,
        };
        Ok(self.x_from_center_region(region))
    }

    fn dialog_button_y(mode: SkillLevelMode) -> i32 {
        match mode {
            SkillLevelMode::Skill => 1180,
            SkillLevelMode::Append => 1132,
        }
    }

    pub fn dialog_cancel(&self, mode: SkillLevelMode) -> Location {
        self.x_from_center_location(Location::new(-390, Self::dialog_button_y(mode)))
    }

    pub fn dialog_confirm(&self, mode: SkillLevelMode) -> Location {
        self.x_from_center_location(Location::new(400, Self::dialog_button_y(mode)))
    }
}

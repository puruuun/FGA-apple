use automata::Region;

use super::location_error::LocationResult;

pub struct SupportSkillLayout;

impl SupportSkillLayout {
    pub const LEVEL_X: i32 = 1610;
    pub const LEVEL_Y: i32 = 325;
    pub const LEVEL_WIDTH: i32 = 60;
    pub const LEVEL_HEIGHT: i32 = 50;
    pub const NORMAL_SPACING: i32 = 90;
    pub const APPEND_SPACING: i32 = 75;

    pub fn level_regions(bounds: Region, append: bool) -> LocationResult<Vec<Region>> {
        let spacing = if append {
            Self::APPEND_SPACING
        } else {
            Self::NORMAL_SPACING
        };
        let count = if append { 5 } else { 3 };
        let mut regions = Vec::with_capacity(count);
        for index in 0..count as i32 {
            regions.push(Region::new(
                bounds
                    .x()
                    .wrapping_add(Self::LEVEL_X)
                    .wrapping_add(index * spacing),
                bounds.y().wrapping_add(Self::LEVEL_Y),
                Self::LEVEL_WIDTH,
                Self::LEVEL_HEIGHT,
            )?);
        }
        Ok(regions)
    }
}

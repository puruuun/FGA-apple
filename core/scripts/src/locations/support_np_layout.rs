use automata::Region;

use super::location_error::LocationResult;

pub struct SupportNpLayout;

impl SupportNpLayout {
    pub const NAME_X: i32 = 648;
    pub const NAME_Y: i32 = 260;
    pub const NAME_WIDTH: i32 = 896;
    pub const NAME_HEIGHT: i32 = 104;
    pub const SUFFIX_MARGIN: i32 = 4;
    pub const DIGIT_WIDTH: i32 = 38;
    pub const SUFFIX_RIGHT_OVERHANG: i32 = 12;
    pub const BAND_X: i32 = 500;
    pub const BAND_WIDTH: i32 = Self::NAME_X + Self::NAME_WIDTH - Self::BAND_X;
    pub const GRAND_ICON_X: i32 = 821;
    pub const NORMAL_ICON_X: i32 = 596;
    pub const ICON_CROP_MARGIN: i32 = 6;
    pub const TEXT_START_OFFSET: i32 = 58;
    pub const LEVEL_CROP_GAP: i32 = 8;
    pub const CROP_VERTICAL_MARGIN: i32 = 10;
    pub const NAME_LINE_OFFSET: i32 = 62;
    pub const BAND_TOP_MARGIN: i32 = 10;
    pub const BAND_BOTTOM_MARGIN: i32 = Self::CROP_VERTICAL_MARGIN;

    pub fn name_region(bounds: Region) -> LocationResult<Region> {
        Ok(Region::new(
            bounds.x().wrapping_add(Self::NAME_X),
            bounds.y().wrapping_add(Self::NAME_Y),
            Self::NAME_WIDTH,
            Self::NAME_HEIGHT,
        )?)
    }

    pub fn noble_phantasm_band(bounds: Region) -> LocationResult<Region> {
        let top = Self::NAME_Y - Self::NAME_LINE_OFFSET - Self::BAND_TOP_MARGIN;
        Ok(Region::new(
            bounds.x().wrapping_add(Self::BAND_X),
            bounds.y().wrapping_add(top),
            Self::BAND_WIDTH,
            Self::NAME_HEIGHT + Self::NAME_Y - top + Self::BAND_BOTTOM_MARGIN,
        )?)
    }

    pub fn noble_phantasm_crop(
        bounds: Region,
        grand: bool,
        level_prefix: Region,
    ) -> LocationResult<Region> {
        let icon_x = if grand {
            Self::GRAND_ICON_X
        } else {
            Self::NORMAL_ICON_X
        };
        let left = bounds
            .x()
            .wrapping_add(icon_x)
            .wrapping_add(Self::TEXT_START_OFFSET);
        let right = level_prefix.x().wrapping_sub(Self::LEVEL_CROP_GAP);
        let top = level_prefix
            .y()
            .wrapping_sub(Self::NAME_LINE_OFFSET)
            .wrapping_sub(Self::CROP_VERTICAL_MARGIN);
        let bottom = level_prefix
            .y()
            .wrapping_add(level_prefix.height())
            .wrapping_add(Self::CROP_VERTICAL_MARGIN);
        Ok(Region::new(
            left,
            top,
            right.wrapping_sub(left),
            bottom.wrapping_sub(top),
        )?)
    }

    pub fn suffix_region(prefix: Region, digit_width: i32) -> LocationResult<Region> {
        Ok(Region::new(
            prefix.x().wrapping_sub(Self::SUFFIX_MARGIN),
            prefix.y().wrapping_sub(Self::SUFFIX_MARGIN),
            prefix
                .width()
                .wrapping_add(digit_width)
                .wrapping_add(Self::SUFFIX_MARGIN),
            prefix.height().wrapping_add(Self::SUFFIX_MARGIN * 2),
        )?)
    }
}

use automata::{Location, Region};

use super::location_error::LocationResult;
use super::script_area_transforms::{Transforms, delegate_script_area_transforms};

pub struct WeeklyRouteLocations {
    transforms: Transforms,
    pub top_left_button: Region,
    pub repeat_dialog_signal: Region,
    pub repeat_dialog_close: Location,
    pub list_area: Region,
    pub home_other_banner: Region,
    pub top_scrollbar_region: Region,
    pub bottom_scrollbar_region: Region,
    pub list_scroll_start: Location,
    pub list_scroll_end: Location,
    pub map_motion: Region,
    pub pinch_bounds: Region,
    pub panel_column: Region,
}

delegate_script_area_transforms!(WeeklyRouteLocations, transforms);

impl WeeklyRouteLocations {
    pub fn new(transforms: Transforms) -> LocationResult<Self> {
        let t = &*transforms;
        let script_width = t.script_area().width();

        let top_left_button = t.x_from_left_edge_region(Region::new(10, 30, 380, 140)?);
        let repeat_dialog_signal = t.x_from_center_region(Region::new(180, 1100, 560, 130)?);
        let repeat_dialog_close = t.x_from_center_location(Location::new(-394, 1166));
        let list_area = t.x_from_right_region(Region::new(-1500, 200, 1500, 1140)?);
        let home_other_banner = Region::new(list_area.x(), list_area.y(), list_area.width(), 360)?;
        let top_scrollbar_region = t.x_from_right_edge_region(Region::new(-82, 178, 74, 38)?);
        let bottom_scrollbar_region = t.x_from_right_edge_region(Region::new(-82, 1152, 74, 34)?);
        let list_scroll_start = t.x_from_right_location(Location::new(-700, 1180));
        let list_scroll_end = t.x_from_right_location(Location::new(-700, 540));
        let map_motion = Region::new(500, 360, script_width.wrapping_sub(740), 600)?;
        let pinch_bounds = Region::new(400, 300, script_width.wrapping_sub(800), 800)?;
        let panel_column = t.x_from_right_edge_region(Region::new(-1330, 250, 330, 1100)?);

        Ok(Self {
            transforms,
            top_left_button,
            repeat_dialog_signal,
            repeat_dialog_close,
            list_area,
            home_other_banner,
            top_scrollbar_region,
            bottom_scrollbar_region,
            list_scroll_start,
            list_scroll_end,
            map_motion,
            pinch_bounds,
            panel_column,
        })
    }
}

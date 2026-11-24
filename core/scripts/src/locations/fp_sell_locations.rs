use automata::{Location, Region};

use super::location_error::LocationResult;
use super::script_area_transforms::{
    IScriptAreaTransforms, Transforms, delegate_script_area_transforms,
};

pub struct FpSellLocations {
    transforms: Transforms,
    pub full_dialog_line: Region,
    pub full_sell: Region,
    pub servant_tab: Region,
    pub servant_tab_click: Location,
    grid_shift: i32,
    pub grid: Region,
    pub bottom_scrollbar_region: Region,
    pub size_switch: Location,
    pub confirm: Region,
    pub convert: Region,
    pub done_close: Region,
    pub close: Location,
    pub left_arrow_region: Region,
    pub summon_button: Region,
    pub daily_summon_button: Region,
    pub menu: Region,
    pub summon_menu: Region,
    pub banner_title: Region,
    pub left_arrow: Location,
    pub swipe_start: Location,
}

delegate_script_area_transforms!(FpSellLocations, transforms);

impl FpSellLocations {
    pub const ROW_PITCH: i32 = 284;
    pub const CARD_WIDTH: i32 = 232;
    pub const CARD_HEIGHT: i32 = 252;
    pub const FRAME_OFFSETS: [i32; 17] =
        [-8, -7, -6, -5, -4, -3, -2, -1, 0, 1, 2, 3, 4, 5, 6, 7, 8];
    pub const FRAME_BANDS: [i32; 3] = [72, 120, 160];

    pub fn new(transforms: Transforms) -> LocationResult<Self> {
        let t = &*transforms;
        let wide = t.is_wide();
        let full_dialog_line = t.x_from_center_region(Region::new(-920, 290, 1840, 90)?);
        let full_sell = t.x_from_center_region(Region::new(-850, 880, 580, 160)?);
        let servant_tab = t.x_from_left_edge_region(Region::new(30, 185, 340, 140)?);
        let servant_tab_click = t.x_from_left_edge_location(Location::new(160, 254));
        let grid_shift = if wide { 10 } else { 0 };
        let grid = t.x_from_center_region(Region::new(-1130 + grid_shift, 338, 1850, 1102)?);
        let bottom_scrollbar_region = t.x_from_right_edge_region(Region::new(-554, 1330, 72, 34)?);
        let size_switch = t.x_from_left_edge_location(if wide {
            Location::new(100, 1268)
        } else {
            Location::new(100, 1320)
        });
        let confirm = t.x_from_right_edge_region(Region::new(-460, 1240, 440, 180)?);
        let convert = t.x_from_center_region(Region::new(100, 1170, 620, 170)?);
        let done_close = t.x_from_center_region(Region::new(-300, 1170, 600, 170)?);
        let close = t.x_from_left_edge_location(Location::new(160, 80));
        let left_arrow_region = t.x_from_left_edge_region(Region::new(0, 600, 150, 250)?);
        let summon_button = t.x_from_center_region(Region::new(250, 940, 500, 180)?);
        let daily_summon_button = t.x_from_center_region(Region::new(-220, 940, 440, 180)?);
        let menu = t.x_from_right_edge_region(Region::new(-390, 1210, 380, 200)?);
        let summon_menu = t.x_from_center_region(Region::new(-260, 1180, 340, 180)?);
        let banner_title = t.x_from_center_region(Region::new(-650, 600, 1300, 260)?);
        let left_arrow = t.x_from_left_edge_location(Location::new(50, 720));
        let swipe_start = t.x_from_center_location(Location::new(0, 1310));
        Ok(Self {
            transforms,
            full_dialog_line,
            full_sell,
            servant_tab,
            servant_tab_click,
            grid_shift,
            grid,
            bottom_scrollbar_region,
            size_switch,
            confirm,
            convert,
            done_close,
            close,
            left_arrow_region,
            summon_button,
            daily_summon_button,
            menu,
            summon_menu,
            banner_title,
            left_arrow,
            swipe_start,
        })
    }

    pub fn swipe_end(&self, distance: i32) -> Location {
        self.swipe_start - Location::new(0, distance)
    }

    pub fn column_x(&self, column: i32) -> i32 {
        let scaled = (f64::from(column) * 266.5).round_ties_even() as i32;
        self.x_from_center_location(Location::new(
            (-1124i32)
                .wrapping_add(self.grid_shift)
                .wrapping_add(scaled),
            0,
        ))
        .x
    }

    pub fn layout_columns(&self, columns: i32) -> Vec<i32> {
        if columns == 7 {
            (0..=6)
                .map(|it| self.column_x(it).wrapping_sub(self.grid.x()) / 2)
                .collect()
        } else {
            (0..columns.max(0))
                .map(|it| {
                    5i32.wrapping_add(
                        (f64::from(it) * 937.5 / f64::from(columns)).round_ties_even() as i32,
                    )
                })
                .collect()
        }
    }

    pub fn layout_pitch(&self, columns: i32) -> i32 {
        match columns {
            5 => 200,
            6 => 166,
            _ => Self::ROW_PITCH / 2,
        }
    }

    pub fn layout_card_height(&self, columns: i32) -> i32 {
        match columns {
            5 => 180,
            6 => 150,
            _ => Self::CARD_HEIGHT / 2,
        }
    }

    pub fn card(&self, column: i32, top: i32) -> LocationResult<Region> {
        Ok(Region::new(
            self.column_x(column),
            top,
            Self::CARD_WIDTH,
            Self::CARD_HEIGHT,
        )?)
    }

    pub fn presence(&self, card: Region) -> LocationResult<Region> {
        Ok(Region::new(
            card.x().wrapping_sub(16),
            card.y().wrapping_add(24),
            card.width().wrapping_add(32),
            184,
        )?)
    }

    pub fn frame_strip(&self, card: Region, offset: i32, band: i32) -> LocationResult<Region> {
        Ok(Region::new(
            card.x().wrapping_add(offset.wrapping_mul(2)),
            card.y().wrapping_add(band),
            2,
            32,
        )?)
    }

    pub fn lock(&self, card: Region) -> LocationResult<Region> {
        Ok(Region::new(
            card.x().wrapping_sub(32),
            card.y(),
            card.width().wrapping_add(32),
            card.height(),
        )?)
    }
}

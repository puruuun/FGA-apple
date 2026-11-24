use automata::kotlin_math::round_to_int;
use automata::{Location, Region};

use super::location_error::LocationResult;
use super::script_area_transforms::{Transforms, delegate_script_area_transforms};
use crate::enums::support_class::SupportClass;

pub struct SupportScreenLocations {
    transforms: Transforms,
    header_offset: Location,
    pub screen_check_region: Region,
    pub extra_region: Region,
    pub update_click: Location,
    pub skill_display_switch_click: Location,
    pub list_top_click: Location,
    pub first_support_click: Location,
    pub update_yes_click: Location,
    pub skill_display_labels_region: Region,
    pub list_region: Region,
    pub grand_ce_list_region: Region,
    pub bond_ce_region: Region,
    pub grand_ce_label_region: Region,
    pub grand_ce_region1: Region,
    pub grand_ce_region2: Region,
    pub grand_ce_region3: Region,
    pub grand_ce_clip_region: Region,
    pub grand_limit_break_region: Region,
    pub friend_region: Region,
    pub friends_region: Region,
    pub max_ascended_region: Region,
    pub limit_break_region: Region,
    pub confirm_setup_button_region: Region,
    pub default_bounds: Region,
    pub default_ce_bounds: Region,
    pub not_found_region: Region,
    pub list_swipe_start: Location,
    pub list_swipe_end: Location,
    pub top_scrollbar_region: Region,
    pub bottom_scrollbar_region: Region,
    pub refresh_region: Region,
}

delegate_script_area_transforms!(SupportScreenLocations, transforms);

impl SupportScreenLocations {
    pub fn new(transforms: Transforms) -> LocationResult<Self> {
        let t = &*transforms;
        let wide = t.is_wide();
        let long_swipe = t.can_long_swipe();
        let header_offset = Location::new(t.edge_inset(), 0);

        let screen_check_region = Region::new(0, 0, 200, 400)? + header_offset;
        let extra_region = Region::new(1200, 200, 130, 130)? + header_offset;
        let update_click = Location::new(1865, 260) + header_offset;
        let skill_display_switch_click = Location::new(1695, 260) + header_offset;
        let list_top_click =
            t.x_from_right_location(Location::new(if wide { -218 } else { -80 }, 360));
        let first_support_click = t.x_from_center_location(Location::new(0, 500));
        let update_yes_click = t.x_from_center_location(Location::new(200, 1110));

        let support_offset = if wide {
            let width = 2560 - 94 - 145;
            let total = t.script_area().width().wrapping_sub(305).wrapping_sub(270);
            let border = round_to_int(f64::from(total.wrapping_sub(width)) / 2.0)?;
            Location::new(305i32.wrapping_add(border), 0)
        } else {
            Location::new(94, 0)
        };

        let skill_display_labels_region = Region::new(1550, 332, 500, 1091)? + support_offset;
        let list_region = Region::new(-24, 332, 378, 1091)? + support_offset;
        let grand_ce_list_region = Region::new(335, 332, 342, 1091)? + support_offset;
        let bond_ce_region = Region::new(348, 192, 53, 55)? + support_offset;
        let grand_ce_label_region = Region::new(335, 0, 60, 60)? + support_offset;
        let grand_ce_region1 = Region::new(346, 100, 320, 94)? + support_offset;
        let grand_ce_region2 = Region::new(346, 192, 320, 94)? + support_offset;
        let grand_ce_region3 = Region::new(346, 284, 320, 94)? + support_offset;
        let grand_ce_clip_region = Region::new(354, -34, 284, 270)? + support_offset;
        let grand_limit_break_region = Region::new(611, 0, 40, 90)? + support_offset;
        let friend_region =
            Region::new(2140, list_region.y(), 120, list_region.height())? + support_offset;
        let friends_region = Region::new(354, 332, 1210, 1091)? + support_offset;
        let max_ascended_region = Region::new(270, 0, 40, 170)? + support_offset;
        let limit_break_region = Region::new(270, 0, 40, 90)? + support_offset;
        let confirm_setup_button_region = Region::new(2006, 0, 370, 1440)? + support_offset;
        let default_bounds = Region::new(-18, 0, 2356, 428)? + support_offset;
        let default_ce_bounds = Region::new(-18, 270, 378, 150)? + support_offset;
        let not_found_region = t.x_from_center_region(Region::new(-180, 810, 360, 166)?);

        let list_swipe_start =
            Location::new(-59, if long_swipe { 1000 } else { 1190 }) + support_offset;
        let list_swipe_end =
            Location::new(-89, if long_swipe { 300 } else { 660 }) + support_offset;

        let top_scrollbar_region = if wide {
            t.x_from_right_region(Region::new(-255, 325, 140, 55)?)
        } else {
            t.x_from_center_region(Region::new(1195, 325, 100, 55)?)
        };
        let bottom_scrollbar_region = if wide {
            t.x_from_right_region(Region::new(-255, 1390, 140, 55)?)
        } else {
            t.x_from_center_region(Region::new(1195, 1390, 100, 55)?)
        };

        let refresh_region = Region::new(1770, 195, 175, 165)? + header_offset;

        Ok(Self {
            transforms,
            header_offset,
            screen_check_region,
            extra_region,
            update_click,
            skill_display_switch_click,
            list_top_click,
            first_support_click,
            update_yes_click,
            skill_display_labels_region,
            list_region,
            grand_ce_list_region,
            bond_ce_region,
            grand_ce_label_region,
            grand_ce_region1,
            grand_ce_region2,
            grand_ce_region3,
            grand_ce_clip_region,
            grand_limit_break_region,
            friend_region,
            friends_region,
            max_ascended_region,
            limit_break_region,
            confirm_setup_button_region,
            default_bounds,
            default_ce_bounds,
            not_found_region,
            list_swipe_start,
            list_swipe_end,
            top_scrollbar_region,
            bottom_scrollbar_region,
            refresh_region,
        })
    }

    pub fn locate(&self, support_class: SupportClass) -> Location {
        let x = match support_class {
            SupportClass::None => 0,
            SupportClass::All => 184,
            SupportClass::Saber => 320,
            SupportClass::Archer => 454,
            SupportClass::Lancer => 568,
            SupportClass::Rider => 724,
            SupportClass::Caster => 858,
            SupportClass::Assassin => 994,
            SupportClass::Berserker => 1130,
            SupportClass::Extra => 1264,
            SupportClass::Mix => 1402,
        };
        Location::new(x, 256) + self.header_offset
    }
}

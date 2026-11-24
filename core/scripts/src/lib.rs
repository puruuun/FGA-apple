#![forbid(unsafe_code)]

pub mod enums;
pub mod fgo_game_area_manager;
pub mod kotlin;
pub mod locations {
    pub mod attack_screen_locations;
    pub mod battle_screen_locations;
    pub mod bond_locations;
    pub mod ce_enhance_locations;
    pub mod fp_locations;
    pub mod fp_sell_locations;
    pub mod game_area_calibration_locations;
    pub mod location_error;
    #[allow(clippy::module_inception)]
    pub mod locations;
    pub mod lottery_locations;
    pub mod master_locations;
    pub mod quest_item_locations;
    pub mod quest_panel_locations;
    pub mod script_area_transforms;
    pub mod servant_level_locations;
    pub mod skill_level_locations;
    pub mod support_np_layout;
    pub mod support_score_layout;
    pub mod support_screen_locations;
    pub mod support_skill_layout;
    pub mod weekly_route_locations;
}
pub mod models;
pub mod prefs;
pub mod modules {
    pub mod skill_level_flow;
}
pub mod script_log;

use automata::{Location, Region};

use super::attack_screen_locations::AttackScreenLocations;
use super::battle_screen_locations::BattleScreenLocations;
use super::bond_locations::BondLocations;
use super::fp_locations::FPLocations;
use super::game_area_calibration_locations::GameAreaCalibrationLocations;
use super::location_error::LocationResult;
use super::lottery_locations::LotteryLocations;
use super::quest_item_locations::QuestItemLocations;
use super::script_area_transforms::{
    IScriptAreaTransforms, Transforms, delegate_script_area_transforms,
};
use super::servant_level_locations::ServantLevelLocations;
use super::support_screen_locations::SupportScreenLocations;
use crate::enums::refill_resource_enum::RefillResourceEnum;
use crate::models::boost_item::Enabled;

pub struct Locations {
    transforms: Transforms,
    pub fp: FPLocations,
    pub lottery: LotteryLocations,
    pub support: SupportScreenLocations,
    pub attack: AttackScreenLocations,
    pub battle: BattleScreenLocations,
    pub servant: ServantLevelLocations,
    pub quest_items: QuestItemLocations,
    pub bond: BondLocations,
    pub continue_region: Region,
    pub continue_boost_click: Location,
    pub mission_popup_clear_click: Location,
    pub inventory_full_region: Region,
    pub ordeal_call_confirm_pod_use_region: Region,
    pub ordeal_call_out_of_pods_region: Region,
    pub ordeal_call_out_of_pods_click: Location,
    pub interlude_close_click: Location,
    pub interlude_end_screen_close: Region,
    pub menu_screen_region: Region,
    pub menu_select_quest_click: Location,
    pub menu_start_quest_click: Location,
    pub menu_story_skip_yes_click: Location,
    pub retry_region: Region,
    pub stamina_screen_region: Region,
    pub stamina_ok_click: Location,
    pub stamina_close_click: Location,
    pub withdraw_region: Region,
    pub withdraw_accept_click: Location,
    pub withdraw_close_click: Location,
    pub turn_limit_retreat_region: Region,
    pub turn_limit_retreat_click: Location,
    pub turn_limit_retreat_ok_region: Region,
    pub battle_menu_close_click: Location,
    pub selected_party_region: Region,
    pub party_selection_array: Vec<Location>,
    pub menu_story_skip_region: Region,
    pub menu_story_skip_click: Location,
    pub result_friend_request_region: Region,
    pub result_friend_request_reject_click: Location,
    pub result_friend_request_send_click: Location,
    pub result_mat_rewards_region: Region,
    pub result_click: Location,
    pub result_quest_reward_region: Region,
    pub result_drop_scrollbar_region: Region,
    pub result_drop_scroll_end_click: Location,
    pub result_master_exp_region: Region,
    pub result_master_lvl_up_region: Region,
    pub result_screen_region: Region,
    pub result_bond_region: Region,
    pub result_ce_reward_region: Region,
    pub result_ce_reward_details_region: Region,
    pub result_ce_reward_close_click: Location,
    pub gift_box_swipe_start: Location,
    pub gift_box_swipe_end: Location,
    pub empty_enhance_region: Region,
    pub np_started_region: Region,
    pub rank_up_region: Region,
    pub middle_of_screen_click: Location,
    pub enhancement_banner_region: Region,
    pub enhancement_click: Location,
    pub temp_servant_enhancement_region: Region,
    pub enhancement_skip_rapid_click: Location,
    pub temp_servant_enhancement_location: Location,
}

delegate_script_area_transforms!(Locations, transforms);

impl Locations {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        transforms: Transforms,
        fp: FPLocations,
        lottery: LotteryLocations,
        support: SupportScreenLocations,
        attack: AttackScreenLocations,
        battle: BattleScreenLocations,
        servant: ServantLevelLocations,
        quest_items: QuestItemLocations,
        bond: BondLocations,
    ) -> LocationResult<Self> {
        let t = &*transforms;
        let wide = t.is_wide();
        let can_long_swipe = t.can_long_swipe();
        let script_area = t.script_area();

        let continue_region = t.x_from_center_region(Region::new(120, 1000, 800, 300)?);
        let continue_boost_click = t.x_from_center_location(Location::new(-20, 1120));
        let mission_popup_clear_click = t.x_from_center_location(Location::new(0, 100));
        let inventory_full_region = t.x_from_center_region(Region::new(-280, 860, 560, 190)?);
        let ordeal_call_confirm_pod_use_region =
            t.x_from_center_region(Region::new(190, 1096, 420, 62)?);
        let ordeal_call_out_of_pods_region =
            t.x_from_center_region(Region::new(-112, 1088, 219, 72)?);
        let ordeal_call_out_of_pods_click = t.x_from_center_location(Location::new(-2, 1124));
        let interlude_close_click = t.x_from_center_location(Location::new(-399, 1125));
        let interlude_end_screen_close = t.x_from_center_region(Region::new(-515, 1080, 230, 90)?);

        let menu_screen_region =
            GameAreaCalibrationLocations::menu_search(script_area.width(), wide)?
                + Location::new(script_area.x(), 0);

        let menu_select_quest_click = t.x_from_right_location(if wide {
            Location::new(-460, 440)
        } else {
            Location::new(-270, 440)
        });
        let menu_start_quest_click = t.y_from_bottom_location(t.x_from_right_location(if wide {
            Location::new(-350, -160)
        } else {
            Location::new(-160, -90)
        }));
        let menu_story_skip_yes_click = t.x_from_center_location(Location::new(320, 1100));

        let retry_region = t.x_from_center_region(Region::new(20, 1000, 700, 300)?);

        let stamina_screen_region = t.x_from_center_region(Region::new(-680, 200, 300, 300)?);
        let stamina_ok_click = t.x_from_center_location(Location::new(370, 1120));
        let stamina_close_click = t.x_from_center_location(Location::new(0, 1240));

        let withdraw_region = t.x_from_center_region(Region::new(-880, 540, 1800, 333)?);
        let withdraw_accept_click = t.x_from_center_location(Location::new(485, 720));
        let withdraw_close_click = t.x_from_center_location(Location::new(-10, 1140));

        let turn_limit_retreat_region = t.x_from_center_region(Region::new(380, 430, 340, 130)?);
        let turn_limit_retreat_click = t.x_from_center_location(Location::new(547, 496));
        let turn_limit_retreat_ok_region = t.x_from_center_region(Region::new(285, 640, 400, 160)?);
        let battle_menu_close_click = t.x_from_center_location(Location::new(1073, 220));

        let selected_party_region = t.x_from_center_region(Region::new(-370, 62, 740, 72)?);

        let party_selection_array = (0..=14)
            .map(|it| t.x_from_center_location(Location::new((it - 7) * 50, 100)))
            .collect();

        let menu_story_skip_region = t.x_from_center_region(Region::new(960, 20, 300, 120)?);
        let menu_story_skip_click = t.x_from_center_location(Location::new(1080, 80));

        let result_friend_request_region = t.x_from_center_region(Region::new(600, 150, 100, 94)?);
        let result_friend_request_reject_click =
            t.x_from_center_location(Location::new(-680, 1200));
        let result_friend_request_send_click = t.x_from_center_location(Location::new(680, 1200));
        let result_mat_rewards_region = t.x_from_center_region(Region::new(800, 1220, 280, 130)?);
        let result_click = t.x_from_center_location(Location::new(320, 1350));
        let result_quest_reward_region = t.x_from_center_region(Region::new(350, 140, 370, 250)?);
        let result_drop_scrollbar_region = t.x_from_center_region(Region::new(980, 167, 100, 88)?);
        let result_drop_scroll_end_click = t.x_from_center_location(Location::new(1030, 968));
        let result_master_exp_region = t.x_from_center_region(Region::new(0, 350, 400, 110)?);
        let result_master_lvl_up_region = t.x_from_center_region(Region::new(710, 160, 250, 270)?);
        let result_screen_region = t.x_from_center_region(Region::new(-1180, 300, 700, 200)?);
        let result_bond_region = t.x_from_center_region(Region::new(720, 600, 120, 400)?);

        let result_ce_reward_region = t.x_from_center_region(Region::new(-230, 1216, 33, 28)?);
        let result_ce_reward_details_region =
            Region::new(if wide { 193 } else { 0 }, 512, 135, 115)?;
        let result_ce_reward_close_click = Location::new(if wide { 265 } else { 80 }, 60);

        let gift_box_swipe_start =
            t.x_from_center_location(Location::new(120, if can_long_swipe { 1200 } else { 1050 }));
        let gift_box_swipe_end =
            t.x_from_center_location(Location::new(120, if can_long_swipe { 350 } else { 575 }));

        let empty_enhance_region = if wide {
            t.x_from_center_region(Region::new(-1100, 600, 400, 400)?)
        } else {
            Region::new(200, 600, 400, 400)?
        };

        let np_started_region = t.x_from_center_region(Region::new(-400, 500, 800, 400)?);

        let rank_up_region = t.x_from_center_region(Region::new(270, 730, 220, 340)?);

        let middle_of_screen_click = t.x_from_center_location(Location::new(0, 720));

        let enhancement_banner_region = if wide {
            t.x_from_center_region(Region::new(-412, 282, 241, 37)?)
        } else {
            t.x_from_center_region(Region::new(-413, 324, 241, 37)?)
        };

        let enhancement_click = if wide {
            t.x_from_right_location(Location::new(-396, 1284))
        } else {
            t.x_from_right_location(Location::new(-281, 1343))
        };

        let temp_servant_enhancement_region =
            t.x_from_center_region(Region::new(176, 1080, 480, 104)?);

        let enhancement_skip_rapid_click = t.x_from_center_location(Location::new(0, 1400));

        let temp_servant_enhancement_location = t.x_from_center_location(Location::new(402, 1124));

        Ok(Self {
            transforms,
            fp,
            lottery,
            support,
            attack,
            battle,
            servant,
            quest_items,
            bond,
            continue_region,
            continue_boost_click,
            mission_popup_clear_click,
            inventory_full_region,
            ordeal_call_confirm_pod_use_region,
            ordeal_call_out_of_pods_region,
            ordeal_call_out_of_pods_click,
            interlude_close_click,
            interlude_end_screen_close,
            menu_screen_region,
            menu_select_quest_click,
            menu_start_quest_click,
            menu_story_skip_yes_click,
            retry_region,
            stamina_screen_region,
            stamina_ok_click,
            stamina_close_click,
            withdraw_region,
            withdraw_accept_click,
            withdraw_close_click,
            turn_limit_retreat_region,
            turn_limit_retreat_click,
            turn_limit_retreat_ok_region,
            battle_menu_close_click,
            selected_party_region,
            party_selection_array,
            menu_story_skip_region,
            menu_story_skip_click,
            result_friend_request_region,
            result_friend_request_reject_click,
            result_friend_request_send_click,
            result_mat_rewards_region,
            result_click,
            result_quest_reward_region,
            result_drop_scrollbar_region,
            result_drop_scroll_end_click,
            result_master_exp_region,
            result_master_lvl_up_region,
            result_screen_region,
            result_bond_region,
            result_ce_reward_region,
            result_ce_reward_details_region,
            result_ce_reward_close_click,
            gift_box_swipe_start,
            gift_box_swipe_end,
            empty_enhance_region,
            np_started_region,
            rank_up_region,
            middle_of_screen_click,
            enhancement_banner_region,
            enhancement_click,
            temp_servant_enhancement_region,
            enhancement_skip_rapid_click,
            temp_servant_enhancement_location,
        })
    }

    pub fn locate_refill_resource(&self, refill_resource: RefillResourceEnum) -> Vec<Location> {
        let scroll_bar_y = match refill_resource {
            RefillResourceEnum::Copper => 1040,
            _ => 300,
        };
        let scroll_bar_loc = self.x_from_center_location(Location::new(750, scroll_bar_y));

        let resource_y = match refill_resource {
            RefillResourceEnum::Copper => 980,
            RefillResourceEnum::Bronze => 1140,
            RefillResourceEnum::Silver => 922,
            RefillResourceEnum::Gold => 634,
            RefillResourceEnum::SQ => 345,
        };
        let resource_loc = self.x_from_center_location(Location::new(-530, resource_y));
        vec![scroll_bar_loc, resource_loc]
    }

    pub fn locate_boost(&self, boost: Enabled) -> Location {
        let location = match boost {
            Enabled::Skip => Location::new(1652, 1304),
            Enabled::BoostItem1 => Location::new(1280, 418),
            Enabled::BoostItem2 => Location::new(1280, 726),
            Enabled::BoostItem3 => Location::new(1280, 1000),
        };
        self.x_from_center_location(location)
    }
}

use automata::{Location, Region};

use super::location_error::LocationResult;
use super::master_locations::MasterLocations;
use super::script_area_transforms::{
    IScriptAreaTransforms, Transforms, delegate_script_area_transforms,
};
use crate::models::enemy_target::{EnemyTarget, Normal};
use crate::models::field_slot::FieldSlot;
use crate::models::order_change_member::{OrderChangeMember, Starting, Sub};
use crate::models::servant_target::{ServantTarget, SpecialTarget};
use crate::models::skill::Servant;

pub struct BattleScreenLocations {
    transforms: Transforms,
    pub master: MasterLocations,
    pub screen_check_region: Region,
    pub attack_click: Location,
    pub skill_ok_click: Location,
    pub order_change_ok_click: Location,
    pub extra_info_window_close_click: Location,
    pub skill_use_region: Region,
    pub shuffle_np_shortage_region: Region,
    pub servant_details_info_click: Location,
    pub servant_details_face_card_region: Region,
    pub battle_safe_middle_of_screen_click: Location,
}

delegate_script_area_transforms!(BattleScreenLocations, transforms);

impl BattleScreenLocations {
    pub fn new(transforms: Transforms, master: MasterLocations) -> LocationResult<Self> {
        let t = &*transforms;
        let wide = t.is_wide();

        let screen_check_region = t.y_from_bottom_region(t.x_from_right_region(if wide {
            Region::new(-660, -210, 400, 175)?
        } else {
            Region::new(-455, -181, 336, 116)?
        }));

        let attack_click = t.y_from_bottom_location(t.x_from_right_location(if wide {
            Location::new(-460, -230)
        } else {
            Location::new(-260, -240)
        }));

        let skill_ok_click = t.x_from_center_location(Location::new(400, 850));
        let order_change_ok_click = t.x_from_center_location(Location::new(0, 1260));
        let extra_info_window_close_click = t.x_from_right_location(Location::new(-300, 940));
        let skill_use_region = t.x_from_center_region(Region::new(-210, 320, 420, 85)?);
        let shuffle_np_shortage_region = t.x_from_center_region(Region::new(-160, 1060, 320, 130)?);
        let servant_details_info_click = t.x_from_center_location(Location::new(-660, 110));
        let servant_details_face_card_region =
            t.x_from_center_region(Region::new(-685, 410, 110, 60)?);
        let battle_safe_middle_of_screen_click = t.x_from_center_location(Location::new(0, 550));

        Ok(Self {
            transforms,
            master,
            screen_check_region,
            attack_click,
            skill_ok_click,
            order_change_ok_click,
            extra_info_window_close_click,
            skill_use_region,
            shuffle_np_shortage_region,
            servant_details_info_click,
            servant_details_face_card_region,
            battle_safe_middle_of_screen_click,
        })
    }

    pub fn locate_order_change_member(&self, order_change_member: OrderChangeMember) -> Location {
        let x = match order_change_member {
            OrderChangeMember::Starting(Starting::A) => -1000,
            OrderChangeMember::Starting(Starting::B) => -600,
            OrderChangeMember::Starting(Starting::C) => -200,
            OrderChangeMember::Sub(Sub::A) => 200,
            OrderChangeMember::Sub(Sub::B) => 600,
            OrderChangeMember::Sub(Sub::C) => 1000,
        };
        self.x_from_center_location(Location::new(x, 700))
    }

    pub fn locate_servant_target(&self, servant_target: ServantTarget) -> Option<Location> {
        let x = match servant_target {
            ServantTarget::A => -580,
            ServantTarget::B => 0,
            ServantTarget::C => 660,
            ServantTarget::Left => -290,
            ServantTarget::Right => 330,
            ServantTarget::Transform => return None,
            ServantTarget::Option1
            | ServantTarget::SpecialTarget(SpecialTarget::Choice2OptionA) => 0,
            ServantTarget::Option2
            | ServantTarget::SpecialTarget(SpecialTarget::Choice2OptionB) => 470,
            ServantTarget::SpecialTarget(SpecialTarget::Choice3OptionA) => -200,
            ServantTarget::SpecialTarget(SpecialTarget::Choice3OptionB) => 300,
            ServantTarget::SpecialTarget(SpecialTarget::Choice3OptionC) => 670,
        };
        Some(self.x_from_center_location(Location::new(x, 880)))
    }

    pub fn locate_skill(&self, skill: Servant) -> Location {
        let x = match skill {
            Servant::A1 => 148,
            Servant::A2 => 324,
            Servant::A3 => 500,
            Servant::B1 => 784,
            Servant::B2 => 960,
            Servant::B3 => 1136,
            Servant::C1 => 1418,
            Servant::C2 => 1594,
            Servant::C3 => 1770,
        };
        let wide = self.is_wide();
        Location::new(
            x + if wide { 108 } else { 0 },
            if wide { 1117 } else { 1158 },
        )
    }

    pub fn locate_enemy(&self, enemy: EnemyTarget) -> Location {
        match enemy {
            EnemyTarget::A => self.normal_enemy(90),
            EnemyTarget::B => self.normal_enemy(570),
            EnemyTarget::C => self.normal_enemy(1050),
            EnemyTarget::SixTopLeft => self.layout_enemy(460, 80),
            EnemyTarget::SixTopMiddle => self.layout_enemy(860, 80),
            EnemyTarget::SixTopRight => self.layout_enemy(1260, 80),
            EnemyTarget::SixBottomLeft => self.layout_enemy(260, 276),
            EnemyTarget::SixBottomMiddle => self.layout_enemy(660, 276),
            EnemyTarget::SixBottomRight => self.layout_enemy(1060, 276),
            EnemyTarget::RaidBoss => self.layout_enemy(780, 80),
            EnemyTarget::RaidLeft => self.layout_enemy(300, 288),
            EnemyTarget::RaidMiddle => self.layout_enemy(800, 288),
            EnemyTarget::RaidRight => self.layout_enemy(1250, 288),
            EnemyTarget::RaidLowerLeft => self.layout_enemy(460, 433),
            EnemyTarget::RaidLowerMiddle => self.layout_enemy(860, 433),
            EnemyTarget::RaidLowerRight => self.layout_enemy(1260, 433),
        }
    }

    fn normal_enemy(&self, x: i32) -> Location {
        Location::new(x + if self.is_wide() { 183 } else { 0 }, 80)
    }

    fn layout_enemy(&self, x: i32, y: i32) -> Location {
        Location::new(x + if self.is_wide() { 155 } else { 0 }, y)
    }

    pub fn danger_region(&self, enemy: Normal) -> LocationResult<Region> {
        let region = match enemy {
            Normal::A => Region::new(0, 0, 485, 220)?,
            Normal::B => Region::new(485, 0, 482, 220)?,
            Normal::C => Region::new(967, 0, 476, 220)?,
        };
        Ok(region + Location::new(if self.is_wide() { 150 } else { 0 }, 0))
    }

    pub fn servant_present_region(&self, slot: FieldSlot) -> LocationResult<Region> {
        let skill3_location = self.locate_skill(slot.skill3());
        Ok(Region::new(
            skill3_location.x.wrapping_add(35),
            skill3_location.y.wrapping_add(67),
            120,
            120,
        )?)
    }

    pub fn servant_open_details_click(&self, slot: FieldSlot) -> Location {
        Location::new(self.locate_skill(slot.skill2()).x, 810)
    }

    pub fn servant_change_check_region(&self, slot: FieldSlot) -> LocationResult<Region> {
        let x = self.locate_skill(slot.skill2()).x;
        Ok(Region::new(x.wrapping_add(20), 865, 40, 80)?)
    }

    pub fn servant_change_support_check_region(&self, slot: FieldSlot) -> LocationResult<Region> {
        let x = self.locate_skill(slot.skill2()).x;
        Ok(Region::new(x.wrapping_add(25), 710, 300, 170)?)
    }

    pub fn image_region(&self, skill: Servant) -> LocationResult<Region> {
        Ok(Region::new(22, 28, 30, 30)? + self.locate_skill(skill))
    }
}

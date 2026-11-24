use automata::{Location, Region};

use super::location_error::LocationResult;
use super::script_area_transforms::{
    IScriptAreaTransforms, Transforms, delegate_script_area_transforms,
};
use crate::enums::game_server_enum::GameServer;
use crate::models::command_card::{CommandCard, Face, NP};

pub struct AttackScreenLocations {
    transforms: Transforms,
    face_card_delta_y: Location,
    pub back_click: Location,
}

delegate_script_area_transforms!(AttackScreenLocations, transforms);

impl AttackScreenLocations {
    pub fn new(transforms: Transforms) -> Self {
        let face_card_delta_y = Location::new(
            0,
            if transforms.game_server() == GameServer::Cn && transforms.is_wide() {
                -42
            } else {
                0
            },
        );
        let back_click = transforms.x_from_right_location(if transforms.is_wide() {
            Location::new(-325, 1310)
        } else {
            Location::new(-160, 1370)
        });
        Self {
            transforms,
            face_card_delta_y,
            back_click,
        }
    }

    fn face_click_location(&self, card: Face) -> Location {
        let x = match card {
            Face::A => -980,
            Face::B => -530,
            Face::C => 20,
            Face::D => 520,
            Face::E => 1070,
        };
        Location::new(x, 1000)
    }

    pub fn click_location(&self, card: CommandCard) -> Location {
        let location = match card {
            CommandCard::Face(face) => self.face_click_location(face),
            CommandCard::NP(NP::A) => Location::new(-280, 220),
            CommandCard::NP(NP::B) => Location::new(20, 400),
            CommandCard::NP(NP::C) => Location::new(460, 400),
        };
        self.x_from_center_location(location)
    }

    pub fn affinity_region(&self, card: Face) -> LocationResult<Region> {
        let x = match card {
            Face::A => -985,
            Face::B => -470,
            Face::C => 41,
            Face::D => 554,
            Face::E => 1068,
        };
        Ok(self.x_from_center_region(Region::new(x, 590, 250, 260)? + self.face_card_delta_y))
    }

    fn type_column_x(card: Face) -> i32 {
        match card {
            Face::A => -1280,
            Face::B => -768,
            Face::C => -256,
            Face::D => 256,
            Face::E => 768,
        }
    }

    pub fn type_region(&self, card: Face) -> LocationResult<Region> {
        Ok(self.x_from_center_region(
            Region::new(Self::type_column_x(card), 1060, 512, 200)? + self.face_card_delta_y,
        ))
    }

    pub fn swap_icon_region(&self, card: Face) -> LocationResult<Region> {
        Ok(self.x_from_center_region(
            Region::new(Self::type_column_x(card) + 240, 1040, 272, 240)? + self.face_card_delta_y,
        ))
    }

    pub fn servant_match_region(&self, card: Face) -> LocationResult<Region> {
        let x = match card {
            Face::A => -1174,
            Face::B => -660,
            Face::C => -150,
            Face::D => 364,
            Face::E => 880,
        };
        Ok(
            self.x_from_center_region(
                Region::new(x - 100, 700, 500, 400)? + self.face_card_delta_y,
            ),
        )
    }

    pub fn support_check_region(&self, card: Face) -> LocationResult<Region> {
        Ok(self.affinity_region(card)? + Location::new(-50, 100))
    }
}

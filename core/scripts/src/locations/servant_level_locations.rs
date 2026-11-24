use automata::{Location, Region};

use super::location_error::LocationResult;
use super::script_area_transforms::{
    IScriptAreaTransforms, Transforms, delegate_script_area_transforms,
};
use crate::enums::game_server_enum::GameServer;

pub struct ServantLevelLocations {
    transforms: Transforms,
    pub ember_confirmation_dialog_region: Region,
    pub servant_auto_select_region: Region,
    pub empty_ember_or_qp_dialog_region: Region,
    pub final_confirm_region: Region,
    pub servant_max_level_region: Region,
    pub auto_select_min_ember_low_qp_region: Region,
    pub auto_select_min_ember_low_qp_location: Location,
    pub return_to_servant_menu_from_ascension_location: Location,
    pub ascension_return_to_level_region: Region,
    pub auto_fill_state_region: Region,
}

delegate_script_area_transforms!(ServantLevelLocations, transforms);

impl ServantLevelLocations {
    pub fn new(transforms: Transforms) -> LocationResult<Self> {
        let t = &*transforms;
        let wide = t.is_wide();
        let tw = t.game_server() == GameServer::Tw;

        let ember_confirmation_dialog_region = if tw {
            t.x_from_center_region(Region::new(338, 1229, 130, 70)?)
        } else {
            t.x_from_center_region(Region::new(250, 1204, 300, 116)?)
        };

        let servant_auto_select_region = if tw {
            let region = t.x_from_center_region(Region::new(1022, 0, 258, 73)?);
            Region::new(
                region.x(),
                if wide { 258 } else { 298 },
                region.width(),
                region.height(),
            )?
        } else {
            let region = t.x_from_center_region(Region::new(742, 0, 304, 72)?);
            Region::new(
                region.x(),
                if wide { 268 } else { 308 },
                region.width(),
                region.height(),
            )?
        };

        let empty_ember_or_qp_dialog_region =
            t.x_from_center_region(Region::new(-113, 1086, 224, 76)?);
        let final_confirm_region = t.x_from_center_region(Region::new(339, 1143, 124, 64)?);
        let servant_max_level_region = if wide {
            t.x_from_center_region(Region::new(613, 1007, 58, 128)?)
        } else {
            t.x_from_center_region(Region::new(613, 1047, 58, 128)?)
        };
        let auto_select_min_ember_low_qp_region =
            t.x_from_center_region(Region::new(339, 1092, 128, 66)?);
        let auto_select_min_ember_low_qp_location =
            t.x_from_center_location(Location::new(402, 1126));
        let return_to_servant_menu_from_ascension_location =
            t.x_from_center_location(Location::new(816, 572));
        let ascension_return_to_level_region = if wide {
            t.x_from_center_region(Region::new(566, 487, 47, 86)?)
        } else {
            t.x_from_center_region(Region::new(566, 527, 47, 86)?)
        };
        let auto_fill_state_region = t.x_from_center_region(Region::new(265, 836, 100, 80)?);

        Ok(Self {
            transforms,
            ember_confirmation_dialog_region,
            servant_auto_select_region,
            empty_ember_or_qp_dialog_region,
            final_confirm_region,
            servant_max_level_region,
            auto_select_min_ember_low_qp_region,
            auto_select_min_ember_low_qp_location,
            return_to_servant_menu_from_ascension_location,
            ascension_return_to_level_region,
            auto_fill_state_region,
        })
    }

    pub fn servant_redirect_check_region(&self, server: GameServer) -> LocationResult<Region> {
        let wide = self.is_wide();
        let region = match server {
            GameServer::En(_) | GameServer::Jp(_) => {
                if wide {
                    Region::new(687, 1013, 47, 115)?
                } else {
                    Region::new(687, 1058, 47, 115)?
                }
            }
            GameServer::Cn | GameServer::Tw | GameServer::Kr => {
                if wide {
                    Region::new(774, 1013, 50, 114)?
                } else {
                    Region::new(774, 1058, 50, 114)?
                }
            }
        };
        Ok(self.x_from_center_region(region))
    }
}

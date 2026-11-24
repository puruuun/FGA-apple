use std::sync::{Mutex, OnceLock};

use automata::{Location, Match, Region};

use super::location_error::LocationResult;
use super::script_area_transforms::{
    IScriptAreaTransforms, Transforms, delegate_script_area_transforms,
};
use crate::models::skill::{CommandSpell, Master};
use crate::script_log::ScriptLog;

pub trait BattleMenuFinder: Send + Sync {
    fn find_battle_menu(&self, search: Region) -> Option<Match>;
}

pub struct MasterLocations {
    transforms: Transforms,
    finder: Box<dyn BattleMenuFinder>,
    master_offset_new_ui: OnceLock<Location>,
    logged: Mutex<Vec<ScriptLog>>,
    pub command_spell_click: Location,
    pub cancel_command_spell_region: Region,
}

delegate_script_area_transforms!(MasterLocations, transforms);

impl MasterLocations {
    pub fn new(transforms: Transforms, finder: Box<dyn BattleMenuFinder>) -> LocationResult<Self> {
        let t = &*transforms;
        let command_spell_click = t.x_from_right_location(if t.is_wide() {
            Location::new(-494, 150)
        } else {
            Location::new(-424, 150)
        });
        let cancel_command_spell_region = t.x_from_center_region(Region::new(-202, 764, 95, 174)?);

        Ok(Self {
            transforms,
            finder,
            master_offset_new_ui: OnceLock::new(),
            logged: Mutex::new(Vec::new()),
            command_spell_click,
            cancel_command_spell_region,
        })
    }

    pub fn logged(&self) -> Vec<ScriptLog> {
        match self.logged.lock() {
            Ok(guard) => guard.clone(),
            Err(poisoned) => poisoned.into_inner().clone(),
        }
    }

    pub fn default_offset_logged(&self) -> bool {
        self.logged().contains(&ScriptLog::DefaultMasterOffset)
    }

    fn log(&self, item: ScriptLog) {
        match self.logged.lock() {
            Ok(mut guard) => guard.push(item),
            Err(poisoned) => poisoned.into_inner().push(item),
        }
    }

    pub fn battle_menu_search_region(&self) -> LocationResult<Region> {
        Ok(self.x_from_right_region(Region::new(-400, 360, 400, 80)?))
    }

    fn master_offset_new_ui(&self) -> LocationResult<Location> {
        if let Some(offset) = self.master_offset_new_ui.get() {
            return Ok(*offset);
        }
        let found = self
            .finder
            .find_battle_menu(self.battle_menu_search_region()?)
            .map(|found| Location::new(found.region.center().x, 0));
        let offset = match found {
            Some(offset) => offset,
            None => {
                let offset = self.x_from_right_location(Location::new(-298, 0));
                self.log(ScriptLog::DefaultMasterOffset);
                offset
            }
        };
        Ok(*self.master_offset_new_ui.get_or_init(|| offset))
    }

    pub fn locate_master(&self, skill: Master) -> LocationResult<Location> {
        let x = match skill {
            Master::A => -740,
            Master::B => -560,
            Master::C => -400,
        };
        Ok(Location::new(x + 178, 620) + self.master_offset_new_ui()?)
    }

    pub fn stage_count_region(&self) -> LocationResult<Region> {
        let x = if self.is_wide() { -571 } else { -638 };
        Ok(Region::new(x, 23, 33, 53)? + self.master_offset_new_ui()?)
    }

    pub fn master_skill_open_click(&self) -> LocationResult<Location> {
        Ok(Location::new(0, 640) + self.master_offset_new_ui()?)
    }

    pub fn locate_command_spell(&self, skill: CommandSpell) -> Location {
        let y = match skill {
            CommandSpell::NpCharge => 700,
        };
        self.x_from_center_location(Location::new(0, y))
    }
}

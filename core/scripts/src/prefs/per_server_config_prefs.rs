use crate::enums::refill_resource_enum::RefillResourceEnum;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PerServerConfigPrefs {
    pub rainbow_apple: i32,
    pub gold_apple: i32,
    pub silver_apple: i32,
    pub blue_apple: i32,
    pub copper_apple: i32,
    pub wait_for_ap_regen: bool,
    pub use_storm_pod: bool,
    pub use_teapot: bool,
    pub selected_apple: RefillResourceEnum,
    pub resources: Vec<RefillResourceEnum>,
    pub should_limit_runs: bool,
    pub limit_runs: i32,
    pub should_limit_mats: bool,
    pub limit_mats: i32,
    pub should_limit_ces: bool,
    pub limit_ces: i32,
}

impl Default for PerServerConfigPrefs {
    fn default() -> PerServerConfigPrefs {
        PerServerConfigPrefs {
            rainbow_apple: 0,
            gold_apple: 0,
            silver_apple: 0,
            blue_apple: 0,
            copper_apple: 0,
            wait_for_ap_regen: false,
            use_storm_pod: false,
            use_teapot: false,
            selected_apple: RefillResourceEnum::Copper,
            resources: Vec::new(),
            should_limit_runs: false,
            limit_runs: 1,
            should_limit_mats: false,
            limit_mats: 1,
            should_limit_ces: false,
            limit_ces: 1,
        }
    }
}

impl PerServerConfigPrefs {
    pub fn current_apple_count(&self) -> i32 {
        match self.selected_apple {
            RefillResourceEnum::Copper => self.copper_apple,
            RefillResourceEnum::Bronze => self.blue_apple,
            RefillResourceEnum::Silver => self.silver_apple,
            RefillResourceEnum::Gold => self.gold_apple,
            RefillResourceEnum::SQ => self.rainbow_apple,
        }
    }
}

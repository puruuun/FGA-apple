use std::collections::BTreeMap;
use std::sync::LazyLock;

use super::execution_profile::ExecutionProfile;
use super::game_server_stand_in::GameServerStandIn;
use super::pref_maker::{PrefMaker, PrefStore, PrefType};
use super::support_preferences::SupportPreferences;
use crate::enums::command_start_combine_mode::CommandStartCombineMode;
use crate::enums::command_start_mode::CommandStartMode;
use crate::enums::material_enum::MaterialEnum;
use crate::enums::shuffle_cards_enum::ShuffleCardsEnum;
use crate::kotlin::{LinkedSet, to_int};
use crate::models::model_error::ModelError;
use crate::models::spam_config::{ServantSpamConfig, parse_servant_spam_configs};

const STREAK_COUNT_KEY: &str = "turn_limit_retreat_streak_count";
const DEFAULT_STREAK_COUNT: i32 = 3;

static PREF_TYPES: LazyLock<BTreeMap<String, PrefType>> = LazyLock::new(|| {
    let mut maker = PrefMaker::schema_only();
    BattleConfig::read("", &mut maker);
    maker.into_types()
});

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BattleConfig {
    pub id: String,
    pub name: String,
    pub folder: String,
    pub notes: String,
    pub execution: ExecutionProfile,
    pub secondary_execution: ExecutionProfile,
    pub tertiary_execution: ExecutionProfile,
    pub command_start_mode: CommandStartMode,
    pub command_start_servant: i32,
    pub command_start_card_count: i32,
    pub command_start_buster: i32,
    pub command_start_arts: i32,
    pub command_start_quick: i32,
    pub command_start_combine_mode: CommandStartCombineMode,
    pub command_start_max_turns: i32,
    pub secondary_start_enabled: bool,
    pub secondary_start_servant: i32,
    pub secondary_start_card_count: i32,
    pub secondary_start_buster: i32,
    pub secondary_start_arts: i32,
    pub secondary_start_quick: i32,
    pub secondary_start_combine_mode: CommandStartCombineMode,
    pub tertiary_start_enabled: bool,
    pub tertiary_start_servant: i32,
    pub tertiary_start_card_count: i32,
    pub tertiary_start_buster: i32,
    pub tertiary_start_arts: i32,
    pub tertiary_start_quick: i32,
    pub tertiary_start_combine_mode: CommandStartCombineMode,
    pub spam: Vec<ServantSpamConfig>,
    pub track_bond: bool,
    pub stop_on_bond_max: bool,
    pub bond_max_ignored_characters: LinkedSet<i32>,
    pub shuffle_cards: ShuffleCardsEnum,
    pub shuffle_cards_wave: i32,
    pub card_swap_detection: bool,
    pub party: i32,
    pub materials: LinkedSet<MaterialEnum>,
    pub support: SupportPreferences,
    pub auto_choose_target: bool,
    pub server: Option<GameServerStandIn>,
    pub add_raid_turn_delay: bool,
    pub raid_turn_delay_seconds: i32,
    pub turn_limit_retreat_enabled: bool,
    pub turn_limit_retreat_turns: i32,
    pub turn_limit_retreat_stop_after_streak: bool,
    pub turn_limit_retreat_streak_count: i32,
}

impl BattleConfig {
    pub fn pref_types() -> &'static BTreeMap<String, PrefType> {
        &PREF_TYPES
    }

    pub fn from_store(id: &str, store: &PrefStore) -> BattleConfig {
        BattleConfig::read(id, &mut PrefMaker::new(store))
    }

    pub fn execution_profile(&self, index: i32) -> Result<&ExecutionProfile, ModelError> {
        match index {
            0 => Ok(&self.execution),
            1 => Ok(&self.secondary_execution),
            2 => Ok(&self.tertiary_execution),
            _ => Err(ModelError::IllegalState(format!(
                "Invalid execution profile: {index}"
            ))),
        }
    }

    fn read(id: &str, maker: &mut PrefMaker) -> BattleConfig {
        let name = maker.string("autoskill_name", "--");
        let folder = maker.string("battle_config_folder", "");
        let execution = ExecutionProfile::read(maker, "");
        let secondary_execution = ExecutionProfile::read(maker, "secondary_");
        let tertiary_execution = ExecutionProfile::read(maker, "tertiary_");
        let card_swap_detection = maker.bool("card_swap_detection", false);
        let shuffle_cards = maker.enum_pref(
            "shuffle_cards",
            ShuffleCardsEnum::None,
            ShuffleCardsEnum::from_name,
        );
        let shuffle_cards_wave = maker.int("shuffle_cards_wave", 3);
        let default_spam: Vec<ServantSpamConfig> =
            (1..=6).map(|_| ServantSpamConfig::default()).collect();
        let spam = maker.serialized("spam_x", default_spam.clone(), |text| {
            parse_servant_spam_configs(text).unwrap_or(default_spam)
        });
        let command_start_mode = maker.enum_pref(
            "command_start_mode",
            CommandStartMode::Basic,
            CommandStartMode::from_name,
        );
        let command_start_servant = maker.int("command_start_servant", 1);
        let command_start_card_count = maker.int("command_start_card_count", 2);
        let command_start_buster = maker.int("command_start_buster", 0);
        let command_start_arts = maker.int("command_start_arts", 0);
        let command_start_quick = maker.int("command_start_quick", 0);
        let command_start_combine_mode = maker.enum_pref(
            "command_start_combine_mode",
            CommandStartCombineMode::And,
            CommandStartCombineMode::from_name,
        );
        let command_start_max_turns = maker.int("command_start_max_turns", 0);
        let secondary_start_enabled = maker.bool("secondary_start_enabled", false);
        let secondary_start_servant = maker.int("secondary_start_servant", 2);
        let secondary_start_card_count = maker.int("secondary_start_card_count", 2);
        let secondary_start_buster = maker.int("secondary_start_buster", 0);
        let secondary_start_arts = maker.int("secondary_start_arts", 0);
        let secondary_start_quick = maker.int("secondary_start_quick", 0);
        let secondary_start_combine_mode = maker.enum_pref(
            "secondary_start_combine_mode",
            CommandStartCombineMode::And,
            CommandStartCombineMode::from_name,
        );
        let tertiary_start_enabled = maker.bool("tertiary_start_enabled", false);
        let tertiary_start_servant = maker.int("tertiary_start_servant", 3);
        let tertiary_start_card_count = maker.int("tertiary_start_card_count", 2);
        let tertiary_start_buster = maker.int("tertiary_start_buster", 0);
        let tertiary_start_arts = maker.int("tertiary_start_arts", 0);
        let tertiary_start_quick = maker.int("tertiary_start_quick", 0);
        let tertiary_start_combine_mode = maker.enum_pref(
            "tertiary_start_combine_mode",
            CommandStartCombineMode::And,
            CommandStartCombineMode::from_name,
        );
        let track_bond = maker.bool("bond_result_track", false);
        let stop_on_bond_max = maker.bool("bond_result_stop_on_max", false);
        let bond_max_ignored_characters = maker.serialized(
            "bond_result_stop_characters",
            LinkedSet::new(),
            ignored_characters,
        );
        let notes = maker.string("autoskill_notes", "");
        let party = maker.string_as_int("autoskill_party", -1);
        let materials: LinkedSet<MaterialEnum> = maker
            .string_set("battle_config_mat")
            .iter()
            .filter_map(|name| MaterialEnum::from_name(name))
            .collect();
        let server = maker.serialized("battle_config_server", None, GameServerStandIn::deserialize);
        let support = SupportPreferences::read(maker);
        let auto_choose_target = maker.bool("auto_choose_target", false);
        let add_raid_turn_delay = maker.bool("add_raid_delay", false);
        let raid_turn_delay_seconds = maker.string_as_int("raid_delay_seconds", 3);
        let turn_limit_retreat_enabled = maker.bool("turn_limit_retreat_enabled", false);
        let turn_limit_retreat_turns = maker.string_as_int("turn_limit_retreat_turns", 0);
        let turn_limit_retreat_stop_after_streak =
            maker.bool("turn_limit_retreat_stop_after_streak", false);
        let turn_limit_retreat_streak_count = maker.int(STREAK_COUNT_KEY, DEFAULT_STREAK_COUNT);

        BattleConfig {
            id: id.to_string(),
            name,
            folder,
            notes,
            execution,
            secondary_execution,
            tertiary_execution,
            command_start_mode,
            command_start_servant,
            command_start_card_count,
            command_start_buster,
            command_start_arts,
            command_start_quick,
            command_start_combine_mode,
            command_start_max_turns,
            secondary_start_enabled,
            secondary_start_servant,
            secondary_start_card_count,
            secondary_start_buster,
            secondary_start_arts,
            secondary_start_quick,
            secondary_start_combine_mode,
            tertiary_start_enabled,
            tertiary_start_servant,
            tertiary_start_card_count,
            tertiary_start_buster,
            tertiary_start_arts,
            tertiary_start_quick,
            tertiary_start_combine_mode,
            spam,
            track_bond,
            stop_on_bond_max,
            bond_max_ignored_characters,
            shuffle_cards,
            shuffle_cards_wave,
            card_swap_detection,
            party,
            materials,
            support,
            auto_choose_target,
            server,
            add_raid_turn_delay,
            raid_turn_delay_seconds,
            turn_limit_retreat_enabled,
            turn_limit_retreat_turns,
            turn_limit_retreat_stop_after_streak,
            turn_limit_retreat_streak_count,
        }
    }
}

fn ignored_characters(raw: &str) -> LinkedSet<i32> {
    let watched: LinkedSet<i32> = raw
        .split(',')
        .filter_map(|part| to_int(part).ok())
        .filter(|number| (1..=5).contains(number))
        .collect();
    (1..=5).filter(|number| !watched.contains(number)).collect()
}

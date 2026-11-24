use super::pref_maker::PrefMaker;
use crate::enums::bond_ce_effect_enum::BondCEEffectEnum;
use crate::enums::support_class::SupportClass;
use crate::enums::support_selection_mode_enum::SupportSelectionModeEnum;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SupportPreferences {
    pub friend_names: Vec<String>,
    pub preferred_servants: Vec<String>,
    pub mlb: bool,
    pub preferred_ces: Vec<String>,
    pub preferred_noble_phantasms: Vec<String>,
    pub friends_only: bool,
    pub auto_friend_request: bool,
    pub selection_mode: SupportSelectionModeEnum,
    pub fallback_to: SupportSelectionModeEnum,
    pub support_class: SupportClass,
    pub also_check_all: bool,
    pub max_ascended: bool,
    pub skill1_max: bool,
    pub skill2_max: bool,
    pub skill3_max: bool,
    pub append_skill_check: bool,
    pub append_skills_max: Vec<bool>,
    pub score62_check: bool,
    pub grand_score16_check: bool,
    pub np_level_check: bool,
    pub minimum_np_level: i32,
    pub grand_servant: bool,
    pub bond_ce_effect: BondCEEffectEnum,
    pub require_both_normal_and_reward_match: bool,
}

impl SupportPreferences {
    pub fn read(maker: &mut PrefMaker) -> SupportPreferences {
        let friend_names = maker.string_set("support_friend_names_list");
        let preferred_servants = maker.string_set("support_pref_servant_list");
        let mlb = maker.bool("support_pref_ce_mlb", false);
        let preferred_ces = maker.string_set("support_pref_ce_list");
        let preferred_noble_phantasms = maker.string_set("support_pref_np_list");
        let friends_only = maker.bool("support_friends_only", false);
        let auto_friend_request = maker.bool("support_auto_friend_request", false);
        let selection_mode = maker.enum_pref(
            "support_mode",
            SupportSelectionModeEnum::Preferred,
            SupportSelectionModeEnum::from_name,
        );
        let fallback_to = maker.enum_pref(
            "support_fallback",
            SupportSelectionModeEnum::Manual,
            SupportSelectionModeEnum::from_name,
        );
        let support_class = maker.enum_pref(
            "autoskill_support_class",
            SupportClass::None,
            SupportClass::from_name,
        );
        let also_check_all = maker.bool("also_check_all", false);
        let max_ascended = maker.bool("support_max_ascended", false);
        let skill1_max = maker.bool("support_skill_max_1", false);
        let skill2_max = maker.bool("support_skill_max_2", false);
        let skill3_max = maker.bool("support_skill_max_3", false);
        let append_skill_check = maker.bool("support_append_skill_check", false);
        let append_skills_max = (1..=5)
            .map(|index| maker.bool(&format!("support_append_skill_max_{index}"), false))
            .collect();
        let score62_check = maker.bool("support_score_62_check", false);
        let grand_score16_check = maker.bool("support_grand_score_16_check", false);
        let minimum_np_level = maker.int("support_np_min_level", 5);
        let stored_np_check = maker.bool("support_np_level_check", false);
        let np_level_check = stored_np_check && minimum_np_level > 1;
        let grand_servant = maker.bool("support_grand_servant", false);
        let bond_ce_effect = maker.enum_pref(
            "support_bond_ce_effect",
            BondCEEffectEnum::Ignore,
            BondCEEffectEnum::from_name,
        );
        let require_both_normal_and_reward_match =
            maker.bool("support_require_both_normal_and_reward_match", false);

        SupportPreferences {
            friend_names,
            preferred_servants,
            mlb,
            preferred_ces,
            preferred_noble_phantasms,
            friends_only,
            auto_friend_request,
            selection_mode,
            fallback_to,
            support_class,
            also_check_all,
            max_ascended,
            skill1_max,
            skill2_max,
            skill3_max,
            append_skill_check,
            append_skills_max,
            score62_check,
            grand_score16_check,
            np_level_check,
            minimum_np_level,
            grand_servant,
            bond_ce_effect,
            require_both_normal_and_reward_match,
        }
    }
}

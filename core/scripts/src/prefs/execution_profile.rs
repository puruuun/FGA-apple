use super::pref_maker::PrefMaker;
use crate::enums::brave_chain_enum::BraveChainEnum;
use crate::enums::card_priority_split::CardPrioritySplit;
use crate::enums::card_priority_style::CardPriorityStyle;
use crate::models::card_priority_per_wave::CardPriorityPerWave;
use crate::models::model_error::ModelError;
use crate::models::per_entry_settings;
use crate::models::servant_card_priority_per_wave::ServantCardPriorityPerWave;
use crate::models::servant_priority_per_wave::ServantPriorityPerWave;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionProfile {
    pub skill_command: String,
    pub card_priority: Result<CardPriorityPerWave, ModelError>,
    pub card_priority_split: CardPrioritySplit,
    pub card_priority_style: CardPriorityStyle,
    pub card_priority_styles: Vec<CardPriorityStyle>,
    pub servant_card_priority: ServantCardPriorityPerWave,
    pub use_servant_priority: bool,
    pub use_servant_priorities: Vec<bool>,
    pub servant_priority: ServantPriorityPerWave,
    pub rearrange_cards: Vec<bool>,
    pub brave_chains: Vec<BraveChainEnum>,
}

impl ExecutionProfile {
    pub fn read(maker: &mut PrefMaker, prefix: &str) -> ExecutionProfile {
        let key = |name: &str| format!("{prefix}{name}");

        let skill_command = maker.string(&key("autoskill_cmd"), "");
        let card_priority = maker.serialized(
            &key("card_priority"),
            Ok(CardPriorityPerWave::default()),
            CardPriorityPerWave::of,
        );
        let card_priority_split = maker.enum_pref(
            &key("card_priority_split"),
            CardPrioritySplit::Wave,
            CardPrioritySplit::from_name,
        );
        let card_priority_style = maker.enum_pref(
            &key("card_priority_style"),
            CardPriorityStyle::Affinity,
            CardPriorityStyle::from_name,
        );
        let card_priority_styles = maker.serialized(
            &key("card_priority_styles"),
            Vec::new(),
            per_entry_settings::parse_styles,
        );
        let servant_card_priority = maker.serialized(
            &key("card_priority_servant_list"),
            ServantCardPriorityPerWave::default(),
            |text| ServantCardPriorityPerWave::of(text).unwrap_or_default(),
        );
        let rearrange_cards =
            maker.serialized(&key("auto_skill_rearrange_cards"), Vec::new(), |text| {
                text.split(',').map(|part| part == "T").collect()
            });
        let brave_chains = maker.serialized(&key("auto_skill_brave_chains"), Vec::new(), |text| {
            text.split(',')
                .map(|part| BraveChainEnum::from_name(part).unwrap_or(BraveChainEnum::None))
                .collect()
        });
        let use_servant_priority = maker.bool(&key("use_servant_priority"), false);
        let use_servant_priorities = maker.serialized(
            &key("use_servant_priorities"),
            Vec::new(),
            per_entry_settings::parse_flags,
        );
        let servant_priority = maker.serialized(
            &key("servant_priority"),
            ServantPriorityPerWave::default(),
            |text| ServantPriorityPerWave::of(text).unwrap_or_default(),
        );

        ExecutionProfile {
            skill_command,
            card_priority,
            card_priority_split,
            card_priority_style,
            card_priority_styles,
            servant_card_priority,
            use_servant_priority,
            use_servant_priorities,
            servant_priority,
            rearrange_cards,
            brave_chains,
        }
    }
}

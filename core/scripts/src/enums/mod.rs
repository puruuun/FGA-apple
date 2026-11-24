macro_rules! kotlin_enum {
    ($name:ident { $($variant:ident),+ $(,)? }) => {
        #[derive(
            Debug,
            Clone,
            Copy,
            PartialEq,
            Eq,
            Hash,
            PartialOrd,
            Ord,
            serde::Serialize,
            serde::Deserialize,
        )]
        pub enum $name {
            $($variant),+
        }

        impl $name {
            pub const ENTRIES: &'static [$name] = &[$($name::$variant),+];

            pub fn name(self) -> &'static str {
                match self {
                    $($name::$variant => stringify!($variant)),+
                }
            }

            pub fn from_name(name: &str) -> Option<$name> {
                Self::ENTRIES.iter().copied().find(|entry| entry.name() == name)
            }

            pub fn ordinal(self) -> usize {
                self as usize
            }
        }

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.name())
            }
        }
    };
}

pub mod bond_ce_effect_enum;
pub mod brave_chain_enum;
pub mod card_affinity_enum;
pub mod card_priority_split;
pub mod card_priority_style;
pub mod card_type_enum;
pub mod command_start_combine_mode;
pub mod command_start_mode;
pub mod material_enum;
pub mod refill_resource_enum;
pub mod script_mode_enum;
pub mod shuffle_cards_enum;
pub mod spam_enum;
pub mod support_class;
pub mod support_selection_mode_enum;

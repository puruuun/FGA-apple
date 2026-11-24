#![forbid(unsafe_code)]

pub mod compare_by;
pub mod custom_game_area;
pub mod game_area_manager;
pub mod game_area_offsets;
pub mod kotlin_math;
pub mod location;
pub mod r#match;
pub mod region;
pub mod scale;
pub mod size;
pub mod transformer;

pub use compare_by::CompareBy;
pub use custom_game_area::custom_game_area;
pub use game_area_manager::GameAreaManager;
pub use game_area_offsets::GameAreaOffsets;
pub use kotlin_math::GeometryError;
pub use location::Location;
pub use r#match::Match;
pub use region::Region;
pub use scale::{RealScale, Scale};
pub use size::Size;
pub use transformer::{RealTransformer, Transformer};

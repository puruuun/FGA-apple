#![forbid(unsafe_code)]

pub mod cv_pattern;
pub mod gray_image;
pub mod score_map;
mod templmatch;
pub mod vision_error;

pub use cv_pattern::{CvPattern, FindMatches, TemplateMatch};
pub use gray_image::GrayImage;
pub use score_map::ScoreMap;
pub use vision_error::VisionError;

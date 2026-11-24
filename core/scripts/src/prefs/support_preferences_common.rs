#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SupportPreferencesCommon {
    pub mlb_similarity: f64,
    pub swipes_per_update: i32,
    pub max_updates: i32,
}

use crate::compare_by::CompareBy;
use crate::region::Region;

pub trait GameAreaManager {
    fn script_dimension(&self) -> CompareBy;

    fn compare_dimension(&self) -> CompareBy;

    fn game_area(&self) -> Region;
}

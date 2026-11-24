use crate::custom_game_area::custom_game_area;
use crate::kotlin_math::GeometryError;
use crate::region::Region;
use crate::size::Size;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct GameAreaOffsets {
    pub left: i32,
    pub top: i32,
    pub right: i32,
    pub bottom: i32,
}

fn coerce_in_ordered(value: i32, min: i32, max: i32) -> i32 {
    value.max(min).min(max)
}

impl GameAreaOffsets {
    pub const fn new(left: i32, top: i32, right: i32, bottom: i32) -> Self {
        Self {
            left,
            top,
            right,
            bottom,
        }
    }

    pub fn region(&self, screen: Size, reverse_landscape: bool) -> Result<Region, GeometryError> {
        custom_game_area(
            screen,
            self.left,
            self.top,
            self.right,
            self.bottom,
            reverse_landscape,
        )
    }

    pub fn maximums(&self, screen: Size) -> GameAreaOffsets {
        if screen.width() < 1280 || screen.height() < 720 {
            return GameAreaOffsets::default();
        }
        GameAreaOffsets::new(
            screen
                .width()
                .wrapping_sub(1280)
                .wrapping_sub(self.right)
                .max(0),
            screen
                .height()
                .wrapping_sub(720)
                .wrapping_sub(self.bottom)
                .max(0),
            screen
                .width()
                .wrapping_sub(1280)
                .wrapping_sub(self.left)
                .max(0),
            screen
                .height()
                .wrapping_sub(720)
                .wrapping_sub(self.top)
                .max(0),
        )
    }

    pub fn clamped(&self, screen: Size) -> GameAreaOffsets {
        if screen.width() < 1280 || screen.height() < 720 {
            return GameAreaOffsets::default();
        }
        let l = coerce_in_ordered(self.left, 0, screen.width() - 1280);
        let t = coerce_in_ordered(self.top, 0, screen.height() - 720);
        GameAreaOffsets::new(
            l,
            t,
            coerce_in_ordered(self.right, 0, screen.width() - 1280 - l),
            coerce_in_ordered(self.bottom, 0, screen.height() - 720 - t),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn size(width: i32, height: i32) -> Size {
        Size::new(width, height).unwrap()
    }

    fn offsets(left: i32, top: i32, right: i32, bottom: i32) -> GameAreaOffsets {
        GameAreaOffsets::new(left, top, right, bottom)
    }

    #[test]
    fn maximums_of_a_screen_that_is_too_small_are_zero() {
        let o = offsets(1, 2, 3, 4);
        assert_eq!(o.maximums(size(1279, 720)), offsets(0, 0, 0, 0));
        assert_eq!(o.maximums(size(1280, 719)), offsets(0, 0, 0, 0));
        assert_eq!(
            GameAreaOffsets::default().maximums(size(1280, 720)),
            offsets(0, 0, 0, 0)
        );
    }

    #[test]
    fn maximums_leave_room_for_the_opposite_side() {
        assert_eq!(
            offsets(10, 20, 30, 40).maximums(size(2000, 1000)),
            offsets(690, 240, 710, 260)
        );
        assert_eq!(
            offsets(900, 900, 900, 900).maximums(size(2000, 1000)),
            offsets(0, 0, 0, 0)
        );
    }

    #[test]
    fn clamped_of_a_screen_that_is_too_small_is_zero() {
        let o = offsets(1, 2, 3, 4);
        assert_eq!(o.clamped(size(1279, 720)), offsets(0, 0, 0, 0));
        assert_eq!(o.clamped(size(1280, 719)), offsets(0, 0, 0, 0));
    }

    #[test]
    fn clamped_keeps_valid_values_and_limits_the_rest() {
        let screen = size(2000, 1000);
        assert_eq!(
            offsets(10, 20, 30, 40).clamped(screen),
            offsets(10, 20, 30, 40)
        );
        assert_eq!(
            offsets(900, 900, 900, 900).clamped(screen),
            offsets(720, 280, 0, 0)
        );
        assert_eq!(offsets(-5, -6, -7, -8).clamped(screen), offsets(0, 0, 0, 0));
        assert_eq!(
            offsets(700, 10, 100, 10).clamped(screen),
            offsets(700, 10, 20, 10)
        );
    }

    #[test]
    fn region_uses_the_flag_for_the_origin() {
        let o = offsets(10, 20, 30, 40);
        assert_eq!(
            o.region(size(2000, 1000), false),
            Region::new(10, 20, 1960, 940)
        );
        assert_eq!(
            o.region(size(2000, 1000), true),
            Region::new(30, 20, 1960, 940)
        );
        assert_eq!(
            offsets(1000, 0, 1000, 0).region(size(2000, 1000), false),
            Err(GeometryError::WidthNotPositive)
        );
    }
}

use crate::kotlin_math::GeometryError;
use crate::region::Region;
use crate::size::Size;

pub fn custom_game_area(
    screen_size: Size,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
    reverse_landscape: bool,
) -> Result<Region, GeometryError> {
    Region::new(
        if reverse_landscape { right } else { left },
        top,
        screen_size.width().wrapping_sub(left).wrapping_sub(right),
        screen_size.height().wrapping_sub(top).wrapping_sub(bottom),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn size(width: i32, height: i32) -> Size {
        Size::new(width, height).unwrap()
    }

    #[test]
    fn origin_follows_the_reverse_landscape_flag() {
        assert_eq!(
            custom_game_area(size(2400, 1080), 50, 0, 100, 10, false),
            Region::new(50, 0, 2250, 1070)
        );
        assert_eq!(
            custom_game_area(size(2400, 1080), 50, 0, 100, 10, true),
            Region::new(100, 0, 2250, 1070)
        );
    }

    #[test]
    fn fails_when_the_offsets_leave_nothing() {
        assert_eq!(
            custom_game_area(size(100, 100), 60, 0, 60, 0, false),
            Err(GeometryError::WidthNotPositive)
        );
    }
}

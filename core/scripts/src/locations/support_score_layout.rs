use automata::Region;

use super::location_error::{LocationError, LocationResult};

pub struct SupportScoreLayout;

fn divide(value: i32, divisor: i32) -> LocationResult<i32> {
    if divisor == 0 {
        return Err(LocationError::Arithmetic("/ by zero".to_owned()));
    }
    Ok(value.wrapping_div(divisor))
}

impl SupportScoreLayout {
    pub const SCORE_X: i32 = 1960;
    pub const SCORE_Y: i32 = 310;
    pub const SCORE_WIDTH: i32 = 120;
    pub const SCORE_HEIGHT: i32 = 60;

    pub fn number_region(bounds: Region) -> LocationResult<Region> {
        Ok(Region::new(
            bounds.x().wrapping_add(Self::SCORE_X),
            bounds.y().wrapping_add(Self::SCORE_Y),
            Self::SCORE_WIDTH,
            Self::SCORE_HEIGHT,
        )?)
    }

    pub fn grand_last_digit(template_width: i32, template_height: i32) -> LocationResult<Region> {
        let width = if template_width >= 34 { 12 } else { 10 };
        Ok(Region::new(
            template_width.wrapping_sub(width),
            0,
            width,
            template_height,
        )?)
    }

    pub fn matched_pixels(
        matched: Region,
        template_width: i32,
        template_height: i32,
        pixels: Region,
    ) -> LocationResult<Region> {
        let x = matched.x().wrapping_add(divide(
            pixels.x().wrapping_mul(matched.width()),
            template_width,
        )?);
        let y = matched.y().wrapping_add(divide(
            pixels.y().wrapping_mul(matched.height()),
            template_height,
        )?);
        let width = divide(pixels.width().wrapping_mul(matched.width()), template_width)?;
        let height = divide(
            pixels.height().wrapping_mul(matched.height()),
            template_height,
        )?;
        Ok(Region::new(x, y, width, height)?)
    }
}

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GeometryError {
    WidthNotPositive,
    HeightNotPositive,
    NotANumber,
    EmptyRange,
}

impl fmt::Display for GeometryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            GeometryError::WidthNotPositive => "width must be positive",
            GeometryError::HeightNotPositive => "height must be positive",
            GeometryError::NotANumber => "cannot round NaN value",
            GeometryError::EmptyRange => "cannot coerce value to an empty range",
        };
        f.write_str(text)
    }
}

impl std::error::Error for GeometryError {}

pub fn round_to_int(value: f64) -> Result<i32, GeometryError> {
    if value.is_nan() {
        return Err(GeometryError::NotANumber);
    }
    if value > f64::from(i32::MAX) {
        return Ok(i32::MAX);
    }
    if value < f64::from(i32::MIN) {
        return Ok(i32::MIN);
    }
    let floor = value.floor();
    let rounded = if value - floor >= 0.5 {
        floor + 1.0
    } else {
        floor
    };
    Ok(rounded as i32)
}

pub fn coerce_in(value: i32, min: i32, max: i32) -> Result<i32, GeometryError> {
    if min > max {
        return Err(GeometryError::EmptyRange);
    }
    Ok(value.max(min).min(max))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounds_half_toward_positive_infinity() {
        assert_eq!(round_to_int(2.5), Ok(3));
        assert_eq!(round_to_int(-2.5), Ok(-2));
        assert_eq!(round_to_int(1.5), Ok(2));
        assert_eq!(round_to_int(-1.5), Ok(-1));
        assert_eq!(round_to_int(-0.5), Ok(0));
        assert_eq!(round_to_int(0.5), Ok(1));
    }

    #[test]
    fn rounds_just_below_half_down() {
        assert_eq!(round_to_int(0.49999999999999994), Ok(0));
        assert_eq!(round_to_int(-0.5000000000000001), Ok(-1));
    }

    #[test]
    fn saturates_and_rejects_nan() {
        assert_eq!(round_to_int(1e20), Ok(i32::MAX));
        assert_eq!(round_to_int(-1e20), Ok(i32::MIN));
        assert_eq!(round_to_int(f64::INFINITY), Ok(i32::MAX));
        assert_eq!(round_to_int(f64::NEG_INFINITY), Ok(i32::MIN));
        assert_eq!(round_to_int(f64::NAN), Err(GeometryError::NotANumber));
    }

    #[test]
    fn coerces_into_range() {
        assert_eq!(coerce_in(5, 1, 3), Ok(3));
        assert_eq!(coerce_in(-5, 1, 3), Ok(1));
        assert_eq!(coerce_in(2, 1, 3), Ok(2));
        assert_eq!(coerce_in(2, 3, 3), Ok(3));
        assert_eq!(coerce_in(2, 4, 3), Err(GeometryError::EmptyRange));
    }
}

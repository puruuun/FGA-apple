use crate::vision_error::VisionError;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GrayImage {
    width: usize,
    height: usize,
    pixels: Vec<u8>,
}

impl GrayImage {
    pub fn new(width: usize, height: usize, pixels: Vec<u8>) -> Result<Self, VisionError> {
        if i32::try_from(width).is_err() || i32::try_from(height).is_err() {
            return Err(VisionError::SideTooLarge);
        }
        let expected = width.checked_mul(height).ok_or(VisionError::SideTooLarge)?;
        if pixels.len() != expected {
            return Err(VisionError::PixelCountMismatch {
                expected,
                actual: pixels.len(),
            });
        }
        Ok(Self {
            width,
            height,
            pixels,
        })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }

    pub fn pixels(&self) -> &[u8] {
        &self.pixels
    }

    pub fn is_empty(&self) -> bool {
        self.pixels.is_empty()
    }

    pub fn row(&self, y: usize) -> &[u8] {
        let start = y * self.width;
        self.pixels.get(start..start + self.width).unwrap_or(&[])
    }

    pub fn crop(&self, x: i32, y: i32, width: i32, height: i32) -> Result<GrayImage, VisionError> {
        let (left, top, w, h) = clip(self.width, self.height, x, y, width, height)?;
        let mut pixels = Vec::with_capacity(w * h);
        for row in top..top + h {
            let start = row * self.width + left;
            pixels.extend_from_slice(self.pixels.get(start..start + w).unwrap_or(&[]));
        }
        GrayImage::new(w, h, pixels)
    }
}

fn positive(width: i32, height: i32) -> Result<(), VisionError> {
    if width <= 0 {
        return Err(VisionError::WidthNotPositive);
    }
    if height <= 0 {
        return Err(VisionError::HeightNotPositive);
    }
    Ok(())
}

fn coerce_in(value: i32, min: i32, max: i32) -> Result<i32, VisionError> {
    if min > max {
        return Err(VisionError::EmptyRange);
    }
    Ok(value.max(min).min(max))
}

fn clip(
    image_width: usize,
    image_height: usize,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<(usize, usize, usize, usize), VisionError> {
    positive(width, height)?;
    let bounds_width = i32::try_from(image_width).map_err(|_| VisionError::SideTooLarge)?;
    let bounds_height = i32::try_from(image_height).map_err(|_| VisionError::SideTooLarge)?;
    positive(bounds_width, bounds_height)?;

    let left = coerce_in(x, 0, bounds_width - 1)?;
    let right = coerce_in(x.wrapping_add(width), 1, bounds_width)?;
    let top = coerce_in(y, 0, bounds_height - 1)?;
    let bottom = coerce_in(y.wrapping_add(height), 1, bounds_height)?;
    let clipped_width = right.wrapping_sub(left);
    let clipped_height = bottom.wrapping_sub(top);
    positive(clipped_width, clipped_height)?;

    let to_usize = |value: i32| usize::try_from(value).map_err(|_| VisionError::EmptyRange);
    Ok((
        to_usize(left)?,
        to_usize(top)?,
        to_usize(clipped_width)?,
        to_usize(clipped_height)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ramp(width: usize, height: usize) -> GrayImage {
        let pixels = (0..width * height).map(|i| (i % 251) as u8).collect();
        GrayImage::new(width, height, pixels).unwrap()
    }

    #[test]
    fn rejects_a_pixel_count_that_does_not_fit() {
        assert_eq!(
            GrayImage::new(3, 2, vec![0; 5]),
            Err(VisionError::PixelCountMismatch {
                expected: 6,
                actual: 5
            })
        );
        assert!(GrayImage::new(0, 0, Vec::new()).unwrap().is_empty());
    }

    #[test]
    fn crop_inside_copies_the_rows() {
        let image = ramp(5, 4);
        let cropped = image.crop(1, 2, 3, 2).unwrap();
        assert_eq!((cropped.width(), cropped.height()), (3, 2));
        assert_eq!(cropped.pixels(), &[11, 12, 13, 16, 17, 18]);
    }

    #[test]
    fn crop_clips_like_region_clip() {
        let image = ramp(10, 8);
        let partly = image.crop(-3, 6, 5, 5).unwrap();
        assert_eq!((partly.width(), partly.height()), (2, 2));
        assert_eq!(partly.pixels(), &[60, 61, 70, 71]);

        let beyond = image.crop(20, 20, 4, 4).unwrap();
        assert_eq!((beyond.width(), beyond.height()), (1, 1));
        assert_eq!(beyond.pixels(), &[79]);

        let before = image.crop(-20, -20, 4, 4).unwrap();
        assert_eq!(before.pixels(), &[0]);

        let whole = image.crop(-1, -1, 100, 100).unwrap();
        assert_eq!(whole, image);
    }

    #[test]
    fn crop_rejects_what_region_rejects() {
        let image = ramp(10, 8);
        assert_eq!(image.crop(0, 0, 0, 4), Err(VisionError::WidthNotPositive));
        assert_eq!(image.crop(0, 0, 4, -1), Err(VisionError::HeightNotPositive));
        let empty = GrayImage::new(0, 0, Vec::new()).unwrap();
        assert_eq!(empty.crop(0, 0, 4, 4), Err(VisionError::WidthNotPositive));
    }

    #[test]
    fn crop_fails_when_the_right_edge_overflows_like_kotlin() {
        let image = ramp(10, 8);
        assert_eq!(
            image.crop(i32::MAX - 1, 0, 5, 3),
            Err(VisionError::WidthNotPositive)
        );
    }
}

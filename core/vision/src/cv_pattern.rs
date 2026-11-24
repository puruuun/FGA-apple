use crate::gray_image::GrayImage;
use crate::score_map::ScoreMap;
use crate::templmatch::{ccoeff_normed, ccoeff_normed_masked};
use crate::vision_error::VisionError;

const FLOOD_FILL_MAX_DIFF: f32 = 0.3;

#[derive(Debug, Clone, PartialEq)]
pub struct TemplateMatch {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub score: f64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CvPattern {
    image: GrayImage,
    mask: Option<GrayImage>,
}

impl CvPattern {
    pub fn new(image: GrayImage) -> Self {
        Self { image, mask: None }
    }

    pub fn with_mask(image: GrayImage, mask: GrayImage) -> Result<Self, VisionError> {
        if mask.width() != image.width() || mask.height() != image.height() {
            return Err(VisionError::MaskSizeMismatch);
        }
        Ok(Self {
            image,
            mask: Some(mask),
        })
    }

    pub fn width(&self) -> usize {
        self.image.width()
    }

    pub fn height(&self) -> usize {
        self.image.height()
    }

    pub fn image(&self) -> &GrayImage {
        &self.image
    }

    pub fn mask(&self) -> Option<&GrayImage> {
        self.mask.as_ref()
    }

    pub fn crop(&self, x: i32, y: i32, width: i32, height: i32) -> Result<CvPattern, VisionError> {
        if self.mask.is_some() {
            return Err(VisionError::MaskedNotSupported);
        }
        Ok(CvPattern::new(self.image.crop(x, y, width, height)?))
    }

    pub fn match_template(&self, template: &CvPattern) -> Result<Option<ScoreMap>, VisionError> {
        if template.image.is_empty() {
            return Err(VisionError::EmptyTemplate);
        }
        if template.width() > self.width() || template.height() > self.height() {
            return Ok(None);
        }
        let map = match &template.mask {
            Some(mask) => {
                let mut map = ccoeff_normed_masked(&self.image, &template.image, mask);
                map.sanitize_masked_result();
                map
            }
            None => ccoeff_normed(&self.image, &template.image),
        };
        Ok(Some(map))
    }

    pub fn find_matches(
        &self,
        template: &CvPattern,
        similarity: f64,
    ) -> Result<FindMatches, VisionError> {
        let map = self.match_template(template)?;
        let side = |value: usize| i32::try_from(value).map_err(|_| VisionError::SideTooLarge);
        Ok(FindMatches::over(
            map,
            similarity,
            side(template.width())?,
            side(template.height())?,
        ))
    }

    pub fn find(
        &self,
        template: &CvPattern,
        similarity: f64,
    ) -> Result<Option<TemplateMatch>, VisionError> {
        Ok(self.find_matches(template, similarity)?.next())
    }

    pub fn exists(&self, template: &CvPattern, similarity: f64) -> Result<bool, VisionError> {
        Ok(self.find(template, similarity)?.is_some())
    }
}

#[derive(Debug, Clone)]
pub struct FindMatches {
    map: Option<ScoreMap>,
    similarity: f64,
    width: i32,
    height: i32,
}

impl FindMatches {
    pub(crate) fn over(map: Option<ScoreMap>, similarity: f64, width: i32, height: i32) -> Self {
        Self {
            map,
            similarity,
            width,
            height,
        }
    }
}

impl Iterator for FindMatches {
    type Item = TemplateMatch;

    fn next(&mut self) -> Option<TemplateMatch> {
        let map = self.map.as_mut()?;
        let found = map
            .max_loc()
            .filter(|&(score, _, _)| score >= self.similarity);
        let Some((score, x, y)) = found else {
            self.map = None;
            return None;
        };
        map.flood_fill(x, y, FLOOD_FILL_MAX_DIFF, 0.0);
        Some(TemplateMatch {
            x: i32::try_from(x).ok()?,
            y: i32::try_from(y).ok()?,
            width: self.width,
            height: self.height,
            score,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn found(map: ScoreMap) -> Vec<(i32, i32, f64)> {
        FindMatches::over(Some(map), 0.8, 1, 1)
            .take(10)
            .map(|m| (m.x, m.y, m.score))
            .collect()
    }

    #[test]
    fn flood_fill_is_four_connected_with_a_fixed_range_of_point_three() {
        let mut values = vec![0.0_f32; 7 * 5];
        for (x, y, value) in [
            (1, 1, 1.0),
            (2, 2, 0.95),
            (5, 1, 0.98),
            (5, 2, 0.73),
            (5, 3, 0.9),
            (3, 3, 0.97),
            (3, 4, 0.57),
            (4, 4, 0.85),
        ] {
            values[y * 7 + x] = value;
        }
        let expected = [
            (1, 1, 1.0),
            (5, 1, 0.9800000190734863),
            (3, 3, 0.9700000286102295),
            (2, 2, 0.949999988079071),
            (4, 4, 0.8500000238418579),
        ];
        let actual = found(ScoreMap::new(7, 5, values));
        assert_eq!(actual.len(), expected.len(), "{actual:?}");
        for (a, e) in actual.iter().zip(&expected) {
            assert_eq!((a.0, a.1), (e.0, e.1), "{actual:?}");
            assert!((a.2 - e.2).abs() <= 0.01, "{actual:?}");
        }
    }

    #[test]
    fn stops_at_the_first_maximum_below_similarity() {
        let map = ScoreMap::new(3, 1, vec![0.9, 0.0, 0.79]);
        let mut matches = FindMatches::over(Some(map), 0.8, 2, 3);
        let first = matches.next();
        assert_eq!(
            first,
            Some(TemplateMatch {
                x: 0,
                y: 0,
                width: 2,
                height: 3,
                score: f64::from(0.9_f32)
            })
        );
        assert_eq!(matches.next(), None);
        assert_eq!(matches.next(), None);
    }

    #[test]
    fn a_score_equal_to_similarity_is_a_match() {
        let map = ScoreMap::new(2, 1, vec![0.5, 0.8]);
        let found = FindMatches::over(Some(map), f64::from(0.8_f32), 1, 1).next();
        assert_eq!(found.map(|m| (m.x, m.y)), Some((1, 0)));
    }

    fn noise(seed: u32, width: usize, height: usize) -> GrayImage {
        let mut state = seed;
        let pixels = (0..width * height)
            .map(|_| {
                state = state.wrapping_mul(1103515245).wrapping_add(12345) & 0x7fff_ffff;
                (state >> 16) as u8
            })
            .collect();
        GrayImage::new(width, height, pixels).unwrap()
    }

    #[test]
    fn a_template_larger_than_the_image_is_no_match() {
        let image = CvPattern::new(noise(1, 5, 5));
        let wide = CvPattern::new(noise(2, 6, 2));
        let tall = CvPattern::new(noise(2, 2, 6));
        assert_eq!(image.match_template(&wide), Ok(None));
        assert_eq!(image.find_matches(&tall, -2.0).unwrap().next(), None);
        assert_eq!(image.exists(&wide, 0.0), Ok(false));
        let empty = CvPattern::new(GrayImage::new(0, 0, Vec::new()).unwrap());
        assert_eq!(empty.find(&tall, 0.0), Ok(None));
    }

    #[test]
    fn an_empty_template_or_a_wrong_mask_is_an_error() {
        let image = CvPattern::new(noise(1, 5, 5));
        let empty = CvPattern::new(GrayImage::new(0, 3, Vec::new()).unwrap());
        assert_eq!(
            image.match_template(&empty),
            Err(VisionError::EmptyTemplate)
        );
        assert_eq!(
            CvPattern::with_mask(noise(2, 3, 3), noise(3, 3, 2)),
            Err(VisionError::MaskSizeMismatch)
        );
    }

    #[test]
    fn a_masked_pattern_cannot_be_cropped() {
        let masked = CvPattern::with_mask(noise(2, 3, 3), noise(3, 3, 3)).unwrap();
        assert_eq!(
            masked.crop(0, 0, 2, 2),
            Err(VisionError::MaskedNotSupported)
        );
        let plain = CvPattern::new(noise(2, 3, 3));
        assert_eq!(plain.crop(1, 1, 5, 5).unwrap().width(), 2);
    }
}

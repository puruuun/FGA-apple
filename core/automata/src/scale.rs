use crate::compare_by::CompareBy;
use crate::game_area_manager::GameAreaManager;

pub const NO_SCALING: f64 = 1.0;

pub trait Scale {
    fn screen_to_image(&self) -> Option<f64>;

    fn script_to_screen(&self) -> f64;

    fn script_to_image(&self) -> f64;
}

pub struct RealScale<'a> {
    game_area_manager: &'a dyn GameAreaManager,
}

impl<'a> RealScale<'a> {
    pub fn new(game_area_manager: &'a dyn GameAreaManager) -> Self {
        Self { game_area_manager }
    }
}

impl Scale for RealScale<'_> {
    fn screen_to_image(&self) -> Option<f64> {
        let target_dimensions = if self.game_area_manager.compare_dimension() != CompareBy::None {
            self.game_area_manager.compare_dimension()
        } else {
            self.game_area_manager.script_dimension()
        };

        let game_area = self.game_area_manager.game_area();

        match target_dimensions {
            CompareBy::Width(width) => {
                if width == game_area.width() {
                    None
                } else {
                    Some(f64::from(width) / f64::from(game_area.width()))
                }
            }
            CompareBy::Height(height) => {
                if height == game_area.height() {
                    None
                } else {
                    Some(f64::from(height) / f64::from(game_area.height()))
                }
            }
            CompareBy::None => None,
        }
    }

    fn script_to_screen(&self) -> f64 {
        let source_region = self.game_area_manager.script_dimension();

        let target_region = self.game_area_manager.game_area();

        match source_region {
            CompareBy::Width(width) => {
                if target_region.width() == width {
                    NO_SCALING
                } else {
                    f64::from(target_region.width()) / f64::from(width)
                }
            }
            CompareBy::Height(height) => {
                if target_region.height() == height {
                    NO_SCALING
                } else {
                    f64::from(target_region.height()) / f64::from(height)
                }
            }
            CompareBy::None => NO_SCALING,
        }
    }

    fn script_to_image(&self) -> f64 {
        self.script_to_screen() * self.screen_to_image().unwrap_or(NO_SCALING)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::region::Region;

    struct FixedManager {
        script: CompareBy,
        compare: CompareBy,
        area: Region,
    }

    impl GameAreaManager for FixedManager {
        fn script_dimension(&self) -> CompareBy {
            self.script
        }

        fn compare_dimension(&self) -> CompareBy {
            self.compare
        }

        fn game_area(&self) -> Region {
            self.area
        }
    }

    fn scales(
        script: CompareBy,
        compare: CompareBy,
        width: i32,
        height: i32,
    ) -> (Option<f64>, f64, f64) {
        let manager = FixedManager {
            script,
            compare,
            area: Region::new(0, 0, width, height).unwrap(),
        };
        let scale = RealScale::new(&manager);
        (
            scale.screen_to_image(),
            scale.script_to_screen(),
            scale.script_to_image(),
        )
    }

    #[test]
    fn compare_none_falls_back_to_the_script_dimension() {
        assert_eq!(
            scales(CompareBy::Height(1440), CompareBy::None, 1280, 720),
            (Some(2.0), 0.5, 1.0)
        );
        assert_eq!(
            scales(CompareBy::Width(1000), CompareBy::None, 1000, 562),
            (None, 1.0, 1.0)
        );
    }

    #[test]
    fn equal_sizes_give_no_image_scale() {
        assert_eq!(
            scales(CompareBy::Width(2560), CompareBy::Width(1280), 1280, 720),
            (None, 0.5, 0.5)
        );
        assert_eq!(
            scales(CompareBy::Height(1440), CompareBy::Height(720), 2560, 1440),
            (Some(0.5), 1.0, 0.5)
        );
    }

    #[test]
    fn nothing_to_compare_means_no_scaling() {
        assert_eq!(
            scales(CompareBy::None, CompareBy::None, 1000, 500),
            (None, 1.0, 1.0)
        );
    }

    #[test]
    fn uneven_sizes_keep_the_double_arithmetic() {
        assert_eq!(
            scales(CompareBy::Height(1440), CompareBy::Height(720), 1000, 562),
            (Some(1.2811387900355873), 0.3902777777777778, 0.5)
        );
        assert_eq!(
            scales(CompareBy::Width(2560), CompareBy::Width(1280), 1001, 562),
            (Some(1.2787212787212787), 0.391015625, 0.49999999999999994)
        );
    }
}

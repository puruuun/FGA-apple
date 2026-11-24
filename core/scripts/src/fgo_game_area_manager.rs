use automata::kotlin_math::round_to_int;
use automata::{CompareBy, GameAreaManager, GeometryError, Location, Region, Size};

#[derive(Debug, Clone, Copy)]
enum ScaleBy {
    Width(f64),
    Height(f64),
}

impl ScaleBy {
    fn rate(self) -> f64 {
        match self {
            ScaleBy::Width(rate) | ScaleBy::Height(rate) => rate,
        }
    }
}

fn decide_scale_method(original_size: Size, desired_size: Size) -> ScaleBy {
    let width = ScaleBy::Width(f64::from(desired_size.width()) / f64::from(original_size.width()));
    let height =
        ScaleBy::Height(f64::from(desired_size.height()) / f64::from(original_size.height()));
    if width.rate() <= height.rate() {
        width
    } else {
        height
    }
}

fn calculate_border_thickness(outer: i32, inner: i32) -> Result<i32, GeometryError> {
    round_to_int(f64::from(outer.wrapping_sub(inner).wrapping_abs()) / 2.0)
}

fn calculate_game_area_without_borders(
    script_size: Size,
    screen_size: Size,
    scale_rate: f64,
) -> Result<Region, GeometryError> {
    let scaled_script_size = (script_size * scale_rate)?;

    Region::new(
        calculate_border_thickness(screen_size.width(), scaled_script_size.width())?,
        calculate_border_thickness(screen_size.height(), scaled_script_size.height())?,
        scaled_script_size.width(),
        scaled_script_size.height(),
    )
}

pub fn wider_than(size: Size, x: i32, y: i32) -> bool {
    f64::from(size.width()) / f64::from(size.height()) > f64::from(x) / f64::from(y)
}

pub fn is_wide(size: Size) -> bool {
    wider_than(size, 18, 9)
}

pub struct FgoGameAreaManager {
    script_dimension: CompareBy,
    compare_dimension: CompareBy,
    game_area_ignoring_notch: Region,
    offset: Box<dyn Fn() -> Location>,
}

impl FgoGameAreaManager {
    const IMAGE_WIDTH: i32 = 1280;
    const IMAGE_HEIGHT: i32 = 720;
    const SCRIPT_WIDTH: i32 = 2560;
    const SCRIPT_HEIGHT: i32 = 1440;
    const ULTRA_WIDE_SCRIPT_WIDTH: i32 = 3360;

    pub fn new(
        game_size_with_borders: Size,
        offset: impl Fn() -> Location + 'static,
    ) -> Result<Self, GeometryError> {
        let image_size = Size::new(Self::IMAGE_WIDTH, Self::IMAGE_HEIGHT)?;
        let script_size = Size::new(Self::SCRIPT_WIDTH, Self::SCRIPT_HEIGHT)?;
        let ultra_wide_script_size = Size::new(Self::ULTRA_WIDE_SCRIPT_WIDTH, Self::SCRIPT_HEIGHT)?;

        let scale_by = decide_scale_method(script_size, game_size_with_borders);

        let (script_dimension, compare_dimension) = match scale_by {
            ScaleBy::Width(_) => (
                CompareBy::Width(script_size.width()),
                CompareBy::Width(image_size.width()),
            ),
            ScaleBy::Height(_) => (
                CompareBy::Height(script_size.height()),
                CompareBy::Height(image_size.height()),
            ),
        };

        let game_area_ignoring_notch = if wider_than(game_size_with_borders, 21, 9) {
            calculate_game_area_without_borders(
                ultra_wide_script_size,
                game_size_with_borders,
                scale_by.rate(),
            )?
        } else if is_wide(game_size_with_borders) {
            Region::from_location_size(Location::default(), game_size_with_borders)
        } else {
            calculate_game_area_without_borders(
                script_size,
                game_size_with_borders,
                scale_by.rate(),
            )?
        };

        Ok(Self {
            script_dimension,
            compare_dimension,
            game_area_ignoring_notch,
            offset: Box::new(offset),
        })
    }
}

impl GameAreaManager for FgoGameAreaManager {
    fn script_dimension(&self) -> CompareBy {
        self.script_dimension
    }

    fn compare_dimension(&self) -> CompareBy {
        self.compare_dimension
    }

    fn game_area(&self) -> Region {
        self.game_area_ignoring_notch + (self.offset)()
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;
    use std::rc::Rc;

    use automata::{RealScale, RealTransformer, Scale, Transformer};

    use super::*;

    fn size(width: i32, height: i32) -> Size {
        Size::new(width, height).unwrap()
    }

    fn region(x: i32, y: i32, width: i32, height: i32) -> Region {
        Region::new(x, y, width, height).unwrap()
    }

    struct Observed {
        script: CompareBy,
        compare: CompareBy,
        area: Region,
        screen_to_image: Option<f64>,
        script_to_screen: f64,
        script_to_image: f64,
        wide: bool,
    }

    fn observe(width: i32, height: i32, offset: Location) -> Observed {
        let screen = size(width, height);
        let manager = FgoGameAreaManager::new(screen, move || offset).unwrap();
        let scale = RealScale::new(&manager);
        Observed {
            script: manager.script_dimension(),
            compare: manager.compare_dimension(),
            area: manager.game_area(),
            screen_to_image: scale.screen_to_image(),
            script_to_screen: scale.script_to_screen(),
            script_to_image: scale.script_to_image(),
            wide: is_wide(screen),
        }
    }

    fn check(
        width: i32,
        height: i32,
        offset: Location,
        dims: (CompareBy, CompareBy),
        area: Region,
        scales: (Option<f64>, f64, f64),
        wide: bool,
    ) {
        let o = observe(width, height, offset);
        assert_eq!((o.script, o.compare), dims, "{width}x{height} dimensions");
        assert_eq!(o.area, area, "{width}x{height} game area");
        assert_eq!(
            (o.screen_to_image, o.script_to_screen, o.script_to_image),
            scales,
            "{width}x{height} scales"
        );
        assert_eq!(o.wide, wide, "{width}x{height} wide");
    }

    const BY_WIDTH: (CompareBy, CompareBy) = (CompareBy::Width(2560), CompareBy::Width(1280));
    const BY_HEIGHT: (CompareBy, CompareBy) = (CompareBy::Height(1440), CompareBy::Height(720));
    const NO_OFFSET: Location = Location::new(0, 0);

    #[test]
    fn width_bound_screens() {
        check(
            1280,
            720,
            NO_OFFSET,
            BY_WIDTH,
            region(0, 0, 1280, 720),
            (None, 0.5, 0.5),
            false,
        );
        check(
            1000,
            1000,
            NO_OFFSET,
            BY_WIDTH,
            region(0, 219, 1000, 563),
            (Some(1.28), 0.390625, 0.5),
            false,
        );
        check(
            1,
            1,
            NO_OFFSET,
            BY_WIDTH,
            region(0, 0, 1, 1),
            (Some(1280.0), 3.90625E-4, 0.5),
            false,
        );
        check(
            1,
            3000,
            NO_OFFSET,
            BY_WIDTH,
            region(0, 1500, 1, 1),
            (Some(1280.0), 3.90625E-4, 0.5),
            false,
        );
        check(
            2560,
            1440,
            NO_OFFSET,
            BY_WIDTH,
            region(0, 0, 2560, 1440),
            (Some(0.5), 1.0, 0.5),
            false,
        );
        check(
            800,
            600,
            NO_OFFSET,
            BY_WIDTH,
            region(0, 75, 800, 450),
            (Some(1.6), 0.3125, 0.5),
            false,
        );
        check(
            1920,
            1200,
            NO_OFFSET,
            BY_WIDTH,
            region(0, 60, 1920, 1080),
            (Some(0.6666666666666666), 0.75, 0.5),
            false,
        );
    }

    #[test]
    fn height_bound_screens() {
        check(
            3000,
            1,
            NO_OFFSET,
            BY_HEIGHT,
            region(1499, 0, 2, 1),
            (Some(720.0), 6.944444444444445E-4, 0.5),
            true,
        );
        check(
            1366,
            768,
            NO_OFFSET,
            BY_HEIGHT,
            region(1, 0, 1365, 768),
            (Some(0.9375), 0.5333333333333333, 0.5),
            false,
        );
        check(
            2000,
            1001,
            NO_OFFSET,
            BY_HEIGHT,
            region(110, 0, 1780, 1001),
            (
                Some(0.7192807192807192),
                0.6951388888888889,
                0.49999999999999994,
            ),
            false,
        );
    }

    #[test]
    fn the_18_to_9_boundary_is_exclusive() {
        check(
            1800,
            900,
            NO_OFFSET,
            BY_HEIGHT,
            region(100, 0, 1600, 900),
            (Some(0.8), 0.625, 0.5),
            false,
        );
        check(
            1801,
            900,
            NO_OFFSET,
            BY_HEIGHT,
            region(0, 0, 1801, 900),
            (Some(0.8), 0.625, 0.5),
            true,
        );
    }

    #[test]
    fn wide_screens_use_the_whole_screen_until_21_to_9() {
        check(
            2100,
            900,
            NO_OFFSET,
            BY_HEIGHT,
            region(0, 0, 2100, 900),
            (Some(0.8), 0.625, 0.5),
            true,
        );
    }

    #[test]
    fn screens_wider_than_21_to_9_lose_the_side_borders() {
        check(
            2101,
            900,
            NO_OFFSET,
            BY_HEIGHT,
            region(1, 0, 2100, 900),
            (Some(0.8), 0.625, 0.5),
            true,
        );
        check(
            3440,
            1440,
            NO_OFFSET,
            BY_HEIGHT,
            region(40, 0, 3360, 1440),
            (Some(0.5), 1.0, 0.5),
            true,
        );
        check(
            3441,
            1440,
            NO_OFFSET,
            BY_HEIGHT,
            region(41, 0, 3360, 1440),
            (Some(0.5), 1.0, 0.5),
            true,
        );
        check(
            4000,
            1000,
            NO_OFFSET,
            BY_HEIGHT,
            region(834, 0, 2333, 1000),
            (Some(0.72), 0.6944444444444444, 0.49999999999999994),
            true,
        );
    }

    #[test]
    fn the_offset_moves_the_game_area() {
        check(
            1920,
            1080,
            Location::new(10, 20),
            BY_WIDTH,
            region(10, 20, 1920, 1080),
            (Some(0.6666666666666666), 0.75, 0.5),
            false,
        );
        check(
            2796,
            1290,
            Location::new(7, 9),
            BY_HEIGHT,
            region(7, 9, 2796, 1290),
            (Some(0.5581395348837209), 0.8958333333333334, 0.5),
            true,
        );
        check(
            3840,
            1600,
            Location::new(-3, 4),
            BY_HEIGHT,
            region(51, 4, 3733, 1600),
            (Some(0.45), 1.1111111111111112, 0.5),
            true,
        );
        check(
            1000,
            1000,
            Location::new(5, 5),
            BY_WIDTH,
            region(5, 224, 1000, 563),
            (Some(1.28), 0.390625, 0.5),
            false,
        );
    }

    #[test]
    fn the_offset_is_read_on_every_access() {
        let offset = Rc::new(Cell::new(Location::new(0, 0)));
        let shared = Rc::clone(&offset);
        let manager = FgoGameAreaManager::new(size(1920, 1080), move || shared.get()).unwrap();
        let before = manager.game_area();
        offset.set(Location::new(40, 50));
        assert_eq!(before, region(0, 0, 1920, 1080));
        assert_eq!(manager.game_area(), region(40, 50, 1920, 1080));
    }

    struct Transformed {
        location: Location,
        screen_region: Region,
        image_region: Region,
        from_image: Region,
        round_trip: Region,
    }

    fn transform(width: i32, height: i32, offset: Location) -> Transformed {
        let manager = FgoGameAreaManager::new(size(width, height), move || offset).unwrap();
        let scale = RealScale::new(&manager);
        let transformer = RealTransformer::new(&manager, &scale);
        let script = region(1234, 567, 321, 123);
        let image_region = transformer.to_image(script).unwrap();
        Transformed {
            location: transformer
                .to_screen_location(Location::new(1234, 567))
                .unwrap(),
            screen_region: transformer.to_screen_region(script).unwrap(),
            image_region,
            from_image: transformer
                .image_to_script(region(100, 50, 40, 30))
                .unwrap(),
            round_trip: transformer.image_to_script(image_region).unwrap(),
        }
    }

    fn check_transform(
        width: i32,
        height: i32,
        offset: Location,
        location: (i32, i32),
        screen: (i32, i32, i32, i32),
        image: (i32, i32, i32, i32),
    ) {
        let t = transform(width, height, offset);
        let label = format!("{width}x{height}");
        assert_eq!(t.location, Location::new(location.0, location.1), "{label}");
        assert_eq!(
            t.screen_region,
            region(screen.0, screen.1, screen.2, screen.3),
            "{label}"
        );
        assert_eq!(
            t.image_region,
            region(image.0, image.1, image.2, image.3),
            "{label}"
        );
        assert_eq!(t.from_image, region(200, 100, 80, 60), "{label}");
        assert_eq!(t.round_trip, region(1234, 568, 322, 124), "{label}");
    }

    #[test]
    fn transforms_match_kotlin() {
        let image = (617, 284, 161, 62);
        check_transform(
            1920,
            1080,
            NO_OFFSET,
            (926, 425),
            (926, 425, 241, 92),
            image,
        );
        check_transform(1280, 720, NO_OFFSET, (617, 284), (617, 284, 161, 62), image);
        check_transform(
            2796,
            1290,
            NO_OFFSET,
            (1105, 508),
            (1105, 508, 288, 110),
            image,
        );
        check_transform(
            3840,
            1600,
            NO_OFFSET,
            (1425, 630),
            (1425, 630, 357, 137),
            image,
        );
        check_transform(
            1000,
            1000,
            NO_OFFSET,
            (482, 440),
            (482, 440, 125, 48),
            image,
        );
        check_transform(
            2732,
            2048,
            NO_OFFSET,
            (1317, 861),
            (1317, 861, 343, 131),
            image,
        );
        check_transform(
            3441,
            1440,
            NO_OFFSET,
            (1275, 567),
            (1275, 567, 321, 123),
            image,
        );
    }

    #[test]
    fn transforms_add_the_offset_to_screen_coordinates_only() {
        let image = (617, 284, 161, 62);
        check_transform(
            1920,
            1080,
            Location::new(10, 20),
            (936, 445),
            (936, 445, 241, 92),
            image,
        );
        check_transform(
            2796,
            1290,
            Location::new(7, 9),
            (1112, 517),
            (1112, 517, 288, 110),
            image,
        );
    }

    #[test]
    fn transform_to_image_of_a_one_pixel_region_stays_valid() {
        let manager = FgoGameAreaManager::new(size(1280, 720), Location::default).unwrap();
        let scale = RealScale::new(&manager);
        let transformer = RealTransformer::new(&manager, &scale);
        assert_eq!(
            transformer.to_image(region(1, 1, 1, 1)),
            Ok(region(1, 1, 1, 1))
        );
    }

    #[test]
    fn wider_than_compares_the_ratio_exclusively() {
        assert!(!wider_than(size(1800, 900), 18, 9));
        assert!(wider_than(size(1801, 900), 18, 9));
        assert!(!wider_than(size(2100, 900), 21, 9));
        assert!(wider_than(size(2101, 900), 21, 9));
        assert!(!is_wide(size(1920, 1080)));
        assert!(is_wide(size(2400, 1080)));
    }
}

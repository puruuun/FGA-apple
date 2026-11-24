use crate::game_area_manager::GameAreaManager;
use crate::kotlin_math::GeometryError;
use crate::location::Location;
use crate::region::Region;
use crate::scale::Scale;

pub trait Transformer {
    fn to_screen_location(&self, location: Location) -> Result<Location, GeometryError>;

    fn to_screen_region(&self, region: Region) -> Result<Region, GeometryError>;

    fn to_image(&self, region: Region) -> Result<Region, GeometryError>;

    fn image_to_script(&self, region: Region) -> Result<Region, GeometryError>;
}

pub struct RealTransformer<'a> {
    game_area_manager: &'a dyn GameAreaManager,
    scale: &'a dyn Scale,
}

impl<'a> RealTransformer<'a> {
    pub fn new(game_area_manager: &'a dyn GameAreaManager, scale: &'a dyn Scale) -> Self {
        Self {
            game_area_manager,
            scale,
        }
    }
}

impl Transformer for RealTransformer<'_> {
    fn to_screen_location(&self, location: Location) -> Result<Location, GeometryError> {
        let scaled_point = (location * self.scale.script_to_screen())?;
        let game_area = self.game_area_manager.game_area();

        Ok(scaled_point + game_area.location())
    }

    fn to_screen_region(&self, region: Region) -> Result<Region, GeometryError> {
        let scaled_point = self.to_screen_location(region.location())?;
        let scaled_size = (region.size() * self.scale.script_to_screen())?;

        Ok(Region::from_location_size(scaled_point, scaled_size))
    }

    fn to_image(&self, region: Region) -> Result<Region, GeometryError> {
        region * self.scale.script_to_image()
    }

    fn image_to_script(&self, region: Region) -> Result<Region, GeometryError> {
        region * (1.0 / self.scale.script_to_image())
    }
}

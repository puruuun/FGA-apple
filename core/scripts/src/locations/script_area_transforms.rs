use automata::{GameAreaManager, GeometryError, Location, Region, Scale};

use crate::enums::game_server_enum::GameServer;
use crate::fgo_game_area_manager::is_wide;

pub const WIDE_EDGE_INSET: i32 = 171;

pub const WIDE_ACTION_INSET: i32 = 115;

pub trait IScriptAreaTransforms {
    fn script_area(&self) -> Region;

    fn is_wide(&self) -> bool;

    fn game_server(&self) -> GameServer;

    fn can_long_swipe(&self) -> bool;

    fn x_from_center_location(&self, location: Location) -> Location;

    fn x_from_center_region(&self, region: Region) -> Region;

    fn x_from_right_location(&self, location: Location) -> Location;

    fn x_from_right_region(&self, region: Region) -> Region;

    fn y_from_bottom_location(&self, location: Location) -> Location;

    fn y_from_bottom_region(&self, region: Region) -> Region;

    fn edge_inset(&self) -> i32 {
        if self.is_wide() { WIDE_EDGE_INSET } else { 0 }
    }

    fn x_from_left_edge_location(&self, location: Location) -> Location {
        location + Location::new(self.edge_inset(), 0)
    }

    fn x_from_left_edge_region(&self, region: Region) -> Region {
        region + Location::new(self.edge_inset(), 0)
    }

    fn x_from_right_edge_location(&self, location: Location) -> Location {
        location
            + Location::new(
                self.script_area().right().wrapping_sub(self.edge_inset()),
                0,
            )
    }

    fn x_from_right_edge_region(&self, region: Region) -> Region {
        region
            + Location::new(
                self.script_area().right().wrapping_sub(self.edge_inset()),
                0,
            )
    }
}

pub struct ScriptAreaTransforms {
    script_area: Region,
    is_wide: bool,
    game_server: GameServer,
    can_long_swipe: bool,
}

impl ScriptAreaTransforms {
    pub fn new(
        scale: &dyn Scale,
        game_area_manager: &dyn GameAreaManager,
        game_server: GameServer,
        can_long_swipe: bool,
    ) -> Result<Self, GeometryError> {
        let script_area = Region::from_location_size(
            Location::default(),
            (game_area_manager.game_area().size() * (1.0 / scale.script_to_screen()))?,
        );

        Ok(Self {
            script_area,
            is_wide: is_wide(script_area.size()),
            game_server,
            can_long_swipe,
        })
    }
}

impl IScriptAreaTransforms for ScriptAreaTransforms {
    fn script_area(&self) -> Region {
        self.script_area
    }

    fn is_wide(&self) -> bool {
        self.is_wide
    }

    fn game_server(&self) -> GameServer {
        self.game_server
    }

    fn can_long_swipe(&self) -> bool {
        self.can_long_swipe
    }

    fn x_from_center_location(&self, location: Location) -> Location {
        location + Location::new(self.script_area.center().x, 0)
    }

    fn x_from_center_region(&self, region: Region) -> Region {
        region + Location::new(self.script_area.center().x, 0)
    }

    fn x_from_right_location(&self, location: Location) -> Location {
        location + Location::new(self.script_area.right(), 0)
    }

    fn x_from_right_region(&self, region: Region) -> Region {
        region + Location::new(self.script_area.right(), 0)
    }

    fn y_from_bottom_location(&self, location: Location) -> Location {
        location + Location::new(0, self.script_area.bottom())
    }

    fn y_from_bottom_region(&self, region: Region) -> Region {
        region + Location::new(0, self.script_area.bottom())
    }
}

macro_rules! delegate_script_area_transforms {
    ($type:ty, $field:ident) => {
        impl $crate::locations::script_area_transforms::IScriptAreaTransforms for $type {
            fn script_area(&self) -> ::automata::Region {
                self.$field.script_area()
            }

            fn is_wide(&self) -> bool {
                self.$field.is_wide()
            }

            fn game_server(&self) -> $crate::enums::game_server_enum::GameServer {
                self.$field.game_server()
            }

            fn can_long_swipe(&self) -> bool {
                self.$field.can_long_swipe()
            }

            fn x_from_center_location(
                &self,
                location: ::automata::Location,
            ) -> ::automata::Location {
                self.$field.x_from_center_location(location)
            }

            fn x_from_center_region(&self, region: ::automata::Region) -> ::automata::Region {
                self.$field.x_from_center_region(region)
            }

            fn x_from_right_location(
                &self,
                location: ::automata::Location,
            ) -> ::automata::Location {
                self.$field.x_from_right_location(location)
            }

            fn x_from_right_region(&self, region: ::automata::Region) -> ::automata::Region {
                self.$field.x_from_right_region(region)
            }

            fn y_from_bottom_location(
                &self,
                location: ::automata::Location,
            ) -> ::automata::Location {
                self.$field.y_from_bottom_location(location)
            }

            fn y_from_bottom_region(&self, region: ::automata::Region) -> ::automata::Region {
                self.$field.y_from_bottom_region(region)
            }
        }
    };
}

pub(crate) use delegate_script_area_transforms;

pub type Transforms = std::sync::Arc<dyn IScriptAreaTransforms + Send + Sync>;

#[cfg(test)]
mod tests {
    use automata::{RealScale, Size};

    use super::*;
    use crate::enums::game_server_enum::GameServers;
    use crate::fgo_game_area_manager::FgoGameAreaManager;

    struct Row {
        screen: (i32, i32),
        area: (i32, i32),
        wide: bool,
        inset: i32,
        center_x: i32,
        right_x: i32,
        bottom_y: i32,
        left_edge_x: i32,
        right_edge_x: i32,
    }

    const fn row(
        screen: (i32, i32),
        area: (i32, i32),
        wide: bool,
        xs: (i32, i32, i32, i32),
        bottom_y: i32,
    ) -> Row {
        Row {
            screen,
            area,
            wide,
            inset: if wide { 171 } else { 0 },
            center_x: xs.0,
            right_x: xs.1,
            bottom_y,
            left_edge_x: xs.2,
            right_edge_x: xs.3,
        }
    }

    const ROWS: [Row; 10] = [
        row(
            (1920, 1080),
            (2560, 1440),
            false,
            (1290, 2570, 10, 2570),
            1460,
        ),
        row(
            (1280, 720),
            (2560, 1440),
            false,
            (1290, 2570, 10, 2570),
            1460,
        ),
        row(
            (2796, 1290),
            (3121, 1440),
            true,
            (1570, 3131, 181, 2960),
            1460,
        ),
        row(
            (2400, 1080),
            (3200, 1440),
            true,
            (1610, 3210, 181, 3039),
            1460,
        ),
        row(
            (3840, 1600),
            (3360, 1440),
            true,
            (1690, 3370, 181, 3199),
            1460,
        ),
        row(
            (1000, 1000),
            (2560, 1441),
            false,
            (1290, 2570, 10, 2570),
            1461,
        ),
        row(
            (2732, 2048),
            (2560, 1440),
            false,
            (1290, 2570, 10, 2570),
            1460,
        ),
        row(
            (3441, 1440),
            (3360, 1440),
            true,
            (1690, 3370, 181, 3199),
            1460,
        ),
        row(
            (1801, 900),
            (2882, 1440),
            true,
            (1451, 2892, 181, 2721),
            1460,
        ),
        row(
            (1800, 900),
            (2560, 1440),
            false,
            (1290, 2570, 10, 2570),
            1460,
        ),
    ];

    #[test]
    fn helpers_match_kotlin() {
        for row in &ROWS {
            let screen = Size::new(row.screen.0, row.screen.1).unwrap();
            let manager = FgoGameAreaManager::new(screen, Location::default).unwrap();
            let scale = RealScale::new(&manager);
            let t =
                ScriptAreaTransforms::new(&scale, &manager, GameServers::DEFAULT, true).unwrap();
            let label = format!("{}x{}", row.screen.0, row.screen.1);

            assert_eq!(
                t.script_area(),
                Region::new(0, 0, row.area.0, row.area.1).unwrap(),
                "{label}"
            );
            assert_eq!(t.is_wide(), row.wide, "{label}");
            assert_eq!(t.edge_inset(), row.inset, "{label}");
            assert_eq!(t.game_server(), GameServers::DEFAULT, "{label}");
            assert!(t.can_long_swipe(), "{label}");

            let l = Location::new(10, 20);
            let r = Region::new(10, 20, 30, 40).unwrap();
            let at = |x: i32, y: i32| Location::new(x, y);
            let rg = |x: i32, y: i32| Region::new(x, y, 30, 40).unwrap();

            assert_eq!(t.x_from_center_location(l), at(row.center_x, 20), "{label}");
            assert_eq!(t.x_from_center_region(r), rg(row.center_x, 20), "{label}");
            assert_eq!(t.x_from_right_location(l), at(row.right_x, 20), "{label}");
            assert_eq!(t.x_from_right_region(r), rg(row.right_x, 20), "{label}");
            assert_eq!(t.y_from_bottom_location(l), at(10, row.bottom_y), "{label}");
            assert_eq!(t.y_from_bottom_region(r), rg(10, row.bottom_y), "{label}");
            assert_eq!(
                t.x_from_left_edge_location(l),
                at(row.left_edge_x, 20),
                "{label}"
            );
            assert_eq!(
                t.x_from_left_edge_region(r),
                rg(row.left_edge_x, 20),
                "{label}"
            );
            assert_eq!(
                t.x_from_right_edge_location(l),
                at(row.right_edge_x, 20),
                "{label}"
            );
            assert_eq!(
                t.x_from_right_edge_region(r),
                rg(row.right_edge_x, 20),
                "{label}"
            );
        }
    }
}

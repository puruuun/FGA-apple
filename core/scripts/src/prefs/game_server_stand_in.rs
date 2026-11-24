#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GameServerStandIn {
    simple: &'static str,
    better_fgo: bool,
}

impl GameServerStandIn {
    pub const VALUES: [GameServerStandIn; 7] = [
        GameServerStandIn::new("En", false),
        GameServerStandIn::new("En", true),
        GameServerStandIn::new("Jp", false),
        GameServerStandIn::new("Jp", true),
        GameServerStandIn::new("Cn", false),
        GameServerStandIn::new("Tw", false),
        GameServerStandIn::new("Kr", false),
    ];

    const fn new(simple: &'static str, better_fgo: bool) -> Self {
        GameServerStandIn { simple, better_fgo }
    }

    pub fn simple(self) -> &'static str {
        self.simple
    }

    pub fn better_fgo(self) -> bool {
        self.better_fgo
    }

    pub fn serialize(self) -> String {
        if self.better_fgo {
            format!("{} BFGO", self.simple)
        } else {
            self.simple.to_string()
        }
    }

    pub fn deserialize(value: &str) -> Option<GameServerStandIn> {
        Self::VALUES
            .iter()
            .copied()
            .find(|server| server.serialize() == value)
    }
}

const BETTER_FGO_SUFFIX: &str = " BFGO";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameServer {
    En(En),
    Jp(Jp),
    Cn,
    Tw,
    Kr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum En {
    Original,
    BetterFgo,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Jp {
    Original,
    BetterFgo,
}

impl GameServer {
    pub fn simple(self) -> &'static str {
        match self {
            GameServer::En(_) => "En",
            GameServer::Jp(_) => "Jp",
            GameServer::Cn => "Cn",
            GameServer::Tw => "Tw",
            GameServer::Kr => "Kr",
        }
    }

    pub fn better_fgo(self) -> bool {
        matches!(
            self,
            GameServer::En(En::BetterFgo) | GameServer::Jp(Jp::BetterFgo)
        )
    }

    pub fn serialize(self) -> String {
        let suffix = if self.better_fgo() {
            BETTER_FGO_SUFFIX
        } else {
            ""
        };
        format!("{}{suffix}", self.simple())
    }
}

impl std::fmt::Display for GameServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.serialize())
    }
}

pub struct GameServers;

impl GameServers {
    pub const DEFAULT: GameServer = GameServer::En(En::Original);

    pub const VALUES: [GameServer; 7] = [
        GameServer::En(En::Original),
        GameServer::En(En::BetterFgo),
        GameServer::Jp(Jp::Original),
        GameServer::Jp(Jp::BetterFgo),
        GameServer::Cn,
        GameServer::Tw,
        GameServer::Kr,
    ];

    pub fn packages(server: GameServer) -> &'static [&'static str] {
        match server {
            GameServer::En(En::Original) => &["com.aniplex.fategrandorder.en"],
            GameServer::En(En::BetterFgo) => &["io.rayshift.betterfgo.en"],
            GameServer::Jp(Jp::Original) => &["com.aniplex.fategrandorder"],
            GameServer::Jp(Jp::BetterFgo) => &["io.rayshift.betterfgo"],
            GameServer::Cn => &[
                "com.bilibili.fatego",
                "com.bilibili.fatego.sharejoy",
                "com.bilibili.fgo.mi",
                "com.bilibili.fgo.uc",
                "com.bilibili.fgo.vivo",
                "com.bilibili.fgo.m4399",
                "com.tencent.tmgp.fgo",
                "com.bilibili.uo.fatego",
            ],
            GameServer::Tw => &["com.komoe.fgomycard", "com.xiaomeng.fategrandorder"],
            GameServer::Kr => &["com.netmarble.fgok"],
        }
    }

    pub fn from_package_name(package_name: &str) -> Option<GameServer> {
        Self::VALUES
            .iter()
            .copied()
            .find(|server| Self::packages(*server).contains(&package_name))
    }

    pub fn deserialize(value: &str) -> Option<GameServer> {
        Self::VALUES
            .iter()
            .copied()
            .find(|server| server.serialize() == value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_like_kotlin() {
        let names: Vec<String> = GameServers::VALUES.iter().map(|s| s.serialize()).collect();
        assert_eq!(names, ["En", "En BFGO", "Jp", "Jp BFGO", "Cn", "Tw", "Kr"]);
        for server in GameServers::VALUES {
            assert_eq!(GameServers::deserialize(&server.serialize()), Some(server));
        }
        assert_eq!(GameServers::deserialize("Kr BFGO"), None);
    }

    #[test]
    fn maps_package_names() {
        assert_eq!(
            GameServers::from_package_name("com.netmarble.fgok"),
            Some(GameServer::Kr)
        );
        assert_eq!(
            GameServers::from_package_name("com.bilibili.uo.fatego"),
            Some(GameServer::Cn)
        );
        assert_eq!(GameServers::from_package_name("x"), None);
    }
}

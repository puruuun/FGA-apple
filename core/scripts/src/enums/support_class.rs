kotlin_enum!(SupportClass {
    None,
    All,
    Saber,
    Archer,
    Lancer,
    Rider,
    Caster,
    Assassin,
    Berserker,
    Extra,
    Mix
});

impl SupportClass {
    pub fn can_also_check_all(self) -> bool {
        !matches!(
            self,
            SupportClass::None | SupportClass::All | SupportClass::Mix
        )
    }
}

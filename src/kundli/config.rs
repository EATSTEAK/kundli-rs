use crate::kundli::astro::{Ayanamsha, HouseSystem, NodeType, ZodiacType};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KundliConfig {
    pub zodiac: ZodiacType,
    pub ayanamsha: Ayanamsha,
    pub house_system: HouseSystem,
    pub node_type: NodeType,
    pub include_d9: bool,
    pub include_dasha: bool,
}

impl Default for KundliConfig {
    fn default() -> Self {
        Self {
            zodiac: ZodiacType::Sidereal,
            ayanamsha: Ayanamsha::Lahiri,
            house_system: HouseSystem::WholeSign,
            node_type: NodeType::True,
            include_d9: false,
            include_dasha: false,
        }
    }
}

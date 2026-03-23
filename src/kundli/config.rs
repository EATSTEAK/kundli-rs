use crate::kundli::astro::{Ayanamsha, HouseSystem, NodeType, ZodiacType};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KundliConfig {
    pub zodiac: ZodiacType,
    pub ayanamsha: Ayanamsha,
    pub house_system: HouseSystem,
    pub node_type: NodeType,
    pub include_d9: bool,
    pub include_dasha: bool,
}

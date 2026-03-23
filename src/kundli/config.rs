use crate::kundli::astro::{Ayanamsha, HouseSystem, NodeType, ZodiacType};

/// Declarative options that control how a kundli is derived.
///
/// Several fields duplicate settings present on
/// [`AstroRequest`](crate::kundli::astro::AstroRequest). Those values must match
/// when calling the high-level calculation entrypoints.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KundliConfig {
    /// Zodiac mode used throughout the calculation.
    pub zodiac: ZodiacType,
    /// Ayanamsha applied to sidereal calculations.
    pub ayanamsha: Ayanamsha,
    /// House system used for D1 house derivation.
    pub house_system: HouseSystem,
    /// Lunar node mode used for Rahu and Ketu positions.
    pub node_type: NodeType,
    /// Whether to include a derived Navamsa (D9) chart.
    pub include_d9: bool,
    /// Whether to include derived Vimshottari dasha periods.
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

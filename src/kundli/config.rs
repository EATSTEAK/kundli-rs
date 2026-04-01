use crate::kundli::astro::{
    AstroRequest, Ayanamsha, HouseSystem, NodeType, ZodiacType,
};

/// Known high-level chart layers that can be requested from the public API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KnownChart {
    D1,
    D9,
    VimshottariDasha,
}

/// Declarative options that control how a kundli is derived.
///
/// Several fields duplicate settings present on
/// [`AstroRequest`]. Use
/// [`KundliConfig::from_request`] when you want those settings to match by
/// construction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KundliConfig {
    /// Zodiac mode used throughout the calculation.
    pub zodiac: ZodiacType,
    /// Ayanamsha applied to sidereal calculations.
    pub ayanamsha: Ayanamsha,
    /// House system used for D1 house derivation.
    pub house_system: HouseSystem,
    /// Lunar node mode used for Rahu and Ketu positions.
    pub node_type: NodeType,
    /// Requested chart layers to derive from the astronomical result.
    pub charts: Vec<KnownChart>,
}

impl KundliConfig {
    /// Creates a config with explicit duplicated astro settings.
    pub fn new(
        zodiac: ZodiacType,
        ayanamsha: Ayanamsha,
        house_system: HouseSystem,
        node_type: NodeType,
    ) -> Self {
        Self {
            zodiac,
            ayanamsha,
            house_system,
            node_type,
            charts: vec![],
        }
    }

    /// Creates a config whose duplicated settings match the given request.
    pub fn from_request(request: &AstroRequest) -> Self {
        Self::new(
            request.zodiac,
            request.ayanamsha,
            request.house_system,
            request.node_type,
        )
    }

    /// Returns a copy with a different zodiac mode.
    pub fn with_zodiac(mut self, zodiac: ZodiacType) -> Self {
        self.zodiac = zodiac;
        self
    }

    /// Returns a copy with a different ayanamsha.
    pub fn with_ayanamsha(mut self, ayanamsha: Ayanamsha) -> Self {
        self.ayanamsha = ayanamsha;
        self
    }

    /// Returns a copy with a different house system.
    pub fn with_house_system(mut self, house_system: HouseSystem) -> Self {
        self.house_system = house_system;
        self
    }

    /// Returns a copy with a different node mode.
    pub fn with_node_type(mut self, node_type: NodeType) -> Self {
        self.node_type = node_type;
        self
    }

    /// Returns a copy with an explicit set of requested chart layers.
    pub fn with_charts(mut self, charts: &[KnownChart]) -> Self {
        self.charts = charts.to_vec();
        self
    }
}

impl Default for KundliConfig {
    fn default() -> Self {
        Self::new(
            ZodiacType::Sidereal,
            Ayanamsha::Lahiri,
            HouseSystem::WholeSign,
            NodeType::True,
        )
    }
}

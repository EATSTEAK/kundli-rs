use crate::kundli::astro::{AstroBody, AstroRequest, Ayanamsha, HouseSystem, NodeType, ZodiacType};
use crate::kundli::error::{ChartSelectionError, KundliError};

/// High-level chart family requested from the public API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ChartKind {
    /// The base Rāśi chart.
    Rasi,
    /// A divisional chart such as D2, D9, or D60.
    Varga { division: u8 },
    /// A cusp-based bhava chart.
    Bhava,
    /// A bhava-chalit style chart.
    Chalit,
    /// A cusp-based chart derived after a divisional transform.
    DivisionalBhava { division: u8 },
    /// Vimshottari dasha output.
    VimshottariDasha,
}

/// Reference anchor used to re-interpret a chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ReferenceKey {
    Lagna,
    Moon,
    Sun,
    Planet(AstroBody),
}

/// House derivation mode for a chart request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum HouseMode {
    /// Reuse `KundliConfig::house_system`.
    Configured,
    /// Always derive whole-sign houses.
    WholeSign,
    /// Always derive cusp-based houses with the given system.
    CuspBased(HouseSystem),
    /// This chart does not expose houses.
    None,
}

/// Declarative chart selection spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ChartSpec {
    pub kind: ChartKind,
    pub reference: ReferenceKey,
    pub house_mode: HouseMode,
}

impl ChartSpec {
    pub const fn new(kind: ChartKind, reference: ReferenceKey, house_mode: HouseMode) -> Self {
        Self {
            kind,
            reference,
            house_mode,
        }
    }

    pub const fn d1() -> Self {
        Self::new(ChartKind::Rasi, ReferenceKey::Lagna, HouseMode::Configured)
    }

    pub const fn rasi() -> Self {
        Self::d1()
    }

    pub const fn moon_chart() -> Self {
        Self::new(ChartKind::Rasi, ReferenceKey::Moon, HouseMode::Configured)
    }

    pub const fn sun_chart() -> Self {
        Self::new(ChartKind::Rasi, ReferenceKey::Sun, HouseMode::Configured)
    }

    pub const fn d9() -> Self {
        Self::new(
            ChartKind::Varga { division: 9 },
            ReferenceKey::Lagna,
            HouseMode::WholeSign,
        )
    }

    pub const fn varga(division: u8) -> Self {
        Self::new(
            ChartKind::Varga { division },
            ReferenceKey::Lagna,
            HouseMode::WholeSign,
        )
    }

    pub const fn bhava() -> Self {
        Self::new(ChartKind::Bhava, ReferenceKey::Lagna, HouseMode::Configured)
    }

    pub const fn chalit() -> Self {
        Self::new(
            ChartKind::Chalit,
            ReferenceKey::Lagna,
            HouseMode::Configured,
        )
    }

    pub const fn divisional_bhava(division: u8) -> Self {
        Self::new(
            ChartKind::DivisionalBhava { division },
            ReferenceKey::Lagna,
            HouseMode::Configured,
        )
    }

    pub const fn vimshottari_dasha() -> Self {
        Self::new(
            ChartKind::VimshottariDasha,
            ReferenceKey::Moon,
            HouseMode::None,
        )
    }

    pub const fn with_reference(self, reference: ReferenceKey) -> Self {
        Self { reference, ..self }
    }

    pub const fn with_house_mode(self, house_mode: HouseMode) -> Self {
        Self { house_mode, ..self }
    }
}

/// Declarative options that control how a kundli is derived.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KundliConfig {
    /// Zodiac mode used throughout the calculation.
    pub zodiac: ZodiacType,
    /// Ayanamsha applied to sidereal calculations.
    pub ayanamsha: Ayanamsha,
    /// Default house system used throughout derivation.
    pub house_system: HouseSystem,
    /// Lunar node mode used for Rahu and Ketu positions.
    pub node_type: NodeType,
    /// Requested chart layers to derive from the astronomical result.
    pub charts: Vec<ChartSpec>,
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

    pub fn with_zodiac(mut self, zodiac: ZodiacType) -> Self {
        self.zodiac = zodiac;
        self
    }

    pub fn with_ayanamsha(mut self, ayanamsha: Ayanamsha) -> Self {
        self.ayanamsha = ayanamsha;
        self
    }

    pub fn with_house_system(mut self, house_system: HouseSystem) -> Self {
        self.house_system = house_system;
        self
    }

    pub fn with_node_type(mut self, node_type: NodeType) -> Self {
        self.node_type = node_type;
        self
    }

    pub fn with_charts(mut self, charts: &[ChartSpec]) -> Self {
        self.charts = charts.to_vec();
        self
    }

    /// Validates and normalizes the requested chart layers.
    pub fn validate(&mut self) -> Result<(), KundliError> {
        if self.charts.is_empty() {
            return Err(ChartSelectionError::Empty.into());
        }

        for chart in &self.charts {
            match chart.kind {
                ChartKind::Varga { division } | ChartKind::DivisionalBhava { division }
                    if division == 0 =>
                {
                    return Err(ChartSelectionError::InvalidDivision(division).into());
                }
                _ => {}
            }

            match chart.kind {
                ChartKind::VimshottariDasha if chart.house_mode != HouseMode::None => {
                    return Err(ChartSelectionError::UnexpectedHouseMode(chart.kind).into());
                }
                ChartKind::Varga { .. } if chart.house_mode == HouseMode::None => {
                    return Err(ChartSelectionError::UnexpectedHouseMode(chart.kind).into());
                }
                ChartKind::Rasi if chart.house_mode == HouseMode::None => {
                    return Err(ChartSelectionError::UnexpectedHouseMode(chart.kind).into());
                }
                ChartKind::Bhava | ChartKind::Chalit | ChartKind::DivisionalBhava { .. }
                    if !matches!(chart.house_mode, HouseMode::CuspBased(_)) =>
                {
                    return Err(ChartSelectionError::CuspBasedHouseModeRequired(chart.kind).into());
                }
                _ => {}
            }
        }

        self.charts.sort();
        self.charts.dedup();

        Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_rejects_empty_chart_selection() {
        let mut config = KundliConfig::default();

        let error = config.validate().unwrap_err();

        assert_eq!(
            error,
            KundliError::ChartSelection(ChartSelectionError::Empty)
        );
    }

    #[test]
    fn validate_rejects_zero_division_varga() {
        let mut config = KundliConfig::default().with_charts(&[ChartSpec::varga(0)]);

        let error = config.validate().unwrap_err();

        assert_eq!(
            error,
            KundliError::ChartSelection(ChartSelectionError::InvalidDivision(0))
        );
    }

    #[test]
    fn validate_sorts_and_deduplicates_chart_selection() {
        let mut config = KundliConfig::default().with_charts(&[
            ChartSpec::vimshottari_dasha(),
            ChartSpec::d1(),
            ChartSpec::d1(),
            ChartSpec::d9(),
        ]);

        config.validate().unwrap();

        assert_eq!(
            config.charts,
            vec![
                ChartSpec::d1(),
                ChartSpec::d9(),
                ChartSpec::vimshottari_dasha()
            ]
        );
    }

    #[test]
    fn validate_rejects_bhava_without_explicit_cusp_based_mode() {
        let mut config = KundliConfig::default().with_charts(&[ChartSpec::bhava()]);

        let error = config.validate().unwrap_err();

        assert_eq!(
            error,
            KundliError::ChartSelection(ChartSelectionError::CuspBasedHouseModeRequired(
                ChartKind::Bhava,
            ))
        );
    }

    #[test]
    fn validate_rejects_chalit_without_explicit_cusp_based_mode() {
        let mut config = KundliConfig::default().with_charts(&[ChartSpec::chalit()]);

        let error = config.validate().unwrap_err();

        assert_eq!(
            error,
            KundliError::ChartSelection(ChartSelectionError::CuspBasedHouseModeRequired(
                ChartKind::Chalit,
            ))
        );
    }

    #[test]
    fn validate_rejects_divisional_bhava_without_cusp_based_mode() {
        let mut config = KundliConfig::default().with_charts(&[ChartSpec::divisional_bhava(9)]);

        let error = config.validate().unwrap_err();

        assert_eq!(
            error,
            KundliError::ChartSelection(ChartSelectionError::CuspBasedHouseModeRequired(
                ChartKind::DivisionalBhava { division: 9 },
            ))
        );
    }
}

use std::collections::BTreeMap;

use crate::kundli::astro::{AstroBody, Ayanamsha, HouseSystem, NodeType, ZodiacType};
use crate::kundli::config::ChartSpec;

/// The twelve zodiac signs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Sign {
    Aries,
    Taurus,
    Gemini,
    Cancer,
    Leo,
    Virgo,
    Libra,
    Scorpio,
    Sagittarius,
    Capricorn,
    Aquarius,
    Pisces,
}

/// The twenty-seven nakshatras used in Vedic astrology.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Nakshatra {
    Ashwini,
    Bharani,
    Krittika,
    Rohini,
    Mrigashira,
    Ardra,
    Punarvasu,
    Pushya,
    Ashlesha,
    Magha,
    PurvaPhalguni,
    UttaraPhalguni,
    Hasta,
    Chitra,
    Swati,
    Vishakha,
    Anuradha,
    Jyeshtha,
    Mula,
    PurvaAshadha,
    UttaraAshadha,
    Shravana,
    Dhanishta,
    Shatabhisha,
    PurvaBhadrapada,
    UttaraBhadrapada,
    Revati,
}

/// Nakshatra quarter number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pada(u8);

impl Pada {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 4;

    pub fn new(value: u8) -> Option<Self> {
        (Self::MIN..=Self::MAX)
            .contains(&value)
            .then_some(Self(value))
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Placement of a longitude within a nakshatra and pada.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NakshatraPlacement {
    pub nakshatra: Nakshatra,
    pub pada: Pada,
    pub degrees_in_nakshatra: f64,
}

/// One-based house number.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HouseNumber(u8);

impl HouseNumber {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 12;

    pub fn new(value: u8) -> Option<Self> {
        (Self::MIN..=Self::MAX)
            .contains(&value)
            .then_some(Self(value))
    }

    pub const fn get(self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LagnaResult {
    pub sign: Sign,
    pub degrees_in_sign: f64,
    pub longitude: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlanetPlacement {
    pub body: AstroBody,
    pub longitude: f64,
    pub sign: Sign,
    pub degrees_in_sign: f64,
    pub house: HouseNumber,
    pub nakshatra: NakshatraPlacement,
    pub is_retrograde: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct HouseResult {
    pub house: HouseNumber,
    pub cusp_longitude: f64,
    pub sign: Sign,
}

/// Common chart result shape produced by the derive pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartResult {
    pub lagna: LagnaResult,
    pub planets: Vec<PlanetPlacement>,
    pub houses: Vec<HouseResult>,
}

/// The primary natal chart layer derived from the astronomical result.
#[derive(Debug, Clone, PartialEq)]
pub struct D1Chart {
    pub lagna: LagnaResult,
    pub planets: Vec<PlanetPlacement>,
    pub houses: Vec<HouseResult>,
}

/// The Navamsa (D9) chart derived from the primary astronomical result.
#[derive(Debug, Clone, PartialEq)]
pub struct D9Chart {
    pub lagna: LagnaResult,
    pub planets: Vec<PlanetPlacement>,
}

impl From<ChartResult> for D1Chart {
    fn from(value: ChartResult) -> Self {
        Self {
            lagna: value.lagna,
            planets: value.planets,
            houses: value.houses,
        }
    }
}

impl From<ChartResult> for D9Chart {
    fn from(value: ChartResult) -> Self {
        Self {
            lagna: value.lagna,
            planets: value.planets,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DashaLord {
    Ketu,
    Venus,
    Sun,
    Moon,
    Mars,
    Rahu,
    Jupiter,
    Saturn,
    Mercury,
}

impl DashaLord {
    pub const SEQUENCE: [Self; 9] = [
        Self::Ketu,
        Self::Venus,
        Self::Sun,
        Self::Moon,
        Self::Mars,
        Self::Rahu,
        Self::Jupiter,
        Self::Saturn,
        Self::Mercury,
    ];
}

#[derive(Debug, Clone, PartialEq)]
pub struct DashaPeriod {
    pub lord: DashaLord,
    pub start_jd_ut: f64,
    pub end_jd_ut: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VimshottariDasha {
    pub moon_nakshatra: Nakshatra,
    pub current_mahadasha: DashaPeriod,
    pub mahadashas: Vec<DashaPeriod>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CalculationMeta {
    pub jd_ut: f64,
    pub zodiac: ZodiacType,
    pub ayanamsha: Ayanamsha,
    pub ayanamsha_value: Option<f64>,
    pub house_system: HouseSystem,
    pub node_type: NodeType,
    pub sidereal_time: f64,
    pub body_count: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalculationWarning {
    pub code: &'static str,
    pub message: &'static str,
}

/// A chart-layer payload stored in the high-level multi-chart response.
#[derive(Debug, Clone, PartialEq)]
pub enum ChartLayer {
    Chart(ChartResult),
    VimshottariDasha(VimshottariDasha),
}

impl ChartLayer {
    pub fn as_chart(&self) -> Option<&ChartResult> {
        match self {
            Self::Chart(chart) => Some(chart),
            _ => None,
        }
    }

    pub fn as_vimshottari_dasha(&self) -> Option<&VimshottariDasha> {
        match self {
            Self::VimshottariDasha(dasha) => Some(dasha),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct KundliResult {
    pub meta: CalculationMeta,
    pub charts: BTreeMap<ChartSpec, ChartLayer>,
    pub warnings: Vec<CalculationWarning>,
}

impl KundliResult {
    pub fn chart(&self, spec: ChartSpec) -> Option<&ChartLayer> {
        self.charts.get(&spec)
    }
}

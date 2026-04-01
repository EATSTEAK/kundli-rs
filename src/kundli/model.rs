use crate::kundli::astro::{AstroBody, Ayanamsha, HouseSystem, NodeType, ZodiacType};

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
///
/// Valid values are in the range `1..=4`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pada(u8);

impl Pada {
    /// Smallest valid pada value.
    pub const MIN: u8 = 1;
    /// Largest valid pada value.
    pub const MAX: u8 = 4;

    /// Creates a checked [`Pada`] value.
    pub fn new(value: u8) -> Option<Self> {
        (Self::MIN..=Self::MAX)
            .contains(&value)
            .then_some(Self(value))
    }

    /// Returns the raw one-based pada value.
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Placement of a longitude within a nakshatra and pada.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NakshatraPlacement {
    /// Resolved nakshatra.
    pub nakshatra: Nakshatra,
    /// Resolved pada.
    pub pada: Pada,
    /// Degrees progressed within the nakshatra.
    pub degrees_in_nakshatra: f64,
}

/// One-based house number.
///
/// Valid values are in the range `1..=12`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HouseNumber(u8);

impl HouseNumber {
    /// Smallest valid house number.
    pub const MIN: u8 = 1;
    /// Largest valid house number.
    pub const MAX: u8 = 12;

    /// Creates a checked [`HouseNumber`] value.
    pub fn new(value: u8) -> Option<Self> {
        (Self::MIN..=Self::MAX)
            .contains(&value)
            .then_some(Self(value))
    }

    /// Returns the raw one-based house number.
    pub const fn get(self) -> u8 {
        self.0
    }
}

/// Derived lagna information.
#[derive(Debug, Clone, PartialEq)]
pub struct LagnaResult {
    /// Lagna sign.
    pub sign: Sign,
    /// Degrees elapsed within the lagna sign.
    pub degrees_in_sign: f64,
    /// Absolute lagna longitude in degrees.
    pub longitude: f64,
}

/// Derived placement for a canonical astronomical body.
#[derive(Debug, Clone, PartialEq)]
pub struct PlanetPlacement {
    /// The astronomical body.
    pub body: AstroBody,
    /// Absolute ecliptic longitude in degrees.
    pub longitude: f64,
    /// Zodiac sign resolved from the longitude.
    pub sign: Sign,
    /// Degrees elapsed within the resolved sign.
    pub degrees_in_sign: f64,
    /// Derived house placement.
    pub house: HouseNumber,
    /// Derived nakshatra placement.
    pub nakshatra: NakshatraPlacement,
    /// Whether the body is retrograde according to longitudinal speed.
    pub is_retrograde: bool,
}

/// Derived house metadata.
///
/// House numbers are assigned relative to the pipeline reference point.
/// `cusp_longitude` represents the start longitude of the materialized house.
/// For cusp-based systems this is the reported cusp after reference-relative
/// renumbering, and for WholeSign houses this is the sign boundary that anchors
/// the reference-relative house.
#[derive(Debug, Clone, PartialEq)]
pub struct HouseResult {
    /// One-based house number.
    pub house: HouseNumber,
    /// House start longitude in degrees.
    pub cusp_longitude: f64,
    /// Sign anchored to the house start.
    pub sign: Sign,
}

/// Common chart result shape produced by the derive pipeline.
#[derive(Debug, Clone, PartialEq)]
pub struct ChartResult {
    /// Derived lagna.
    pub lagna: LagnaResult,
    /// Derived placements for canonical astronomical bodies.
    pub planets: Vec<PlanetPlacement>,
    /// Derived houses according to the configured house system.
    pub houses: Vec<HouseResult>,
}

/// The primary natal chart layer derived from the astronomical result.
#[derive(Debug, Clone, PartialEq)]
pub struct D1Chart {
    /// Derived lagna.
    pub lagna: LagnaResult,
    /// Derived placements for canonical astronomical bodies.
    pub planets: Vec<PlanetPlacement>,
    /// Derived houses according to the configured house system.
    pub houses: Vec<HouseResult>,
}

/// The Navamsa (D9) chart derived from the primary astronomical result.
#[derive(Debug, Clone, PartialEq)]
pub struct D9Chart {
    /// Derived D9 lagna.
    pub lagna: LagnaResult,
    /// Derived D9 planet placements.
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

/// Lords used in the Vimshottari mahadasha cycle.
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
    /// Canonical Vimshottari mahadasha sequence.
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

/// A single mahadasha period.
#[derive(Debug, Clone, PartialEq)]
pub struct DashaPeriod {
    /// Mahadasha lord.
    pub lord: DashaLord,
    /// Inclusive start Julian day in Universal Time.
    pub start_jd_ut: f64,
    /// End Julian day in Universal Time.
    pub end_jd_ut: f64,
}

/// Derived Vimshottari dasha summary.
#[derive(Debug, Clone, PartialEq)]
pub struct VimshottariDasha {
    /// The Moon's nakshatra used to anchor the dasha sequence.
    pub moon_nakshatra: Nakshatra,
    /// The currently active mahadasha at the request time.
    pub current_mahadasha: DashaPeriod,
    /// The full mahadasha cycle beginning with `current_mahadasha`.
    pub mahadashas: Vec<DashaPeriod>,
}

/// Metadata describing how a [`KundliResult`] was calculated.
#[derive(Debug, Clone, PartialEq)]
pub struct CalculationMeta {
    /// Requested Julian day in Universal Time.
    pub jd_ut: f64,
    /// Zodiac mode used during calculation.
    pub zodiac: ZodiacType,
    /// Ayanamsha used during calculation.
    pub ayanamsha: Ayanamsha,
    /// Numeric ayanamsha value reported by the astronomical backend, when available.
    pub ayanamsha_value: Option<f64>,
    /// House system used for D1 house derivation.
    pub house_system: HouseSystem,
    /// Node mode used for Rahu and Ketu positions.
    pub node_type: NodeType,
    /// Sidereal time reported by the astronomical backend.
    pub sidereal_time: f64,
    /// Number of astronomical bodies included in the raw result.
    pub body_count: usize,
}

/// Non-fatal warning emitted during calculation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CalculationWarning {
    /// Stable warning code.
    pub code: &'static str,
    /// Human-readable warning message.
    pub message: &'static str,
}

/// Fully assembled result returned by the high-level kundli API.
///
/// The top-level `lagna`, `planets`, and `houses` fields mirror the contents of
/// [`KundliResult::d1`] for ergonomic access to the primary chart layer.
#[derive(Debug, Clone, PartialEq)]
pub struct KundliResult {
    /// Calculation metadata.
    pub meta: CalculationMeta,
    /// Convenience mirror of `d1.lagna`.
    pub lagna: LagnaResult,
    /// Convenience mirror of `d1.planets`.
    pub planets: Vec<PlanetPlacement>,
    /// Convenience mirror of `d1.houses`.
    pub houses: Vec<HouseResult>,
    /// Primary natal chart layer.
    pub d1: D1Chart,
    /// Optional Navamsa chart.
    pub d9: Option<D9Chart>,
    /// Optional Vimshottari dasha summary.
    pub dasha: Option<VimshottariDasha>,
    /// Non-fatal warnings collected during assembly.
    pub warnings: Vec<CalculationWarning>,
}

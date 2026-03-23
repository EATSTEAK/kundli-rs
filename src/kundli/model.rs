use crate::kundli::astro::AstroBody;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pada(pub u8);

impl Pada {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 4;

    pub fn new(value: u8) -> Option<Self> {
        (Self::MIN..=Self::MAX)
            .contains(&value)
            .then_some(Self(value))
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NakshatraPlacement {
    pub nakshatra: Nakshatra,
    pub pada: Pada,
    pub degrees_in_nakshatra: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct HouseNumber(pub u8);

impl HouseNumber {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 12;

    pub fn new(value: u8) -> Option<Self> {
        (Self::MIN..=Self::MAX)
            .contains(&value)
            .then_some(Self(value))
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

/// Derived house metadata.
///
/// `cusp_longitude` represents the start longitude of the house.
/// For cusp-based systems this is the reported cusp longitude, and for
/// WholeSign houses this is the sign boundary that anchors the house.
#[derive(Debug, Clone, PartialEq)]
pub struct HouseResult {
    pub house: HouseNumber,
    pub cusp_longitude: f64,
    pub sign: Sign,
}

#[derive(Debug, Clone, PartialEq)]
pub struct D1Chart {
    pub lagna: LagnaResult,
    pub planets: Vec<PlanetPlacement>,
    pub houses: Vec<HouseResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct D9Chart {
    pub lagna: LagnaResult,
    pub planets: Vec<PlanetPlacement>,
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

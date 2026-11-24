//! W3: giants (weather design section 3). Belts and zones between alternating
//! jets, storm families drawn from pack rates, and the polar regime chosen by
//! rule from the Burger number.
//!
//! - **Bands**: the S6 cloud deck's layout (`sse-surface` `regimes::cloud_deck`):
//!   jets = round(sqrt(a Ω / U)) from the Rhines scale, 2 × jets bands with
//!   edges evenly spaced in sin(latitude). Here the bands are mirrored about
//!   the equator: an equatorial zone, then belt, zone, ... poleward, so the
//!   prograde jets sit on the poleward edge of each zone.
//! - **Polar jet**: one polar Rhines length from the pole, L = (U a² / 2Ω)^⅓
//!   (Rhines arrest on the polar γ-plane f = 2Ω(1 − r²/2a²)).
//! - **Polar regime**: Burger number Bu = (L_d / a)², L_d = N H / 2Ω.
//!   Low Bu: a ring of cyclones; middle: one polar vortex; high Bu or weak
//!   internal heat: a diffuse cap (Brueshaber, Sayanagi and Dowling 2019;
//!   O'Neill, Emanuel and Flierl 2015).
//! - **Polygon jet**: a rule fitted to Saturn's hexagon. Sides
//!   n = round(2π a cos φ_jet / (k L_d(φ_jet))), clamped to the pack's range.
//! - **Storms**: great ovals and strings of ovals in zones (gas giants), dark
//!   spots (ice giants), planet-circling outbreaks (cool gas giants). Each
//!   family is drawn per era window keyed by the window's index, so a storm
//!   alive on a day has the same id, track and position whichever date asks.
//!   Storms drift with the local zonal wind: Δλ = u Δt / (a cos φ).

use crate::conditions::Conditions;
use crate::contract::*;
use crate::settings::Reference;
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Seed, Stream};
use sse_core::math as m;
use sse_surface::atlas::TileKey;
use std::collections::BTreeMap;

const G: f64 = sse_core::sheet::G;
const M_EARTH_KG: f64 = sse_core::sheet::M_EARTH;
const DAY_S: f64 = sse_core::sheet::DAY;
const YEAR_D: f64 = sse_core::sheet::YR_JULIAN / sse_core::sheet::DAY;
const DEG: f64 = 180.0 / m::PI;

/// Keyed stream for storms (ovals, strings, dark spots, outbreaks).
pub const STORMS_STREAM: &str = "weather.giants.storms.v1";
/// Keyed stream for the polar features (ring counts, transition, orientation).
pub const POLES_STREAM: &str = "weather.giants.poles.v1";
/// `born` of a permanent feature (the contract's `ends` uses `f64::MAX`).
pub const ALWAYS: f64 = f64::MIN;

// ---------------------------------------------------------------- settings

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Bands {
    /// Jet speed scale, m/s: the bands' winds and the polar jet.
    pub wind_gas_m_s: f64,
    pub wind_ice_m_s: f64,
    /// Local eddy speed the Rhines scale reads, m/s (not the jet speed).
    pub eddy_gas_m_s: f64,
    pub eddy_ice_m_s: f64,
    pub ice_giant_mass_earth: f64,
    /// Retrograde jets are this fraction of the prograde (S6: 0.5).
    pub retrograde_ratio: f64,
    /// Equatorial wind as a fraction of the wind scale.
    pub equator_gas: f64,
    pub equator_ice: f64,
    /// Bands (from the equator) the equatorial jet spans.
    pub equatorial_bands_gas: u32,
    pub equatorial_bands_ice: u32,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Stratification {
    /// Brunt-Väisälä frequency, s⁻¹.
    pub brunt_s: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Polar {
    /// Bu at or below: ring of cyclones (a / L_d ≥ 40).
    pub ring_bu_max: f64,
    /// Bu at or above: one polar vortex (a / L_d ≤ 30). Between: drawn.
    pub single_bu_min: f64,
    /// Bu at or above: diffuse cap.
    pub diffuse_bu_min: f64,
    /// Internal heat below this: diffuse cap (weak convective forcing).
    pub diffuse_internal_max_w_m2: f64,
    pub ring_counts: Vec<u32>,
    /// Ring cyclone diameter in deformation radii, and the gap between them.
    pub ring_cyclone_ld: f64,
    pub ring_gap: f64,
    /// Polar vortex diameter in deformation radii.
    pub vortex_ld: f64,
    /// Polar jet speed as a fraction of the wind scale.
    pub polar_jet_u: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct PolarJet {
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Polygon {
    /// Polar regimes that keep a coherent circumpolar jet.
    pub regimes: Vec<String>,
    pub min_lat_deg: f64,
    /// k: one polygon side is k deformation radii (at the jet) long.
    pub wavelength_ld: f64,
    pub min_sides: u32,
    pub max_sides: u32,
    pub min_jet_m_s: f64,
    pub source: String,
}

/// A family of drawn storms.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Family {
    /// Contract kind (`oval`, `dark-spot`).
    pub kind: String,
    /// `gas` or `ice`.
    pub applies: String,
    /// New systems per Earth year on the whole giant.
    pub per_year: f64,
    /// Era window, days: storms are drawn per window, keyed by its index.
    pub window_days: f64,
    /// Life, years, log-uniform between the two.
    pub life_years: [f64; 2],
    /// Size as a fraction of the zone's width, when nonzero.
    pub size_band: [f64; 2],
    /// Size in km, when `size_band` is zero.
    #[serde(default)]
    pub size_km: [f64; 2],
    /// Peak wind as a fraction of the wind scale.
    pub strength_u: [f64; 2],
    /// Members per system (strings), inclusive.
    pub members: [u32; 2],
    /// |latitude| range; zero: inside a zone poleward of the equatorial one.
    pub lat_deg: [f64; 2],
    pub because: String,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Outbreaks {
    /// Gas giants at or below this 1 bar temperature.
    pub max_t_k: f64,
    pub chance_per_cycle: f64,
    pub duration_days: f64,
    /// Days for the disturbance to wrap the planet.
    pub encircle_days: f64,
    pub head_km: f64,
    /// Width of the disturbed band, km.
    pub band_km: f64,
    pub lat_max_deg: f64,
    pub strength_m_s: f64,
    pub because: String,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Chemistry {
    pub max_k: f64,
    /// Condensates of the low, middle and high layer.
    pub layers: [String; 3],
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GiantClouds {
    pub zone_cover: f64,
    pub belt_cover: f64,
    pub zone_tau: f64,
    pub belt_tau: f64,
    pub mid_cover: f64,
    pub mid_tau: f64,
    pub low_cover: f64,
    pub low_tau: f64,
    /// Haze optical depth in a diffuse polar cap.
    pub cap_tau: f64,
    pub chemistry: Vec<Chemistry>,
    /// Typical effective radius of each condensate's deck, µm (K-c); a
    /// condensate not listed has no measured size held.
    #[serde(default)]
    pub particle_radius_um: BTreeMap<String, f64>,
    /// The red chromophore layer above an ammonia deck (W10a).
    #[serde(default)]
    pub chromophore: Option<Chromophore>,
    pub source: String,
}

/// A thin layer of red chromophore just above the ammonia cloud top, thicker
/// over belts than zones (Sromovsky et al. 2017).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Chromophore {
    /// The deck it sits on.
    pub above: String,
    /// Optical depth at 1 µm over zones and over belts.
    pub tau_zone_1um: f64,
    pub tau_belt_1um: f64,
    /// Effective radius over zones and belts, µm.
    pub radius_zone_um: f64,
    pub radius_belt_um: f64,
    /// Linear sRGB stand-in until the sheet's chromophore row lands.
    pub colour: [f64; 3],
    pub source: String,
}

/// The weather pack's giant settings (`provides.other.weather_giants`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GiantSettings {
    pub bands: Bands,
    pub stratification: Stratification,
    pub polar: Polar,
    pub polar_jet: PolarJet,
    pub polygon: Polygon,
    pub ovals: Family,
    pub strings: Family,
    pub dark_spots: Family,
    pub outbreaks: Outbreaks,
    pub clouds: GiantClouds,
    pub references: Vec<Reference>,
}

impl GiantSettings {
    /// The last pack in the stack that provides `weather_giants` wins.
    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("weather_giants"))
            .ok_or("no pack provides weather_giants settings")?;
        let s: GiantSettings = serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("weather_giants settings: {e}"))?;
        if s.references
            .iter()
            .any(|r| r.doi.is_none() && r.url.is_none())
        {
            return Err("every weather_giants reference needs a DOI or a URL".into());
        }
        let p = &s.polygon;
        if p.wavelength_ld <= 0.0 || p.min_sides < 3 || p.max_sides < p.min_sides {
            return Err("polygon needs wavelength_ld > 0 and 3 <= min_sides <= max_sides".into());
        }
        if s.polar.ring_counts.is_empty() || s.clouds.chemistry.is_empty() {
            return Err("ring_counts and chemistry must not be empty".into());
        }
        if !(s.polar.ring_bu_max <= s.polar.single_bu_min
            && s.polar.single_bu_min <= s.polar.diffuse_bu_min)
        {
            return Err("polar Burger thresholds must rise: ring <= single <= diffuse".into());
        }
        for f in [&s.ovals, &s.strings, &s.dark_spots] {
            if f.window_days <= 0.0 || f.life_years[0] <= 0.0 || f.life_years[1] < f.life_years[0] {
                return Err(format!("storm family {}: bad window or life", f.kind));
            }
        }
        Ok(s)
    }
}

/// The base weather pack's giant settings.
pub fn base_settings() -> GiantSettings {
    GiantSettings::from_packs(&[crate::scenario::base_weather_pack()]).expect("giant settings")
}

// ---------------------------------------------------------------- layout

/// One pole's regime and features.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Pole {
    pub north: bool,
    /// `cyclone-ring`, `single-vortex` or `diffuse-cap`.
    pub regime: String,
    /// `rule`, or `drawn` when the Burger number sits in the transition.
    pub evidence: String,
    /// Cyclones in the ring (0 unless a ring).
    pub ring_count: u32,
    /// Longitude of the first ring cyclone or polygon vertex.
    pub orient_lon_deg: f64,
    /// Polygon sides (0: no polygon).
    pub polygon_sides: u32,
    pub why: Vec<String>,
}

/// Everything about a giant's bands and poles, from the conditions alone.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct GiantLayout {
    /// From gravity and radius: g a² / G.
    pub mass_earth: f64,
    pub ice: bool,
    pub wind_scale_m_s: f64,
    /// Local eddy speed the Rhines scale reads, m/s.
    #[serde(default)]
    pub eddy_m_s: f64,
    pub jets: u32,
    /// Band edges pole to pole (the S6 cloud deck's).
    pub band_edges_deg: Vec<f64>,
    pub scale_height_km: f64,
    /// Deformation radius at the pole, km.
    pub deformation_km: f64,
    pub burger: f64,
    pub a_over_ld: f64,
    pub polar_jet_lat_deg: f64,
    pub polar_jet_m_s: f64,
    /// Deformation radius at the polar jet's latitude, km.
    pub jet_deformation_km: f64,
    /// Waves the jet's latitude circle holds: 2π a cos φ / (k L_d), unrounded.
    pub polygon_waves: f64,
    /// The condition for a polygon, apart from the pole's regime.
    pub polygon_jet_ok: bool,
    pub polygon_why: String,
    /// North, then south.
    pub poles: Vec<Pole>,
    /// Condensates of the low, middle and high layer.
    pub layers: [String; 3],
    pub why: Vec<String>,
}

fn draws(c: &Conditions, stream: &str) -> Draws {
    Draws::new(
        Seed::from_text(&c.address.to_string()),
        c.address.clone(),
        Stream::new(stream).expect("stream name"),
    )
}

/// Mass in Earth masses from surface gravity and radius.
pub fn mass_earth(c: &Conditions) -> f64 {
    c.gravity_m_s2 * c.radius_m * c.radius_m / G / M_EARTH_KG
}

/// Jets from the Rhines scale and the eddy speed, as the S6 cloud deck (the
/// one count, W7a).
pub fn jets(a: f64, omega: f64, eddy: f64) -> u32 {
    sse_climate::circulation::giant_jets(a, omega, eddy)
}

/// The S6 band edges, pole to pole: asin(−1 + 2i / 2J).
pub fn band_edges(jets: u32) -> Vec<f64> {
    sse_climate::circulation::giant_band_edges(jets)
}

/// Deformation radius N H / f at a latitude, m.
pub fn deformation_m(gs: &GiantSettings, c: &Conditions, lat_deg: f64) -> f64 {
    let f = 2.0 * c.spin_rate() * m::sin(lat_deg.abs().max(1.0) / DEG);
    gs.stratification.brunt_s * c.scale_height_m() / f
}

/// Polar jet latitude: one polar Rhines length L = (U a² / 2Ω)^⅓ from the pole.
pub fn polar_jet_lat(a: f64, omega: f64, u: f64) -> f64 {
    sse_climate::circulation::polar_jet_lat(a, omega, u)
}

pub fn layout(gs: &GiantSettings, c: &Conditions) -> GiantLayout {
    let b = &gs.bands;
    let a = c.radius_m;
    let omega = c.spin_rate();
    let me = mass_earth(c);
    let ice = me > 0.0 && me < b.ice_giant_mass_earth;
    let u = if ice { b.wind_ice_m_s } else { b.wind_gas_m_s };
    let eddy = if ice { b.eddy_ice_m_s } else { b.eddy_gas_m_s };
    let nj = jets(a, omega, eddy);
    let edges = band_edges(nj);
    let h = c.scale_height_m();
    let ld = deformation_m(gs, c, 90.0);
    let bu = (ld / a) * (ld / a);
    let lat_p = polar_jet_lat(a, omega, u);
    let up = gs.polar.polar_jet_u * u;
    let ld_jet = deformation_m(gs, c, lat_p);
    let pg = &gs.polygon;
    let waves = 2.0 * m::PI * a * m::cos(lat_p / DEG) / (pg.wavelength_ld * ld_jet);
    let lo = pg.min_sides as f64 - 0.5;
    let hi = pg.max_sides as f64 + 0.5;
    let jet_ok = lat_p >= pg.min_lat_deg && up >= pg.min_jet_m_s && waves >= lo && waves < hi;
    let polygon_why = if jet_ok {
        format!(
            "Rule (fitted to Saturn's hexagon): the circumpolar jet at {lat_p:.1}° runs at {up:.0} m/s and its latitude circle holds {waves:.2} waves of {:.0} deformation radii ({:.0} km each), so it can settle into a polygon of {} sides.",
            pg.wavelength_ld,
            ld_jet / 1000.0,
            sides(gs, waves)
        )
    } else if lat_p < pg.min_lat_deg {
        format!(
            "No polygon: the circumpolar jet sits at {lat_p:.1}°, below the {:.0}° the rule needs.",
            pg.min_lat_deg
        )
    } else if up < pg.min_jet_m_s {
        format!(
            "No polygon: the circumpolar jet runs at {up:.0} m/s, below the {:.0} m/s the rule needs.",
            pg.min_jet_m_s
        )
    } else {
        format!(
            "No polygon: the jet's latitude circle holds {waves:.2} deformation-scale waves, outside the {} to {} a stable polygon needs (too {}).",
            pg.min_sides,
            pg.max_sides,
            if waves < lo { "broad for its circle" } else { "narrow: it breaks into eddies" }
        )
    };
    let t = c.t_surface_k;
    let chem = gs
        .clouds
        .chemistry
        .iter()
        .find(|x| t < x.max_k)
        .or(gs.clouds.chemistry.last())
        .expect("chemistry")
        .layers
        .clone();
    let d = draws(c, POLES_STREAM);
    let poles = [true, false]
        .iter()
        .map(|&north| pole(gs, c, &d, north, bu, jet_ok, waves))
        .collect();
    let why = vec![
        format!(
            "{nj} jets from the Rhines scale, round(sqrt(a Ω / U)), with a {:.0} km, a {:.2} h day and winds of {u:.0} m/s ({} giant of {me:.0} Earth masses); {} bands with edges evenly spaced in sin(latitude), as the S6 cloud deck.",
            a / 1000.0,
            c.rotation_hours,
            if ice { "an ice" } else { "a gas" },
            2 * nj
        ),
        format!(
            "Deformation radius at the pole {:.0} km (N H / 2Ω, N {} s⁻¹, H {:.1} km): Burger number {bu:.2e}, a / L_d {:.0}.",
            ld / 1000.0,
            gs.stratification.brunt_s,
            h / 1000.0,
            a / ld
        ),
        format!(
            "Circumpolar jet at {lat_p:.1}°, one polar Rhines length (U a² / 2Ω)^⅓ from the pole."
        ),
        polygon_why.clone(),
    ];
    GiantLayout {
        mass_earth: me,
        ice,
        wind_scale_m_s: u,
        eddy_m_s: eddy,
        jets: nj,
        band_edges_deg: edges,
        scale_height_km: h / 1000.0,
        deformation_km: ld / 1000.0,
        burger: bu,
        a_over_ld: a / ld,
        polar_jet_lat_deg: lat_p,
        polar_jet_m_s: up,
        jet_deformation_km: ld_jet / 1000.0,
        polygon_waves: waves,
        polygon_jet_ok: jet_ok,
        polygon_why,
        poles,
        layers: chem,
        why,
    }
}

/// Polygon sides from the unrounded wave count: round, clamped to the pack range.
pub fn sides(gs: &GiantSettings, waves: f64) -> u32 {
    (m::round(waves) as i64).clamp(gs.polygon.min_sides as i64, gs.polygon.max_sides as i64) as u32
}

fn pole(
    gs: &GiantSettings,
    c: &Conditions,
    d: &Draws,
    north: bool,
    bu: f64,
    jet_ok: bool,
    waves: f64,
) -> Pole {
    let p = &gs.polar;
    let base = if north { 0 } else { 16 };
    let name = if north { "north" } else { "south" };
    let mut evidence = "rule";
    let (regime, why) = if c.internal_flux_w_m2 < p.diffuse_internal_max_w_m2 {
        (
            "diffuse-cap",
            format!(
                "Rule: internal heat of {:.2} W/m² is below {:.2} W/m², too weak a convective forcing to spin up a polar cyclone, so the {name} pole is a broad, diffuse cap (O'Neill, Emanuel and Flierl 2015).",
                c.internal_flux_w_m2, p.diffuse_internal_max_w_m2
            ),
        )
    } else if bu >= p.diffuse_bu_min {
        (
            "diffuse-cap",
            format!(
                "Rule: Burger number {bu:.2e} is at or above {:.1e}, ice-giant-like, so the {name} pole is a broad, diffuse cap (Brueshaber, Sayanagi and Dowling 2019).",
                p.diffuse_bu_min
            ),
        )
    } else if bu <= p.ring_bu_max {
        (
            "cyclone-ring",
            format!(
                "Rule: Burger number {bu:.2e} is at or below {:.2e} (a / L_d ≥ 40), so storms at the {name} pole stay small against the planet and gather as a ring of cyclones around a central one, as on Jupiter (Brueshaber, Sayanagi and Dowling 2019; O'Neill, Emanuel and Flierl 2015).",
                p.ring_bu_max
            ),
        )
    } else if bu >= p.single_bu_min {
        (
            "single-vortex",
            format!(
                "Rule: Burger number {bu:.2e} is at or above {:.2e} (a / L_d ≤ 30), so storms drifting to the {name} pole merge into one strong polar vortex, as on Saturn (Brueshaber, Sayanagi and Dowling 2019; O'Neill, Emanuel and Flierl 2015).",
                p.single_bu_min
            ),
        )
    } else {
        evidence = "drawn";
        let ring = d.unit(base) < 0.5;
        (
            if ring { "cyclone-ring" } else { "single-vortex" },
            format!(
                "Rule with a draw: Burger number {bu:.2e} lies in the transition between a ring of cyclones and one polar vortex (a / L_d between 30 and 40, O'Neill, Emanuel and Flierl 2015); the {name} pole drew {}.",
                if ring { "a ring" } else { "one vortex" }
            ),
        )
    };
    let ring_count = if regime == "cyclone-ring" {
        evidence = "drawn";
        p.ring_counts[d.below(base + 1, p.ring_counts.len() as u64) as usize]
    } else {
        0
    };
    let polygon = jet_ok && gs.polygon.regimes.iter().any(|r| r == regime);
    let mut why = vec![why];
    if ring_count > 0 {
        why.push(format!(
            "The ring holds {ring_count} cyclones, drawn from {:?} (Juno saw 8 around Jupiter's north pole and 5 around its south: Adriani et al. 2018).",
            p.ring_counts
        ));
    }
    if polygon {
        why.push(format!(
            "Rule (fitted to Saturn): the coherent circumpolar jet carries a polygon of {} sides.",
            sides(gs, waves)
        ));
    }
    Pole {
        north,
        regime: regime.into(),
        evidence: evidence.into(),
        ring_count,
        orient_lon_deg: 360.0 * d.unit(base + 2) - 180.0,
        polygon_sides: if polygon { sides(gs, waves) } else { 0 },
        why,
    }
}

/// Fractional band coordinate from the equator: x = |sin φ| × jets.
fn band_x(l: &GiantLayout, lat: f64) -> f64 {
    m::sin(lat.abs() / DEG).min(1.0) * l.jets as f64
}

/// Zone (1) or belt (0) weight: zones at even bands from the equator.
pub fn zone_weight(l: &GiantLayout, lat: f64) -> f64 {
    0.5 + 0.5 * m::sin(m::PI * band_x(l, lat))
}

/// The banded wind without the polar jet, m/s eastward.
fn band_wind(gs: &GiantSettings, l: &GiantLayout, lat: f64) -> f64 {
    let b = &gs.bands;
    let x = band_x(l, lat);
    let eb = if l.ice {
        b.equatorial_bands_ice
    } else {
        b.equatorial_bands_gas
    }
    .clamp(1, l.jets) as f64;
    let eq = if l.ice { b.equator_ice } else { b.equator_gas };
    let p = if x < eb {
        eq + (1.0 - eq) * 0.5 * (1.0 - m::cos(m::PI * x / eb))
    } else {
        let g = -m::cos(m::PI * x);
        if g < 0.0 {
            g * b.retrograde_ratio
        } else {
            g
        }
    };
    l.wind_scale_m_s * m::cos(lat / DEG) * p
}

fn blend_jet(l: &GiantLayout, base: f64, dist_km: f64) -> f64 {
    let w = l.jet_deformation_km.max(1.0);
    let g = m::exp(-(dist_km / w) * (dist_km / w));
    base + (l.polar_jet_m_s - base) * g
}

/// Eastward wind at a latitude, m/s: bands plus the circumpolar jet.
pub fn zonal_wind(gs: &GiantSettings, l: &GiantLayout, a_m: f64, lat: f64) -> f64 {
    let dist = (lat.abs() - l.polar_jet_lat_deg).abs() / DEG * a_m / 1000.0;
    blend_jet(l, band_wind(gs, l, lat), dist)
}

/// The jets by latitude, for drawing the curls and drift (visual polish
/// K-a): one entry per band, at its centre.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct JetProfile {
    /// Band centres, south to north, degrees.
    pub lat_deg: Vec<f64>,
    /// Eastward wind at the band centre, m/s (+ eastward).
    pub jet_speed_m_s: Vec<f64>,
    /// Change in eastward wind from the band's south edge to its north
    /// edge, m/s per band.
    pub jet_shear_m_s: Vec<f64>,
    /// Local eddy speed, m/s: one value per giant kind for now.
    pub eddy_speed_m_s: Vec<f64>,
    /// Rhines width of one jet, π sqrt(2U/β), U the eddy speed, m.
    pub rhines_scale_m: Vec<f64>,
    /// Deformation radius N H / f, m.
    pub deformation_radius_m: Vec<f64>,
    pub source: String,
}

pub fn jet_profile(gs: &GiantSettings, c: &Conditions, l: &GiantLayout) -> JetProfile {
    let a = c.radius_m;
    let omega = c.spin_rate();
    let e = &l.band_edges_deg;
    let mut p = JetProfile {
        lat_deg: vec![],
        jet_speed_m_s: vec![],
        jet_shear_m_s: vec![],
        eddy_speed_m_s: vec![],
        rhines_scale_m: vec![],
        deformation_radius_m: vec![],
        source: "Band winds of the S6 layout; Rhines width pi sqrt(2U/beta) with U the eddy speed (Rhines 1975; Gastine et al. 2014: the factor of 2 is not settled); deformation radius N H / f.".into(),
    };
    for w in e.windows(2) {
        let lat = 0.5 * (w[0] + w[1]);
        p.lat_deg.push(lat);
        p.jet_speed_m_s.push(zonal_wind(gs, l, a, lat));
        p.jet_shear_m_s
            .push(zonal_wind(gs, l, a, w[1]) - zonal_wind(gs, l, a, w[0]));
        p.eddy_speed_m_s.push(l.eddy_m_s);
        p.rhines_scale_m
            .push(sse_climate::circulation::rhines_width_m(
                a, omega, l.eddy_m_s, lat,
            ));
        p.deformation_radius_m.push(deformation_m(gs, c, lat));
    }
    p
}

/// Longitude at `day` of something at `lat` that was at `lon0` on `born`,
/// drifting with the local wind: Δλ = u Δt / (a cos φ).
pub fn drift_lon(u: f64, a_m: f64, lat: f64, lon0: f64, born: f64, day: f64) -> f64 {
    let r = a_m * m::cos(lat / DEG).max(1e-6);
    wrap(lon0 + u * (day - born) * DAY_S / r * DEG)
}

fn wrap(lon: f64) -> f64 {
    let x = m::fmod(lon + 180.0, 360.0);
    (if x < 0.0 { x + 360.0 } else { x }) - 180.0
}

// ---------------------------------------------------------------- systems

/// Counter for (family, window, slot): windows are keyed by index.
fn ctr(fam: u64, window: i64, slot: u64) -> u64 {
    let w = (window.wrapping_add(1 << 40) as u64) & ((1u64 << 44) - 1);
    (fam << 56) | (w << 12) | (slot & 0xfff)
}

fn poisson(u: f64, lam: f64) -> u32 {
    let mut p = m::exp(-lam);
    let mut cdf = p;
    let mut k = 0u32;
    while u > cdf && k < 15 {
        k += 1;
        p *= lam / k as f64;
        cdf += p;
    }
    k
}

fn track(u: f64, a: f64, lat: f64, lon0: f64, born: f64, ends: f64) -> Vec<[f64; 3]> {
    (0..12)
        .map(|i| {
            let t = born + (ends - born) * i as f64 / 11.0;
            [t, lat, drift_lon(u, a, lat, lon0, born, t)]
        })
        .collect()
}

fn applies(f: &str, l: &GiantLayout) -> bool {
    match f {
        "ice" => l.ice,
        "gas" => !l.ice,
        _ => true,
    }
}

/// Zones poleward of the equatorial one: (band index from the equator).
fn zones(l: &GiantLayout) -> Vec<u32> {
    (2..l.jets).step_by(2).collect()
}

#[allow(clippy::too_many_arguments)]
fn family(
    gs: &GiantSettings,
    c: &Conditions,
    l: &GiantLayout,
    f: &Family,
    fam: u64,
    name: &str,
    day: f64,
    d: &Draws,
    out: &mut Vec<WeatherSystem>,
) {
    if !applies(&f.applies, l) || f.per_year <= 0.0 {
        return;
    }
    let zs = zones(l);
    let by_zone = f.lat_deg[1] <= 0.0;
    if by_zone && zs.is_empty() {
        return;
    }
    let a = c.radius_m;
    let w = f.window_days;
    let now = m::floor(day / w) as i64;
    let back = m::ceil(f.life_years[1] * YEAR_D / w) as i64;
    let lam = f.per_year * w / YEAR_D;
    let nj = l.jets as f64;
    for i in (now - back)..=now {
        let n = poisson(d.unit(ctr(fam, i, 0)), lam);
        for j in 0..n as u64 {
            let u = |k: u64| d.unit(ctr(fam, i, 1 + 32 * j + k));
            let born = (i as f64 + u(0)) * w;
            let life = f.life_years[0] * m::exp(u(1) * m::ln(f.life_years[1] / f.life_years[0]));
            let ends = born + life * YEAR_D;
            if day < born || day >= ends {
                continue;
            }
            let hemi = if u(2) < 0.5 { 1.0 } else { -1.0 };
            let (lat, band_km) = if by_zone {
                let k = zs[(u(3) * zs.len() as f64) as usize % zs.len()] as f64;
                let s = (k + 0.3 + 0.4 * u(4)) / nj;
                let wkm = (m::asin(((k + 1.0) / nj).min(1.0)) - m::asin(k / nj)) * a / 1000.0;
                (hemi * m::asin(s) * DEG, wkm)
            } else {
                (
                    hemi * (f.lat_deg[0] + (f.lat_deg[1] - f.lat_deg[0]) * u(4)),
                    0.0,
                )
            };
            let size = if f.size_band[1] > 0.0 {
                band_km * (f.size_band[0] + (f.size_band[1] - f.size_band[0]) * u(5))
            } else {
                f.size_km[0] + (f.size_km[1] - f.size_km[0]) * u(5)
            };
            let strength =
                l.wind_scale_m_s * (f.strength_u[0] + (f.strength_u[1] - f.strength_u[0]) * u(6));
            let members = f.members[0]
                + (u(7) * (f.members[1] - f.members[0] + 1) as f64) as u32
                    % (f.members[1] - f.members[0] + 1);
            let wind = zonal_wind(gs, l, a, lat);
            let lon_first = 360.0 * u(8) - 180.0;
            // Members of a string sit about three sizes apart along the zone.
            let step = 3.0 * size * 1000.0 / (a * m::cos(lat / DEG).max(1e-6)) * DEG;
            for k in 0..members {
                let lon0 = wrap(lon_first + step * k as f64);
                let id = if members > 1 {
                    format!("{name}:{i}:{j}:{k}")
                } else {
                    format!("{name}:{i}:{j}")
                };
                out.push(WeatherSystem {
                    id,
                    kind: f.kind.clone(),
                    lat_deg: lat,
                    lon_deg: drift_lon(wind, a, lat, lon0, born, day),
                    track: track(wind, a, lat, lon0, born, ends),
                    size_km: size,
                    strength,
                    born,
                    ends,
                    count: if members > 1 { members } else { 0 },
                    why: format!(
                        "Rule: {}. This one sits at {:.0}°{} and drifts {:+.0} m/s with the local wind; drawn from the pack's rate of {} a year, it lasts {:.1} years.",
                        f.because,
                        lat.abs(),
                        if lat >= 0.0 { "N" } else { "S" },
                        wind,
                        f.per_year,
                        life
                    ),
                    evidence: "drawn".into(),
                });
            }
        }
    }
}

fn outbreaks(
    gs: &GiantSettings,
    c: &Conditions,
    l: &GiantLayout,
    day: f64,
    d: &Draws,
    out: &mut Vec<WeatherSystem>,
) {
    let o = &gs.outbreaks;
    if l.ice || c.t_surface_k > o.max_t_k || c.year_days <= 0.0 {
        return;
    }
    let a = c.radius_m;
    let y = c.year_days;
    let now = m::floor(day / y) as i64;
    let back = m::ceil(o.duration_days / y) as i64;
    for i in (now - back)..=now {
        let u = |k: u64| d.unit(ctr(9, i, k));
        if u(0) >= o.chance_per_cycle {
            continue;
        }
        let born = (i as f64 + u(1)) * y;
        let ends = born + o.duration_days;
        if day < born || day >= ends {
            continue;
        }
        let hemi = if u(2) < 0.5 { 1.0 } else { -1.0 };
        let lat = hemi * o.lat_max_deg * u(3);
        let lon0 = 360.0 * u(4) - 180.0;
        let wind = zonal_wind(gs, l, a, lat);
        out.push(WeatherSystem {
            id: format!("outbreak:{i}"),
            kind: "outbreak".into(),
            lat_deg: lat,
            lon_deg: drift_lon(wind, a, lat, lon0, born, day),
            track: track(wind, a, lat, lon0, born, ends),
            size_km: o.head_km,
            strength: o.strength_m_s,
            born,
            ends,
            count: 0,
            why: format!(
                "Rule: {} ({:.0} K at 1 bar, at or below {:.0} K); at most one per orbit of {:.0} days, onset and latitude drawn; it wraps the planet in about {:.0} days.",
                o.because, c.t_surface_k, o.max_t_k, y, o.encircle_days
            ),
            evidence: "drawn".into(),
        });
    }
}

fn polar_systems(gs: &GiantSettings, c: &Conditions, l: &GiantLayout) -> Vec<WeatherSystem> {
    let a = c.radius_m;
    let ld = l.deformation_km;
    let mut out = vec![];
    for p in &l.poles {
        let h = if p.north { 1.0 } else { -1.0 };
        let tag = if p.north { "n" } else { "s" };
        let why = p.why.join(" ");
        let sys = |id: String,
                   kind: &str,
                   lat: f64,
                   lon: f64,
                   size: f64,
                   strength: f64,
                   count: u32,
                   ev: &str| {
            WeatherSystem {
                id,
                kind: kind.into(),
                lat_deg: lat,
                lon_deg: lon,
                track: vec![],
                size_km: size,
                strength,
                born: ALWAYS,
                ends: f64::MAX,
                count,
                why: why.clone(),
                evidence: ev.into(),
            }
        };
        match p.regime.as_str() {
            "cyclone-ring" => {
                let dkm = gs.polar.ring_cyclone_ld * ld;
                let n = p.ring_count as f64;
                let r = n * dkm * (1.0 + gs.polar.ring_gap) / (2.0 * m::PI);
                let lat = h * (90.0 - r * 1000.0 / a * DEG);
                out.push(sys(
                    format!("pole:{tag}:ring"),
                    "cyclone-ring",
                    h * 90.0,
                    0.0,
                    2.0 * r + dkm,
                    l.polar_jet_m_s,
                    p.ring_count,
                    &p.evidence,
                ));
                out.push(sys(
                    format!("pole:{tag}:centre"),
                    "cyclone",
                    h * 90.0,
                    0.0,
                    dkm,
                    l.polar_jet_m_s,
                    0,
                    &p.evidence,
                ));
                for k in 0..p.ring_count {
                    let lon = wrap(p.orient_lon_deg + 360.0 * k as f64 / n);
                    out.push(sys(
                        format!("pole:{tag}:ring:{k}"),
                        "cyclone",
                        lat,
                        lon,
                        dkm,
                        l.polar_jet_m_s,
                        0,
                        &p.evidence,
                    ));
                }
            }
            "single-vortex" => {
                out.push(sys(
                    format!("pole:{tag}:vortex"),
                    "polar-vortex",
                    h * 90.0,
                    0.0,
                    gs.polar.vortex_ld * ld,
                    l.polar_jet_m_s,
                    0,
                    &p.evidence,
                ));
            }
            _ => {
                let r = (90.0 - l.polar_jet_lat_deg) / DEG * a / 1000.0;
                out.push(sys(
                    format!("pole:{tag}:cap"),
                    "diffuse-cap",
                    h * 90.0,
                    0.0,
                    2.0 * r,
                    0.0,
                    0,
                    &p.evidence,
                ));
            }
        }
        if p.polygon_sides > 0 {
            let r = (90.0 - l.polar_jet_lat_deg) / DEG * a / 1000.0;
            let mut s = sys(
                format!("pole:{tag}:polygon"),
                "polygon-jet",
                h * l.polar_jet_lat_deg,
                p.orient_lon_deg,
                2.0 * r,
                l.polar_jet_m_s,
                p.polygon_sides,
                "rule",
            );
            s.why = format!("{} {}", l.polygon_why, p.why[0]);
            out.push(s);
        }
    }
    out
}

/// The nameable systems of a giant alive on `date`: polar features
/// (permanent), then great ovals, strings, dark spots and outbreaks. Keyed on
/// the world's address: the same on every device.
pub fn giant_systems(gs: &GiantSettings, c: &Conditions, date: Date) -> Vec<WeatherSystem> {
    systems_for(gs, c, &layout(gs, c), date)
}

fn systems_for(
    gs: &GiantSettings,
    c: &Conditions,
    l: &GiantLayout,
    date: Date,
) -> Vec<WeatherSystem> {
    let d = draws(c, STORMS_STREAM);
    let mut out = polar_systems(gs, c, l);
    family(gs, c, l, &gs.ovals, 1, "oval", date.day, &d, &mut out);
    family(gs, c, l, &gs.strings, 2, "string", date.day, &d, &mut out);
    family(
        gs,
        c,
        l,
        &gs.dark_spots,
        3,
        "dark-spot",
        date.day,
        &d,
        &mut out,
    );
    outbreaks(gs, c, l, date.day, &d, &mut out);
    out
}

// ---------------------------------------------------------------- cells

/// A giant worked out once per date, for many cells.
#[derive(Clone, PartialEq, Debug)]
pub struct GiantPrepared {
    pub layout: GiantLayout,
    pub systems: Vec<WeatherSystem>,
    pub day: f64,
}

pub fn prepare(gs: &GiantSettings, c: &Conditions, date: Date) -> GiantPrepared {
    let l = layout(gs, c);
    let systems = systems_for(gs, c, &l, date);
    GiantPrepared {
        layout: l,
        systems,
        day: date.day,
    }
}

fn unit(lat: f64, lon: f64) -> [f64; 3] {
    let (p, l) = (lat / DEG, lon / DEG);
    [m::cos(p) * m::cos(l), m::cos(p) * m::sin(l), m::sin(p)]
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Distance between two places on the giant, km.
pub fn distance_km(a_m: f64, lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    m::acos(dot(unit(lat1, lon1), unit(lat2, lon2)).clamp(-1.0, 1.0)) * a_m / 1000.0
}

/// Eastward and northward components of the unit direction from a storm
/// centre to the cell, in the cell's tangent plane; None at the centre or a pole.
fn radial(lat: f64, lon: f64, lat_s: f64, lon_s: f64) -> Option<(f64, f64)> {
    let p = unit(lat, lon);
    let s = unit(lat_s, lon_s);
    let q = [p[0] - s[0], p[1] - s[1], p[2] - s[2]];
    let k = dot(q, p);
    let qt = [q[0] - k * p[0], q[1] - k * p[1], q[2] - k * p[2]];
    let (fl, ll) = (lat / DEG, lon / DEG);
    let e = [-m::sin(ll), m::cos(ll), 0.0];
    let n = [
        -m::sin(fl) * m::cos(ll),
        -m::sin(fl) * m::sin(ll),
        m::cos(fl),
    ];
    let (re, rn) = (dot(qt, e), dot(qt, n));
    let len = m::sqrt(re * re + rn * rn);
    (len > 1e-12 && m::cos(fl) > 1e-9).then(|| (re / len, rn / len))
}

/// Distance from the pole to the polygon's edge along longitude `lon`, km.
/// Vertices sit on the jet circle (azimuthal equidistant).
pub fn polygon_edge_km(r_jet_km: f64, n: u32, orient_deg: f64, lon: f64) -> f64 {
    let seg = 2.0 * m::PI / n as f64;
    let t = m::fmod(((lon - orient_deg) / DEG).rem_euclid(2.0 * m::PI), seg);
    r_jet_km * m::cos(m::PI / n as f64) / m::cos(t - m::PI / n as f64)
}

fn add_cover(l: &mut Layer, g: f64, tau: f64) {
    l.cloud_cover += (1.0 - l.cloud_cover) * g.clamp(0.0, 1.0);
    l.optical_depth += tau * g;
}

/// The weather at one place on a giant: belts and zones by latitude, winds
/// from the jets, and the storms and polar features darkening or brightening
/// the cloud around them.
pub fn giant_cell(
    gs: &GiantSettings,
    c: &Conditions,
    p: &GiantPrepared,
    lat: f64,
    lon: f64,
) -> Cell {
    let l = &p.layout;
    let cl = &gs.clouds;
    let a = c.radius_m;
    let w = zone_weight(l, lat);
    let deck = |cover: f64, cond: &str, tau: f64| Layer {
        cloud_cover: cover,
        cloud_type: "deck".into(),
        condensate: cond.into(),
        optical_depth: tau,
        wind_m_s: [0.0, 0.0],
        turbulence: 0.0,
        particle_radius_um: cl.particle_radius_um.get(cond).copied().unwrap_or(0.0),
    };
    let mut ly = vec![
        deck(cl.low_cover, &l.layers[0], cl.low_tau),
        deck(cl.mid_cover, &l.layers[1], cl.mid_tau),
        deck(
            cl.belt_cover + (cl.zone_cover - cl.belt_cover) * w,
            &l.layers[2],
            cl.belt_tau + (cl.zone_tau - cl.belt_tau) * w,
        ),
    ];
    // The circumpolar jet follows the polygon where this pole has one.
    let north = lat >= 0.0;
    let pole = l.poles.iter().find(|q| q.north == north);
    let r_jet = (90.0 - l.polar_jet_lat_deg) / DEG * a / 1000.0;
    let r_pole = (90.0 - lat.abs()) / DEG * a / 1000.0;
    let jet_dist = match pole {
        Some(q) if q.polygon_sides > 0 => {
            let edge = polygon_edge_km(r_jet, q.polygon_sides, q.orient_lon_deg, lon);
            if r_pole < edge {
                // Inside the polygon the high cloud thins (Saturn's hexagon interior).
                ly[2].cloud_cover = ly[2].cloud_cover.min(cl.belt_cover);
                ly[2].optical_depth = ly[2].optical_depth.min(cl.belt_tau);
            }
            (r_pole - edge).abs()
        }
        _ => (r_pole - r_jet).abs(),
    };
    let u = blend_jet(l, band_wind(gs, l, lat), jet_dist);
    let mut wind = [u, 0.0];
    let mut turb = 0.1 + 0.15 * (1.0 - w);
    let mut aerosols = vec![];
    // The chromophore: more over belts (w 0) than zones (w 1).
    if let Some(ch) = &cl.chromophore {
        if l.layers[2] == ch.above {
            aerosols.push(Aerosol {
                kind: "chromophore".into(),
                optical_depth: ch.tau_belt_1um + (ch.tau_zone_1um - ch.tau_belt_1um) * w,
                colour: ch.colour,
                particle_radius_um: ch.radius_belt_um + (ch.radius_zone_um - ch.radius_belt_um) * w,
            });
        }
    }
    let mut precip = "none";
    for s in &p.systems {
        let dist = distance_km(a, lat, lon, s.lat_deg, s.lon_deg);
        let rr = 0.5 * s.size_km.max(1.0);
        let r = dist / rr;
        let g = m::exp(-r * r);
        let spin = |sense: f64, wind: &mut [f64; 2]| {
            if r > 3.0 {
                return;
            }
            if let Some((re, rn)) = radial(lat, lon, s.lat_deg, s.lon_deg) {
                let h = if s.lat_deg >= 0.0 { 1.0 } else { -1.0 };
                let vt = s.strength * r * m::exp(0.5 * (1.0 - r * r));
                wind[0] += sense * h * vt * -rn;
                wind[1] += sense * h * vt * re;
            }
        };
        match s.kind.as_str() {
            "oval" => {
                add_cover(&mut ly[2], g, 4.0);
                turb += 0.3 * g;
                spin(-1.0, &mut wind);
            }
            "dark-spot" => {
                ly[2].optical_depth *= 1.0 - 0.85 * g;
                ly[2].cloud_cover *= 1.0 - 0.6 * g;
                // Bright companion clouds on the poleward side.
                let h = if s.lat_deg >= 0.0 { 1.0 } else { -1.0 };
                let lat_c = (s.lat_deg + h * 0.8 * rr * 1000.0 / a * DEG).clamp(-90.0, 90.0);
                let dc = distance_km(a, lat, lon, lat_c, s.lon_deg) / (0.35 * rr);
                let gc = m::exp(-dc * dc);
                if gc > 0.05 {
                    ly[2].cloud_type = "cirrus".into();
                }
                add_cover(&mut ly[2], gc, 3.0);
                turb += 0.2 * g;
                spin(-1.0, &mut wind);
            }
            "cyclone" => {
                add_cover(&mut ly[1], g, 2.0);
                add_cover(&mut ly[2], 0.5 * g, 2.0);
                turb += 0.4 * g;
                spin(1.0, &mut wind);
            }
            "polar-vortex" => {
                let eye = m::exp(-(r / 0.3) * (r / 0.3));
                ly[2].cloud_cover *= 1.0 - 0.9 * eye;
                ly[2].optical_depth *= 1.0 - 0.9 * eye;
                let wall = m::exp(-((r - 0.5) / 0.2) * ((r - 0.5) / 0.2));
                add_cover(&mut ly[1], wall, 3.0);
                turb += 0.4 * wall;
                spin(1.0, &mut wind);
            }
            "diffuse-cap" if (s.lat_deg >= 0.0) == north => {
                let x = r_pole / rr;
                let cap = m::exp(-x * x * x * x);
                if cap > 0.01 {
                    aerosols.push(Aerosol {
                        kind: "haze".into(),
                        optical_depth: cl.cap_tau * cap,
                        colour: [0.85, 0.9, 0.95],
                        particle_radius_um: 0.0,
                    });
                }
            }
            "outbreak" => {
                let o = &gs.outbreaks;
                let dy = (lat - s.lat_deg).abs() / DEG * a / 1000.0 / (0.5 * o.band_km);
                let age = p.day - s.born;
                let reach = 180.0 * (age / o.encircle_days).min(1.0);
                let dl = wrap(lon - s.lon_deg).abs();
                let gb = m::exp(-dy * dy) * if dl <= reach { 1.0 } else { 0.0 };
                let gg = gb.max(g);
                if gg > 0.05 {
                    ly[2].cloud_type = "cumulonimbus".into();
                    precip = "virga";
                }
                add_cover(&mut ly[2], gg, 15.0);
                turb += 0.5 * gg;
            }
            _ => {}
        }
    }
    for (k, layer) in ly.iter_mut().enumerate() {
        // Deep winds on a giant: a little weaker below the top layer.
        let f = [0.8, 0.9, 1.0][k];
        layer.wind_m_s = [wind[0] * f, wind[1] * f];
        layer.turbulence = turb.clamp(0.0, 1.0);
        layer.cloud_cover = layer.cloud_cover.clamp(0.0, 1.0);
        layer.optical_depth = layer.optical_depth.max(0.0);
    }
    Cell {
        lat_deg: lat,
        lon_deg: lon,
        layers: ly,
        precip_kind: precip.into(),
        precip_mm_day: 0.0,
        temp_offset_k: 0.0,
        lightning_km2_yr: 0.0,
        aerosols,
        wind10_m_s: 0.0,
    }
}

/// A field tile on a giant (the shape `RuleBlock::field` returns).
pub fn giant_tile(
    gs: &GiantSettings,
    c: &Conditions,
    date: Date,
    key: TileKey,
    n: u32,
) -> FieldTile {
    let p = prepare(gs, c, date);
    let cells = crate::rules::cell_dirs(key, n)
        .into_iter()
        .map(|d| {
            let (lat, lon) = crate::rules::lat_lon(d);
            giant_cell(gs, c, &p, lat, lon)
        })
        .collect();
    FieldTile { key, n, cells }
}

// ---------------------------------------------------------------- fixtures

#[derive(Clone, Debug, Deserialize)]
pub struct GiantFixture {
    /// The expected north polar regime.
    pub expect: String,
    pub conditions: Conditions,
}

pub fn fixtures() -> Vec<GiantFixture> {
    #[derive(Deserialize)]
    struct F {
        worlds: Vec<GiantFixture>,
    }
    let f: F = serde_json::from_str(include_str!("../../../tests/fixtures/weather/giants.json"))
        .expect("giant fixtures");
    f.worlds
}

pub fn fixture(label: &str) -> Conditions {
    fixtures()
        .into_iter()
        .find(|f| f.conditions.label == label)
        .unwrap_or_else(|| panic!("no giant fixture {label}"))
        .conditions
}

#[cfg(test)]
#[path = "giants_tests.rs"]
mod tests;

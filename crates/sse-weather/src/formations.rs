//! W2: formations on worlds with a surface (weather design section 2, step
//! 2), drawn storm systems with tracks, and the weather report (section 5).
//!
//! Systems are keyed draws from climate statistics: the same storm, with the
//! same id and track, on every device and whichever date it is asked from.
//! Storms are drawn per day of birth, so a storm alive on day d was born on
//! one of the days before it and is found by looking back one lifetime.

use crate::circulation::{self, bump};
use crate::conditions::Conditions;
use crate::contract::*;
use crate::rules::Prepared;
use crate::settings::Reference;
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Seed, Stream};
use sse_core::math as m;

const KM: f64 = 1000.0;

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Cyclones {
    /// Brunt-Väisälä frequency of the troposphere, s⁻¹.
    pub n_s: f64,
    /// Most unstable wavelength in deformation radii (Eady 1949).
    pub eady_wavelengths: f64,
    pub life_days: f64,
    pub peak_wind_m_s: f64,
    /// Winter strengthening of storms (fraction at the solstice).
    pub winter_boost: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Tropical {
    /// Least sea surface temperature for genesis, K.
    pub min_sst_k: f64,
    /// Never closer to the equator than this, degrees.
    pub min_lat_deg: f64,
    /// Storms a year on a world with Earth's warm ocean area.
    pub per_year_earth: f64,
    /// Earth's ocean area warmer than `min_sst_k` at peak season, as a fraction of the sphere.
    pub earth_warm_fraction: f64,
    pub life_days: f64,
    pub peak_wind_m_s: f64,
    pub size_km: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Dust {
    /// Chance per year of a planet-encircling storm on a dry thin-air world.
    pub global_per_year: f64,
    /// Year phase (0 = northern spring) of the dust season peak.
    pub season_phase: f64,
    pub regional_per_year: f64,
    pub optical_depth: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Orography {
    /// Extra rain per m/s of forced uplift, as a multiple of the local mean.
    pub rain_per_m_s: f64,
    pub sample_km: f64,
    pub cap_cloud_m: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Lightning {
    /// Thunderstorm cells per 10⁴ km² where convection is deep.
    pub storms_per_1e4_km2: f64,
    /// Price and Rind: flashes a minute = a H^b, H cloud top km.
    pub land: [f64; 2],
    pub ocean: [f64; 2],
    pub source: String,
}

/// W7c: eastern-boundary upwelling, the cold water under the marine decks.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Currents {
    /// How far offshore the cold water reaches, km (e-folding).
    pub upwelling_km: f64,
    /// Cooling of the sea surface at the coast under full sinking air, K.
    pub upwelling_cooling_k: f64,
    /// Western boundary current: its warm water's reach offshore, km, its
    /// warming at the coast, K, and the gyre's half-width about the Hadley
    /// edge, degrees of latitude.
    pub boundary_km: f64,
    pub boundary_warming_k: f64,
    pub gyre_half_width_deg: f64,
    pub source: String,
}

/// Nearness to a coast at a sea place, 0 to 1: exp(−d / L), L the
/// `upwelling_km`, d the distance to the nearest land lying spin-east
/// (`spin` +1) or spin-west (−1) of it along the parallel, within 4 L.
fn upwelling(cu: &Currents, c: &Conditions, t: &dyn Terrain, lat: f64, lon: f64, spin: f64) -> f64 {
    let l = cu.upwelling_km * KM;
    if l <= 0.0 {
        return 0.0;
    }
    let per_deg = c.radius_m * m::cos(rad(lat)).max(0.05) * m::PI / 180.0;
    let steps = 16;
    for k in 1..=steps {
        let d = 4.0 * l * k as f64 / steps as f64;
        let (_, wet) = t.ground(dir(lat, lon + spin.signum() * d / per_deg));
        if !wet {
            return m::exp(-d / l);
        }
    }
    0.0
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FormationSettings {
    pub cyclones: Cyclones,
    pub tropical: Tropical,
    pub dust: Dust,
    pub orography: Orography,
    pub lightning: Lightning,
    pub marine_layer: f64,
    /// W7c: cold upwelling off eastern-boundary coasts.
    pub currents: Currents,
    pub clear_visibility_km: f64,
    pub references: Vec<Reference>,
}

impl FormationSettings {
    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("weather_formations"))
            .ok_or("no pack provides weather formations")?;
        let s: FormationSettings = serde_json::to_value(v)
            .and_then(serde_json::from_value)
            .map_err(|e| format!("weather formations: {e}"))?;
        if s.references
            .iter()
            .any(|r| r.doi.is_none() && r.url.is_none())
        {
            return Err("every weather reference needs a DOI or a URL".into());
        }
        Ok(s)
    }
}

/// The ground under the weather, when the surface atlas is known.
pub trait Terrain {
    /// Height above sea level (m) and whether it is under liquid.
    fn ground(&self, d: [f64; 3]) -> (f64, bool);
}

impl Terrain for sse_surface::atlas::Atlas {
    fn ground(&self, d: [f64; 3]) -> (f64, bool) {
        let (h, p) = self.height(d, 0);
        (h, p.liquid)
    }
}

fn rad(x: f64) -> f64 {
    x * m::PI / 180.0
}

pub fn dir(lat: f64, lon: f64) -> [f64; 3] {
    let (la, lo) = (rad(lat), rad(lon));
    [m::cos(la) * m::cos(lo), m::cos(la) * m::sin(lo), m::sin(la)]
}

/// Great-circle distance, km.
pub fn distance_km(c: &Conditions, a: (f64, f64), b: (f64, f64)) -> f64 {
    let (p, q) = (dir(a.0, a.1), dir(b.0, b.1));
    let dot = (p[0] * q[0] + p[1] * q[1] + p[2] * q[2]).clamp(-1.0, 1.0);
    m::acos(dot) * c.radius_m / KM
}

fn wrap(lon: f64) -> f64 {
    let x = m::fmod(lon + 180.0, 360.0);
    if x < 0.0 {
        x + 180.0
    } else {
        x - 180.0
    }
}

/// Near-surface temperature at a latitude and season, K: the one climate's
/// season (W7a). A star-locked world's heat follows the star, not latitude,
/// so it reads the mean and the cell adds the day-night offset.
pub fn t_at(p: &Prepared, c: &Conditions, lat: f64, phase: f64) -> f64 {
    if c.star_locked {
        return c.t_surface_k;
    }
    let sea = p.core.season_at(phase);
    p.core.lerp(&sea.t_k, lat)
}

/// Weather draws are keyed by the world's address (which carries its
/// universe), a stream per kind of system, and a counter per day of birth.
fn seed() -> Seed {
    Seed::from_text("sse4-weather-rules")
}

/// Counter for day `k` in lane `lane`: unit draws use `base .. base + 64`.
fn slot(lane: u64, k: i64) -> u64 {
    let day = (k + (1i64 << 35)).max(0) as u64 & ((1u64 << 36) - 1);
    ((lane << 36) | day) << 6
}

/// A normal draw from its own range of counters, apart from the unit draws.
fn nrm(d: &Draws, i: u64) -> f64 {
    d.normal((1u64 << 61) | (i & ((1u64 << 60) - 1)))
}

fn draws(c: &Conditions, stream: &str) -> Draws {
    Draws::new(
        seed(),
        c.address.clone(),
        Stream::new(stream).expect("stream"),
    )
}

/// Track points every 6 hours over a life, moving at (east, north) m/s.
fn track(
    c: &Conditions,
    born: f64,
    ends: f64,
    lat0: f64,
    lon0: f64,
    v: impl Fn(f64, f64) -> (f64, f64),
) -> Vec<[f64; 3]> {
    let mut out = vec![];
    let (mut lat, mut lon, mut t) = (lat0, lon0, born);
    let dt = 0.25;
    while t <= ends + 1e-9 {
        out.push([t, lat, lon]);
        let (ue, vn) = v(lat, t - born);
        let s = dt * sse_core::sheet::DAY;
        lat = (lat + vn * s / c.radius_m * 180.0 / m::PI).clamp(-89.0, 89.0);
        let cl = m::cos(rad(lat)).max(0.05);
        lon = wrap(lon + ue * s / (c.radius_m * cl) * 180.0 / m::PI);
        t += dt;
    }
    out
}

/// Where a system is on a date (nearest track point at or before it).
pub fn position_at(sys: &WeatherSystem, day: f64) -> (f64, f64) {
    let mut p = (sys.lat_deg, sys.lon_deg);
    for q in &sys.track {
        if q[0] <= day + 1e-9 {
            p = (q[1], q[2]);
        }
    }
    p
}

fn steering(s: &crate::WeatherSettings, p: &Prepared, lat: f64) -> f64 {
    circulation::zonal_wind(s, &p.circ, lat) * 2.5
}

/// The drawn and permanent systems alive on a date.
pub fn systems(
    fs: &FormationSettings,
    s: &crate::WeatherSettings,
    c: &Conditions,
    p: &Prepared,
    date: Date,
) -> Vec<WeatherSystem> {
    let mut out = vec![];
    let day = date.day;
    let regime = p.circ.regime.as_str();
    if regime == "airless" || regime == "banded-giant" {
        return out;
    }
    let omega = c.spin_rate();
    // Mid-latitude cyclones along every storm track.
    if p.cond.is_some() && !p.circ.storm_tracks_deg.is_empty() && regime != "thin-air" {
        let cy = &fs.cyclones;
        let d = draws(c, "weather.cyclones.v1");
        let life_max = cy.life_days * 1.4;
        for (ti, st) in p.circ.storm_tracks_deg.iter().enumerate() {
            let f = 2.0 * omega * m::sin(rad(*st));
            let h = s.climate.circulation.tropopause_scale_heights * c.scale_height_m();
            let lambda = cy.eady_wavelengths * cy.n_s * h / f;
            let circ_km = 2.0 * m::PI * c.radius_m * m::cos(rad(*st));
            let waves = (circ_km / lambda).max(1.0);
            let per_day = waves / cy.life_days;
            for hemi in [1.0f64, -1.0] {
                let first = m::floor(day - life_max) as i64;
                for k in first..=(m::floor(day) as i64) {
                    let i = slot(2 * ti as u64 + (hemi > 0.0) as u64, k);
                    let nb =
                        m::floor(per_day) as u64 + (d.unit(i) < per_day - m::floor(per_day)) as u64;
                    for j in 0..nb.min(8) {
                        let ij = i + 2 + j * 6;
                        let born = k as f64 + d.unit(ij);
                        let life = cy.life_days * (0.6 + 0.8 * d.unit(ij + 1));
                        let ends = born + life;
                        if !(born <= day && day < ends) {
                            continue;
                        }
                        let lat0 = hemi * (st + 4.0 * nrm(&d, ij + 2)).clamp(15.0, 85.0);
                        let lon0 = d.uniform(ij + 3, -180.0, 180.0);
                        let winter = -hemi
                            * circulation::declination_deg(
                                c,
                                crate::rules::year_phase(c, Date { day: born }),
                            )
                            / c.obliquity_deg.max(1.0);
                        let strength = cy.peak_wind_m_s
                            * (1.0 + cy.winter_boost * winter.clamp(-1.0, 1.0))
                            * (0.7 + 0.6 * d.unit(ij + 4));
                        let u = steering(s, p, lat0).max(3.0);
                        let tr = track(c, born, ends, lat0, lon0, |_, _| (u, hemi * 1.5));
                        let (la, lo) = (tr[0][1], tr[0][2]);
                        out.push(WeatherSystem {
                            id: format!("cyclone-{}{}-{}-{}", if hemi > 0.0 { "n" } else { "s" }, ti, k, j),
                            kind: "cyclone".into(),
                            lat_deg: la,
                            lon_deg: lo,
                            track: tr,
                            size_km: lambda / 2.0 / KM,
                            strength,
                            born,
                            ends,
                            count: 0,
                            why: format!(
                                "A storm-track cyclone near {st:.0}°: baroclinic waves about {:.0} km long (Eady 1949) travel east with the westerlies and live days.",
                                lambda / KM
                            ),
                            evidence: "drawn".into(),
                        });
                    }
                }
            }
        }
    }
    // Tropical cyclones: warm ocean, off the equator, a spin neither slow nor locked.
    let tc = &fs.tropical;
    let warm_ocean = c.liquid.as_deref() == Some("water") && c.ocean_fraction > 0.0;
    if warm_ocean && matches!(regime, "earth-like" | "fast-rotator") {
        let d = draws(c, "weather.tropical.v1");
        let lo_lat = tc.min_lat_deg;
        let hi_lat = p.circ.hadley_edge_deg.max(lo_lat + 1.0);
        let first = m::floor(day - tc.life_days * 1.5) as i64;
        for k in first..=(m::floor(day) as i64) {
            for hemi in [1.0f64, -1.0] {
                let ph = crate::rules::year_phase(c, Date { day: k as f64 });
                // Warm area this season in this hemisphere (fraction of sphere).
                let mut warm = 0.0;
                for q in 0..18 {
                    let la = lo_lat + (hi_lat - lo_lat) * (q as f64 + 0.5) / 18.0;
                    if t_at(p, c, hemi * la, ph) >= tc.min_sst_k {
                        warm += m::cos(rad(la)) * rad((hi_lat - lo_lat) / 18.0) / 2.0;
                    }
                }
                warm *= c.ocean_fraction;
                let rate = tc.per_year_earth / 365.25 * warm / tc.earth_warm_fraction;
                let i = slot((hemi > 0.0) as u64, k);
                if rate <= 0.0 || d.unit(i) >= rate.min(1.0) {
                    continue;
                }
                let born = k as f64 + d.unit(i + 1);
                let ends = born + tc.life_days * (0.6 + 0.8 * d.unit(i + 2));
                if !(born <= day && day < ends) {
                    continue;
                }
                // Born where the sea is warm enough.
                let mut lat0 = hemi * (lo_lat + (hi_lat - lo_lat) * 0.5 * d.unit(i + 3));
                if t_at(p, c, lat0, ph) < tc.min_sst_k {
                    lat0 = hemi * (lo_lat + 2.0);
                }
                let lon0 = d.uniform(i + 4, -180.0, 180.0);
                let edge = p.circ.hadley_edge_deg * 0.7;
                let tr = track(c, born, ends, lat0, lon0, |la, _| {
                    // West in the trades, then recurving east past the subtropics.
                    if la.abs() < edge {
                        (-5.0, hemi * 1.5)
                    } else {
                        (6.0, hemi * 3.0)
                    }
                });
                out.push(WeatherSystem {
                    id: format!("tropical-{}-{}", if hemi > 0.0 { "n" } else { "s" }, k),
                    kind: "tropical-cyclone".into(),
                    lat_deg: tr[0][1],
                    lon_deg: tr[0][2],
                    track: tr,
                    size_km: tc.size_km,
                    strength: tc.peak_wind_m_s * (0.6 + 0.6 * d.unit(i + 5)),
                    born,
                    ends,
                    count: 0,
                    why: format!(
                        "A tropical cyclone: sea above {:.0} °C, at least {:.0}° from the equator where spin can organise it (Emanuel 2003).",
                        tc.min_sst_k - sse_core::sheet::T0C,
                        tc.min_lat_deg
                    ),
                    evidence: "drawn".into(),
                });
            }
        }
    }
    // Dust on dry thin-air worlds.
    if regime == "thin-air" && c.liquid.is_none() && c.solid_surface {
        let du = &fs.dust;
        let d = draws(c, "weather.dust.v1");
        let year = m::floor(day / c.year_days) as i64;
        for y in [year - 1, year] {
            let i = slot(0, y);
            if d.unit(i) < du.global_per_year {
                let born = (y as f64 + du.season_phase + 0.05 * nrm(&d, i + 1)) * c.year_days;
                let ends = born + 0.15 * c.year_days * (0.7 + 0.6 * d.unit(i + 2));
                if born <= day && day < ends {
                    out.push(WeatherSystem {
                        id: format!("dust-global-{y}"),
                        kind: "dust-storm".into(),
                        lat_deg: -20.0,
                        lon_deg: d.uniform(i + 3, -180.0, 180.0),
                        track: vec![[born, -20.0, 0.0]],
                        size_km: m::PI * c.radius_m / KM,
                        strength: du.optical_depth,
                        born,
                        ends,
                        count: 0,
                        why: "A planet-encircling dust storm in the dust season: dry loose ground, thin air and strong seasonal winds (Zurek and Martin 1993).".into(),
                        evidence: "drawn".into(),
                    });
                }
            }
            let nr = m::round(du.regional_per_year) as u64;
            for j in 0..nr.min(12) {
                let ij = i + 8 + j * 4;
                let born = (y as f64 + d.unit(ij)) * c.year_days;
                let ends = born + 3.0 + 20.0 * d.unit(ij + 1);
                if born <= day && day < ends {
                    let lat0 = d.uniform(ij + 2, -60.0, 60.0);
                    let lon0 = d.uniform(ij + 3, -180.0, 180.0);
                    out.push(WeatherSystem {
                        id: format!("dust-{y}-{j}"),
                        kind: "dust-storm".into(),
                        lat_deg: lat0,
                        lon_deg: lon0,
                        track: track(c, born, ends, lat0, lon0, |_, _| (5.0, 0.0)),
                        size_km: 800.0,
                        strength: du.optical_depth * 0.5,
                        born,
                        ends,
                        count: 0,
                        why: "A regional dust storm: wind over dry, loose ground.".into(),
                        evidence: "drawn".into(),
                    });
                }
            }
        }
    }
    // Star-locked: a permanent storm under the star and twin night-side gyres.
    if regime == "tidally-locked" && p.cond.is_some() {
        out.push(WeatherSystem {
            id: "substellar-storm".into(),
            kind: "substellar-storm".into(),
            lat_deg: 0.0,
            lon_deg: 0.0,
            track: vec![],
            size_km: 0.35 * m::PI * c.radius_m / KM,
            strength: 1.0,
            born: f64::MIN,
            ends: f64::MAX,
            count: 0,
            why: "Under the star the air always rises: a permanent storm and a thick cloud shield that reflects much of the light (Yang, Cowan and Abbot 2013).".into(),
            evidence: "rule".into(),
        });
        for (h, n) in [(1.0, "n"), (-1.0, "s")] {
            out.push(WeatherSystem {
                id: format!("gyre-{n}"),
                kind: "gyre".into(),
                lat_deg: 35.0 * h,
                lon_deg: 180.0,
                track: vec![],
                size_km: 0.2 * m::PI * c.radius_m / KM,
                strength: 0.5,
                born: f64::MIN,
                ends: f64::MAX,
                count: 0,
                why: "Twin standing gyres off the equator, set up by the day-night heating contrast on a locked world (Showman and Polvani 2011).".into(),
                evidence: "rule".into(),
            });
        }
    }
    // A winter polar vortex where there are seasons.
    if matches!(regime, "earth-like" | "fast-rotator" | "thin-air") && c.obliquity_deg > 5.0 {
        let decl = circulation::declination_deg(c, p.phase);
        if decl.abs() > 0.3 * c.obliquity_deg {
            let h = -decl.signum();
            out.push(WeatherSystem {
                id: format!("polar-vortex-{}-{}", if h > 0.0 { "n" } else { "s" }, m::floor(day / c.year_days) as i64),
                kind: "polar-vortex".into(),
                lat_deg: 90.0 * h,
                lon_deg: 0.0,
                track: vec![],
                size_km: 2500.0 * c.radius_m / sse_core::sheet::R_EARTH_MEAN,
                strength: 1.0,
                born: m::floor(day / c.year_days) * c.year_days,
                ends: (m::floor(day / c.year_days) + 1.0) * c.year_days,
                count: 0,
                why: "The winter pole in polar night: a cold vortex with frost, snow and polar-night cloud.".into(),
                evidence: "rule".into(),
            });
        }
    }
    out
}

/// Adds the formations and systems at one place to a zonal cell.
#[allow(clippy::too_many_arguments)]
pub fn imprint(
    fs: &FormationSettings,
    s: &crate::WeatherSettings,
    c: &Conditions,
    p: &Prepared,
    date: Date,
    systems: &[WeatherSystem],
    terrain: Option<&dyn Terrain>,
    cell: &mut Cell,
) {
    let (lat, lon) = (cell.lat_deg, cell.lon_deg);
    let regime = p.circ.regime.as_str();
    let Some(cond) = p.cond.as_ref() else {
        return;
    };
    let substance = cond.substance.clone();
    let mean_rain = p.climate.precip_mean_m_yr * 1000.0 / 365.25;
    let here = dir(lat, lon);
    let (height, wet_ground) = match terrain {
        Some(t) => t.ground(here),
        None => (0.0, c.ocean_fraction >= 0.5),
    };
    let ocean_w = if terrain.is_some() {
        wet_ground as u8 as f64
    } else {
        c.ocean_fraction
    };
    // Locked worlds: cloud shield by day, clear by night.
    if regime == "tidally-locked" {
        let cosz = m::cos(rad(lat)) * m::cos(rad(lon));
        let day_side = cosz.max(0.0);
        let shield = (1.5 * day_side).min(1.0);
        cell.layers[1].cloud_cover = (0.15 + 0.85 * shield).min(1.0);
        cell.layers[1].cloud_type = if shield > 0.5 {
            "cumulonimbus"
        } else {
            "altostratus"
        }
        .into();
        cell.layers[1].condensate = substance.clone();
        cell.layers[1].optical_depth = 2.0 + 25.0 * shield;
        cell.layers[0].cloud_cover *= 0.3 + 0.7 * shield;
        cell.precip_mm_day = mean_rain * 3.0 * shield * shield;
        cell.temp_offset_k = 45.0 * (day_side - 0.32);
    }
    // Marine layer: low deck over the sea under sinking subtropical air,
    // where the water is cold. With a map, that is upwelling water off an
    // eastern-boundary coast (Klein and Hartmann 1993): the trades' flank
    // of the subtropical high blows along the coast toward the equator,
    // Ekman transport carries the surface water offshore and cold water
    // rises behind it. "East" is the spin's: on a backward-spinning world
    // the gyres and the upwelling coasts swap sides.
    let edge = p.circ.hadley_edge_deg;
    if regime != "tidally-locked" && p.climate.precip_mean_m_yr > 0.0 {
        let sink = bump(lat.abs() - (edge - 8.0), 7.0);
        let cold = match terrain {
            Some(t) if wet_ground && sink > 0.01 => {
                upwelling(&fs.currents, c, t, lat, lon, p.circ.spin_sign)
            }
            Some(_) => 0.0,
            None => 1.0,
        };
        if cold > 0.0 {
            cell.temp_offset_k -= fs.currents.upwelling_cooling_k * cold * sink;
        }
        // The same gyre's western boundary current (the Gulf Stream, the
        // Kuroshio) carries warm water poleward along the coast lying to
        // the spin's west, across the gyre's span.
        if let Some(t) = terrain {
            let gyre = bump(lat.abs() - edge, fs.currents.gyre_half_width_deg);
            if wet_ground && gyre > 0.01 {
                let cu = Currents {
                    upwelling_km: fs.currents.boundary_km,
                    ..fs.currents.clone()
                };
                let warm = upwelling(&cu, c, t, lat, lon, -p.circ.spin_sign);
                cell.temp_offset_k += fs.currents.boundary_warming_k * warm * gyre;
            }
        }
        let extra = fs.marine_layer * sink * ocean_w * cold;
        if extra > 0.01 {
            let l = &mut cell.layers[0];
            l.cloud_cover = (l.cloud_cover + extra).min(1.0);
            l.cloud_type = "stratocumulus".into();
            l.condensate = substance.clone();
            l.optical_depth = l.optical_depth.max(6.0);
            if cell.precip_mm_day < 0.5 {
                cell.precip_kind = "drizzle".into();
            }
        }
    }
    // Mountains: cloud banked on the windward side, dry behind, caps on peaks.
    if let Some(t) = terrain {
        if !wet_ground {
            let u = circulation::zonal_wind(s, &p.circ, lat);
            let step = fs.orography.sample_km * KM;
            let dlon = step / (c.radius_m * m::cos(rad(lat)).max(0.05)) * 180.0 / m::PI;
            let up = if u >= 0.0 { lon - dlon } else { lon + dlon };
            let (h_up, _) = t.ground(dir(lat, up));
            let slope = (height.max(0.0) - h_up.max(0.0)) / step;
            let w = u.abs() * slope;
            if w > 0.0 {
                let f = (fs.orography.rain_per_m_s * w).min(4.0);
                cell.precip_mm_day += mean_rain * f;
                let l = &mut cell.layers[0];
                l.cloud_cover = (l.cloud_cover + 0.5 * f.min(1.0)).min(1.0);
                l.cloud_type = "nimbostratus".into();
                l.condensate = substance.clone();
            } else if w < 0.0 {
                let dry = (1.0 + fs.orography.rain_per_m_s * (-w)).recip();
                cell.precip_mm_day *= dry;
                cell.layers[0].cloud_cover *= dry;
            }
            if height > fs.orography.cap_cloud_m && p.climate.precip_mean_m_yr > 0.0 {
                let l = &mut cell.layers[1];
                l.cloud_cover = l.cloud_cover.max(0.6);
                l.cloud_type = "cap".into();
                l.condensate = substance.clone();
            }
        }
    }
    // Systems.
    for sys in systems {
        let (sl, so) = position_at(sys, date.day);
        let r = sys.size_km.max(1.0);
        let dkm = distance_km(c, (lat, lon), (sl, so));
        if dkm > 1.5 * r {
            continue;
        }
        let x = dkm / r;
        let core = bump(x, 0.45);
        match sys.kind.as_str() {
            "cyclone" => {
                // The comma: a cloud shield east and poleward of the low.
                let bearing = m::atan2(lat - sl, wrap(lon - so));
                let comma = 0.5 + 0.5 * m::cos(bearing - 0.6 * sl.signum());
                let k = core * (0.4 + 0.6 * comma);
                let l = &mut cell.layers[1];
                l.cloud_cover = (l.cloud_cover + k).min(1.0);
                l.cloud_type = "nimbostratus".into();
                l.condensate = substance.clone();
                l.optical_depth = l.optical_depth.max(10.0 * k);
                cell.precip_mm_day += 8.0 * k * (mean_rain / 2.77).max(0.05);
                spin(cell, lat, sl, so, lon, sys.strength * core, c);
            }
            "tropical-cyclone" => {
                let over_land = terrain.is_some() && !wet_ground;
                let k = core * if over_land { 0.4 } else { 1.0 };
                for l in cell.layers.iter_mut() {
                    l.cloud_cover = (l.cloud_cover + k).min(1.0);
                    l.condensate = substance.clone();
                    l.turbulence = l.turbulence.max(k);
                }
                cell.layers[1].cloud_type = "cumulonimbus".into();
                cell.layers[1].optical_depth = 40.0 * k;
                cell.precip_mm_day += 60.0 * k;
                cell.lightning_km2_yr += 20.0 * k;
                spin(cell, lat, sl, so, lon, sys.strength * k, c);
            }
            "dust-storm" => {
                cell.aerosols.push(Aerosol {
                    kind: "dust".into(),
                    optical_depth: sys.strength * bump(x, 0.6),
                    colour: [0.75, 0.5, 0.3],
                });
                cell.layers[0].turbulence = cell.layers[0].turbulence.max(0.5 * core);
            }
            "substellar-storm" => {
                cell.lightning_km2_yr += 10.0 * core;
                cell.layers[1].turbulence = cell.layers[1].turbulence.max(0.6 * core);
            }
            "polar-vortex" => {
                cell.temp_offset_k -= 8.0 * core;
                let l = &mut cell.layers[2];
                l.cloud_cover = l.cloud_cover.max(0.4 * core);
                if l.cloud_type == "none" {
                    l.cloud_type = "cirrus".into();
                    l.condensate = substance.clone();
                }
            }
            _ => {}
        }
    }
    // Lightning where convection is deep (Price and Rind 1992).
    let ordered_clear = orders_here(c, systems, date, lat, lon, "order-clear");
    let deep = cell.layers[1].cloud_type == "cumulonimbus";
    if deep {
        let top_km = s.climate.circulation.tropopause_scale_heights * c.scale_height_m() / KM * 0.9;
        let ab = if ocean_w > 0.5 {
            fs.lightning.ocean
        } else {
            fs.lightning.land
        };
        let per_min = ab[0] * m::pow(top_km, ab[1]);
        let cells = fs.lightning.storms_per_1e4_km2 * cell.layers[1].cloud_cover / 1.0e4;
        cell.lightning_km2_yr += per_min * 60.0 * 24.0 * 365.25 * cells;
    }
    // GM orders win over everything drawn (authored, marked as such).
    if ordered_clear {
        for l in cell.layers.iter_mut() {
            l.cloud_cover = 0.0;
            l.cloud_type = "none".into();
            l.condensate.clear();
            l.optical_depth = 0.0;
        }
        cell.precip_mm_day = 0.0;
        cell.precip_kind = "none".into();
        cell.lightning_km2_yr = 0.0;
    }
    if orders_here(c, systems, date, lat, lon, "order-calm") {
        for l in cell.layers.iter_mut() {
            // Light air at most: 1 m/s.
            let v = m::sqrt(l.wind_m_s[0] * l.wind_m_s[0] + l.wind_m_s[1] * l.wind_m_s[1]);
            let k = if v > 1.0 { 1.0 / v } else { 1.0 };
            l.wind_m_s = [k * l.wind_m_s[0], k * l.wind_m_s[1]];
            l.turbulence = 0.0;
        }
    }
    // What falls: snow where it is freezing.
    if cell.precip_mm_day > 0.0 && c.liquid.as_deref() == Some("water") {
        let t = t_at(p, c, lat, p.phase) + cell.temp_offset_k - 0.0065 * height.max(0.0);
        cell.precip_kind = if t < sse_core::sheet::T0C {
            "snow"
        } else if cell.precip_kind == "drizzle" {
            "drizzle"
        } else {
            "rain"
        }
        .into();
    }
}

/// Cyclonic wind round a low: anticlockwise in the north, clockwise in the south.
fn spin(cell: &mut Cell, lat: f64, sl: f64, so: f64, lon: f64, v: f64, c: &Conditions) {
    let east = wrap(lon - so) * m::cos(rad(lat));
    let north = lat - sl;
    let r = m::sqrt(east * east + north * north).max(1e-6);
    let h = if sl >= 0.0 { 1.0 } else { -1.0 };
    let _ = c;
    for l in cell.layers.iter_mut() {
        l.wind_m_s[0] += -h * v * north / r;
        l.wind_m_s[1] += h * v * east / r;
    }
}

/// A plain report for a place and date.
pub fn report(
    fs: &FormationSettings,
    cell: &Cell,
    c: &Conditions,
    systems: &[WeatherSystem],
    day: f64,
    provenance: Provenance,
) -> Report {
    let l0 = &cell.layers[0];
    let w = l0.wind_m_s;
    let speed = m::sqrt(w[0] * w[0] + w[1] * w[1]);
    // Direction the wind comes from, clockwise from north.
    let from = wrap(m::atan2(-w[0], -w[1]) * 180.0 / m::PI);
    let from = if from < 0.0 { from + 360.0 } else { from };
    let tau: f64 = cell.aerosols.iter().map(|a| a.optical_depth).sum();
    let fog = if l0.cloud_type == "stratocumulus" && l0.cloud_cover > 0.9 {
        4.0
    } else {
        1.0
    };
    let vis = fs.clear_visibility_km / (1.0 + 8.0 * tau + cell.precip_mm_day / 3.0) / fog;
    let cover = cell
        .layers
        .iter()
        .map(|l| l.cloud_cover)
        .fold(0.0, f64::max);
    let mut nearby: Vec<(String, f64)> = systems
        .iter()
        .map(|s| {
            let p = position_at(s, day);
            (
                s.id.clone(),
                distance_km(c, (cell.lat_deg, cell.lon_deg), p),
            )
        })
        .filter(|(_, d)| *d < 3000.0)
        .collect();
    nearby.sort_by(|a, b| a.1.total_cmp(&b.1).then(a.0.cmp(&b.0)));
    let compass = [
        "north",
        "north-east",
        "east",
        "south-east",
        "south",
        "south-west",
        "west",
        "north-west",
    ];
    let ci = (m::round(from / 45.0) as usize) % 8;
    let mut lines = vec![format!(
        "Wind {:.0} km/h from the {}",
        speed * 3.6,
        compass[ci]
    )];
    let sky = match cover {
        x if x < 0.1 => "clear",
        x if x < 0.4 => "partly cloudy",
        x if x < 0.8 => "mostly cloudy",
        _ => "overcast",
    };
    lines.push(format!("Sky {sky}"));
    if cell.precip_mm_day > 0.2 {
        lines.push(format!(
            "{} {:.0} mm a day",
            capital(&cell.precip_kind),
            cell.precip_mm_day
        ));
    }
    for a in &cell.aerosols {
        if a.optical_depth > 0.3 {
            lines.push(format!("{} in the air", capital(&a.kind)));
        }
    }
    lines.push(format!("Visibility {:.0} km", vis.max(0.1)));
    if cell.lightning_km2_yr > 5.0 {
        lines.push("Thunderstorms about".into());
    }
    // Approaching storms in the next day.
    for s in systems {
        if s.track.len() < 2 || s.kind == "dust-storm" && s.size_km > 5000.0 {
            continue;
        }
        let now = distance_km(c, (cell.lat_deg, cell.lon_deg), position_at(s, day));
        if now <= s.size_km {
            continue;
        }
        for h in [6.0, 12.0, 18.0, 24.0] {
            let p = position_at(s, day + h / 24.0);
            if distance_km(c, (cell.lat_deg, cell.lon_deg), p) <= s.size_km * 0.8 {
                let what = if s.kind == "tropical-cyclone" {
                    "Tropical cyclone"
                } else {
                    "Storm front"
                };
                lines.push(format!("{what} in about {h:.0} hours"));
                break;
            }
        }
    }
    Report {
        lat_deg: cell.lat_deg,
        lon_deg: cell.lon_deg,
        day,
        lines,
        wind_km_h: speed * 3.6,
        wind_from_deg: from,
        visibility_km: vis.max(0.1),
        cloud_cover: cover,
        precip_mm_day: cell.precip_mm_day,
        nearby,
        provenance,
    }
}

fn capital(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

fn orders_here(
    c: &Conditions,
    systems: &[WeatherSystem],
    date: Date,
    lat: f64,
    lon: f64,
    kind: &str,
) -> bool {
    systems.iter().any(|s| {
        s.kind == kind
            && s.born <= date.day
            && date.day < s.ends
            && distance_km(c, (lat, lon), (s.lat_deg, s.lon_deg)) <= s.size_km / 2.0
    })
}

//! W4: smoke, ash, haze, smog and spray tied to their sources, and the
//! climate effect of large veils (weather design section 4, stage W4).
//!
//! Sources are inputs ([`Sources`]): the volcanism, chronicle, life and built
//! layers that will publish them (S7, S12, L1c) are not built yet, so the
//! caller passes them. Nothing here draws: the same conditions, date and
//! sources give the same answer on every device.
//!
//! - Plume height on Earth from Mastin et al. (2009): H = 2.00 V^0.241 km
//!   above the vent, V the dense-rock-equivalent rate (m³/s).
//! - Other worlds: buoyant plume rise H ∝ F^¼ N^-¾ (Morton, Taylor and Turner
//!   1956), F ∝ g MER / ρ, N² = g² / (c_p T) for an isothermal stratosphere.
//!   Earth's constant is carried over by ratio: a stated fudge.
//! - Thin or no air: ballistic, H = v² / 2g (vent speeds from McEwen and
//!   Soderblom 1983).
//! - Sulphate veil: SO₂ becomes sulphate, which e-folds over about a year;
//!   cooling peaks about a year after the eruption (Robock 2000).
//!
//! Every plume is a [`WeatherSystem`] of kind `plume` (strength = height km).
//! In-air aerosols go into the cell; ash on the ground and acid rain are in
//! [`AerosolOut`] (the cell contract is fixed).

use crate::circulation::{self, bump, Circulation};
use crate::conditions::Conditions;
use crate::contract::{Aerosol, Cell, Date, WeatherSystem};
use crate::formations::distance_km;
use crate::settings::{Photochemical, Reference, WeatherSettings};
use serde::{Deserialize, Serialize};
use sse_core::math as m;

const R_GAS: f64 = sse_core::sheet::R_GAS;
const DAY_S: f64 = sse_core::sheet::DAY;
const YEAR_DAYS: f64 = sse_core::sheet::YR_JULIAN / sse_core::sheet::DAY;
const TG: f64 = 1.0e9;
const M_SO2: f64 = sse_core::sheet::molar_mass::SO2;

// ---------------------------------------------------------------- settings

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Mastin {
    pub coef_km: f64,
    pub exponent: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct EarthRef {
    pub gravity_m_s2: f64,
    pub molar_kg: f64,
    pub t_strat_k: f64,
    pub density_kg_m3: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Ballistic {
    /// Long-lived (Prometheus-type) vents, m/s.
    pub steady_speed_m_s: f64,
    /// Short (Pele-type) eruptions, m/s.
    pub eruption_speed_m_s: f64,
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Ash {
    pub settle_m_s: f64,
    /// Sideways growth of the streak's half-width per km downwind.
    pub spread_per_km: f64,
    /// Umbrella diameter per km of height.
    pub umbrella_per_height: f64,
    pub tau_ref: f64,
    pub mer_ref_kg_s: f64,
    pub tau_max: f64,
    pub deposit_kg_m3: f64,
    pub linger_days: f64,
    /// Ash depth that darkens the ground to 1 − 1/e.
    pub darkening_mm: f64,
    pub min_streak_km: f64,
    /// Least half-width of a plume at the vent, km.
    pub min_width_km: f64,
    /// Eruption rate when an event gives neither rate nor volume, kg/s.
    pub default_mer_kg_s: f64,
    pub colour: [f64; 3],
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Vog {
    pub tau_per_kg_s: f64,
    pub tau_max: f64,
    /// Haze depth above which rain falling through it is acid.
    pub acid_tau: f64,
    pub length_km: f64,
    pub colour: [f64; 3],
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Veil {
    pub conversion_years: f64,
    pub residence_years: f64,
    pub response_years: f64,
    pub cooling_k_per_tg_so2: f64,
    pub max_cooling_k: f64,
    pub tau_per_tg_so2: f64,
    /// Eruptions this close to the equator veil the whole world.
    pub tropics_deg: f64,
    /// Share of the veil a high-latitude eruption gives the other hemisphere.
    pub other_hemisphere: f64,
    pub colour: [f64; 3],
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct ImpactSettings {
    pub density_kg_m3: f64,
    pub speed_m_s: f64,
    pub fireball_j: f64,
    pub regional_j: f64,
    pub winter_j: f64,
    pub plume_km_at_fireball: f64,
    pub fireball_days: f64,
    pub regional_radius_km: f64,
    pub regional_tau: f64,
    pub regional_days: f64,
    pub winter_tau: f64,
    pub winter_k: f64,
    pub winter_k_per_decade: f64,
    pub max_cooling_k: f64,
    pub rise_years: f64,
    pub residence_years: f64,
    pub response_years: f64,
    pub dust_colour: [f64; 3],
    pub soot_colour: [f64; 3],
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct FireSettings {
    pub min_o2: f64,
    pub dry_ratio: f64,
    pub plume_km_ref: f64,
    pub area_ref_km2: f64,
    pub tau_ref: f64,
    pub tau_max: f64,
    pub settle_m_s: f64,
    pub colour: [f64; 3],
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Smog {
    pub tau_ref: f64,
    pub radius_km_ref: f64,
    pub inversion_boost: f64,
    pub inversion_width_deg: f64,
    pub basin_boost: f64,
    pub colour: [f64; 3],
    pub source: String,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Spray {
    pub u_min_m_s: f64,
    pub tau_per_m_s: f64,
    pub tau_max: f64,
    pub colour: [f64; 3],
    pub source: String,
}

/// The pack key `provides.other.weather_aerosols`.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct AerosolSettings {
    pub thin_air_bar: f64,
    pub thin_air_source: String,
    pub magma_dre_kg_m3: f64,
    pub mastin: Mastin,
    pub earth: EarthRef,
    pub scaling_source: String,
    pub ballistic: Ballistic,
    pub ash: Ash,
    pub vog: Vog,
    pub veil: Veil,
    pub impact: ImpactSettings,
    pub fire: FireSettings,
    pub smog: Smog,
    pub spray: Spray,
    /// Hazes beyond the main list (`rules::haze`), with outgassed gas counted in.
    pub photochemical: Vec<Photochemical>,
    /// Outgassed gas stays in the air this long before it is removed, years.
    pub outgassing_mix_years: f64,
    /// Aerosols thinner than this are left out.
    pub tau_min: f64,
    pub references: Vec<Reference>,
}

impl AerosolSettings {
    pub fn from_packs(stack: &[sse_pack::PackManifest]) -> Result<Self, String> {
        let v = stack
            .iter()
            .rev()
            .find_map(|p| p.provides.other.get("weather_aerosols"))
            .ok_or("no pack provides weather aerosols")?;
        let s: AerosolSettings = sse_reference::refs::with_sheet(stack, v)
            .and_then(|j| serde_json::from_value(j).map_err(|e| e.to_string()))
            .map_err(|e| format!("weather aerosols: {e}"))?;
        if s.references
            .iter()
            .any(|r| r.doi.is_none() && r.url.is_none())
        {
            return Err("every weather reference needs a DOI or a URL".into());
        }
        let distinct = |a: f64, b: f64, c: f64| a != b && b != c && a != c;
        if !distinct(
            s.veil.conversion_years,
            s.veil.residence_years,
            s.veil.response_years,
        ) || !distinct(
            s.impact.rise_years,
            s.impact.residence_years,
            s.impact.response_years,
        ) {
            return Err("veil time constants must differ from one another".into());
        }
        Ok(s)
    }
}

// ----------------------------------------------------------------- sources

/// A volcano that is always degassing.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Volcano {
    pub id: String,
    pub lat_deg: f64,
    pub lon_deg: f64,
    /// Vent above the surrounding ground, m.
    #[serde(default)]
    pub vent_height_m: f64,
    /// Steady outgassing (mostly SO₂), kg/s.
    pub outgassing_kg_s: f64,
    /// Vent speed in thin air, m/s (default: the pack's Prometheus-type speed).
    #[serde(default)]
    pub vent_speed_m_s: Option<f64>,
}

/// A dated eruption.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Eruption {
    pub id: String,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub day: f64,
    pub duration_days: f64,
    #[serde(default)]
    pub vent_height_m: f64,
    /// Mass eruption rate, kg/s.
    #[serde(default)]
    pub mer_kg_s: Option<f64>,
    /// Or the erupted volume, dense-rock equivalent, m³.
    #[serde(default)]
    pub volume_m3: Option<f64>,
    /// Sulphur dioxide put into the air, kg.
    #[serde(default)]
    pub so2_kg: f64,
    /// Vent speed in thin air, m/s (default: the pack's Pele-type speed).
    #[serde(default)]
    pub vent_speed_m_s: Option<f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Impact {
    pub id: String,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub day: f64,
    #[serde(default)]
    pub energy_j: Option<f64>,
    /// Or the impactor's diameter, m, with the pack's density and speed.
    #[serde(default)]
    pub diameter_m: Option<f64>,
    #[serde(default)]
    pub speed_m_s: Option<f64>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Fire {
    pub id: String,
    pub lat_deg: f64,
    pub lon_deg: f64,
    pub area_km2: f64,
    pub day_start: f64,
    pub day_end: f64,
}

/// A settlement or industrial centre.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Industry {
    pub id: String,
    pub lat_deg: f64,
    pub lon_deg: f64,
    /// 0 (none) to 1 (clean, post-industrial).
    pub tech: f64,
    pub population: f64,
    /// In a basin that traps air.
    #[serde(default)]
    pub basin: bool,
}

/// Everything that puts aerosol in the air, passed by the caller until the
/// volcanism (S7), chronicle (S12) and built (L1c) layers publish them.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct Sources {
    #[serde(default)]
    pub volcanoes: Vec<Volcano>,
    #[serde(default)]
    pub eruptions: Vec<Eruption>,
    #[serde(default)]
    pub impacts: Vec<Impact>,
    #[serde(default)]
    pub fires: Vec<Fire>,
    #[serde(default)]
    pub industry: Vec<Industry>,
}

// ----------------------------------------------------------------- outputs

/// What the cell contract has no room for.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub struct AerosolOut {
    /// Ash fallen on the ground so far, mm.
    pub ash_fall_mm: f64,
    /// How much the ash darkens ground and ice, 0 (not) to 1 (black).
    pub ground_darkening: f64,
    /// Rain here falls through volcanic sulphate haze.
    pub acid_rain: bool,
}

// ----------------------------------------------------------------- physics

/// Mastin et al. (2009): Earth plume height above the vent, km, for a
/// dense-rock-equivalent rate in m³/s.
pub fn mastin_height_km(a: &AerosolSettings, v_dre_m3_s: f64) -> f64 {
    a.mastin.coef_km * m::pow(v_dre_m3_s.max(0.0), a.mastin.exponent)
}

/// Mastin's mass form, km, for a mass eruption rate in kg/s.
pub fn mastin_height_mer_km(a: &AerosolSettings, mer_kg_s: f64) -> f64 {
    mastin_height_km(a, mer_kg_s / a.magma_dre_kg_m3)
}

/// Brunt-Väisälä frequency of an isothermal stratosphere, s⁻¹.
pub fn n_strat(g: f64, molar_kg: f64, t_k: f64) -> f64 {
    let cp = 3.5 * R_GAS / molar_kg;
    g / m::sqrt(cp * t_k)
}

/// Height ratio of a buoyant plume on this world to the same plume on Earth.
pub fn atmosphere_factor(a: &AerosolSettings, c: &Conditions) -> f64 {
    let e = &a.earth;
    let n_e = n_strat(e.gravity_m_s2, e.molar_kg, e.t_strat_k);
    let t_strat = c.t_eff_k() * m::pow(0.5, 0.25);
    let n = n_strat(c.gravity_m_s2, c.mean_molar_kg, t_strat);
    let rho = c.pressure_bar * 1.0e5 * c.mean_molar_kg / (R_GAS * c.t_surface_k);
    m::pow(c.gravity_m_s2 / e.gravity_m_s2, 0.25)
        * m::pow(e.density_kg_m3 / rho, 0.25)
        * m::pow(n_e / n, 0.75)
}

/// Ballistic plume height, km.
pub fn ballistic_height_km(speed_m_s: f64, g: f64) -> f64 {
    speed_m_s * speed_m_s / (2.0 * g) / 1000.0
}

fn thin(a: &AerosolSettings, c: &Conditions) -> bool {
    c.pressure_bar < a.thin_air_bar
}

/// Plume height above the vent, km, and whether it is ballistic.
pub fn plume_height_km(
    a: &AerosolSettings,
    c: &Conditions,
    mer_kg_s: f64,
    speed: f64,
) -> (f64, bool) {
    if thin(a, c) {
        (ballistic_height_km(speed, c.gravity_m_s2), true)
    } else {
        (
            mastin_height_mer_km(a, mer_kg_s) * atmosphere_factor(a, c),
            false,
        )
    }
}

/// A first-order chain source → load → response: the response to a unit
/// pulse at t = 0 decaying at rate 1/τc into a load removed at 1/τd, felt
/// with lag τr. Unnormalised; all τ distinct.
fn chain(t: f64, tc: f64, td: f64, tr: f64) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    let (a, b, c) = (1.0 / tc, 1.0 / td, 1.0 / tr);
    c * a / (a - b)
        * ((m::exp(-b * t) - m::exp(-c * t)) / (c - b)
            - (m::exp(-a * t) - m::exp(-c * t)) / (c - a))
}

/// The load itself (source → load), unnormalised.
fn load(t: f64, tc: f64, td: f64) -> f64 {
    if t <= 0.0 {
        return 0.0;
    }
    let (a, b) = (1.0 / tc, 1.0 / td);
    a / (a - b) * (m::exp(-b * t) - m::exp(-a * t))
}

/// Peak of a single-humped curve over [0, 12 τd]: coarse samples, then a
/// fixed number of ternary steps (deterministic).
fn peak(f: impl Fn(f64) -> f64, td: f64) -> f64 {
    let n = 240;
    let dt = 12.0 * td / n as f64;
    let mut best = 1;
    for i in 1..=n {
        if f(dt * i as f64) > f(dt * best as f64) {
            best = i;
        }
    }
    let (mut lo, mut hi) = (dt * (best - 1) as f64, dt * (best + 1) as f64);
    for _ in 0..60 {
        let (p, q) = (lo + (hi - lo) / 3.0, hi - (hi - lo) / 3.0);
        if f(p) < f(q) {
            lo = p;
        } else {
            hi = q;
        }
    }
    f(0.5 * (lo + hi)).max(f(dt * best as f64))
}

/// Normalised curves of a veil: (optical depth share, cooling share), 0 to 1.
fn veil_curves(t_years: f64, tc: f64, td: f64, tr: f64) -> (f64, f64) {
    let lp = peak(|t| load(t, tc, td), td).max(1e-30);
    let cp = peak(|t| chain(t, tc, td, tr), td.max(tr)).max(1e-30);
    (load(t_years, tc, td) / lp, chain(t_years, tc, td, tr) / cp)
}

fn eruption_mer(a: &AerosolSettings, e: &Eruption) -> f64 {
    e.mer_kg_s
        .or_else(|| {
            e.volume_m3
                .map(|v| v * a.magma_dre_kg_m3 / (e.duration_days.max(1e-6) * DAY_S))
        })
        .unwrap_or(a.ash.default_mer_kg_s)
}

/// Impact energy, J.
pub fn impact_energy_j(a: &AerosolSettings, i: &Impact) -> f64 {
    i.energy_j.unwrap_or_else(|| {
        let d = i.diameter_m.unwrap_or(0.0);
        let v = i.speed_m_s.unwrap_or(a.impact.speed_m_s);
        let mass = a.impact.density_kg_m3 * m::PI / 6.0 * d * d * d;
        0.5 * mass * v * v
    })
}

fn hemi(a: &AerosolSettings, src_lat: f64, lat: f64) -> f64 {
    if src_lat.abs() < a.veil.tropics_deg || (src_lat >= 0.0) == (lat >= 0.0) {
        1.0
    } else {
        a.veil.other_hemisphere
    }
}

fn tropopause_m(w: &WeatherSettings, c: &Conditions) -> f64 {
    w.climate.circulation.tropopause_scale_heights * c.scale_height_m()
}

/// A sulphate veil from one eruption: (source latitude, Tg SO₂, years since).
fn veils(
    a: &AerosolSettings,
    w: &WeatherSettings,
    c: &Conditions,
    date: Date,
    src: &Sources,
) -> Vec<(f64, f64, f64)> {
    let mut out = vec![];
    if thin(a, c) {
        return out;
    }
    let trop = tropopause_m(w, c);
    for e in &src.eruptions {
        if e.so2_kg <= 0.0 || date.day < e.day {
            continue;
        }
        let (h, _) = plume_height_km(a, c, eruption_mer(a, e), 0.0);
        if h * 1000.0 + e.vent_height_m <= trop {
            continue; // tropospheric sulphate rains out within weeks
        }
        out.push((e.lat_deg, e.so2_kg / TG, (date.day - e.day) / YEAR_DAYS));
    }
    out
}

/// Cooling of the climate at a latitude from large veils (sulphate after
/// eruptions that reach the stratosphere, soot and dust after big impacts),
/// K, negative. Add it to the cell's `temp_offset_k`.
pub fn veil_cooling_k(
    a: &AerosolSettings,
    w: &WeatherSettings,
    c: &Conditions,
    date: Date,
    sources: &Sources,
    lat_deg: f64,
) -> f64 {
    let v = &a.veil;
    let mut k = 0.0;
    for (src_lat, tg, t) in veils(a, w, c, date, sources) {
        let (_, cool) = veil_curves(t, v.conversion_years, v.residence_years, v.response_years);
        let peak_k = (v.cooling_k_per_tg_so2 * tg).min(v.max_cooling_k);
        k -= peak_k * cool * hemi(a, src_lat, lat_deg);
    }
    let im = &a.impact;
    if !thin(a, c) {
        for i in &sources.impacts {
            let e = impact_energy_j(a, i);
            if e < im.winter_j || date.day < i.day {
                continue;
            }
            let t = (date.day - i.day) / YEAR_DAYS;
            let (_, cool) = veil_curves(t, im.rise_years, im.residence_years, im.response_years);
            let peak_k = (im.winter_k + im.winter_k_per_decade * m::log10(e / im.winter_j))
                .min(im.max_cooling_k);
            k -= peak_k * cool;
        }
    }
    k
}

// ------------------------------------------------------------------ plumes

/// One active plume and its streak.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Plume {
    pub id: String,
    pub lat_deg: f64,
    pub lon_deg: f64,
    /// Above the vent, km.
    pub height_km: f64,
    pub ballistic: bool,
    pub born: f64,
    pub ends: f64,
    /// `ash`, `smoke`, `dust` or `haze`.
    pub aerosol: String,
    pub colour: [f64; 3],
    /// Optical depth at the vent now.
    pub tau0: f64,
    /// Eastward wind carrying it, m/s (0: an umbrella with no streak).
    pub wind_m_s: f64,
    pub length_km: f64,
    /// Half-width of the umbrella, km.
    pub sigma0_km: f64,
    pub spread_per_km: f64,
    /// Ash fallen so far, kg (0 for smoke and haze).
    pub deposited_kg: f64,
    pub why: String,
}

impl Plume {
    /// Streak shape at a place, 1 at the vent: falls off along the wind
    /// (e-folding over `length_km`), spreads sideways, and thins as it widens.
    fn shape(&self, c: &Conditions, lat: f64, lon: f64) -> f64 {
        let s0 = self.sigma0_km.max(1.0);
        if self.wind_m_s.abs() < 0.5 {
            let r = distance_km(c, (self.lat_deg, self.lon_deg), (lat, lon)) / s0;
            return m::exp(-0.5 * r * r);
        }
        let km_deg = c.radius_m / 1000.0 * m::PI / 180.0;
        let mut dl = lon - self.lon_deg;
        dl -= 360.0 * m::round(dl / 360.0);
        let x = dl * m::cos(self.lat_deg * m::PI / 180.0) * km_deg * self.wind_m_s.signum();
        let y = (lat - self.lat_deg) * km_deg;
        if x < 0.0 {
            return m::exp(-0.5 * ((x / s0) * (x / s0) + (y / s0) * (y / s0)));
        }
        let s = s0 + self.spread_per_km * x;
        m::exp(-x / self.length_km) * (s0 / s) * m::exp(-0.5 * (y / s) * (y / s))
    }

    /// Area under the shape, km² (so the deposit holds the erupted mass).
    fn area_km2(&self) -> f64 {
        let s0 = self.sigma0_km.max(1.0);
        if self.wind_m_s.abs() < 0.5 {
            2.0 * m::PI * s0 * s0
        } else {
            m::sqrt(2.0 * m::PI) * s0 * self.length_km + m::PI * s0 * s0
        }
    }

    pub fn system(&self) -> WeatherSystem {
        WeatherSystem {
            id: format!("plume-{}", self.id),
            kind: "plume".into(),
            lat_deg: self.lat_deg,
            lon_deg: self.lon_deg,
            track: if self.born == f64::MIN {
                vec![]
            } else {
                vec![[self.born, self.lat_deg, self.lon_deg]]
            },
            size_km: 2.0 * self.sigma0_km,
            strength: self.height_km,
            born: self.born,
            ends: self.ends,
            count: 0,
            why: self.why.clone(),
            evidence: "rule".into(),
        }
    }
}

/// Everything about the aerosols at one date, worked out once.
pub struct Prepared {
    pub circ: Circulation,
    pub plumes: Vec<Plume>,
    /// Sulphate veils: (source latitude, optical depth at full share).
    pub veils: Vec<(f64, f64)>,
    /// Impact veils: (lat, lon, radius km or 0 for world-wide, kind, tau, colour).
    pub impact_veils: Vec<(f64, f64, f64, String, f64, [f64; 3])>,
    /// Industry with its smog depth at the centre and radius, km.
    pub smog: Vec<(f64, f64, f64, f64)>,
    pub haze: Vec<Aerosol>,
}

#[allow(clippy::too_many_arguments)]
fn streak_plume(
    a: &AerosolSettings,
    w: &WeatherSettings,
    c: &Conditions,
    circ: &Circulation,
    id: String,
    lat: f64,
    lon: f64,
    height_km: f64,
    vent_m: f64,
    ballistic: bool,
    settle: f64,
) -> Plume {
    let top = height_km * 1000.0 + vent_m;
    let u = if ballistic || circ.regime == "airless" {
        0.0
    } else {
        let h = c.scale_height_m().max(1.0);
        circulation::zonal_wind(w, circ, lat) * (1.0 + 1.5 * (top / h).min(2.0))
    };
    Plume {
        id,
        lat_deg: lat,
        lon_deg: lon,
        height_km,
        ballistic,
        born: f64::MIN,
        ends: f64::MAX,
        aerosol: String::new(),
        colour: [0.0; 3],
        tau0: 0.0,
        wind_m_s: u,
        length_km: (u.abs() * top / settle / 1000.0).max(a.ash.min_streak_km),
        sigma0_km: (0.5 * a.ash.umbrella_per_height * height_km).max(a.ash.min_width_km),
        spread_per_km: a.ash.spread_per_km,
        deposited_kg: 0.0,
        why: String::new(),
    }
}

/// Is it the dry part of the year at this latitude?
fn dry_season(
    a: &AerosolSettings,
    w: &WeatherSettings,
    c: &Conditions,
    circ: &Circulation,
    date: Date,
    lat: f64,
) -> bool {
    if c.liquid.is_none() {
        return true;
    }
    let ns = w.season_samples.max(1);
    let mut mean = 0.0;
    for k in 0..ns {
        let ph = (k as f64 + 0.5) / ns as f64;
        mean +=
            circulation::wetness(w, circ, lat, circulation::itcz_deg(w, c, circ, ph)) / ns as f64;
    }
    let now = circulation::wetness(
        w,
        circ,
        lat,
        circulation::itcz_deg(w, c, circ, crate::rules::year_phase(c, date)),
    );
    now < a.fire.dry_ratio * mean
}

/// Whether a fire burns, and why not when it does not.
pub fn fire_burns(
    a: &AerosolSettings,
    w: &WeatherSettings,
    c: &Conditions,
    circ: &Circulation,
    date: Date,
    f: &Fire,
) -> Result<(), String> {
    if date.day < f.day_start || date.day >= f.day_end {
        return Err("not burning on this date".into());
    }
    if !c.solid_surface || thin(a, c) {
        return Err("no ground or no air to burn in".into());
    }
    let o2 = c.gases.get("o2").copied().unwrap_or(0.0);
    if o2 < a.fire.min_o2 {
        return Err(format!(
            "too little oxygen to burn ({:.0} % against {:.0} %)",
            100.0 * o2,
            100.0 * a.fire.min_o2
        ));
    }
    if !dry_season(a, w, c, circ, date, f.lat_deg) {
        return Err("not the dry season here".into());
    }
    Ok(())
}

/// Work out every plume, veil and smog dome alive on this date.
pub fn prepare(
    a: &AerosolSettings,
    w: &WeatherSettings,
    c: &Conditions,
    date: Date,
    src: &Sources,
) -> Prepared {
    let circ = circulation::circulation(w, c);
    let mut plumes = vec![];
    let fac = if thin(a, c) {
        1.0
    } else {
        atmosphere_factor(a, c)
    };
    // Steady volcanoes: a degassing plume and a vog streak (no ash).
    for v in &src.volcanoes {
        let speed = v.vent_speed_m_s.unwrap_or(a.ballistic.steady_speed_m_s);
        let (h, bal) = plume_height_km(a, c, v.outgassing_kg_s, speed);
        let mut p = streak_plume(
            a,
            w,
            c,
            &circ,
            v.id.clone(),
            v.lat_deg,
            v.lon_deg,
            h,
            v.vent_height_m,
            bal,
            a.ash.settle_m_s,
        );
        if !bal {
            p.length_km = p.length_km.max(a.vog.length_km);
        }
        p.aerosol = "haze".into();
        p.colour = a.vog.colour;
        p.tau0 = (a.vog.tau_per_kg_s * v.outgassing_kg_s).min(a.vog.tau_max);
        p.why = if bal {
            format!("A {h:.0} km ballistic plume: gas leaves the vent at {speed:.0} m/s into almost no air, height v²/2g (McEwen and Soderblom 1983 speeds).")
        } else {
            format!("A {h:.1} km degassing plume from {:.0} kg/s of gas (Mastin et al. 2009 applied to the gas rate, a stand-in), trailing a sulphate haze downwind.", v.outgassing_kg_s)
        };
        plumes.push(p);
    }
    // Dated eruptions: an ash column, an ash streak and ash fall. Kept after
    // the ash clears from the air, for the ash on the ground.
    for e in &src.eruptions {
        let ends = e.day + e.duration_days;
        if date.day < e.day {
            continue;
        }
        let mer = eruption_mer(a, e);
        let speed = e.vent_speed_m_s.unwrap_or(a.ballistic.eruption_speed_m_s);
        let (h, bal) = plume_height_km(a, c, mer, speed);
        let mut p = streak_plume(
            a,
            w,
            c,
            &circ,
            e.id.clone(),
            e.lat_deg,
            e.lon_deg,
            h,
            e.vent_height_m,
            bal,
            a.ash.settle_m_s,
        );
        p.born = e.day;
        p.ends = ends;
        p.aerosol = "ash".into();
        p.colour = a.ash.colour;
        let fade = if date.day < ends {
            1.0
        } else {
            m::exp(-(date.day - ends) / a.ash.linger_days)
        };
        p.tau0 = (a.ash.tau_ref * m::sqrt(mer / a.ash.mer_ref_kg_s)).min(a.ash.tau_max) * fade;
        p.deposited_kg = mer * (date.day.min(ends) - e.day).max(0.0) * DAY_S;
        p.why = if bal {
            format!("A {h:.0} km ballistic eruption plume: {speed:.0} m/s from the vent into almost no air, height v²/2g.")
        } else {
            format!(
                "A {h:.1} km eruption column from {mer:.2e} kg/s of magma: {:.1} km on Earth (Mastin et al. 2009), × {fac:.2} for this world's gravity, air density and stratification (Morton, Taylor and Turner 1956).",
                mastin_height_mer_km(a, mer)
            )
        };
        plumes.push(p);
    }
    // Fires: smoke plumes and regional haze where they can burn.
    for f in &src.fires {
        if fire_burns(a, w, c, &circ, date, f).is_err() {
            continue;
        }
        let ar = f.area_km2 / a.fire.area_ref_km2;
        let h = a.fire.plume_km_ref * m::pow(ar.max(0.0), 0.25) * fac;
        let mut p = streak_plume(
            a,
            w,
            c,
            &circ,
            f.id.clone(),
            f.lat_deg,
            f.lon_deg,
            h,
            0.0,
            false,
            a.fire.settle_m_s,
        );
        p.born = f.day_start;
        p.ends = f.day_end;
        p.sigma0_km = p.sigma0_km.max(m::sqrt(f.area_km2 / m::PI));
        p.aerosol = "smoke".into();
        p.colour = a.fire.colour;
        p.tau0 = (a.fire.tau_ref * m::sqrt(ar)).min(a.fire.tau_max);
        p.why = format!(
            "A wildfire over {:.0} km² in the dry season with {:.0} % oxygen: a {h:.1} km smoke plume and haze downwind.",
            f.area_km2,
            100.0 * c.gases.get("o2").copied().unwrap_or(0.0)
        );
        plumes.push(p);
    }
    // Impacts: a fireball plume, then regional and world-wide veils.
    let im = &a.impact;
    let mut impact_veils = vec![];
    for i in &src.impacts {
        let e = impact_energy_j(a, i);
        if e < im.fireball_j || date.day < i.day {
            continue;
        }
        let dt = date.day - i.day;
        if dt < im.fireball_days {
            let h = im.plume_km_at_fireball * m::pow(e / im.fireball_j, 0.25) * fac;
            let mut p = streak_plume(
                a,
                w,
                c,
                &circ,
                i.id.clone(),
                i.lat_deg,
                i.lon_deg,
                h,
                0.0,
                true,
                a.ash.settle_m_s,
            );
            p.born = i.day;
            p.ends = i.day + im.fireball_days;
            p.aerosol = "dust".into();
            p.colour = im.dust_colour;
            p.tau0 = im.regional_tau;
            p.why = format!("The fireball and ejecta curtain of a {:.1e} J impact, rising about {h:.0} km (a stand-in scaling).", e);
            plumes.push(p);
        }
        if e >= im.regional_j && !thin(a, c) {
            let r = im.regional_radius_km * m::cbrt(e / im.regional_j);
            let tau = im.regional_tau * m::exp(-dt / im.regional_days);
            impact_veils.push((
                i.lat_deg,
                i.lon_deg,
                r,
                "dust".to_string(),
                tau,
                im.dust_colour,
            ));
        }
        if e >= im.winter_j && !thin(a, c) {
            let t = dt / YEAR_DAYS;
            let (share, _) = veil_curves(t, im.rise_years, im.residence_years, im.response_years);
            impact_veils.push((
                0.0,
                0.0,
                0.0,
                "smoke".to_string(),
                im.winter_tau * share,
                im.soot_colour,
            ));
        }
    }
    // Sulphate veils.
    let vs = &a.veil;
    let veils = veils(a, w, c, date, src)
        .into_iter()
        .map(|(lat, tg, t)| {
            let (share, _) = veil_curves(
                t,
                vs.conversion_years,
                vs.residence_years,
                vs.response_years,
            );
            (lat, vs.tau_per_tg_so2 * tg * share)
        })
        .collect();
    // Smog over industry, deeper under subsiding air and in basins.
    let sm = &a.smog;
    let smog = if thin(a, c) || circ.regime == "airless" {
        vec![]
    } else {
        src.industry
            .iter()
            .filter(|x| x.population > 0.0)
            .map(|x| {
                let t = x.tech.clamp(0.0, 1.0);
                let inv = circ
                    .sinking_deg
                    .iter()
                    .filter(|s| **s < 89.0)
                    .map(|s| bump(x.lat_deg.abs() - s, sm.inversion_width_deg))
                    .fold(0.0, f64::max);
                let pop = x.population / 1.0e6;
                let tau = sm.tau_ref
                    * m::sqrt(pop)
                    * 4.0
                    * t
                    * (1.0 - t)
                    * (1.0 + sm.inversion_boost * inv)
                    * (1.0 + if x.basin { sm.basin_boost } else { 0.0 });
                (x.lat_deg, x.lon_deg, tau, sm.radius_km_ref * m::sqrt(pop))
            })
            .collect()
    };
    // Hazes from the gas mix, with outgassed SO₂ mixed through the column.
    let out_kg_s: f64 = src.volcanoes.iter().map(|v| v.outgassing_kg_s).sum();
    let air_kg = c.column_kg_m2() * 4.0 * m::PI * c.radius_m * c.radius_m;
    let mut c2 = c.clone();
    if out_kg_s > 0.0 && air_kg > 0.0 {
        let x = out_kg_s * a.outgassing_mix_years * YEAR_DAYS * DAY_S * c.mean_molar_kg
            / M_SO2
            / air_kg;
        *c2.gases.entry("so2".into()).or_insert(0.0) += x;
    }
    let haze = crate::rules::haze_from(&a.photochemical, &c2);
    Prepared {
        circ,
        plumes,
        veils,
        impact_veils,
        smog,
        haze,
    }
}

/// The plumes alive on this date as nameable systems (kind `plume`).
pub fn plumes(
    a: &AerosolSettings,
    w: &WeatherSettings,
    c: &Conditions,
    date: Date,
    src: &Sources,
) -> Vec<WeatherSystem> {
    prepare(a, w, c, date, src)
        .plumes
        .iter()
        .filter(|p| p.born <= date.day && date.day < p.ends)
        .map(Plume::system)
        .collect()
}

fn add(list: &mut Vec<Aerosol>, kind: &str, tau: f64, colour: [f64; 3]) {
    if tau <= 0.0 {
        return;
    }
    if let Some(x) = list.iter_mut().find(|x| x.kind == kind) {
        let t = x.optical_depth + tau;
        for (old, new) in x.colour.iter_mut().zip(colour) {
            *old = (*old * x.optical_depth + new * tau) / t;
        }
        x.optical_depth = t;
    } else {
        list.push(Aerosol {
            kind: kind.into(),
            optical_depth: tau,
            colour,
            particle_radius_um: 0.0,
        });
    }
}

/// Add this date's aerosols to a cell (ash, smoke, dust, haze, smog, spray)
/// and return the ground facts the cell cannot hold. `sea` is whether the
/// cell is over open liquid when the ground is known; with `None` spray is
/// weighted by the world's ocean fraction. Leaves `temp_offset_k` alone: add
/// [`veil_cooling_k`] there.
pub fn imprint(
    p: &Prepared,
    a: &AerosolSettings,
    c: &Conditions,
    sea: Option<bool>,
    cell: &mut Cell,
) -> AerosolOut {
    let (lat, lon) = (cell.lat_deg, cell.lon_deg);
    let mut out = AerosolOut::default();
    let mut list: Vec<Aerosol> = vec![];
    let mut vog = 0.0;
    for pl in &p.plumes {
        let f = pl.shape(c, lat, lon);
        if pl.aerosol == "haze" {
            vog += pl.tau0 * f;
        }
        add(&mut list, &pl.aerosol, pl.tau0 * f, pl.colour);
        if pl.deposited_kg > 0.0 {
            let kg_m2 = pl.deposited_kg * f / (pl.area_km2() * 1.0e6);
            out.ash_fall_mm += 1000.0 * kg_m2 / a.ash.deposit_kg_m3;
        }
    }
    for (vlat, tau) in &p.veils {
        add(&mut list, "haze", tau * hemi(a, *vlat, lat), a.veil.colour);
    }
    for (ilat, ilon, r, kind, tau, col) in &p.impact_veils {
        let f = if *r > 0.0 {
            let d = distance_km(c, (*ilat, *ilon), (lat, lon)) / r;
            m::exp(-0.5 * d * d)
        } else {
            1.0
        };
        add(&mut list, kind, tau * f, *col);
    }
    for (slat, slon, tau, r) in &p.smog {
        let d = distance_km(c, (*slat, *slon), (lat, lon)) / r.max(1.0);
        add(&mut list, "smog", tau * m::exp(-0.5 * d * d), a.smog.colour);
    }
    for h in &p.haze {
        add(&mut list, &h.kind, h.optical_depth, h.colour);
    }
    // Spray over open liquid in wind.
    let sp = &a.spray;
    if c.liquid.is_some() && c.solid_surface && !thin(a, c) {
        let share = match sea {
            Some(true) => 1.0,
            Some(false) => 0.0,
            None => c.ocean_fraction,
        };
        let u = cell
            .layers
            .first()
            .map(|l| m::hypot(l.wind_m_s[0], l.wind_m_s[1]))
            .unwrap_or(0.0);
        add(
            &mut list,
            "spray",
            share * (sp.tau_per_m_s * (u - sp.u_min_m_s)).clamp(0.0, sp.tau_max),
            sp.colour,
        );
    }
    out.ground_darkening = 1.0 - m::exp(-out.ash_fall_mm / a.ash.darkening_mm);
    out.acid_rain = vog >= a.vog.acid_tau && cell.precip_mm_day > 0.0;
    cell.aerosols
        .extend(list.into_iter().filter(|x| x.optical_depth >= a.tau_min));
    out
}

/// One-call form of [`prepare`] and [`imprint`] (prepare once per tile when
/// filling many cells).
#[allow(clippy::too_many_arguments)]
pub fn imprint_aerosols(
    a: &AerosolSettings,
    w: &WeatherSettings,
    c: &Conditions,
    date: Date,
    sources: &Sources,
    sea: Option<bool>,
    cell: &mut Cell,
) -> AerosolOut {
    let p = prepare(a, w, c, date, sources);
    imprint(&p, a, c, sea, cell)
}

#[cfg(test)]
#[path = "aerosols_tests.rs"]
mod tests;

//! O10: stellar activity (nebulae-and-light design section 4A). From a
//! star's conditions (mass, radius, temperature, power, age and spin, or a
//! close companion's orbit) this crate gives its activity level, cycle,
//! spots and coronal X-rays, and lays out its flares and mass ejections as
//! keyed events on the timeline, each with a light description and, for an
//! ejection, a hit or miss on every planet by geometry.
//!
//! The scalar physics lives in `sse_atoms::activity` (editable blocks); this
//! crate decides which apply, from conditions, never from a star "kind".
//!
//! Flares are a Poisson process thinned by the cycle, laid out in fixed time
//! bins per energy decade. Every bin has its own draws, so any window of
//! time, and any lowest energy asked for, gives the same flares where they
//! overlap, on every device.

use serde::{Deserialize, Serialize};
use sse_atoms::activity as act;
use sse_core::draw::{Draws, Stream};
use sse_core::math as m;
use sse_core::quantity::consts::*;
use sse_core::{Address, Seed};
use sse_flow::Reference;
use sse_light::{LightDescription, Part};

#[cfg(test)]
mod tests;

pub const M_SUN: f64 = GM_SUN / G;
const KEV_M: f64 = sse_core::sheet::H * sse_core::sheet::C / (1e3 * sse_core::sheet::E);

const PALL81: &str = "Pallavicini, R. et al. 1981, Relations among stellar X-ray emission observed from Einstein, stellar rotation and bolometric luminosity, ApJ 248, 279: hot (O and B) stars give L_X ≈ 10⁻⁷ L_bol from their winds, independent of rotation. doi:10.1086/159152";
const SKUM72: &str = "Skumanich, A. 1972, Time scales for Ca II emission decay, rotational braking, and lithium depletion, ApJ 171, 565: spin falls as the square root of age. doi:10.1086/151310";
const SHIB13_T: &str = "Shibayama, T. et al. 2013, Superflares on solar-type stars observed with Kepler I, ApJS 209, 5: flare light as a 10 000 K black body. https://arxiv.org/abs/1308.1480";
const YASHIRO06: &str = "Yashiro, S. et al. 2006, Different power-law indices in the frequency distributions of flares with and without coronal mass ejections, ApJ 650, L143: the share of flares with ejections rises with flare size, to nearly all of the largest. doi:10.1086/508876";

fn r(text: &str) -> Reference {
    Reference::parse(text)
}

/// A star's conditions. Any body with a churning outer layer qualifies; the
/// crate decides from temperature and size, never from a name.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Star {
    pub address: String,
    pub mass_kg: f64,
    pub radius_m: f64,
    pub luminosity_w: f64,
    pub teff_k: f64,
    /// Time since the star was born.
    pub age_s: f64,
    /// Spin period if the life line knows it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rotation_period_s: Option<f64>,
    /// Orbital period of a close companion, which spins the star up by tides.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub companion_period_s: Option<f64>,
    /// An authored cycle length, which wins over the relation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cycle_period_s: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Wright et al. 2011 parameters.
    pub log_rx_sat: f64,
    pub ro_sat: f64,
    pub beta: f64,
    /// Cool enough to have a churning outer layer below this temperature (the Kraft break).
    pub convective_below_k: f64,
    /// Hot-star winds give L_X/L_bol = 1e-7 above this temperature.
    pub hot_wind_above_k: f64,
    pub hot_rx: f64,
    /// Compact remnants (radius below this share of the Sun's) have no dynamo of this kind.
    pub compact_below_rsun: f64,
    /// Spin at birth (disc-locked), days; spin-down never goes faster than this.
    pub birth_period_d: f64,
    /// A companion closer than this (orbital period, days) locks the star's spin to its orbit.
    pub tidal_lock_below_d: f64,
    /// Stars above this L_X/L_bol cycle on the active branch, below it on the inactive one.
    pub active_branch_above_rx: f64,
    /// Peak-to-mean amplitude of the cycle in X-rays and flare rate.
    pub cycle_amplitude: f64,
    /// Spot coverage of the Sun's mean, and the cap.
    pub sun_spot_coverage: f64,
    pub max_spot_coverage: f64,
    /// Flare energy spread dN/dE ∝ E^−α.
    pub flare_alpha: f64,
    /// Largest flare a star can have, J.
    pub max_flare_j: f64,
    /// Flare decay time at 10²⁵ J, s, and its growth with energy (τ ∝ E^0.39).
    pub decay_at_1e25_s: f64,
    pub decay_exponent: f64,
    pub flare_temperature_k: f64,
    /// Share of a flare's energy in soft X-rays, on top of the white-light flash.
    pub flare_xray_share: f64,
    /// Ejection: share of flares with one at 10²³ J and at 10²⁵ J (linear in log energy between).
    pub cme_share: [f64; 2],
    /// Ejection speed at 10²³ J, km/s, its growth with energy, and the cap.
    pub cme_speed_km_s: f64,
    pub cme_speed_exponent: f64,
    pub cme_max_km_s: f64,
    /// Ejection mass at 10²³ J, kg, and its growth with energy.
    pub cme_mass_kg: f64,
    pub cme_mass_exponent: f64,
    /// Half-width of an ejection's cone, degrees.
    pub cme_half_width_deg: f64,
    /// Ejections leave from within this latitude of the star's equator, degrees.
    pub active_latitude_deg: f64,
    /// Most flares listed in one answer; smaller ones are summarised.
    pub max_flares: usize,
    /// Surface convection blocks' parameters (`sse.convection.*`).
    pub photosphere_mu: f64,
    pub granule_factor: f64,
    pub convective_v_sun: f64,
    pub granule_life_k: f64,
    pub supergranule_ratio: f64,
    pub supergranule_flow_share: f64,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            log_rx_sat: -3.13,
            ro_sat: 0.13,
            beta: -2.70,
            convective_below_k: 6900.0,
            hot_wind_above_k: 10_000.0,
            hot_rx: 1e-7,
            compact_below_rsun: 0.05,
            birth_period_d: 1.0,
            tidal_lock_below_d: 10.0,
            active_branch_above_rx: 1e-5,
            cycle_amplitude: 0.7,
            sun_spot_coverage: 0.003,
            max_spot_coverage: 0.5,
            flare_alpha: 2.0,
            max_flare_j: 1e29,
            decay_at_1e25_s: 600.0,
            decay_exponent: 0.39,
            flare_temperature_k: 10_000.0,
            flare_xray_share: 0.1,
            cme_share: [0.2, 1.0],
            cme_speed_km_s: 400.0,
            cme_speed_exponent: 0.2,
            cme_max_km_s: 4000.0,
            cme_mass_kg: 1e12,
            cme_mass_exponent: 0.6,
            cme_half_width_deg: 30.0,
            active_latitude_deg: 30.0,
            max_flares: 2000,
            photosphere_mu: 1.3,
            granule_factor: 10.0,
            convective_v_sun: 2000.0,
            granule_life_k: 0.5,
            supergranule_ratio: 25.0,
            supergranule_flow_share: 0.2,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Cycle {
    pub period_s: f64,
    /// "inactive", "active" or "authored".
    pub branch: String,
    /// Cycle phase at age zero, 0 to 1 (drawn).
    pub phase_offset: f64,
}

/// A star's activity: the fact the rest of the model reads.
/// Surface convection: granules and supergranules (what the renderer animates,
/// and the network that spots and flares rise through).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Convection {
    pub surface_gravity_m_s2: f64,
    pub pressure_scale_height_m: f64,
    pub granule_size_m: f64,
    pub speed_m_s: f64,
    pub sound_speed_m_s: f64,
    pub granule_lifetime_s: f64,
    pub supergranule_size_m: f64,
    pub supergranule_lifetime_s: f64,
    /// Granules across the star's diameter, and supergranules (network cells)
    /// over its whole surface.
    pub granules_across: f64,
    pub network_cells: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Activity {
    pub address: String,
    /// saturated, active, moderate, quiet, hot-wind, no-corona or compact.
    pub regime: String,
    pub rotation_period_s: f64,
    /// Where the spin came from: given, tides or age.
    pub rotation_from: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub turnover_s: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rossby: Option<f64>,
    pub xray_fraction: f64,
    /// Mean coronal X-ray power, W (feeds `corona_xray_w` in light).
    pub xray_w: f64,
    pub spot_coverage: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cycle: Option<Cycle>,
    /// The churning surface, if it has one (any body cooler than the
    /// granulation boundary, remnants included).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub convection: Option<Convection>,
    pub flares: bool,
    pub why: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stand_ins: Vec<String>,
    pub references: Vec<Reference>,
}

fn addr(a: &str) -> Result<Address, String> {
    a.parse().map_err(|e| format!("{a}: {e:?}"))
}

/// The star's activity level and cycle.
pub fn activity(seed: &Seed, s: &Star, set: &Settings) -> Result<Activity, String> {
    if !(s.mass_kg > 0.0 && s.radius_m > 0.0 && s.luminosity_w > 0.0 && s.teff_k > 0.0) {
        return Err(format!(
            "{}: mass, radius, power and temperature must be positive",
            s.address
        ));
    }
    let mut why = vec![];
    let mut stand_ins = vec![];
    let mut refs = vec![];
    let (rotation_period_s, rotation_from) = match (s.companion_period_s, s.rotation_period_s) {
        (Some(p), _) if p < set.tidal_lock_below_d * DAY => {
            why.push(format!("A companion only {:.1} days round holds the star's spin to its orbit by tides, so it stays active at any age.", p / DAY));
            stand_ins.push(format!("tidal locking is assumed for companions closer than {} days, rather than computed from tidal theory", set.tidal_lock_below_d));
            (p, "tides")
        }
        (_, Some(p)) => (p, "given"),
        _ => {
            let tau = act::convective_turnover(s.mass_kg.clamp(0.09 * M_SUN, 1.36 * M_SUN));
            let tau_sun = act::convective_turnover(M_SUN);
            let p = (26.09 * DAY * tau / tau_sun
                * m::sqrt(s.age_s.max(0.0) / (4.57e9 * JULIAN_YEAR)))
            .max(set.birth_period_d * DAY);
            why.push(format!("Spun down with age as the square root of time (Skumanich), from the Sun's 26.1 days at 4.57 billion years: about {:.1} days now.", p / DAY));
            stand_ins.push("spin from age is a Skumanich law scaled by turnover time, never faster than a 1-day birth spin; the life line's own spin wins when it has one (F7)".into());
            refs.push(r(SKUM72));
            (p, "age")
        }
    };
    let compact = s.radius_m < set.compact_below_rsun * R_SUN;
    let convection = surface_convection(s, set);
    if let Some(c) = &convection {
        why.push(surface_why(c, s));
        stand_ins.push("granules are ten pressure scale heights across and live half a crossing time; supergranules 25 granules across with flows a fifth of the granule speed; convective speed from flux with a fixed photospheric opacity (all fitted on the Sun)".into());
        for x in sse_atoms::decls_of(&[
            "sse.convection.granule_size",
            "sse.convection.speed",
            "sse.convection.cell_lifetime",
            "sse.convection.supergranule_size",
            "sse.convection.supergranule_lifetime",
        ])
        .into_iter()
        .flat_map(|d| d.references)
        {
            if !refs.contains(&x) {
                refs.push(x);
            }
        }
    }
    let refs_w11 = || sse_atoms::decl_of("sse.activity.convective_turnover_time").references;
    let base =
        |regime: &str, rx: f64, why: Vec<String>, stand_ins: Vec<String>, refs: Vec<Reference>| {
            Activity {
                address: s.address.clone(),
                regime: regime.into(),
                rotation_period_s,
                rotation_from: rotation_from.into(),
                turnover_s: None,
                rossby: None,
                xray_fraction: rx,
                xray_w: rx * s.luminosity_w,
                spot_coverage: 0.0,
                cycle: None,
                convection: convection.clone(),
                flares: false,
                why,
                stand_ins,
                references: refs,
            }
        };
    if compact {
        why.push("A compact remnant: no churning outer layer, so no spots, cycles or flares of this kind.".into());
        return Ok(base("compact", 0.0, why, stand_ins, refs));
    }
    if s.teff_k >= set.convective_below_k {
        if s.teff_k >= set.hot_wind_above_k {
            why.push(format!("At {:.0} K the star has no churning outer layer; its X-rays come from shocks in its fast line-driven wind, about a ten-millionth of its power.", s.teff_k));
            refs.push(r(PALL81));
            return Ok(base("hot-wind", set.hot_rx, why, stand_ins, refs));
        }
        why.push(format!("At {:.0} K the star's outer layer barely churns: almost no corona, no spots, no flares.", s.teff_k));
        return Ok(base("no-corona", 0.0, why, stand_ins, refs));
    }
    if s.mass_kg < 0.09 * M_SUN || s.mass_kg > 1.36 * M_SUN {
        stand_ins.push(
            "turnover time extrapolated outside the 0.09 to 1.36 solar-mass range of its fit"
                .into(),
        );
    }
    let tau = act::convective_turnover(s.mass_kg);
    let ro = rotation_period_s / tau;
    let rx = act::xray_fraction(ro, set.log_rx_sat, set.ro_sat, set.beta);
    let regime = if ro <= set.ro_sat {
        "saturated"
    } else if rx > 1e-4 {
        "active"
    } else if rx > 3e-7 {
        "moderate"
    } else {
        "quiet"
    };
    why.push(format!(
        "Its outer layer turns over in {:.1} days and it spins in {:.1}: Rossby number {:.3}, so {}; X-rays are {:.1e} of its power ({:.2e} W).",
        tau / DAY,
        rotation_period_s / DAY,
        ro,
        match regime {
            "saturated" => "saturated: heavily spotted, flaring constantly",
            "active" => "very active",
            "moderate" => "moderately active, with a cycle",
            _ => "quiet",
        },
        rx,
        rx * s.luminosity_w
    ));
    for x in refs_w11() {
        refs.push(x);
    }
    let rx_sun = m::pow(10.0, act::SUN_LOG_RX);
    let spot = (set.sun_spot_coverage * m::sqrt(rx / rx_sun)).min(set.max_spot_coverage);
    stand_ins.push("regime words: active above 10⁻⁴ of the star's power in X-rays, moderate above 3×10⁻⁷, quiet below (labels only; the numbers carry the physics)".into());
    stand_ins.push("spot coverage scales as the square root of the X-ray fraction from the Sun's 0.3 %, capped at half the disc".into());
    let d = Draws::new(
        *seed,
        addr(&s.address)?,
        Stream::new("activity.cycle.v1").map_err(|e| e.0)?,
    );
    let cycle = if let Some(p) = s.cycle_period_s {
        Some(Cycle {
            period_s: p,
            branch: "authored".into(),
            phase_offset: d.unit(0),
        })
    } else if regime == "saturated" {
        why.push("Saturated stars show no clean cycle; their activity is steady and high.".into());
        None
    } else {
        let active = rx > set.active_branch_above_rx;
        let (a, n) = if active { (2.22, 1.28) } else { (1.488, 1.324) };
        let p = act::cycle_period(rotation_period_s, a, n);
        why.push(format!(
            "Its spots and flares wax and wane over a {:.1}-year cycle ({} branch).",
            p / JULIAN_YEAR,
            if active { "active" } else { "inactive" }
        ));
        for x in sse_atoms::decl_of("sse.activity.cycle_period_inactive").references {
            refs.push(x);
        }
        stand_ins.push("the cycle is a sine of fixed amplitude; real cycles vary in length and strength, with grand minima".into());
        Some(Cycle {
            period_s: p,
            branch: if active { "active" } else { "inactive" }.into(),
            phase_offset: d.unit(0),
        })
    };
    let mut a = base(regime, rx, why, stand_ins, refs);
    a.turnover_s = Some(tau);
    a.rossby = Some(ro);
    a.spot_coverage = spot;
    a.cycle = cycle;
    a.flares = true;
    Ok(a)
}

/// Surface convection from the star's own temperature, gravity and size;
/// `None` above the granulation boundary.
pub fn surface_convection(s: &Star, set: &Settings) -> Option<Convection> {
    use sse_atoms::convection as cv;
    if s.teff_k >= set.convective_below_k {
        return None;
    }
    let g = sse_core::quantity::consts::G * s.mass_kg / (s.radius_m * s.radius_m);
    let h = sse_atoms::air::scale_height(s.teff_k, set.photosphere_mu * sse_atoms::air::AMU, g);
    let d = cv::granule_size(h, set.granule_factor);
    let v = cv::convective_speed(s.teff_k, g, set.convective_v_sun, set.photosphere_mu);
    let big = d * set.supergranule_ratio;
    let area = 4.0 * m::PI * s.radius_m * s.radius_m;
    Some(Convection {
        surface_gravity_m_s2: g,
        pressure_scale_height_m: h,
        granule_size_m: d,
        speed_m_s: v,
        sound_speed_m_s: cv::sound_speed(s.teff_k, set.photosphere_mu),
        granule_lifetime_s: cv::cell_lifetime(d, v, set.granule_life_k),
        supergranule_size_m: big,
        supergranule_lifetime_s: cv::cell_lifetime(big, v * set.supergranule_flow_share, 1.0),
        granules_across: 2.0 * s.radius_m / d,
        network_cells: area / (big * big),
    })
}

fn span(t: f64) -> String {
    if t < 3600.0 {
        format!("{:.0} minutes", t / 60.0)
    } else if t < 2.0 * DAY {
        format!("{:.0} hours", t / 3600.0)
    } else if t < 2.0 * JULIAN_YEAR {
        format!("{:.0} days", t / DAY)
    } else {
        format!("{:.1} years", t / JULIAN_YEAR)
    }
}

fn length(x: f64) -> String {
    if x < 1e6 {
        format!("{:.0} km", x / 1e3)
    } else if x < 0.1 * R_SUN {
        format!("{:.0} thousand km", x / 1e6)
    } else {
        format!("{:.1} Sun-widths", x / (2.0 * R_SUN))
    }
}

fn surface_why(c: &Convection, s: &Star) -> String {
    let near_sound = c.speed_m_s >= 0.99 * c.sound_speed_m_s;
    format!(
        "Its {:.0} K surface boils in granules about {} across, each living about {}{}, about {:.0} across the star; larger cells about {} wide, lasting about {}, sweep its magnetic field into a net of about {:.0} cells.",
        s.teff_k,
        length(c.granule_size_m),
        span(c.granule_lifetime_s),
        if near_sound { " and churning near the speed of sound" } else { "" },
        c.granules_across,
        length(c.supergranule_size_m),
        span(c.supergranule_lifetime_s),
        c.network_cells
    )
}

/// The activity multiplier at time `t_s` (age): 1 + A sin(2π (t/P + φ)), or 1 without a cycle.
pub fn cycle_factor(a: &Activity, t_s: f64, set: &Settings) -> f64 {
    match &a.cycle {
        Some(c) => 1.0 + set.cycle_amplitude * m::sin(m::TAU * (t_s / c.period_s + c.phase_offset)),
        None => 1.0,
    }
}

/// The star at one moment.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Moment {
    pub at_s: f64,
    pub cycle_phase: Option<f64>,
    pub xray_w: f64,
    pub spot_coverage: f64,
    /// Flares per year above 10³² erg (10²⁵ J, about the Sun's largest class) and above 10³⁴ erg.
    pub flares_per_year_above_1e25_j: f64,
    pub flares_per_year_above_1e27_j: f64,
}

pub fn moment(a: &Activity, t_s: f64, set: &Settings) -> Moment {
    let f = cycle_factor(a, t_s, set);
    let lx = a.xray_w * f;
    let rate = |e: f64| {
        if a.flares {
            act::flare_rate_above(lx, e, act::FLARE_K, set.flare_alpha) * JULIAN_YEAR
        } else {
            0.0
        }
    };
    Moment {
        at_s: t_s,
        cycle_phase: a
            .cycle
            .as_ref()
            .map(|c| m::fmod(t_s / c.period_s + c.phase_offset, 1.0)),
        xray_w: lx,
        spot_coverage: (a.spot_coverage * f).min(set.max_spot_coverage),
        flares_per_year_above_1e25_j: rate(1e25),
        flares_per_year_above_1e27_j: rate(1e27),
    }
}

/// Something an ejection may hit: a planet on a circular orbit in the star's equator.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Target {
    pub address: String,
    pub distance_m: f64,
    /// Orbital period; without one the target stays put.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub period_s: Option<f64>,
    /// Angle round the star at age zero, radians.
    #[serde(default)]
    pub phase_rad: f64,
}

impl Target {
    fn direction(&self, t: f64) -> [f64; 3] {
        let a = self.phase_rad + self.period_s.map_or(0.0, |p| m::TAU * m::fmod(t / p, 1.0));
        [m::cos(a), m::sin(a), 0.0]
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Hit {
    pub target: String,
    /// When the ejection arrives (age, s).
    pub arrives_s: f64,
    /// Warning: the flare's light arrives this long before the particles.
    pub warning_s: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ejection {
    pub speed_m_s: f64,
    pub mass_kg: f64,
    /// Unit vector from the star (z is its spin axis).
    pub direction: [f64; 3],
    pub half_width_rad: f64,
    pub hits: Vec<Hit>,
    pub misses: Vec<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Flare {
    pub address: String,
    /// Peak time (age, s).
    pub at_s: f64,
    pub energy_j: f64,
    pub rise_s: f64,
    pub decay_s: f64,
    pub peak_w: f64,
    /// Light at the peak.
    pub light: LightDescription,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ejection: Option<Ejection>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Flares {
    pub star: String,
    pub from_s: f64,
    pub to_s: f64,
    /// Lowest energy listed, J (raised above what was asked if the list would be too long).
    pub listed_above_j: f64,
    pub flares: Vec<Flare>,
    /// Expected number and energy of the flares between the energy asked for and `listed_above_j`.
    pub summarised_count: f64,
    pub summarised_energy_j: f64,
    pub why: Vec<String>,
    pub stand_ins: Vec<String>,
    pub references: Vec<Reference>,
}

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Poisson count from a uniform draw (inverse CDF, capped).
fn poisson(mu: f64, u: f64, cap: u64) -> u64 {
    let mut k = 0;
    let mut p = m::exp(-mu);
    let mut c = p;
    while u > c && k < cap {
        k += 1;
        p *= mu / k as f64;
        c += p;
    }
    k
}

/// Flare energy released per second by flares in [e_lo, e_hi):
/// with N(>E) = C E^(1−α), the integral of E dN is (α−1) C ∫ E^(1−α) dE.
fn energy_between(lx: f64, e_lo: f64, e_hi: f64, set: &Settings) -> f64 {
    let a = set.flare_alpha;
    let c = act::FLARE_K * lx / act::E_REF_J * m::pow(act::E_REF_J, a - 1.0);
    if (a - 2.0).abs() < 1e-12 {
        c * m::ln(e_hi / e_lo)
    } else {
        (a - 1.0) * c * (m::pow(e_hi, 2.0 - a) - m::pow(e_lo, 2.0 - a)) / (2.0 - a)
    }
}

/// Expected flares in [e_lo, e_hi) per second at mean X-ray power.
fn decade_rate(lx: f64, e_lo: f64, e_hi: f64, set: &Settings) -> f64 {
    act::flare_rate_above(lx, e_lo, act::FLARE_K, set.flare_alpha)
        - act::flare_rate_above(lx, e_hi, act::FLARE_K, set.flare_alpha)
}

/// Flares between `from_s` and `to_s` (ages) with at least `min_energy_j`.
pub fn flares(
    seed: &Seed,
    a: &Activity,
    from_s: f64,
    to_s: f64,
    min_energy_j: f64,
    targets: &[Target],
    set: &Settings,
) -> Result<Flares, String> {
    if !(to_s >= from_s && min_energy_j > 0.0) {
        return Err("the window must not run backwards and the energy must be positive".into());
    }
    let mut out = Flares {
        star: a.address.clone(),
        from_s,
        to_s,
        listed_above_j: min_energy_j,
        flares: vec![],
        summarised_count: 0.0,
        summarised_energy_j: 0.0,
        why: vec![],
        stand_ins: vec![
            format!("flare durations scale as E^{} from {} s at 10²⁵ J; rise is a tenth of decay", set.decay_exponent, set.decay_at_1e25_s),
            format!("flare light is a {} K black body plus {} % extra in soft X-rays; no radio burst yet", set.flare_temperature_k, set.flare_xray_share * 100.0),
            "ejection share, speed and mass grow with flare energy by stated power laws; cone half-width 30°, leaving within 30° of the star's equator; planets on circular orbits in that equator".into(),
            format!("largest possible flare {:.0e} J", set.max_flare_j),
        ],
        references: sse_atoms::decl_of("sse.activity.flare_rate_above").references,
    };
    out.references.push(r(SHIB13_T));
    out.references.push(r(YASHIRO06));
    if !a.flares {
        out.why
            .push("This star does not flare: no churning outer layer.".into());
        return Ok(out);
    }
    let peak = 1.0
        + if a.cycle.is_some() {
            set.cycle_amplitude
        } else {
            0.0
        };
    let lx_max = a.xray_w * peak;
    let d_lo = m::floor(m::log10(min_energy_j)) as i32;
    let d_hi = m::ceil(m::log10(set.max_flare_j)) as i32;
    let star = addr(&a.address)?;
    let mut listed_above = min_energy_j;
    // From the largest flares down, so a cap on the list drops the smallest.
    let mut decade = d_hi - 1;
    while decade >= d_lo {
        let (e_lo, e_hi) = (
            m::pow(10.0, decade as f64).max(min_energy_j),
            m::pow(10.0, (decade + 1) as f64).min(set.max_flare_j),
        );
        if e_lo >= e_hi {
            decade -= 1;
            continue;
        }
        let edge_lo = m::pow(10.0, decade as f64);
        let rate_max = decade_rate(
            lx_max,
            edge_lo,
            m::pow(10.0, (decade + 1) as f64).min(set.max_flare_j),
            set,
        );
        let expected = decade_rate(a.xray_w, e_lo, e_hi, set) * (to_s - from_s);
        if out.flares.len() as f64 + expected > set.max_flares as f64 {
            // Summarise this decade and everything below.
            out.summarised_count = decade_rate(a.xray_w, min_energy_j, e_hi, set) * (to_s - from_s);
            out.summarised_energy_j =
                energy_between(a.xray_w, min_energy_j, e_hi, set) * (to_s - from_s);
            listed_above = e_hi;
            break;
        }
        // Bins of fixed length for this star and decade: about one flare per bin at the cycle peak.
        let bin = m::pow(2.0, m::round(m::log2(1.0 / rate_max))).max(60.0);
        let stream = Stream::new(&format!("activity.flares.e{}.v1", decade)).map_err(|e| e.0)?;
        let dr = Draws::new(*seed, star.clone(), stream);
        let (b0, b1) = (m::floor(from_s / bin) as i64, m::floor(to_s / bin) as i64);
        if rate_max * bin > 60.0 {
            return Err(format!(
                "flares near {:.0e} J come too often to list one by one: ask for larger flares",
                e_lo
            ));
        }
        if b1 - b0 > 5_000_000 {
            return Err(format!(
                "{:.0e} bins for flares near {:.0e} J: ask for a shorter window or larger flares",
                (b1 - b0) as f64,
                e_lo
            ));
        }
        for b in b0..=b1 {
            let base = (b as u64).wrapping_mul(1024);
            let n = poisson(rate_max * bin, dr.unit(base + 1023), 126);
            for k in 0..n {
                let i = base + k * 8;
                let t = (b as f64 + dr.unit(i)) * bin;
                if t < from_s || t > to_s {
                    continue;
                }
                if dr.unit(i + 1) * peak > cycle_factor(a, t, set) {
                    continue;
                }
                // Truncated power law within the decade: N(>E) ∝ E^(1−α).
                let top = m::pow(10.0, (decade + 1) as f64).min(set.max_flare_j);
                let g = 1.0 - set.flare_alpha;
                let u = dr.unit(i + 2);
                let e = m::pow(
                    m::pow(edge_lo, g) + u * (m::pow(top, g) - m::pow(edge_lo, g)),
                    1.0 / g,
                );
                if e < min_energy_j {
                    continue;
                }
                out.flares
                    .push(flare(a, t, e, &dr, i, targets, set, out.flares.len()));
            }
        }
        decade -= 1;
    }
    out.listed_above_j = listed_above;
    out.flares.sort_by(|x, y| x.at_s.total_cmp(&y.at_s));
    for (n, f) in out.flares.iter_mut().enumerate() {
        f.address = format!("{}/flare:{}", a.address, n);
        f.light.address = f.address.clone();
    }
    out.why.push(format!(
        "{} flares above {:.1e} J in {:.2} years; ten times the energy comes about a tenth as often.",
        out.flares.len(),
        listed_above,
        (to_s - from_s) / JULIAN_YEAR
    ));
    if out.summarised_count > 0.0 {
        out.why.push(format!(
            "About {:.3e} smaller flares are summarised rather than listed.",
            out.summarised_count
        ));
    }
    Ok(out)
}

#[allow(clippy::too_many_arguments)]
fn flare(
    a: &Activity,
    t: f64,
    e: f64,
    dr: &Draws,
    i: u64,
    targets: &[Target],
    set: &Settings,
    n: usize,
) -> Flare {
    let decay = set.decay_at_1e25_s * m::pow(e / 1e25, set.decay_exponent);
    let rise = 0.1 * decay;
    let peak_w = e / (0.5 * rise + decay);
    let tk = set.flare_temperature_k;
    let area = peak_w / (sse_light::SIGMA * tk * tk * tk * tk);
    let mut light = LightDescription::new(&format!("{}/flare:{}", a.address, n));
    light.parts.push(Part::Thermal {
        t_k: tk,
        area_m2: area,
        emissivity: 1.0,
        cause: "flare".into(),
    });
    light.parts.push(Part::PowerLaw {
        power_w: set.flare_xray_share * peak_w,
        from_m: 10.0 * KEV_M / 100.0,
        to_m: KEV_M / 0.1,
        alpha: 0.0,
        cause: "flare X-rays".into(),
    });
    light.why.push(format!(
        "A flare of {:.1e} J: a white-hot flash ({:.0} K over {:.1e} m²) peaking at {:.1e} W, rising in {:.0} s and fading over {:.0} s, with a burst of X-rays.",
        e, tk, area, peak_w, rise, decay
    ));
    // Share with an ejection: linear in log energy through the two stated points, 0 to 1.
    let lg = m::log10(e / 1e23) / 2.0;
    let share = (set.cme_share[0] + (set.cme_share[1] - set.cme_share[0]) * lg).clamp(0.0, 1.0);
    let ejection = {
        if dr.unit(i + 3) < share {
            let x = e / 1e23;
            let v = (set.cme_speed_km_s * m::pow(x, set.cme_speed_exponent)).min(set.cme_max_km_s)
                * 1e3;
            let mass = set.cme_mass_kg * m::pow(x, set.cme_mass_exponent);
            let lon = dr.uniform(i + 4, 0.0, m::TAU);
            let lat = dr.uniform(i + 5, -1.0, 1.0) * set.active_latitude_deg * m::PI / 180.0;
            let dir = [
                m::cos(lat) * m::cos(lon),
                m::cos(lat) * m::sin(lon),
                m::sin(lat),
            ];
            let hw = set.cme_half_width_deg * m::PI / 180.0;
            let (mut hits, mut misses) = (vec![], vec![]);
            for tg in targets {
                let travel = tg.distance_m / v;
                // Aim at where the planet will be when the ejection gets there.
                let there = tg.direction(t + travel);
                if dot(dir, there) >= m::cos(hw) {
                    hits.push(Hit {
                        target: tg.address.clone(),
                        arrives_s: t + travel,
                        warning_s: travel - tg.distance_m / C,
                    });
                } else {
                    misses.push(tg.address.clone());
                }
            }
            Some(Ejection {
                speed_m_s: v,
                mass_kg: mass,
                direction: dir,
                half_width_rad: hw,
                hits,
                misses,
            })
        } else {
            None
        }
    };
    Flare {
        address: String::new(),
        at_s: t,
        energy_j: e,
        rise_s: rise,
        decay_s: decay,
        peak_w,
        light,
        ejection,
    }
}

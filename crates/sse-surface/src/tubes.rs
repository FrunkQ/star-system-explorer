//! S14a: lava tubes (Rock 3 comparison 5.5, accepted 21 Nov), part 1:
//! envelopes.
//!
//! Low-viscosity lava on a gentle slope crusts over and keeps flowing
//! inside its own roof. When the eruption stops, the tube drains and a
//! hollow is left. This stage makes, for every lava source S4 laid down
//! (shields and lava plains), the envelope of the tubes it holds: how
//! likely tubes are there, how wide and deep, how much passage per km²,
//! and how much of the roof is left. Nothing is placed yet. A site's
//! passage graph resolves from this envelope on approach (S14a part 2).
//!
//! Each number is a process strength or a cited scaling, never a class:
//! - **Forming** (0 to 1): tubes need a gentle slope. Too steep and the
//!   flow runs as open channels; too flat and it spreads as sheets
//!   (Greeley 1971; Calvari and Pinkerton 1998: Etna and Hawaii tubes on
//!   slopes of about 1 to 5 degrees). A log-normal curve in slope.
//! - **Width**: the widest roof that holds itself up scales as 1/g (Blair
//!   et al. 2017, lunar tube stability), so Moon tubes can be about six
//!   times wider than Earth's for the same rock.
//! - **Roof kept** (0 to 1): on an airless world, impacts breach a roof
//!   wherever a crater cuts deeper than it, about a fifth of the crater's
//!   width (the same saturation law that thins old craters). On a wet
//!   world the roof also wears down with the land around it.
//! - **Stage** is read off the roof kept: an intact roof, then skylights,
//!   then pit chains, then an open sinuous rille.

use crate::impacts::{CraterField, Features};
use crate::SurfaceEnvelope;
use serde::{Deserialize, Serialize};
use sse_core::draw::Draws;
use sse_core::math as m;

pub const TUBES_VERSION: &str = "surface-tubes-v1";

pub const REFS: [&str; 5] = [
    "Greeley, R. 1971, The Moon 3, 289 (now Earth, Moon, and Planets): lava tubes and channels in the lunar Marius Hills; tubes form in fluid pahoehoe on gentle slopes. doi:10.1007/BF00561842",
    "Calvari, S. and Pinkerton, H. 1998, JGR 103, 27291: formation of lava tubes and extensive flow fields during the 1991-1993 eruption of Mount Etna. doi:10.1029/97JB03388",
    "Blair, D. M. et al. 2017, Icarus 282, 47: the structural stability of lunar lava tubes; under the Moon's gravity tubes kilometres wide can stand (the 1/g scaling of roof span is SSE4's inference from it). doi:10.1016/j.icarus.2016.10.008",
    "Sauro, F. et al. 2020, Earth-Science Reviews 209, 103288: lava tubes on Earth, Moon and Mars; terrestrial tubes up to about 30 m wide. doi:10.1016/j.earscirev.2020.103288",
    "Haruyama, J. et al. 2009, GRL 36, L21206: a possible lunar lava tube skylight in the Marius Hills, about 65 m across. doi:10.1029/2009GL040635",
];

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TubeSettings {
    /// Typical and largest tube width on Earth, m (Sauro et al. 2020).
    pub earth_width_m: [f64; 2],
    /// Roof thickness over width for a tube that formed (a fudge, registered).
    pub roof_over_width: f64,
    /// Slope at which tubes form best, degrees, and the factor either side
    /// at which the strength falls to e^(−1/2).
    pub slope_best_deg: f64,
    pub slope_factor: f64,
    /// Passage length per km² of a lava field where tubes form at full
    /// strength (a fudge, registered: Hawaiian flank fields).
    pub km_per_km2: f64,
    /// Ground worn per metre of rain a year, m per Myr (denudation).
    pub wear_m_myr_per_m_rain: f64,
    /// Skylights per km of passage whose roof has gone.
    pub skylights_per_km_lost: f64,
}

impl Default for TubeSettings {
    fn default() -> Self {
        TubeSettings {
            earth_width_m: [10.0, 30.0],
            roof_over_width: 0.25,
            slope_best_deg: 2.0,
            slope_factor: 2.5,
            km_per_km2: 0.05,
            wear_m_myr_per_m_rain: 20.0,
            skylights_per_km_lost: 2.0,
        }
    }
}

/// The tubes one lava source holds, as an envelope (no passages placed).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct TubeField {
    pub id: String,
    /// The S4 feature the lava came from.
    pub source: String,
    pub dir: [f64; 3],
    pub radius_km: f64,
    pub formed_myr: f64,
    /// Slope of the lava's ground, degrees.
    pub slope_deg: f64,
    /// How readily tubes form there, 0 to 1.
    pub forming: f64,
    pub width_m: f64,
    pub width_max_m: f64,
    pub roof_m: f64,
    /// Passage length in the field, km.
    pub length_km: f64,
    /// Share of the roof still standing, 0 to 1.
    pub roof_kept: f64,
    /// The label read off `roof_kept`.
    pub stage: String,
    pub skylights: u32,
    /// Cryolava on an ice world: labelled so.
    pub speculative: bool,
    pub cause: String,
}

/// What the tube stage needs from the rest of the world.
pub struct TubeInputs<'a> {
    pub features: &'a Features,
    pub craters: &'a CraterField,
    /// Mean rain, m a year, at a direction (None: dry world).
    pub rain_m_yr: &'a dyn Fn([f64; 3]) -> f64,
    /// When the world had rain, Myr since formation.
    pub wet_myr: (f64, f64),
    /// Hot-spot volcanoes on a plate world's land (S2 plumes): id, centre,
    /// radius km, mean date of their lava.
    pub plumes: Vec<(String, [f64; 3], f64, f64)>,
    /// Flank slope of a shield, degrees (from S4's shield aspect).
    pub shield_slope_deg: f64,
}

fn stage_of(kept: f64) -> &'static str {
    if kept >= 0.9 {
        "intact roof"
    } else if kept >= 0.5 {
        "skylights"
    } else if kept >= 0.1 {
        "pit chains"
    } else {
        "sinuous rille"
    }
}

/// How readily tubes form on ground of this slope, 0 to 1.
pub fn forming_strength(t: &TubeSettings, slope_deg: f64) -> f64 {
    if slope_deg <= 0.0 {
        return 0.0;
    }
    let z = m::ln(slope_deg / t.slope_best_deg) / m::ln(t.slope_factor.max(1.0001));
    m::exp(-0.5 * z * z)
}

/// Every lava source's tube envelope at the epoch.
pub fn lava_tubes(
    t: &TubeSettings,
    env: &SurfaceEnvelope,
    d: &Draws,
    inp: &TubeInputs,
) -> Vec<TubeField> {
    let epoch = inp.features.epoch_myr;
    let g = env.gravity_m_s2.max(1e-3);
    let gscale = sse_core::sheet::G0 / g;
    let icy = env.regime_now.contains("ice");
    let mut out = vec![];
    // (source id, dir, radius km, formed, slope degrees, what it is)
    let mut sources: Vec<(String, [f64; 3], f64, f64, f64, String)> = vec![];
    for sh in &inp.features.shields {
        let slope = m::atan(sh.height_m / (sh.radius_km * 1000.0).max(1.0)).to_degrees();
        sources.push((
            sh.id.clone(),
            sh.dir,
            sh.radius_km,
            0.5 * (sh.from_myr + sh.to_myr),
            slope,
            "shield volcano's flanks".into(),
        ));
    }
    for (id, dir, r, formed) in &inp.plumes {
        sources.push((
            id.clone(),
            *dir,
            *r,
            *formed,
            inp.shield_slope_deg,
            "hot-spot volcano's flanks".into(),
        ));
    }
    for (pi, p) in inp.features.plains.iter().enumerate() {
        // Flood lava ponds on near-level ground; its tubes run on the
        // gentle ramps at its margins (a tenth of a degree to a degree).
        let slope = 0.3 * (0.5 + d.unit(9_000 + pi as u64));
        sources.push((
            p.id.clone(),
            p.dir,
            p.radius_km,
            p.formed_myr,
            slope,
            "lava plains".into(),
        ));
    }
    for (i, (src, dir, radius_km, formed, slope, what)) in sources.into_iter().enumerate() {
        let k = |x: u64| ((i as u64 + 1) << 32) | x;
        let forming = forming_strength(t, slope);
        if forming < 1e-3 {
            continue;
        }
        let w_typ = t.earth_width_m[0] * gscale;
        let w_max = t.earth_width_m[1] * gscale;
        let width = w_typ * m::pow(w_max / w_typ, d.unit(k(1)));
        let roof = t.roof_over_width * width;
        let area = m::PI * radius_km * radius_km;
        let length = t.km_per_km2 * forming * area;
        // Impacts breach the roof where a crater's bowl cuts deeper than
        // it (S14a part 3: only the bowl's deep middle, never saturating),
        // and rain wears it down.
        let by_impacts = inp.craters.cut_through_kept(roof, formed, epoch);
        let wet = (epoch.min(inp.wet_myr.1) - formed.max(inp.wet_myr.0)).max(0.0);
        let rain = (inp.rain_m_yr)(dir);
        let worn_m = t.wear_m_myr_per_m_rain * rain * wet;
        let by_wear = m::exp(-worn_m / roof.max(1e-3));
        // Later lava or ice over the field buries its tubes: kept, but out
        // of reach until a pit or a rille opens them.
        let open = inp.craters.renewal_kept(formed);
        if open < 0.01 {
            continue;
        }
        let length = length * open;
        let kept = by_impacts * by_wear;
        let stage = stage_of(kept);
        let skylights = m::round(length * (1.0 - kept).min(0.9) * t.skylights_per_km_lost) as u32;
        let age = epoch - formed;
        let age = if age >= 10.0 {
            format!("{:.2} Gyr ago", age / 1000.0)
        } else {
            format!("{:.1} Myr ago", age)
        };
        let cause = format!(
            "Lava tubes in the {what}, {age}: slope {slope:.1}° gives a forming strength of {forming:.2} (tubes form best near {best:.0}°, Greeley 1971; Calvari and Pinkerton 1998). Gravity {g:.2} m/s² lets a roof span about {w:.0} m ({ratio:.1} times Earth's, scaled as 1/g: SSE4's inference from Blair et al. 2017). {kept_pc:.0}% of the roof still stands: {imp}{wear}.",
            best = t.slope_best_deg,
            w = width,
            ratio = gscale,
            kept_pc = 100.0 * kept,
            imp = if by_impacts < 0.999 {
                format!("craters deeper than the {roof:.0} m roof have breached {:.0}% of it", 100.0 * (1.0 - by_impacts))
            } else {
                "no crater since has cut through the roof".into()
            },
            wear = if worn_m > 0.0 {
                format!(", and rain has worn about {worn_m:.0} m off the land over it")
            } else {
                String::new()
            },
        ) + &if open < 0.99 {
            format!(" Later resurfacing has buried {:.0}% of the field.", 100.0 * (1.0 - open))
        } else {
            String::new()
        };
        out.push(TubeField {
            id: format!("tubes:{}", out.len()),
            source: src,
            dir,
            radius_km,
            formed_myr: formed,
            slope_deg: slope,
            forming,
            width_m: width,
            width_max_m: w_max,
            roof_m: roof,
            length_km: length,
            roof_kept: kept,
            stage: stage.into(),
            skylights,
            speculative: icy,
            cause: if icy {
                format!("{cause} Cryolava tubes on an ice world are speculative.")
            } else {
                cause
            },
        });
    }
    out
}

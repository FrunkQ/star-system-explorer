//! SG6: prominences (nebulae-and-light design section 4A): cool gas held up
//! in a star's magnetic loops, bright at the limb and dark threads on the
//! disc in hydrogen light; they last days to weeks and some erupt.
//!
//! Everything comes from the star's own conditions, never a "kind" of star:
//! - its mean surface field from its Rossby number (`sse.activity.surface_field`);
//! - its corona's temperature from its X-ray surface flux
//!   (`sse.activity.corona_temperature`) and the density at the corona's
//!   foot from how brightly that gas glows (`sse.activity.corona_base_density`);
//! - loops reach a share of the corona's pressure scale height
//!   (`sse.atmosphere.scale_height`), so a giant's are huge and a red dwarf's low;
//! - the cool gas sits in pressure balance with the corona, so it is denser
//!   by the ratio of the temperatures: that and its size give its mass;
//! - a fast spinner whose co-rotation radius lies inside its closed field
//!   (`sse.orbit.semi_major_axis_from_period`, `sse.activity.closed_corona_radius`)
//!   also holds "slingshot" clouds out there, flung off at the end of their lives.
//!
//! How many there are follows the spot coverage (the active regions they
//! form over), waxing and waning with the cycle. Births are a Poisson
//! process laid out in fixed time bins per kind, like the flares, so any
//! window of time gives the same prominences where windows overlap.

use crate::{addr, cycle_factor, Activity, Settings, Star};
use serde::{Deserialize, Serialize};
use sse_atoms::activity as act;
use sse_core::draw::{Draws, Stream};
use sse_core::math as m;
use sse_core::quantity::consts::*;
use sse_flow::Reference;
use sse_light::LightDescription;

const H_ALPHA_M: f64 = 656.28e-9;

const TH95: &str = "Tandberg-Hanssen, E. 1995, The Nature of Solar Prominences (Kluwer): prominences are gas near 10⁴ K, about a hundred times denser than the corona around them and in rough pressure balance with it; quiescent ones reach tens of thousands of kilometres and last weeks to months, active-region ones are lower and live hours to days; many end by erupting.";
const MACKAY10: &str = "Mackay, D. H., Karpen, J. T., Ballester, J. L., Schmieder, B. and Aulanier, G. 2010, Physics of solar prominences: II. Magnetic structure and dynamics, Space Science Reviews 151, 333: filaments lie along polarity-inversion lines, sheared over several rotations by differential rotation, and form at all latitudes up to the polar crown. https://arxiv.org/abs/1001.1635";

/// Settings for prominences (stand-ins in the fudge register).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct ProminenceSettings {
    /// `sse.activity.surface_field` (Reiners et al. 2022): gauss at Ro = 1 and slopes.
    pub field_fast_g: f64,
    pub field_fast_slope: f64,
    pub field_slow_g: f64,
    pub field_slow_slope: f64,
    /// `sse.activity.corona_temperature` (Johnstone and Güdel 2015).
    pub corona_t_scale: f64,
    pub corona_t_index: f64,
    /// `sse.activity.corona_base_density`: radiative loss, W m³, and mean mass per particle.
    pub corona_loss: f64,
    pub corona_mu: f64,
    /// `sse.activity.closed_corona_radius`: the Sun's source surface, its field (gauss), the growth index.
    pub closed_ref_radii: f64,
    pub closed_ref_field_g: f64,
    pub closed_index: f64,
    /// Temperature of prominence gas, K.
    pub temperature_k: f64,
    /// Mass per hydrogen atom of prominence gas, atomic mass units (helium included).
    pub gas_mu: f64,
    /// Prominences on the disc at once for a star with the Sun's mean spot coverage.
    pub sun_count: f64,
    /// Most at once, however spotted the star.
    pub max_at_once: f64,
    /// Share of loop prominences that are quiescent (the rest form in active regions).
    pub quiescent_share: f64,
    /// Highest latitude quiescent ones form at, degrees (the polar crown).
    pub quiescent_max_lat_deg: f64,
    /// Heights in coronal scale heights, low to high, for active and quiescent ones.
    pub active_height: [f64; 2],
    pub quiescent_height: [f64; 2],
    /// Length along the spine over height.
    pub length_over_height: [f64; 2],
    /// Thickness across the spine over height.
    pub thickness_over_height: f64,
    /// Lives: active ones in supergranule lives, quiescent and slingshot ones in rotations.
    pub active_life_cells: [f64; 2],
    pub quiescent_life_turns: [f64; 2],
    pub slingshot_life_turns: [f64; 2],
    /// Share that end by erupting.
    pub erupt_share_active: f64,
    pub erupt_share_quiescent: f64,
    /// Eruption speed in the star's surface escape speed.
    pub erupt_speed_escape: [f64; 2],
    /// Share of prominences that are slingshot clouds when the star can hold them.
    pub slingshot_share: f64,
    /// Slingshot cloud size over its distance from the centre.
    pub slingshot_size: f64,
    /// Width of the hydrogen-alpha line's glow in Doppler widths, and the other lines' extra light over it.
    pub line_doppler_widths: f64,
    pub other_lines: f64,
    /// The eye's band, m.
    pub band_m: [f64; 2],
    /// Most listed in one answer.
    pub max_listed: usize,
}

impl Default for ProminenceSettings {
    fn default() -> Self {
        ProminenceSettings {
            field_fast_g: 2050.0,
            field_fast_slope: -0.11,
            field_slow_g: 199.0,
            field_slow_slope: -1.26,
            corona_t_scale: 0.11,
            corona_t_index: 0.26,
            corona_loss: 1.2e-35,
            corona_mu: 0.6,
            closed_ref_radii: 2.5,
            closed_ref_field_g: 95.0,
            closed_index: 1.0 / 3.0,
            temperature_k: 8000.0,
            gas_mu: 1.3,
            sun_count: 12.0,
            max_at_once: 60.0,
            quiescent_share: 0.6,
            quiescent_max_lat_deg: 60.0,
            active_height: [0.05, 0.3],
            quiescent_height: [0.2, 1.0],
            length_over_height: [2.0, 6.0],
            thickness_over_height: 0.1,
            active_life_cells: [0.5, 5.0],
            quiescent_life_turns: [1.0, 5.0],
            slingshot_life_turns: [1.0, 4.0],
            erupt_share_active: 0.5,
            erupt_share_quiescent: 0.3,
            erupt_speed_escape: [0.2, 1.5],
            slingshot_share: 0.3,
            slingshot_size: 0.1,
            line_doppler_widths: 2.5,
            other_lines: 0.5,
            band_m: [400e-9, 700e-9],
            max_listed: 3000,
        }
    }
}

/// The star's field and corona, which every prominence reads.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Corona {
    pub surface_field_t: f64,
    pub temperature_k: f64,
    pub base_density_m3: f64,
    pub scale_height_m: f64,
    /// Reach of the closed field, stellar radii.
    pub closed_radii: f64,
    /// Co-rotation radius, stellar radii.
    pub corotation_radii: f64,
    /// Whether slingshot clouds can be held (co-rotation inside the closed field).
    pub slingshots: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Eruption {
    /// When it lifts off (age, s): the end of its life.
    pub at_s: f64,
    pub speed_m_s: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Prominence {
    pub address: String,
    /// active, quiescent or slingshot.
    pub kind: String,
    pub born_s: f64,
    pub life_s: f64,
    /// Middle of its spine on the star, in the star's turning frame at birth, degrees.
    pub lat_deg: f64,
    pub lon_deg: f64,
    /// Angle of the spine from east-west, degrees.
    pub tilt_deg: f64,
    /// Top above the surface (a slingshot cloud's middle), m.
    pub height_m: f64,
    pub length_m: f64,
    pub thickness_m: f64,
    pub density_m3: f64,
    pub mass_kg: f64,
    pub temperature_k: f64,
    /// Its surface brightness in the eye's band over the star disc's mean, seen at the limb.
    pub share_visible: f64,
    /// Its light: hydrogen lines.
    pub light: LightDescription,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub eruption: Option<Eruption>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Prominences {
    pub star: String,
    pub from_s: f64,
    pub to_s: f64,
    /// Every prominence alive before this time is listed (`to_s` unless the list was cut at `max_listed`).
    pub complete_to_s: f64,
    pub corona: Option<Corona>,
    pub prominences: Vec<Prominence>,
    /// Mean number at once over the window's start.
    pub mean_at_once: f64,
    pub why: Vec<String>,
    pub stand_ins: Vec<String>,
    pub references: Vec<Reference>,
}

fn refs_of(ids: &[&str]) -> Vec<Reference> {
    let mut out: Vec<Reference> = vec![];
    for d in sse_atoms::decls_of(ids) {
        for r in d.references {
            if !out.contains(&r) {
                out.push(r);
            }
        }
    }
    out
}

/// The star's field and corona from its activity.
pub fn corona(s: &Star, a: &Activity, p: &ProminenceSettings, set: &Settings) -> Option<Corona> {
    let ro = a.rossby?;
    let b = act::surface_field(
        ro,
        p.field_fast_g,
        p.field_fast_slope,
        p.field_slow_g,
        p.field_slow_slope,
        set.ro_sat,
    );
    let t = act::corona_temperature(a.xray_w, s.radius_m, p.corona_t_scale, p.corona_t_index);
    let g = G * s.mass_kg / (s.radius_m * s.radius_m);
    let n = act::corona_base_density(a.xray_w, s.radius_m, t, g, p.corona_loss, p.corona_mu);
    let h = sse_atoms::air::scale_height(t, p.corona_mu * sse_atoms::air::AMU, g);
    let rc = act::closed_corona_radius(b, p.closed_ref_radii, p.closed_ref_field_g, p.closed_index);
    let rk = act::corotation_radius(s.mass_kg, a.rotation_period_s) / s.radius_m;
    Some(Corona {
        surface_field_t: b,
        temperature_k: t,
        base_density_m3: n,
        scale_height_m: h,
        closed_radii: rc,
        corotation_radii: rk,
        slingshots: rk > 1.2 && rk < rc,
    })
}

/// Dilution of the star's light at `r` stellar radii from its centre.
fn dilution(r: f64) -> f64 {
    let r = r.max(1.0);
    0.5 * (1.0 - m::sqrt(1.0 - 1.0 / (r * r)))
}

/// Hydrogen-alpha radiance of the gas, W m⁻² sr⁻¹ (an optically thick line
/// whose source is the star's own light at that wavelength, diluted with
/// height, over a line `width` wide), and its surface brightness in the eye's
/// band over the disc's mean, the other lines adding their share.
fn radiance(teff_k: f64, gas_t_k: f64, r: f64, p: &ProminenceSettings) -> (f64, f64) {
    // Thermal width of hydrogen, its atom taken as one atomic mass unit (0.8 % light).
    let doppler =
        H_ALPHA_M / C * m::sqrt(2.0 * sse_core::constants::k_b() * gas_t_k / sse_atoms::air::AMU);
    let width = p.line_doppler_widths * doppler;
    let ha = dilution(r) * sse_light::planck(H_ALPHA_M, teff_k) * width;
    let (lo, hi) = (p.band_m[0], p.band_m[1]);
    let steps = 30;
    let mut mean = 0.0;
    for k in 0..=steps {
        mean += sse_light::planck(lo + (hi - lo) * k as f64 / steps as f64, teff_k);
    }
    mean /= (steps + 1) as f64;
    if mean <= 0.0 {
        return (ha, 0.0);
    }
    (ha, (1.0 + p.other_lines) * ha / (mean * (hi - lo)))
}

fn log_between(u: f64, r: [f64; 2]) -> f64 {
    r[0] * m::pow(r[1] / r[0], u)
}

/// Mean of a log-uniform draw between the two.
fn log_mean(r: [f64; 2]) -> f64 {
    if (r[1] - r[0]).abs() < 1e-12 * r[0] {
        r[0]
    } else {
        (r[1] - r[0]) / m::ln(r[1] / r[0])
    }
}

/// Prominences alive at any time between `from_s` and `to_s` (ages).
pub fn prominences(
    seed: &sse_core::Seed,
    s: &Star,
    a: &Activity,
    from_s: f64,
    to_s: f64,
    p: &ProminenceSettings,
    set: &Settings,
) -> Result<Prominences, String> {
    if to_s < from_s {
        return Err("the window must not run backwards".into());
    }
    let mut out = Prominences {
        star: a.address.clone(),
        from_s,
        to_s,
        complete_to_s: to_s,
        corona: None,
        prominences: vec![],
        mean_at_once: 0.0,
        why: vec![],
        stand_ins: vec![
            format!("prominence gas at {:.0} K, {:.1} atomic mass units per hydrogen atom, in pressure balance with the corona", p.temperature_k, p.gas_mu),
            format!("about {} on the disc at once for the Sun's mean spot coverage, in proportion to spot coverage, at most {}", p.sun_count, p.max_at_once),
            format!("heights a share of the corona's scale height ({:?} active, {:?} quiescent), lengths {:?} heights, thickness {} of the height", p.active_height, p.quiescent_height, p.length_over_height, p.thickness_over_height),
            format!("lives: active {:?} supergranule lives, quiescent {:?} turns (sheared apart by differential rotation), slingshot {:?} turns", p.active_life_cells, p.quiescent_life_turns, p.slingshot_life_turns),
            format!("{} of active and {} of quiescent ones erupt at the end, at {:?} of the surface escape speed", p.erupt_share_active, p.erupt_share_quiescent, p.erupt_speed_escape),
            format!("brightness: hydrogen-alpha {} Doppler widths wide lit by the diluted disc, other lines {} more; no Thomson-scattered continuum", p.line_doppler_widths, p.other_lines),
        ],
        references: vec![Reference::parse(TH95), Reference::parse(MACKAY10)],
    };
    for r in refs_of(&[
        "sse.activity.surface_field",
        "sse.activity.corona_temperature",
        "sse.activity.corona_base_density",
        "sse.activity.closed_corona_radius",
        "sse.orbit.semi_major_axis_from_period",
    ]) {
        if !out.references.contains(&r) {
            out.references.push(r);
        }
    }
    if !a.flares {
        out.why.push(
            "No churning outer layer, so no magnetic loops to hold cool gas up: no prominences."
                .into(),
        );
        return Ok(out);
    }
    let Some(c) = corona(s, a, p, set) else {
        return Ok(out);
    };
    // Number at once at the cycle's mean, in proportion to spot coverage.
    let mean_n = (p.sun_count * a.spot_coverage / set.sun_spot_coverage).min(p.max_at_once);
    out.mean_at_once = mean_n;
    let v_esc = m::sqrt(2.0 * G * s.mass_kg / s.radius_m);
    let gas_n = c.base_density_m3 * c.temperature_k / p.temperature_k;
    let cell_life = a
        .convection
        .as_ref()
        .map_or(DAY, |cv| cv.supergranule_lifetime_s);
    let turn = a.rotation_period_s;
    let peak = 1.0
        + if a.cycle.is_some() {
            set.cycle_amplitude
        } else {
            0.0
        };
    let max_h = (c.closed_radii - 1.0).max(0.05) * s.radius_m;
    // Each kind: share of the count, mean life, its own stream.
    let sling = if c.slingshots { p.slingshot_share } else { 0.0 };
    let kinds: [(&str, f64, [f64; 2]); 3] = [
        (
            "active",
            (1.0 - sling) * (1.0 - p.quiescent_share),
            [
                p.active_life_cells[0] * cell_life,
                p.active_life_cells[1] * cell_life,
            ],
        ),
        (
            "quiescent",
            (1.0 - sling) * p.quiescent_share,
            [
                p.quiescent_life_turns[0] * turn,
                p.quiescent_life_turns[1] * turn,
            ],
        ),
        (
            "slingshot",
            sling,
            [
                p.slingshot_life_turns[0] * turn,
                p.slingshot_life_turns[1] * turn,
            ],
        ),
    ];
    let star = addr(&a.address)?;
    for (kind, share, life) in kinds {
        if share <= 0.0 || mean_n <= 0.0 {
            continue;
        }
        let mean_life = log_mean(life);
        // Births per second at the cycle's peak.
        let rate = mean_n * share * peak / mean_life;
        let bin = m::pow(2.0, m::round(m::log2(mean_life / 4.0))).max(60.0);
        let stream = Stream::new(&format!("activity.prominences.{kind}.v1")).map_err(|e| e.0)?;
        let dr = Draws::new(*seed, star.clone(), stream);
        let (b0, b1) = (
            m::floor((from_s - life[1]) / bin) as i64,
            m::floor(to_s / bin) as i64,
        );
        if (b1 - b0) as f64 * rate * bin > 20.0 * p.max_listed as f64 {
            return Err(format!(
                "about {:.0} {kind} prominences in this window: ask for a shorter one",
                (b1 - b0) as f64 * rate * bin
            ));
        }
        for b in b0..=b1 {
            let base = (b as u64).wrapping_mul(4096);
            let n = crate::poisson(rate * bin, dr.unit(base + 4095), 400);
            for k in 0..n {
                let i = base + k * 10;
                let born = (b as f64 + dr.unit(i)) * bin;
                // Still a double date until prominences move to dates (TP1 follow-up);
                // the cycle reads it as the exact value of that double.
                let born_at = sse_core::time::Instant::from_seconds_f64(born)
                    .unwrap_or(sse_core::time::Instant::ORIGIN);
                if dr.unit(i + 1) * peak > cycle_factor(a, born_at, set) {
                    continue;
                }
                let life_s = log_between(dr.unit(i + 2), life);
                if born > to_s || born + life_s < from_s {
                    continue;
                }
                let lon = dr.uniform(i + 3, 0.0, 360.0);
                let (lat, tilt, height, length, thick) = match kind {
                    "slingshot" => {
                        let h = (c.corotation_radii - 1.0) * s.radius_m;
                        let size = p.slingshot_size * c.corotation_radii * s.radius_m;
                        (dr.uniform(i + 4, -10.0, 10.0), 0.0, h, size, size)
                    }
                    _ => {
                        let (lat_max, hr) = if kind == "active" {
                            (set.active_latitude_deg, p.active_height)
                        } else {
                            (p.quiescent_max_lat_deg, p.quiescent_height)
                        };
                        let h = (log_between(dr.unit(i + 5), hr) * c.scale_height_m).min(max_h);
                        let len = log_between(dr.unit(i + 6), p.length_over_height) * h;
                        let tilt = dr.uniform(i + 7, -30.0, 30.0);
                        (
                            dr.uniform(i + 4, -1.0, 1.0) * lat_max,
                            tilt,
                            h,
                            len,
                            p.thickness_over_height * h,
                        )
                    }
                };
                let volume = length * height.min(length) * thick;
                let mass = gas_n * p.gas_mu * sse_atoms::air::AMU * volume;
                let erupt_share = match kind {
                    "active" => p.erupt_share_active,
                    "quiescent" => p.erupt_share_quiescent,
                    _ => 1.0,
                };
                let eruption = if dr.unit(i + 8) < erupt_share {
                    let speed = if kind == "slingshot" {
                        // Flung off at the speed it is whirled round at.
                        m::TAU * c.corotation_radii * s.radius_m / turn
                    } else {
                        log_between(dr.unit(i + 9), p.erupt_speed_escape) * v_esc
                    };
                    Some(Eruption {
                        at_s: born + life_s,
                        speed_m_s: speed,
                    })
                } else {
                    None
                };
                let r_mid = 1.0 + 0.5 * height / s.radius_m;
                let (ha, share) = radiance(s.teff_k, p.temperature_k, r_mid, p);
                // Its glow seen side on, one face (π steradians' worth of a flat glowing sheet).
                let face = m::PI * ha * length * height.min(length);
                let mut light = LightDescription::new("");
                for (line, w) in [("ha", 1.0), ("hb", p.other_lines)] {
                    light.lines.push(sse_light::LineEmission {
                        line: line.into(),
                        power_w: w * face,
                        cause: "prominence".into(),
                        zone: None,
                    });
                }
                light.why.push(format!(
                    "Cool hydrogen at {:.0} K held in the star's field, glowing red in hydrogen-alpha (and blue-green in hydrogen-beta) as it scatters the star's own light: about {:.1e} of the disc's brightness to the eye.",
                    p.temperature_k, share
                ));
                out.prominences.push(Prominence {
                    address: String::new(),
                    kind: kind.into(),
                    born_s: born,
                    life_s,
                    lat_deg: lat,
                    lon_deg: lon,
                    tilt_deg: tilt,
                    height_m: height,
                    length_m: length,
                    thickness_m: thick,
                    density_m3: gas_n,
                    mass_kg: mass,
                    temperature_k: p.temperature_k,
                    share_visible: share,
                    light,
                    eruption,
                });
            }
        }
    }
    out.prominences
        .sort_by(|x, y| x.born_s.total_cmp(&y.born_s).then(x.kind.cmp(&y.kind)));
    // A long list keeps the earliest born; the list is whole up to the first one left out.
    if out.prominences.len() > p.max_listed {
        out.complete_to_s = out.prominences[p.max_listed].born_s.max(from_s);
        out.prominences.truncate(p.max_listed);
        out.why.push(format!(
            "The list is cut at {} prominences: it is whole for the first {:.1} days of the window.",
            p.max_listed,
            (out.complete_to_s - from_s) / DAY
        ));
    }
    for (n, pr) in out.prominences.iter_mut().enumerate() {
        pr.address = format!("{}/prominence:{}", a.address, n);
        pr.light.address = pr.address.clone();
    }
    out.why.push(format!(
        "Its field averages about {:.0} gauss over the surface; its corona is near {:.1} million K with {:.1e} particles per cubic metre at its foot and a scale height of {}. Cool gas held in its loops sits {:.0} times denser than the corona around it.",
        c.surface_field_t * 1e4,
        c.temperature_k / 1e6,
        c.base_density_m3,
        crate::length(c.scale_height_m),
        c.temperature_k / p.temperature_k
    ));
    out.why.push(format!(
        "About {:.1} prominences at once on average, following its spots: {} alive in this window.",
        mean_n,
        out.prominences.len()
    ));
    if c.slingshots {
        out.why.push(format!(
            "It spins so fast that gas {:.1} radii out orbits as fast as it turns, inside the {:.1} radii its field holds closed: huge cool clouds are slung round there and flung off.",
            c.corotation_radii, c.closed_radii
        ));
    }
    out.corona = Some(c);
    Ok(out)
}

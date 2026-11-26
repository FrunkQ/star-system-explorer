//! XO3: a disc's light, point by point, as a far viewer at a stated tilt
//! sees it (exotic objects design 3.3). The picture's geometry and the
//! spectrum chart's number come from the same three facts, so they agree:
//!
//! 1. **Where each ray lands.** A ray reaching the viewer at impact parameter
//!    b and angle α round the picture lies in one plane through the hole.
//!    That plane crosses the disc's plane after sweeping φ₀ = π/2 +
//!    atan(cos α · tan i) (i the tilt from the disc's axis), and again every
//!    half turn after. The bend table (`sse-gravity`, XO1) gives the ray's
//!    distance r at each crossing: the first crossing inside the disc's
//!    edges is what the viewer sees there (order 0 the direct image and the
//!    arch over the top, order 1 the thin ring hugging the shadow). The disc
//!    is opaque, so a ray stops at its first hit. Beyond the table's edge
//!    (b over 64 gravitational radii) rays are taken as straight.
//! 2. **How hot it is there**: the Shakura and Sunyaev ring temperature
//!    ([`crate::disc_temperature`], Newtonian brightness law, logged).
//! 3. **How shifted the light is**: the formula block `sse.light.disc_shift`
//!    (Luminet 1979), the same expression a shader evaluates.
//!
//! A glowing surface at T seen through a shift g looks exactly like one
//! glowing at gT (the specific intensity I_ν/ν³ is unchanged along a ray),
//! so each piece of the picture is a blackbody at gT. Summed over the
//! picture, a far viewer receives flux F = Σ I dΩ, which is an
//! isotropic-equivalent luminosity 4π d² F = Σ 4 dA · π B(gT) with dA the
//! piece's area in the picture plane. The pieces are binned by seen
//! temperature (64 bins a decade, each keeping its total σT⁴ exactly) into
//! one [`Part::Thermals`], so the existing band recipes colour it with no
//! new colour code.

use crate::feed::run_block;
use crate::*;
use serde::{Deserialize, Serialize};
use sse_gravity::bend::{self, BendTable};
use std::collections::BTreeMap;

/// Image orders traced: the direct image (with the arch) and the first ring.
/// Higher orders are thinner than a pixel (fudge register).
pub const ORDERS: usize = 2;
/// Steps round the picture.
pub const ANGLES: usize = 192;
/// Picture-plane steps per decade of impact parameter, outside [`FINE_TO`].
pub const STEPS_PER_DECADE: f64 = 48.0;
/// Inside this impact parameter (in r_g) the steps are even, [`FINE_STEP`]
/// apart, so the thin ring near √27 gets several steps.
pub const FINE_TO: f64 = 12.0;
pub const FINE_STEP: f64 = 0.04;
/// Seen-temperature bins per decade in the published spectrum.
pub const BINS_PER_DECADE: f64 = 64.0;

pub const STRAIGHT_FAR: &str = "rays passing more than 64 gravitational radii from the hole are drawn straight: their bending moves the far disc by under 6% of its distance (fudge register: exotic objects, table edges)";
pub const ORDERS_STAND_IN: &str = "only the direct image, the arch and the first thin ring are traced; higher rings are thinner than a pixel (fudge register: exotic objects)";
pub const R_LUMINET: &str = "Luminet, J.-P. 1979, Image of a spherical black hole with thin accretion disk, A&A 75, 228: the image of a thin disc by ray tracing, the shift factor and the bolometric picture";

/// One point of the picture.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spot {
    /// 0 direct image (and arch), 1 the thin ring.
    pub order: usize,
    pub impact_rg: f64,
    pub image_angle_rad: f64,
    /// Where on the disc the ray came from, in gravitational radii.
    pub r_rg: f64,
    pub shift: f64,
    pub t_emitted_k: f64,
    pub t_seen_k: f64,
    /// Bolometric brightness seen there, W m⁻² sr⁻¹.
    pub intensity_w_m2_sr: f64,
}

/// How far a picture must be dimmed to show its detail.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Exposure {
    /// Visible-band brightness of the brightest point, W m⁻² sr⁻¹.
    pub brightest_visible_w_m2_sr: f64,
    /// The same for the Sun's surface (the sheet's T_sun).
    pub sun_surface_visible_w_m2_sr: f64,
    /// Brightest point over the Sun's surface, in visible light.
    pub times_sun_surface: f64,
    pub note: String,
}

/// The disc as one viewer sees it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DiscImage {
    pub tilt_rad: f64,
    pub gravitational_radius_m: f64,
    /// Isotropic-equivalent luminosity for this viewer, W.
    pub luminosity_w: f64,
    /// Share of the light in each image order.
    pub order_shares: Vec<f64>,
    pub brightest: Spot,
    /// Brightest point on the side turning towards the viewer (left).
    pub approaching: Spot,
    /// Brightest point on the side turning away (right).
    pub receding: Spot,
    /// Approaching over receding brightness.
    pub side_ratio: f64,
    pub exposure: Exposure,
    /// [seen temperature K, area m²] pieces, as `Part::Thermals` carries them.
    pub pieces: Vec<[f64; 2]>,
    pub why: Vec<String>,
    pub stand_ins: Vec<String>,
}

/// Picture-plane impact parameter steps: (centre, width), in r_g.
fn b_steps(b_hi: f64) -> Vec<(f64, f64)> {
    let mut edges = vec![];
    let n_fine = m::ceil(FINE_TO / FINE_STEP) as usize;
    for k in 0..=n_fine {
        edges.push(k as f64 * FINE_STEP);
    }
    let top = edges[n_fine];
    if b_hi > top {
        let n = m::ceil(m::log10(b_hi / top) * STEPS_PER_DECADE) as usize;
        for k in 1..=n {
            edges.push(top * m::pow(10.0, k as f64 / STEPS_PER_DECADE));
        }
    }
    edges
        .windows(2)
        .map(|w| (0.5 * (w[0] + w[1]), w[1] - w[0]))
        .collect()
}

/// Where a ray of impact parameter `b` (r_g) at picture angle `alpha`
/// first lands on the disc, for a viewer tilted `tilt` from its axis:
/// (order, r in r_g). `None` when it misses.
pub fn landing(
    table: &BendTable,
    b: f64,
    alpha: f64,
    tilt: f64,
    r_in: f64,
    r_out: f64,
) -> Option<(usize, f64)> {
    let phi0 = m::PI / 2.0 + m::atan(m::cos(alpha) * m::tan(tilt));
    for n in 0..ORDERS {
        let phi = phi0 + n as f64 * m::PI;
        let u = if b <= bend::B_MAX {
            table.u_at(b, phi)?
        } else if n == 0 {
            m::sin(phi) / b
        } else {
            return None;
        };
        if u <= 0.0 || u >= bend::U_HORIZON * 0.999 {
            // Escaped before this crossing, or swallowed.
            return None;
        }
        let r = 1.0 / u;
        if (r_in..=r_out).contains(&r) {
            return Some((n, r));
        }
    }
    None
}

/// Trace a thin disc round `mass_kg` fed at `rate_kg_s`, from `r_inner_m`
/// to `r_outer_m`, seen from `tilt_rad` off its axis.
pub fn trace(
    mass_kg: f64,
    rate_kg_s: f64,
    r_inner_m: f64,
    r_outer_m: f64,
    tilt_rad: f64,
    table: &BendTable,
    light: &LightTable,
) -> Result<DiscImage, String> {
    let rg = run_block("sse.gravity.gravitational_radius", &[mass_kg])?[0];
    let (r_in, r_out) = (r_inner_m / rg, r_outer_m / rg);
    if !(r_in > 0.0 && r_out > r_in) {
        return Err("a disc needs an outer edge beyond its inner edge".into());
    }
    let tilt = tilt_rad.clamp(0.0, m::PI / 2.0 - 1e-6);
    let dalpha = m::TAU / ANGLES as f64;
    let mut bins: BTreeMap<i64, (f64, f64)> = BTreeMap::new();
    let mut by_order = [0.0f64; ORDERS];
    let mut best: Option<Spot> = None;
    let mut left: Option<Spot> = None;
    let mut right: Option<Spot> = None;
    let keep = |slot: &mut Option<Spot>, s: &Spot| {
        if slot
            .as_ref()
            .is_none_or(|o| s.intensity_w_m2_sr > o.intensity_w_m2_sr)
        {
            *slot = Some(s.clone());
        }
    };
    for (b, db) in b_steps(r_out * 1.02) {
        let area = 4.0 * b * db * dalpha * rg * rg;
        for j in 0..ANGLES {
            let alpha = -m::PI + (j as f64 + 0.5) * dalpha;
            let Some((order, r)) = landing(table, b, alpha, tilt, r_in, r_out) else {
                continue;
            };
            let t = disc_temperature(mass_kg, rate_kg_s, r_inner_m, r * rg);
            if t <= 0.0 {
                continue;
            }
            let g = run_block("sse.light.disc_shift", &[1.0, r, b, tilt, alpha])?[0];
            if g <= 0.0 {
                continue;
            }
            let ts = g * t;
            let t4 = ts * ts * ts * ts;
            let power = area * SIGMA * t4;
            by_order[order] += power;
            let key = m::floor(m::log10(ts) * BINS_PER_DECADE) as i64;
            let e = bins.entry(key).or_insert((0.0, 0.0));
            e.0 += area;
            e.1 += area * t4;
            let spot = Spot {
                order,
                impact_rg: b,
                image_angle_rad: alpha,
                r_rg: r,
                shift: g,
                t_emitted_k: t,
                t_seen_k: ts,
                intensity_w_m2_sr: SIGMA * t4 / m::PI,
            };
            keep(&mut best, &spot);
            if alpha > 0.0 {
                keep(&mut left, &spot);
            } else {
                keep(&mut right, &spot);
            }
        }
    }
    let (Some(brightest), Some(approaching), Some(receding)) = (best, left, right) else {
        return Err("no ray lands on the disc: it is too small to trace".into());
    };
    let total: f64 = by_order.iter().sum();
    let pieces: Vec<[f64; 2]> = bins
        .values()
        .filter(|(a, _)| *a > 0.0)
        .map(|(a, at4)| [m::sqrt(m::sqrt(at4 / a)), *a])
        .collect();
    let side_ratio = approaching.intensity_w_m2_sr / receding.intensity_w_m2_sr;
    let exposure = exposure(&brightest, light)?;
    let tilt_deg = tilt * 180.0 / m::PI;
    let why = vec![
        format!(
            "Seen {tilt_deg:.0}° from the disc's axis, traced ray by ray through the bend table: the side turning towards the viewer is {side_ratio:.1} times as bright as the side turning away."
        ),
        format!(
            "Its brightest point is {:.1} gravitational radii out, glowing at {:.3e} K but seen at {:.3e} K (shifted by {:.3}).",
            brightest.r_rg, brightest.t_emitted_k, brightest.t_seen_k, brightest.shift
        ),
        exposure.note.clone(),
    ];
    Ok(DiscImage {
        tilt_rad: tilt,
        gravitational_radius_m: rg,
        luminosity_w: total,
        order_shares: by_order.iter().map(|p| p / total).collect(),
        brightest,
        approaching,
        receding,
        side_ratio,
        exposure,
        pieces,
        why,
        stand_ins: vec![ORDERS_STAND_IN.into(), STRAIGHT_FAR.into()],
    })
}

fn visible_radiance(t_k: f64, light: &LightTable) -> Result<f64, String> {
    let v = light
        .band("visible")
        .ok_or("the light table has no visible band")?;
    Ok(integrate(
        |l| planck(l, t_k),
        v.from_m,
        v.to_m,
        light.steps_per_decade,
    ))
}

/// The exposure note: how much brighter than the Sun's surface the
/// brightest point is in visible light.
fn exposure(brightest: &Spot, light: &LightTable) -> Result<Exposure, String> {
    let lit = visible_radiance(brightest.t_seen_k, light)?;
    let sun = visible_radiance(sse_core::sheet::T_SUN, light)?;
    let x = lit / sun;
    let note = if x >= 1.0 {
        format!("In visible light its brightest point is {x:.3e} times as bright as the Sun's surface: far too bright to look at, so a picture is dimmed about that much to show its detail.")
    } else {
        format!("In visible light its brightest point is {x:.3e} of the Sun's surface brightness.")
    };
    Ok(Exposure {
        brightest_visible_w_m2_sr: lit,
        sun_surface_visible_w_m2_sr: sun,
        times_sun_surface: x,
        note,
    })
}

/// The traced disc as a light description's part.
pub fn part(img: &DiscImage) -> Part {
    Part::Thermals {
        pieces: img.pieces.clone(),
        cause: format!(
            "accretion disc seen {:.0}° from its axis",
            img.tilt_rad * 180.0 / m::PI
        ),
    }
}

pub fn luminet_reference() -> Reference {
    Reference {
        text: R_LUMINET.into(),
        doi: None,
        url: Some("https://ui.adsabs.harvard.edu/abs/1979A&A....75..228L".into()),
    }
}

//! S5's hazard layer: is this spot fit to land on? Read from the one height
//! map (the atlas) at the level whose sample spacing resolves the site, so
//! the answer agrees with every tile drawn there.
//!
//! - **Slope**: a least-squares plane through 25 samples (the centre and three
//!   rings of eight) across the site; the tilt of that plane, and the
//!   steepest step between neighbouring samples at the finest spacing.
//! - **Roughness**: the RMS of the samples about that plane.
//! - **Liquid**: below the datum on a world with surface liquid, the landing
//!   is a splashdown.
//! - **Air at the site**: density and sound speed from the cover's air
//!   profile, for the descent to read.

use crate::atlas::Atlas;
use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// The site limits a lander is judged against (pack data, `nav.descent`).
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct SiteLimits {
    pub slope_limit_deg: f64,
    pub roughness_limit_m: f64,
    pub site_radius_m: f64,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct SiteReport {
    pub lat_deg: f64,
    pub lon_deg: f64,
    /// Atlas level read, and its sample spacing.
    pub level: u8,
    pub spacing_m: f64,
    /// Height above the datum (sea level where there is a sea), m.
    pub height_m: f64,
    /// Tilt of the best-fit plane across the site, degrees.
    pub slope_deg: f64,
    /// Steepest step between neighbouring samples, degrees.
    pub local_slope_deg: f64,
    /// RMS about the plane, m.
    pub roughness_m: f64,
    /// Highest minus lowest sample, m.
    pub relief_m: f64,
    /// The liquid the site lies under, if any.
    pub under_liquid: Option<String>,
    pub ok: bool,
    pub why: Vec<String>,
}

fn from_lat_lon(lat_deg: f64, lon_deg: f64) -> [f64; 3] {
    let (la, lo) = (lat_deg.to_radians(), lon_deg.to_radians());
    [m::cos(la) * m::cos(lo), m::cos(la) * m::sin(lo), m::sin(la)]
}

/// Direction `x` m east and `y` m north of the site on a sphere of radius `r`.
fn offset(lat_deg: f64, lon_deg: f64, x: f64, y: f64, r: f64) -> [f64; 3] {
    let la = lat_deg + (y / r).to_degrees();
    let c = m::cos(lat_deg.to_radians()).max(1e-6);
    let lo = lon_deg + (x / (r * c)).to_degrees();
    from_lat_lon(la, lo)
}

/// Rings across the site.
pub const RINGS: u32 = 3;
/// Samples per ring.
pub const SPOKES: u32 = 8;

/// Judge a landing site.
pub fn site(a: &Atlas, lat_deg: f64, lon_deg: f64, lim: &SiteLimits) -> SiteReport {
    let r = a.radius_m;
    let step = lim.site_radius_m / RINGS as f64;
    let level = a.level_for(step);
    let spacing = a.spacing_m(level);
    let h_at = |x: f64, y: f64| a.height_at(offset(lat_deg, lon_deg, x, y, r), level);
    let mut pts = vec![(0.0, 0.0, h_at(0.0, 0.0))];
    for ring in 1..=RINGS {
        let rad = step * ring as f64;
        for k in 0..SPOKES {
            let th = m::TAU * k as f64 / SPOKES as f64;
            let (x, y) = (rad * m::cos(th), rad * m::sin(th));
            pts.push((x, y, h_at(x, y)));
        }
    }
    // Least-squares plane h = c0 + bx x + by y. The rings are symmetric, so the
    // normal equations are diagonal: Σx = Σy = Σxy = 0.
    let n = pts.len() as f64;
    let mean = pts.iter().map(|p| p.2).sum::<f64>() / n;
    let sxx: f64 = pts.iter().map(|p| p.0 * p.0).sum();
    let syy: f64 = pts.iter().map(|p| p.1 * p.1).sum();
    let bx = pts.iter().map(|p| p.0 * p.2).sum::<f64>() / sxx;
    let by = pts.iter().map(|p| p.1 * p.2).sum::<f64>() / syy;
    let resid2: f64 = pts
        .iter()
        .map(|p| {
            let e = p.2 - (mean + bx * p.0 + by * p.1);
            e * e
        })
        .sum();
    let roughness = m::sqrt(resid2 / n);
    let slope = m::atan(m::sqrt(bx * bx + by * by)).to_degrees();
    // Steepest step at the finest spacing around the centre.
    let fine = spacing.max(1.0);
    let h0 = pts[0].2;
    let mut local: f64 = 0.0;
    for k in 0..SPOKES {
        let th = m::TAU * k as f64 / SPOKES as f64;
        let h = h_at(fine * m::cos(th), fine * m::sin(th));
        local = local.max(m::atan((h - h0).abs() / fine).to_degrees());
    }
    let (lo, hi) = pts
        .iter()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), p| {
            (lo.min(p.2), hi.max(p.2))
        });
    let liquid = a
        .layers()
        .and_then(|l| l.climate.liquid.clone())
        .filter(|_| h0 < 0.0);
    let mut why = vec![format!(
        "read at atlas level {level} ({:.0} m samples) across {:.0} m",
        spacing, lim.site_radius_m
    )];
    let mut ok = true;
    if slope.max(local) > lim.slope_limit_deg {
        ok = false;
        why.push(format!(
            "too steep: {:.1}° across the site, {:.1}° locally (limit {:.0}°)",
            slope, local, lim.slope_limit_deg
        ));
    } else {
        why.push(format!(
            "slope {:.1}° across the site, {:.1}° locally",
            slope, local
        ));
    }
    if roughness > lim.roughness_limit_m {
        ok = false;
        why.push(format!(
            "too rough: {:.1} m about the mean ground (limit {:.0} m)",
            roughness, lim.roughness_limit_m
        ));
    } else {
        why.push(format!("ground smooth to {:.1} m", roughness));
    }
    if let Some(l) = &liquid {
        ok = false;
        why.push(format!("under {:.0} m of {l}: a splashdown", -h0));
    }
    SiteReport {
        lat_deg,
        lon_deg,
        level,
        spacing_m: spacing,
        height_m: h0,
        slope_deg: slope,
        local_slope_deg: local,
        roughness_m: roughness,
        relief_m: hi - lo,
        under_liquid: liquid,
        ok,
        why,
    }
}

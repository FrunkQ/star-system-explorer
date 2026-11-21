//! Reconstructed origins (top-level design 17.1): when a GM makes one
//! system older or younger without moving the campaign's now, work back a
//! plausible birthplace and path for its star, so the universe corrects
//! itself around the worldbuilder's tweak.
//!
//! The pins are where the star is now (its guide radius and azimuth), its
//! new age and, when its composition is kept, its [Fe/H]. Stars are born
//! where the disc was forming stars, and drift in radius afterwards by
//! churning at spiral corotation (Sellwood and Binney 2002), which changes
//! the guide radius without heating the orbit. The spread of that drift
//! grows as the square root of age (Frankel et al. 2018: 3.6 kpc over 8 Gyr).
//! So the weight of a birth radius R is
//!
//! `N(R; R_now, sigma(age)) x SFR(R, t_birth) x R x [N(feh; feh_gas(R, t_birth), feh_sd)]`
//!
//! (the last factor only when the composition is kept), and the answer is
//! the most likely R: one plausible past, deterministic, no draws.
//! IMPOSSIBLE only when no radius fits: born before the disc formed stars,
//! or at a time when nowhere in the disc was forming any.

use crate::envelope::GalaxyEnvelope;
use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// Settings (`provides.other.cosmology.origin`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(default)]
pub struct OriginParams {
    /// Spread of the guide-radius drift after `mig_ref_gyr`, kpc (Frankel et al. 2018).
    pub mig_sigma_kpc: f64,
    pub mig_ref_gyr: f64,
    /// Scatter of a star's [Fe/H] about the gas it formed from, dex.
    pub feh_sd: f64,
    /// A kept composition further than this many `feh_sd` from the gas at
    /// the most likely birthplace is marked unusual for its age.
    pub unusual_sd: f64,
    /// Radius grid step, kpc.
    pub step_kpc: f64,
    /// Points on the returned path (birth to now).
    pub path_points: usize,
    /// Circular speed for the azimuth along the path, km/s (flat rotation curve).
    pub rotation_kms: f64,
}

impl Default for OriginParams {
    fn default() -> Self {
        OriginParams {
            mig_sigma_kpc: 3.6,
            mig_ref_gyr: 8.0,
            feh_sd: 0.1,
            unusual_sd: 3.0,
            step_kpc: 0.05,
            path_points: 9,
            rotation_kms: 220.0,
        }
    }
}

/// What is pinned.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct OriginAsk {
    /// Galaxy time of the campaign's now, Gyr since the Big Bang.
    pub now_gyr: f64,
    /// The star's new age at that now, Gyr.
    pub age_gyr: f64,
    /// Where the star's orbit is now: guide radius (kpc) and azimuth (degrees).
    pub r_now_kpc: f64,
    pub phi_now_deg: f64,
    /// `Some([Fe/H])` keeps the composition (and weighs birthplaces by it);
    /// `None` lets it follow the birthplace.
    pub keep_feh: Option<f64>,
}

/// One plausible past.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Origin {
    pub label: String,
    pub birth_gyr: f64,
    pub age_gyr: f64,
    pub birth_r_kpc: f64,
    pub birth_phi_deg: f64,
    pub now_r_kpc: f64,
    /// Now minus birth radius: positive means it has drifted outward.
    pub migrated_kpc: f64,
    /// The drift spread for this age, kpc.
    pub sigma_kpc: f64,
    /// Gas [Fe/H] at the birthplace and time.
    pub feh_gas: f64,
    /// The star's [Fe/H]: kept, or the birthplace's gas.
    pub feh: f64,
    /// `follows birthplace` or `kept`.
    pub composition: String,
    /// Why this composition is unusual for its age, when it is.
    pub unusual: Option<String>,
    /// Why no origin fits, when none does.
    pub impossible: Option<String>,
    /// `[t_gyr, r_kpc, phi_deg]` from birth to now.
    pub path: Vec<[f64; 3]>,
    pub why: String,
    pub references: Vec<String>,
}

const KMS_KPC_GYR: f64 = 1.022_712;

fn wrap_deg(d: f64) -> f64 {
    let x = d - 360.0 * m::floor(d / 360.0);
    if x >= 360.0 {
        0.0
    } else {
        x
    }
}

fn fmt_feh(x: f64) -> String {
    format!("{:+.2}", x)
}

/// Work back a birthplace and path (deterministic).
pub fn reconstruct(env: &GalaxyEnvelope, p: &OriginParams, a: &OriginAsk) -> Origin {
    let t_b = a.now_gyr - a.age_gyr;
    let sigma = p.mig_sigma_kpc * m::sqrt((a.age_gyr / p.mig_ref_gyr).max(0.0));
    let rings = &env.disc.rings;
    let r_in = rings.first().map_or(0.5, |g| g.inner_kpc.max(p.step_kpc));
    let r_out = rings.last().map_or(20.0, |g| g.outer_kpc);
    let refs = vec![
        "Sellwood, J. A. and Binney, J. J. 2002, MNRAS 336, 785, radial mixing in galactic discs, https://doi.org/10.1046/j.1365-8711.2002.05806.x".to_string(),
        "Frankel, N. et al. 2018, ApJ 865, 96, the radial migration strength of the Milky Way disc, https://doi.org/10.3847/1538-4357/aadba5".to_string(),
    ];
    let base = |impossible: Option<String>, why: String| Origin {
        label: "one plausible past".into(),
        birth_gyr: t_b,
        age_gyr: a.age_gyr,
        birth_r_kpc: a.r_now_kpc,
        birth_phi_deg: wrap_deg(a.phi_now_deg),
        now_r_kpc: a.r_now_kpc,
        migrated_kpc: 0.0,
        sigma_kpc: sigma,
        feh_gas: 0.0,
        feh: a.keep_feh.unwrap_or(0.0),
        composition: if a.keep_feh.is_some() {
            "kept".into()
        } else {
            "follows birthplace".into()
        },
        unusual: None,
        impossible,
        path: vec![],
        why,
        references: refs.clone(),
    };
    if a.age_gyr < 0.0 {
        let why = "a star cannot have a negative age".to_string();
        return base(Some(why.clone()), why);
    }
    if t_b <= 0.0 {
        let why = format!(
            "born {:.2} Gyr ago would be before the Big Bang ({:.2} Gyr ago)",
            a.age_gyr, a.now_gyr
        );
        return base(Some(why.clone()), why);
    }
    if t_b < env.disc.t_start_gyr {
        let why = format!(
            "born {:.2} Gyr ago would be before this galaxy's disc formed any stars ({:.2} Gyr ago)",
            a.age_gyr,
            a.now_gyr - env.disc.t_start_gyr
        );
        return base(Some(why.clone()), why);
    }
    // The most likely birth radius on a fixed grid.
    let n = m::floor((r_out - r_in) / p.step_kpc) as usize;
    let mut best: Option<(f64, f64, f64)> = None; // (weight, r, feh_gas)
    let mut any_sfr = false;
    for k in 0..=n {
        let r = r_in + p.step_kpc * k as f64;
        let s = env.surface(r, t_b);
        let sfr = s.sfr.max(0.0);
        if sfr > 0.0 {
            any_sfr = true;
        }
        let d = (r - a.r_now_kpc) / sigma.max(1.0e-6);
        let mut w = m::exp(-0.5 * d * d) * sfr * r;
        if let Some(f) = a.keep_feh {
            let q = (f - s.feh) / p.feh_sd;
            w *= m::exp(-0.5 * q * q);
        }
        if w > 0.0 && best.is_none_or(|b| w > b.0) {
            best = Some((w, r, s.feh));
        }
    }
    if !any_sfr {
        let why = format!(
            "nowhere in the disc was forming stars {:.2} Gyr ago",
            a.age_gyr
        );
        return base(Some(why.clone()), why);
    }
    let (r_b, feh_gas) = match best {
        Some((_, r, f)) => (r, f),
        None => {
            // A kept composition so far from every birthplace's gas that the
            // weights underflow: fall back to the drift and star formation
            // alone, and say so below.
            let mut b: Option<(f64, f64, f64)> = None;
            for k in 0..=n {
                let r = r_in + p.step_kpc * k as f64;
                let s = env.surface(r, t_b);
                let d = (r - a.r_now_kpc) / sigma.max(1.0e-6);
                let w = m::exp(-0.5 * d * d) * s.sfr.max(0.0) * r;
                if w > 0.0 && b.is_none_or(|x| w > x.0) {
                    b = Some((w, r, s.feh));
                }
            }
            b.map_or((a.r_now_kpc, 0.0), |x| (x.1, x.2))
        }
    };
    let feh = a.keep_feh.unwrap_or(feh_gas);
    let unusual = a.keep_feh.and_then(|f| {
        let q = (f - feh_gas) / p.feh_sd;
        (q.abs() > p.unusual_sd).then(|| {
            format!(
                "its kept [Fe/H] {} is {:.1} sigma from the gas it would have formed from ({} at {:.1} kpc, {:.2} Gyr ago)",
                fmt_feh(f),
                q.abs(),
                fmt_feh(feh_gas),
                r_b,
                a.age_gyr
            )
        })
    });
    // The path: the guide radius drifts from birth to now (its mean course,
    // linear in time), the azimuth advances at the flat-curve rate.
    let np = p.path_points.max(2);
    let omega = |r: f64| p.rotation_kms * KMS_KPC_GYR / r.max(0.1); // rad per Gyr
    let r_at = |t: f64| {
        if a.age_gyr <= 0.0 {
            a.r_now_kpc
        } else {
            r_b + (a.r_now_kpc - r_b) * (t - t_b) / a.age_gyr
        }
    };
    let ts: Vec<f64> = (0..np)
        .map(|k| t_b + a.age_gyr * k as f64 / (np - 1) as f64)
        .collect();
    // Integrate the azimuth backwards from now (trapezoid, 64 steps a segment).
    let mut phi = vec![0.0; np];
    phi[np - 1] = a.phi_now_deg * m::PI / 180.0;
    for k in (0..np - 1).rev() {
        let (t0, t1) = (ts[k], ts[k + 1]);
        let steps = 64;
        let h = (t1 - t0) / steps as f64;
        let mut acc = 0.0;
        for j in 0..steps {
            let ta = t0 + h * j as f64;
            acc += 0.5 * h * (omega(r_at(ta)) + omega(r_at(ta + h)));
        }
        phi[k] = phi[k + 1] - acc;
    }
    let path: Vec<[f64; 3]> = ts
        .iter()
        .zip(&phi)
        .map(|(t, f)| [*t, r_at(*t), wrap_deg(*f * 180.0 / m::PI)])
        .collect();
    let migrated = a.r_now_kpc - r_b;
    let drift = if migrated.abs() < 0.05 {
        "close to where its orbit is now".to_string()
    } else if migrated > 0.0 {
        format!(
            "{:.1} kpc nearer the centre than its orbit now, and has drifted outward since",
            migrated
        )
    } else {
        format!(
            "{:.1} kpc further out than its orbit now, and has drifted inward since",
            -migrated
        )
    };
    let comp = match a.keep_feh {
        None => format!(
            "Its composition follows that birthplace: [Fe/H] {}.",
            fmt_feh(feh)
        ),
        Some(f) => format!(
            "Its composition was kept ([Fe/H] {}); the gas there had {}.",
            fmt_feh(f),
            fmt_feh(feh_gas)
        ),
    };
    let why = format!(
        "Formed {:.2} Gyr ago at {:.1} kpc from the centre, {}. Stars wander in radius by churning at spiral corotation; over {:.2} Gyr the spread is about {:.1} kpc. That place was forming stars then, and is the most likely birthplace given where the star is now{}. {} One plausible past.",
        a.age_gyr,
        r_b,
        drift,
        a.age_gyr,
        sigma,
        if a.keep_feh.is_some() {
            " and its kept composition"
        } else {
            ""
        },
        comp
    );
    Origin {
        label: "one plausible past".into(),
        birth_gyr: t_b,
        age_gyr: a.age_gyr,
        birth_r_kpc: r_b,
        birth_phi_deg: path[0][2],
        now_r_kpc: a.r_now_kpc,
        migrated_kpc: migrated,
        sigma_kpc: sigma,
        feh_gas,
        feh,
        composition: if a.keep_feh.is_some() {
            "kept".into()
        } else {
            "follows birthplace".into()
        },
        unusual,
        impossible: None,
        path,
        why,
        references: refs,
    }
}

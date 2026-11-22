//! Y2: `sky_from_here`, the stars seen from a place at a time (night sky
//! design section 4.3).
//!
//! The walk (which stars) is `sse_cosmos::sky`; this module gives each star
//! its light. A system's components are summed as blackbody photospheres
//! (`sse_light::planck`), seen through the dust column along its own sight
//! line, in the light it sent when it left: its position is read at the
//! time of emission, so a star 1 kpc away is placed where it was 3,262
//! years ago.

use crate::{err, Response};
use sse_core::math as m;
use sse_core::quantity::consts::{C, JULIAN_YEAR, L_SUN};
use sse_cosmos::sky::SkyParams;
use sse_cosmos::stars::{StarSystem, Stars};
use sse_light::photometry::{photometry_of, Sight};

type Out = Result<serde_json::Value, Response>;

pub(crate) struct Ask<'a> {
    pub observer: Option<&'a str>,
    pub position_pc: Option<[f64; 3]>,
    pub t_myr: f64,
    pub limit_mag: f64,
    pub band: &'a str,
    pub bands: Vec<String>,
    pub cone: Option<crate::protocol::SkyCone>,
    pub max_stars: usize,
}

fn unit(v: [f64; 3]) -> [f64; 3] {
    let n = m::sqrt(v[0] * v[0] + v[1] * v[1] + v[2] * v[2]);
    if n > 0.0 {
        [v[0] / n, v[1] / n, v[2] / n]
    } else {
        [1.0, 0.0, 0.0]
    }
}
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The observer's sky axes: x toward the galaxy's centre in the plane, z
/// toward the galaxy's north (+z), y completing a right-handed set.
fn axes(obs: [f64; 3]) -> [[f64; 3]; 3] {
    let ex = unit([-obs[0], -obs[1], 0.0]);
    let ez = [0.0, 0.0, 1.0];
    let ey = [
        ez[1] * ex[2] - ez[2] * ex[1],
        ez[2] * ex[0] - ez[0] * ex[2],
        ez[0] * ex[1] - ez[1] * ex[0],
    ];
    [ex, ey, ez]
}

fn lb(ax: &[[f64; 3]; 3], d: [f64; 3]) -> (f64, f64) {
    let (x, y, z) = (dot(d, ax[0]), dot(d, ax[1]), dot(d, ax[2]));
    let mut l = m::atan2(y, x).to_degrees();
    if l < 0.0 {
        l += 360.0;
    }
    (l, m::asin(z.clamp(-1.0, 1.0)).to_degrees())
}

fn from_lb(ax: &[[f64; 3]; 3], l_deg: f64, b_deg: f64) -> [f64; 3] {
    let (l, b) = (l_deg.to_radians(), b_deg.to_radians());
    let (cl, sl, cb, sb) = (m::cos(l), m::sin(l), m::cos(b), m::sin(b));
    let mut v = [0.0; 3];
    for (i, vi) in v.iter_mut().enumerate() {
        *vi = cb * cl * ax[0][i] + cb * sl * ax[1][i] + sb * ax[2][i];
    }
    v
}

/// The components of a system that shine: (temperature K, radius m).
fn shining(st: &Stars, s: &StarSystem, t_myr: f64) -> Vec<(f64, f64)> {
    let age = t_myr / 1000.0 - s.birth_gyr;
    std::iter::once(s.mass_msun)
        .chain(s.companions.iter().map(|c| c.mass_msun))
        .map(|mass| st.params.state(mass, age))
        .filter(|x| x.teff_k > 0.0 && x.luminosity_lsun > 0.0)
        .map(|x| {
            let t2 = x.teff_k * x.teff_k;
            let l = x.luminosity_lsun * L_SUN;
            let r = m::sqrt(l / (4.0 * m::PI * sse_light::SIGMA * t2 * t2));
            (x.teff_k, r)
        })
        .collect()
}

pub(crate) fn sky_from_here(st: &Stars, pins: &[StarSystem], sky: &SkyParams, ask: Ask) -> Out {
    let t = ask.t_myr;
    if !t.is_finite() || t < 0.0 {
        return Err(err("bad_request", "t_myr must be 0 or more"));
    }
    if !ask.limit_mag.is_finite() || ask.limit_mag > 30.0 {
        return Err(err("bad_request", "limit_mag must be a number up to 30"));
    }
    let table = sse_light::LightTable::base();
    let mut want = ask.bands.clone();
    if !want.is_empty() && !want.iter().any(|b| b == ask.band) {
        want.push(ask.band.to_string());
    }
    let bands =
        sse_light::photometry::bands_of(&table, &want).map_err(|e| err("bad_request", e))?;
    let limit_band = bands
        .iter()
        .find(|b| b.id == ask.band)
        .copied()
        .ok_or_else(|| err("bad_request", format!("unknown passband {}", ask.band)))?;
    let pc_m = sse_core::Units::standard()
        .get("pc")
        .map_err(|e| err("bad_request", format!("{e:?}")))?
        .scale;

    // Where the observer is.
    let (obs_addr, obs) = match (ask.observer, ask.position_pc) {
        (Some(a), _) => {
            let addr: sse_core::Address = a
                .parse()
                .map_err(|e| err("bad_address", format!("{e:?}")))?;
            let s = pins
                .iter()
                .find(|p| p.address == addr)
                .cloned()
                .or(st.draw_one(&addr).map_err(|e| err("bad_address", e))?)
                .ok_or_else(|| err("bad_address", format!("{a}: no such star system")))?;
            (Some(a.to_string()), st.at(&s, t).xyz_pc)
        }
        (None, Some(p)) => (None, p),
        (None, None) => (
            None,
            st.from_grid([st.env.sun_radius_kpc() * 1000.0, 0.0, 0.0], t),
        ),
    };
    let ax = axes(obs);
    let cone = ask.cone.as_ref().map(|c| {
        (
            from_lb(&ax, c.l_deg, c.b_deg),
            m::cos(c.half_angle_deg.clamp(0.0, 180.0).to_radians()),
        )
    });

    // The visibility radius of a class: the dust-free distance at which a
    // blackbody of its top luminosity, at whichever temperature is
    // brightest in the limit band, reaches the limit (plus a margin).
    let probe = |l_lsun: f64| -> f64 {
        let mut best = f64::INFINITY;
        for i in 0..24 {
            let tk = 2000.0 * m::pow(25.0, i as f64 / 23.0);
            let t2 = tk * tk;
            let r = m::sqrt(l_lsun * L_SUN / (4.0 * m::PI * sse_light::SIGMA * t2 * t2));
            let dil = (r / (10.0 * pc_m)) * (r / (10.0 * pc_m));
            let p = photometry_of(
                |lam| m::PI * sse_light::planck(lam, tk) * dil,
                &Sight {
                    distance_m: 10.0 * pc_m,
                    a_v: 0.0,
                    r_v: 3.1,
                },
                &[limit_band],
            );
            best = best.min(p.magnitudes[0].m_vega);
        }
        best
    };
    let classes = st.params.class_edges_lsun.len() + 1;
    let tops: Vec<f64> = (0..classes).map(|c| st.class_top_lsun(c, sky)).collect();
    let radii: Vec<f64> = tops
        .iter()
        .map(|top| {
            (10.0
                * m::pow(
                    10.0,
                    (ask.limit_mag + sky.radius_margin_mag - probe(*top)) / 5.0,
                ))
            .min(sky.max_radius_pc)
        })
        .collect();
    let (found, per, reads) = st
        .sky_walk(&radii, obs, t)
        .map_err(|e| err("bad_request", e))?;
    let walked: Vec<_> = (0..classes)
        .map(|c| serde_json::json!({"class": c, "top_lsun": tops[c], "radius_pc": radii[c], "found": per[c]}))
        .collect();
    let mut found: Vec<_> = found.into_iter().map(|f| (f.system, f.class)).collect();
    // Pins drawn from a reached cell and frame replace that slot; pins from
    // elsewhere are added (fudge F35).
    let frame = st.frame_of(t);
    found.retain(|(s, _)| {
        !pins
            .iter()
            .any(|p| p.cell == s.cell && p.frame == s.frame && p.slot == s.slot)
    });
    for p in pins {
        let l = st.at(p, t).state.luminosity_lsun;
        found.push((p.clone(), st.params.class_of(l)));
    }

    let mut stars = vec![];
    for (s, class) in &found {
        // Light time from where it is now, then its place when the light left.
        let now = st.at(s, t).xyz_pc;
        let d_now = [now[0] - obs[0], now[1] - obs[1], now[2] - obs[2]];
        let dist0 = m::sqrt(dot(d_now, d_now));
        if dist0 <= 0.0 {
            continue; // the observer's own system
        }
        let lt_yr = dist0 * pc_m / C / JULIAN_YEAR;
        let t_emit = (t - lt_yr / 1.0e6).max(0.0);
        let at = st.at(s, t_emit);
        let d = [
            at.xyz_pc[0] - obs[0],
            at.xyz_pc[1] - obs[1],
            at.xyz_pc[2] - obs[2],
        ];
        let dist = m::sqrt(dot(d, d));
        if dist <= 0.0 {
            continue;
        }
        let dir = unit(d);
        if let Some((c, cos_max)) = cone {
            if dot(dir, c) < cos_max {
                continue;
            }
        }
        let parts = shining(st, s, t_emit);
        if parts.is_empty() {
            continue;
        }
        let dm = dist * pc_m;
        let fl = |lam: f64| {
            parts
                .iter()
                .map(|(tk, r)| m::PI * sse_light::planck(lam, *tk) * (r / dm) * (r / dm))
                .sum::<f64>()
        };
        // Dust only dims: a star too faint without it is never seen.
        let clear = Sight {
            distance_m: dm,
            a_v: 0.0,
            r_v: 3.1,
        };
        if photometry_of(fl, &clear, &[limit_band]).magnitudes[0].m_vega > ask.limit_mag {
            continue;
        }
        let a_v = st.dust_av(obs, at.xyz_pc, t, sky);
        let sight = Sight {
            distance_m: dm,
            a_v,
            r_v: 3.1,
        };
        let lim = photometry_of(fl, &sight, &[limit_band]).magnitudes[0].m_vega;
        if lim > ask.limit_mag {
            continue;
        }
        let ph = photometry_of(fl, &sight, &bands);
        let (l, b) = lb(&ax, dir);
        stars.push((
            lim,
            serde_json::json!({
                "address": s.address.to_string(),
                "pinned": s.pinned,
                "class": class,
                "distance_pc": dist,
                "light_time_yr": lt_yr,
                "l_deg": l,
                "b_deg": b,
                "dir": dir,
                "a_v": a_v,
                "teff_k": at.state.teff_k,
                "luminosity_lsun": at.state.luminosity_lsun,
                "phase": at.state.phase,
                "double": !s.companions.is_empty(),
                "components": parts
                    .iter()
                    .map(|(tk, r)| serde_json::json!({"teff_k": tk, "radius_m": r}))
                    .collect::<Vec<_>>(),
                "mag": lim,
                "magnitudes": ph.magnitudes,
                "colours": ph.colours,
            }),
        ));
    }
    // Brightest first; ties by address, so the list is the same everywhere.
    stars.sort_by(|a, b| {
        a.0.total_cmp(&b.0)
            .then_with(|| a.1["address"].as_str().cmp(&b.1["address"].as_str()))
    });
    let mut counts = vec![];
    let mut k = m::floor(stars.first().map_or(0.0, |s| s.0));
    while k <= m::floor(ask.limit_mag) {
        let n = stars.iter().filter(|s| s.0 < k + 1.0).count();
        counts.push(serde_json::json!({"mag": k + 1.0, "n": n}));
        k += 1.0;
    }
    let truncated = stars.len() > ask.max_stars;
    stars.truncate(ask.max_stars);
    Ok(serde_json::json!({
        "galaxy": st.galaxy.to_string(),
        "observer": {"address": obs_addr, "xyz_pc": obs},
        "t_myr": t,
        "frame": frame,
        "limit_mag": ask.limit_mag,
        "band": ask.band,
        "bands": bands.iter().map(|b| b.id.clone()).collect::<Vec<_>>(),
        "cone": ask.cone,
        "stars": stars.into_iter().map(|s| s.1).collect::<Vec<_>>(),
        "counts": counts,
        "walked": {"classes": walked, "reads": reads},
        "truncated": truncated,
        "conventions": "l is measured from the direction of the galaxy's centre, b from its plane toward +z; magnitudes are Vega unless m_ab; counts are cumulative (stars brighter than mag); positions are where each star was when its light left",
    }))
}

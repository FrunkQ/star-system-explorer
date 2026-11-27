//! Y4 part 1: other galaxies in the sky (night sky design section 6).
//!
//! Our galaxy is the universe's home halo (GZ12), one of the web's own
//! catalogued haloes, and the region cells within reach of its place give
//! the other galaxies.
//! Each is grown by G3 (`galaxy::evolve`), the same growth `galaxy_chart`
//! shows, and read at the epoch its light left: the comoving distance on the
//! light cone gives the scale factor. Its light is its star formation
//! history through a simple stellar population built from the fill stars'
//! own model, with the same IMF and the same stellar states as the stars in
//! the sky, each a blackbody. The light is redshifted by the expansion (the
//! O7 shift 1/(1+z), with the luminosity distance (1+z) D_M, Hogg 1999) and
//! dimmed by our own galaxy's dust along the line. Fudge F37.

use crate::{err, Response};
use sse_core::math as m;
use sse_core::quantity::consts::L_SUN;
use sse_cosmos::cosmology::{Cosmology, MPC_M};
use sse_cosmos::galaxy::{Galaxy, GalaxyState};
use sse_cosmos::haloes::{Halo, Haloes};
use sse_cosmos::sky::SkyParams;
use sse_cosmos::stars::{imf_quantile, StarParams, Stars};
use sse_light::photometry::{photometry_of, Sight};
use sse_light::table::Passband;

/// A simple stellar population: L_λ per solar mass formed (W m⁻¹) on a grid
/// of wavelengths and ages.
pub(crate) struct Ssp {
    ln_lam: Vec<f64>,
    ln_age: Vec<f64>,
    /// `l[age][lam]`.
    l: Vec<Vec<f64>>,
}

impl Ssp {
    /// From the fill's IMF and stellar states: each star's L_λ is its
    /// luminosity spread as a blackbody, L π B_λ(T) / (σ T⁴).
    pub(crate) fn of(p: &StarParams) -> Ssp {
        let n_lam = 160;
        let ln_lam: Vec<f64> = (0..n_lam)
            .map(|i| {
                m::ln(5.0e-8) + (m::ln(5.0e-6) - m::ln(5.0e-8)) * i as f64 / (n_lam - 1) as f64
            })
            .collect();
        let n_age = 56;
        let ln_age: Vec<f64> = (0..n_age)
            .map(|i| m::ln(1.0e-4) + (m::ln(20.0) - m::ln(1.0e-4)) * i as f64 / (n_age - 1) as f64)
            .collect();
        // The IMF by number in quantile, finest in the bright tail.
        let mut us = vec![];
        for (lo, hi, n) in [(0.0, 0.99, 256), (0.99, 0.999, 96), (0.999, 1.0, 96)] {
            let du = (hi - lo) / n as f64;
            for i in 0..n {
                us.push((imf_quantile(p, lo + (i as f64 + 0.5) * du), du));
            }
        }
        let mass_formed: f64 = us.iter().map(|(mm, du)| mm * du).sum();
        let lam: Vec<f64> = ln_lam.iter().map(|x| m::exp(*x)).collect();
        let l = ln_age
            .iter()
            .map(|la| {
                let age = m::exp(*la);
                let mut row = vec![0.0; n_lam];
                for (mass, du) in &us {
                    let s = p.state(*mass, age);
                    if s.teff_k <= 0.0 || s.luminosity_lsun <= 0.0 {
                        continue;
                    }
                    let t2 = s.teff_k * s.teff_k;
                    let norm = s.luminosity_lsun * L_SUN * du / (sse_light::SIGMA * t2 * t2);
                    for (k, lk) in lam.iter().enumerate() {
                        row[k] += norm * m::PI * sse_light::planck(*lk, s.teff_k);
                    }
                }
                row.iter().map(|v| v / mass_formed).collect()
            })
            .collect();
        Ssp { ln_lam, ln_age, l }
    }

    /// L_λ per solar mass formed at an age (Gyr) and wavelength (m), by
    /// linear interpolation in ln age and ln λ.
    fn at(&self, age_gyr: f64, lam_m: f64) -> f64 {
        let find = |g: &[f64], x: f64| -> (usize, f64) {
            if x <= g[0] {
                return (0, 0.0);
            }
            let last = g.len() - 1;
            if x >= g[last] {
                return (last - 1, 1.0);
            }
            let step = g[1] - g[0];
            let i = (((x - g[0]) / step) as usize).min(last - 1);
            (i, (x - g[i]) / step)
        };
        let (ia, fa) = find(&self.ln_age, m::ln(age_gyr.max(1.0e-6)));
        let (il, fl) = find(&self.ln_lam, m::ln(lam_m));
        let v = |a: usize| self.l[a][il] * (1.0 - fl) + self.l[a][il + 1] * fl;
        v(ia) * (1.0 - fa) + v(ia + 1) * fa
    }
}

/// Mass formed per age: (age at emission in Gyr, solar masses), from the
/// galaxy's history up to the time `t_gyr`, each history interval split in
/// four.
fn births(g: &Galaxy, t_gyr: f64) -> Vec<(f64, f64)> {
    let h = &g.history;
    let mut out = vec![];
    for w in h.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        if a.t_gyr >= t_gyr {
            break;
        }
        let t1 = b.t_gyr.min(t_gyr);
        let sub = 4;
        let dt = (t1 - a.t_gyr) / sub as f64;
        for j in 0..sub {
            let t = a.t_gyr + (j as f64 + 0.5) * dt;
            let f = (t - a.t_gyr) / (b.t_gyr - a.t_gyr).max(1e-12);
            let sfr = a.sfr + f * (b.sfr - a.sfr); // M☉ per year
            out.push((t_gyr - t, sfr * dt * 1.0e9));
        }
    }
    out
}

/// The galaxy's state at a time (the history point at or before it).
fn state_at(g: &Galaxy, t_gyr: f64) -> &GalaxyState {
    g.history
        .iter()
        .take_while(|s| s.t_gyr <= t_gyr)
        .last()
        .unwrap_or(&g.history[0])
}

pub(crate) struct GalaxyAsk<'a> {
    pub observer_pc: [f64; 3],
    pub axes: &'a [[f64; 3]; 3],
    pub cone: Option<([f64; 3], f64)>,
    pub limit_mag: f64,
    pub limit_band: &'a Passband,
    pub bands: &'a [&'a Passband],
    pub t_myr: f64,
    /// GZ6: also every member's path in the host since infall (survivors,
    /// the torn apart and the merged), for Galaxies nearby's Play and its
    /// streams. Off by default; the sky does not ask.
    pub paths: bool,
}

/// GZ6: one member's path, thinned to one point every `PATH_EVERY` of the
/// orbit's own `path_gyr` points (0.1 Gyr at pack defaults), plus its last.
const PATH_EVERY: usize = 5;

fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// The other galaxies seen from an observer in our galaxy, brightest first.
pub(crate) fn galaxies_seen(
    c: &Cosmology,
    hs: &Haloes,
    gp: &sse_cosmos::galaxy::GalaxyParams,
    st: &Stars,
    sky: &SkyParams,
    host: &Halo,
    ask: GalaxyAsk,
) -> Result<(Vec<(f64, serde_json::Value)>, serde_json::Value), Response> {
    let ssp = Ssp::of(&st.params);
    // GZ12: home is our own halo's place in the web.
    let home = host.position;
    let side = hs.params.cell_mpc;
    let reach = sky.galaxy_reach_mpc;
    let lo = |v: f64| m::floor((v - reach) / side) as i64;
    let hi = |v: f64| m::floor((v + reach) / side) as i64;
    let pc_m = MPC_M / 1.0e6;
    let now_gyr = c.age_gyr();
    // The campaign may sit at another time than the web's now; the light
    // left at (campaign time - look-back) on the web's own clock.
    let t_obs_gyr = ask.t_myr / 1000.0;
    let mut out = vec![];
    let mut cells = 0usize;
    let mut haloes = 0usize;
    for x in lo(home[0])..=hi(home[0]) {
        for y in lo(home[1])..=hi(home[1]) {
            for z in lo(home[2])..=hi(home[2]) {
                // The cell's nearest point to home.
                let mut d2 = 0.0;
                for (i, v) in [x, y, z].iter().enumerate() {
                    let (a, b) = (*v as f64 * side, (*v + 1) as f64 * side);
                    let g = if home[i] < a {
                        a - home[i]
                    } else if home[i] > b {
                        home[i] - b
                    } else {
                        0.0
                    };
                    d2 += g * g;
                }
                if d2 > reach * reach {
                    continue;
                }
                cells += 1;
                let cell = hs.cell([x, y, z]);
                for h in &cell.haloes {
                    haloes += 1;
                    // Our own halo is us, not a galaxy in our sky.
                    if h.address == host.address {
                        continue;
                    }
                    if let Some(v) = seen(
                        c, gp, st, sky, &ssp, h, home, pc_m, now_gyr, t_obs_gyr, &ask,
                    )? {
                        out.push(v);
                    }
                }
            }
        }
    }
    let (sats, sat_walk) = satellites_seen(c, hs, gp, st, sky, &ssp, host, &ask)?;
    out.extend(sats);
    // Brightest first; ties by address.
    out.sort_by(|a, b| {
        a.0.total_cmp(&b.0)
            .then_with(|| a.1["address"].as_str().cmp(&b.1["address"].as_str()))
    });
    let mut walk = serde_json::json!({"cells": cells, "haloes": haloes, "reach_mpc": reach, "home_mpc": home, "satellites": sat_walk});
    // The edge needs the sheet's Sun (no stack, no sheet: left out).
    if let (true, Some(m_sun)) = (ask.paths, sse_physics::constant("M_sun")) {
        let host_msun = walk["satellites"]["host_mass_msun"].as_f64().unwrap_or(0.0);
        walk["group_edge"] = group_edge(host_msun, m_sun, &out, t_obs_gyr)?;
    }
    Ok((out, walk))
}

/// The block that finds a group's edge (Sandage 1986), reading G through Look up.
pub(crate) const ZERO_VELOCITY_BLOCK: &str = "sse.gravity.zero_velocity_radius";

/// GZ6 (design 7.2): the edge of our group, the zero-velocity sphere round
/// home of the mass inside it. It is widened until it holds what it needs:
/// home's halo, then each galaxy's halo inside it, nearest first, which
/// widens it again (it only grows, so it settles in a few steps). Satellites
/// are inside home's halo already, so are not counted twice.
fn group_edge(
    home_halo_msun: f64,
    m_sun: f64,
    seen: &[(f64, serde_json::Value)],
    t_gyr: f64,
) -> Result<serde_json::Value, Response> {
    use sse_flow::Atom as _;
    // Through the physics in force (ER1): the block from its library, read
    // once here, and the Sun's mass from the sheet in force (`m_sun`). The
    // block has no settings, so none are shipped defaults.
    let physics = sse_physics::now().ok_or_else(|| err("eval", "no physics in force"))?;
    let block = physics
        .blocks()
        .iter()
        .find(|a| a.decl().id == ZERO_VELOCITY_BLOCK)
        .ok_or_else(|| err("eval", format!("{ZERO_VELOCITY_BLOCK} is not loaded")))?;
    let t_s = t_gyr * 1.0e9 * sse_core::quantity::consts::JULIAN_YEAR;
    let radius_mpc = |msun: f64| -> Result<f64, Response> {
        let kg = msun.max(0.0) * m_sun;
        let r = block
            .eval(&[kg, t_s], &[])
            .map_err(|x| err("eval", format!("{ZERO_VELOCITY_BLOCK}: {}", x.0)))?;
        Ok(r.first().copied().unwrap_or(0.0) / MPC_M)
    };
    let mut others: Vec<(f64, f64, &str)> = seen
        .iter()
        .filter(|(_, v)| v["satellite"].as_bool() != Some(true))
        .map(|(_, v)| {
            (
                v["distance_mpc"].as_f64().unwrap_or(f64::INFINITY),
                v["halo_mass_msun"].as_f64().unwrap_or(0.0),
                v["address"].as_str().unwrap_or(""),
            )
        })
        .collect();
    others.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.2.cmp(b.2)));
    let mut mass = home_halo_msun;
    let mut r = radius_mpc(mass)?;
    let mut n = 0;
    while n < others.len() && others[n].0 <= r {
        mass += others[n].1;
        n += 1;
        r = radius_mpc(mass)?;
    }
    Ok(serde_json::json!({
        "radius_mpc": r, "mass_msun": mass, "members": n, "t_gyr": t_gyr,
        "block": ZERO_VELOCITY_BLOCK,
    }))
}

#[allow(clippy::too_many_arguments)]
fn seen(
    c: &Cosmology,
    gp: &sse_cosmos::galaxy::GalaxyParams,
    st: &Stars,
    sky: &SkyParams,
    ssp: &Ssp,
    h: &Halo,
    home: [f64; 3],
    pc_m: f64,
    now_gyr: f64,
    t_obs_gyr: f64,
    ask: &GalaxyAsk,
) -> Result<Option<(f64, serde_json::Value)>, Response> {
    let o = ask.observer_pc;
    let rel = [
        (h.position[0] - home[0]) * 1.0e6 - o[0],
        (h.position[1] - home[1]) * 1.0e6 - o[1],
        (h.position[2] - home[2]) * 1.0e6 - o[2],
    ];
    let d_pc = m::sqrt(dot(rel, rel));
    let d_mpc = d_pc / 1.0e6;
    // Inside our own halo's reach: taken to be part of us (fudge F37).
    if d_mpc < sky.home_exclusion_mpc || d_mpc > sky.galaxy_reach_mpc {
        return Ok(None);
    }
    let dir = [rel[0] / d_pc, rel[1] / d_pc, rel[2] / d_pc];
    if let Some((axis, cos_max)) = ask.cone {
        if dot(dir, axis) < cos_max {
            return Ok(None);
        }
    }
    let g = sse_cosmos::galaxy::evolve(c, gp, h);
    look(
        c,
        st,
        sky,
        ssp,
        Look {
            rel_pc: rel,
            g: &g,
            quench_gyr: None,
            address: &h.address,
            halo_mass: h.mass,
            stars_kept: 1.0,
            return_fraction: gp.return_fraction,
        },
        pc_m,
        now_gyr,
        t_obs_gyr,
        ask,
    )
}

/// One galaxy to look at: where it is from the observer (pc), its grown
/// history, and when (if ever) its star formation stopped.
struct Look<'a> {
    rel_pc: [f64; 3],
    g: &'a Galaxy,
    quench_gyr: Option<f64>,
    address: &'a str,
    halo_mass: f64,
    /// GH2: the share of its stars still bound (tidal stripping).
    stars_kept: f64,
    /// G3's share of a formed mass handed back by dying stars.
    return_fraction: f64,
}

#[allow(clippy::too_many_arguments)]
fn look(
    c: &Cosmology,
    st: &Stars,
    sky: &SkyParams,
    ssp: &Ssp,
    lk: Look,
    pc_m: f64,
    now_gyr: f64,
    t_obs_gyr: f64,
    ask: &GalaxyAsk,
) -> Result<Option<(f64, serde_json::Value)>, Response> {
    let o = ask.observer_pc;
    let (rel, g) = (lk.rel_pc, lk.g);
    let d_pc = m::sqrt(dot(rel, rel));
    let d_mpc = d_pc / 1.0e6;
    let dir = [rel[0] / d_pc, rel[1] / d_pc, rel[2] / d_pc];
    if let Some((axis, cos_max)) = ask.cone {
        if dot(dir, axis) < cos_max {
            return Ok(None);
        }
    }
    // When its light left: the light cone, on the web's clock, shifted to
    // the campaign's own now.
    let a_emit = c.scale_at_comoving(d_mpc);
    let z = 1.0 / a_emit - 1.0;
    let lookback = now_gyr - c.time_of_scale(a_emit);
    let t_emit = t_obs_gyr - lookback;
    if t_emit <= 0.0 {
        return Ok(None);
    }
    // A satellite stops forming stars a while after it falls in (its gas
    // is stripped): its stars are those born by then, aged to t_emit.
    let t_stop = lk.quench_gyr.map_or(t_emit, |q| q.min(t_emit));
    let then = state_at(g, t_stop);
    if then.stars <= 0.0 || lk.stars_kept <= 0.0 {
        return Ok(None);
    }
    let kept = lk.stars_kept;
    let formed = births(g, t_stop);
    // Stars brought in by mergers and smooth accretion (G3's ex-situ stars)
    // are in `then.stars` but not in its own star formation history, so the
    // history alone leaves them dark: a group's central elliptical, mostly
    // ex-situ, read M/L_V 13 (found by GZ12). They are lit as if they formed
    // like the galaxy's own stars (fudge F37): the history is scaled so the
    // stars it leaves match the galaxy's.
    let own: f64 = formed.iter().map(|(_, mm)| mm).sum::<f64>() * (1.0 - lk.return_fraction);
    let lit = if own > 0.0 {
        (then.stars / own).max(1.0)
    } else {
        1.0
    };
    let born: Vec<(f64, f64)> = formed
        .into_iter()
        .map(|(age, mass)| (age + (t_emit - t_stop), mass * kept * lit))
        .collect();
    // Rest-frame L_λ (W per m).
    let l_rest = |lam: f64| -> f64 {
        born.iter()
            .map(|(age, mass)| mass * ssp.at(*age, lam))
            .sum()
    };
    // Observed flux density: L_λ(λ/(1+z)) / (4π d_L² (1+z)), d_L = (1+z) D_M.
    let dm_m = d_mpc * MPC_M;
    let dl = (1.0 + z) * dm_m;
    let fl = |lam: f64| l_rest(lam / (1.0 + z)) / (4.0 * m::PI * dl * dl * (1.0 + z));
    // Our galaxy's dust, out to where the disc has none left (four legs, so
    // the thin dust layer near us is sampled finely).
    let mut a_v = 0.0;
    let mut from = o;
    for r_pc in [300.0, 1500.0, 6000.0, 30000.0] {
        let to = [
            o[0] + dir[0] * r_pc,
            o[1] + dir[1] * r_pc,
            o[2] + dir[2] * r_pc,
        ];
        a_v += st.dust_av(from, to, ask.t_myr, sky);
        from = to;
    }
    let lim = photometry_of(
        fl,
        &Sight {
            distance_m: dl,
            a_v,
            r_v: 3.1,
        },
        &[ask.limit_band],
    )
    .magnitudes[0]
        .m_vega;
    if lim > ask.limit_mag {
        return Ok(None);
    }
    let ph = photometry_of(
        fl,
        &Sight {
            distance_m: dl,
            a_v,
            r_v: 3.1,
        },
        ask.bands,
    );
    // Rest-frame absolute magnitudes (at 10 pc, no dust).
    let ten = 10.0 * pc_m;
    let abs = photometry_of(
        |lam| l_rest(lam) / (4.0 * m::PI * ten * ten),
        &Sight {
            distance_m: ten,
            a_v: 0.0,
            r_v: 3.1,
        },
        ask.bands,
    );
    // Shape: an exponential disc's half-light radius is 1.678 scale lengths;
    // tilt and turn keyed by the halo, isotropic.
    let addr: sse_core::Address = lk
        .address
        .parse()
        .map_err(|e| err("bad_address", format!("{e:?}")))?;
    let d = sse_core::Draws::new(
        st.seed,
        addr,
        sse_core::Stream::new("cosmos.galaxy.sky.v1")
            .map_err(|e| err("bad_request", e.to_string()))?,
    );
    let cos_i = d.unit(0);
    let pa = 180.0 * d.unit(1);
    let morph = sse_cosmos::galaxy::classify(then, g.barred);
    let spheroid = matches!(morph, sse_cosmos::galaxy::Morphology::Elliptical);
    let axis_ratio = if spheroid {
        0.6 + 0.4 * cos_i
    } else {
        cos_i.max(0.15)
    };
    let r_half_kpc = 1.678 * then.disc_kpc;
    let d_a_kpc = d_mpc * 1000.0 / (1.0 + z);
    let ang_arcmin = (r_half_kpc / d_a_kpc).to_degrees() * 60.0;
    let (l, b) = crate::sky::lb(ask.axes, dir);
    Ok(Some((
        lim,
        serde_json::json!({
            "address": lk.address,
            "halo_mass_msun": lk.halo_mass,
            "distance_mpc": d_mpc,
            "redshift": z,
            "lookback_myr": lookback * 1000.0,
            "seen_at_gyr": t_emit,
            "l_deg": l,
            "b_deg": b,
            "dir": dir,
            "a_v": a_v,
            "stellar_mass_msun": then.stars * kept,
            "stellar_mass_now_msun": g.now.stars * kept,
            // G3's envelope: the galaxy chart follows its tree (GH3).
            "about": true,
            "sfr_msun_yr": if t_stop < t_emit { 0.0 } else { then.sfr },
            "morphology": morph,
            "half_light_kpc": r_half_kpc,
            "half_light_arcmin": ang_arcmin,
            "axis_ratio": axis_ratio,
            "inclination_deg": m::acos(cos_i).to_degrees(),
            "position_angle_deg": pa,
            "mag": lim,
            "magnitudes": ph.magnitudes,
            "colours": ph.colours,
            "absolute": abs.magnitudes,
        }),
    )))
}

/// The satellites of our own galaxy, seen from an observer in it (Y4 part
/// 2, GH1, GH2).
///
/// The subhaloes are the members of the host's assembly tree read at the
/// campaign's time (GH1). Each falls in on the host's r200 with a keyed
/// speed and direction and is followed on its orbit in the host's growing
/// potential under the four processes of design 7.3 (`sse_cosmos::infall`):
/// sinking, tidal stripping, gas stripping and disc shocks. Those that
/// reached the centre are merged; those torn apart are remains (streams,
/// shells, mixed remains), counted but not drawn as galaxies. Each
/// survivor's galaxy is G3 grown on a halo that had its mass at infall, with
/// its stars cut to the share still bound, forming stars until ram pressure
/// strips its gas.
#[allow(clippy::too_many_arguments)]
pub(crate) fn satellites_seen(
    c: &Cosmology,
    hs: &Haloes,
    gp: &sse_cosmos::galaxy::GalaxyParams,
    st: &Stars,
    sky: &SkyParams,
    ssp: &Ssp,
    host: &Halo,
    ask: &GalaxyAsk,
) -> Result<(Vec<(f64, serde_json::Value)>, serde_json::Value), Response> {
    let op = &sky.satellites.orbits;
    let pc_m = MPC_M / 1.0e6;
    let now_gyr = c.age_gyr();
    let t_obs = ask.t_myr / 1000.0;
    let a_obs = c.scale_of_time(t_obs.max(1.0e-3)).min(1.0);
    let z_obs = 1.0 / a_obs - 1.0;
    let m_host = host.mass_at(z_obs);
    let r200_kpc = gp.virial(c, m_host, a_obs).0;
    let tree = sse_cosmos::assembly::tree(
        c,
        st.seed,
        host,
        z_obs,
        &hs.params.assembly,
        sse_cosmos::assembly::Level::Chart,
    )
    .map_err(|e| err("bad_address", e))?;
    // The host's potential from the first infall to now, one table per
    // slice (GH2 cut 3), built once for every satellite; each member's own
    // galaxy and orbit in it (shared with GH3's feed).
    let host_galaxy = sse_cosmos::galaxy::evolve(c, gp, host);
    let (_, members) = sse_cosmos::feed::members(c, hs, gp, op, host, &host_galaxy, &tree, t_obs)
        .map_err(|e| err("bad_address", e))?;
    let mut out = vec![];
    let (mut drawn, mut merged) = (0u64, 0u64);
    let mut remains: std::collections::BTreeMap<String, u64> = Default::default();
    let mut paths = vec![];
    for mb in &members {
        drawn += 1;
        let p = &mb.progenitor;
        let g = &mb.galaxy;
        let halo = &mb.halo;
        let fate = &mb.fate;
        if ask.paths {
            paths.push(member_path(mb, ask.observer_pc));
        }
        if fate.merged_gyr.is_some() {
            merged += 1;
            continue;
        }
        if fate.disrupted_gyr.is_some() {
            for l in &fate.labels {
                *remains.entry(l.clone()).or_default() += 1;
            }
            continue;
        }
        let pos = [
            fate.pos_kpc[0] * 1000.0,
            fate.pos_kpc[1] * 1000.0,
            fate.pos_kpc[2] * 1000.0,
        ];
        let o = ask.observer_pc;
        let rel = [pos[0] - o[0], pos[1] - o[1], pos[2] - o[2]];
        let dist = m::sqrt(dot(rel, rel));
        if let Some((axis, cos_max)) = ask.cone {
            let dir = [rel[0] / dist, rel[1] / dist, rel[2] / dist];
            if dot(dir, axis) < cos_max {
                continue;
            }
        }
        let seen = look(
            c,
            st,
            sky,
            ssp,
            Look {
                rel_pc: rel,
                g,
                quench_gyr: fate.quench_gyr,
                address: &halo.address,
                halo_mass: p.mass_at_infall_msun,
                stars_kept: fate.stars_kept,
                return_fraction: gp.return_fraction,
            },
            pc_m,
            now_gyr,
            t_obs,
            ask,
        )?;
        if let Some((mag, mut v)) = seen {
            v["satellite"] = true.into();
            v["distance_kpc"] = (dist / 1000.0).into();
            v["from_centre_kpc"] = fate.r_kpc.into();
            v["infall_gyr"] = p.t_infall_gyr.into();
            v["infall_halo_mass_msun"] = p.mass_at_infall_msun.into();
            v["orbit"] = serde_json::json!({
                "labels": fate.labels,
                "dark_kept": fate.dark_kept,
                "stars_kept": fate.stars_kept,
                "pericentres": fate.pericentres,
                "closest_kpc": fate.peri_kpc,
                "farthest_kpc": fate.apo_kpc,
                "disc_passes": fate.disc_passes,
                "heat": fate.heat,
                "quenched_gyr": fate.quench_gyr,
                "velocity_kms": [
                    fate.vel_kpc_gyr[0] / sse_cosmos::infall::kms(),
                    fate.vel_kpc_gyr[1] / sse_cosmos::infall::kms(),
                    fate.vel_kpc_gyr[2] / sse_cosmos::infall::kms(),
                ],
            });
            out.push((mag, v));
        }
    }
    let mut walk = serde_json::json!({
        "host_mass_msun": m_host, "r200_kpc": r200_kpc,
        "subhaloes": drawn, "merged": merged, "remains": remains,
        "expected": tree.expected, "accreted_share": tree.accreted_psi,
        "pinned": tree.progenitors.iter().filter(|p| p.pinned).count(),
    });
    if ask.paths {
        let o = ask.observer_pc;
        walk["centre_kpc"] = serde_json::json!([-o[0] / 1000.0, -o[1] / 1000.0, -o[2] / 1000.0]);
        walk["paths"] = paths.into();
    }
    Ok((out, walk))
}

/// GZ6: where one member went from infall to now, in kpc from the observer
/// in the galactic frame (the frame of each galaxy's `dir`), as
/// `[t_gyr, x, y, z]`; with what became of it and its stars at infall.
fn member_path(mb: &sse_cosmos::feed::Member, observer_pc: [f64; 3]) -> serde_json::Value {
    let f = &mb.fate;
    let o = [
        observer_pc[0] / 1000.0,
        observer_pc[1] / 1000.0,
        observer_pc[2] / 1000.0,
    ];
    let at = |t: f64, x: &[f64; 3]| [t, x[0] - o[0], x[1] - o[1], x[2] - o[2]];
    let mut pts: Vec<[f64; 4]> = f
        .path
        .iter()
        .enumerate()
        .filter(|(i, _)| i % PATH_EVERY == 0)
        .map(|(_, (t, x))| at(*t, x))
        .collect();
    if pts.last().is_none_or(|l| l[0] < f.t_gyr) {
        pts.push(at(f.t_gyr, &f.pos_kpc));
    }
    let fate = if f.merged_gyr.is_some() {
        "merged"
    } else if f.disrupted_gyr.is_some() {
        "torn apart"
    } else {
        "survives"
    };
    serde_json::json!({
        "address": mb.halo.address,
        "fate": fate,
        "labels": f.labels,
        "infall_gyr": mb.progenitor.t_infall_gyr,
        "merged_gyr": f.merged_gyr,
        "disrupted_gyr": f.disrupted_gyr,
        "infall_halo_mass_msun": mb.progenitor.mass_at_infall_msun,
        "infall_stars_msun": state_at(&mb.galaxy, mb.progenitor.t_infall_gyr).stars,
        "path": pts,
    })
}

//! Y4 part 1: other galaxies in the sky (night sky design section 6).
//!
//! Our galaxy (the campaign's median halo) sits at a home point of the
//! cosmic web, and the region cells within reach give the other galaxies.
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
}

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
    let home = sky.home_mpc;
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
    Ok((
        out,
        serde_json::json!({"cells": cells, "haloes": haloes, "reach_mpc": reach, "home_mpc": home, "satellites": sat_walk}),
    ))
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
    // Inside our own halo: that is us (fudge F37).
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
    if then.stars <= 0.0 {
        return Ok(None);
    }
    let born: Vec<(f64, f64)> = births(g, t_stop)
        .into_iter()
        .map(|(age, mass)| (age + (t_emit - t_stop), mass))
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
            "stellar_mass_msun": then.stars,
            "stellar_mass_now_msun": g.now.stars,
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

/// Y4 part 2: the satellites of our own galaxy, seen from an observer in it.
///
/// Subhaloes are drawn from the unevolved subhalo mass function of the host
/// as it is at the campaign's time (keyed Poisson counts per ln ψ bin).
/// Each falls in when the host had grown to a matching share of its mass
/// (accretion follows the host's growth), and is gone if dynamical friction
/// has merged it since. Survivors sit on the Aquarius radial profile,
/// isotropic about the centre. Each satellite's galaxy is G3 grown on a
/// halo that had the subhalo's mass at infall; it stops forming stars a
/// delay after infall. Fudge F39.
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
    let sp = &sky.satellites;
    let pc_m = MPC_M / 1.0e6;
    let now_gyr = c.age_gyr();
    let t_obs = ask.t_myr / 1000.0;
    let a_obs = c.scale_of_time(t_obs.max(1.0e-3)).min(1.0);
    let z_obs = 1.0 / a_obs - 1.0;
    let m_host = host.mass_at(z_obs);
    let r200_kpc = gp.virial(c, m_host, a_obs).0;
    let host_addr: sse_core::Address = host
        .address
        .parse()
        .map_err(|e| err("bad_address", format!("{e:?}")))?;
    let d = sse_core::Draws::new(
        st.seed,
        host_addr,
        sse_core::Stream::new("cosmos.galaxy.satellite.v1")
            .map_err(|e| err("bad_request", e.to_string()))?,
    );
    // The radial profile's inverse CDF: number density n(x) ∝
    // exp(-2/α ((x/x₂)^α - 1)), weighted by x², from r_min to r_200.
    let (ea, e2) = (sp.einasto_alpha, sp.einasto_r2_frac);
    let n_grid = 128usize;
    let xs: Vec<f64> = (0..=n_grid)
        .map(|i| sp.r_min_frac + (1.0 - sp.r_min_frac) * i as f64 / n_grid as f64)
        .collect();
    let w = |x: f64| x * x * m::exp(-2.0 / ea * (m::pow(x / e2, ea) - 1.0));
    let mut cdf = vec![0.0];
    for k in 1..xs.len() {
        let last = cdf[k - 1];
        cdf.push(last + 0.5 * (w(xs[k - 1]) + w(xs[k])) * (xs[k] - xs[k - 1]));
    }
    let total = cdf[n_grid];
    let radius = |u: f64| -> f64 {
        let target = u * total;
        let k = cdf.partition_point(|v| *v < target).clamp(1, n_grid);
        let f = (target - cdf[k - 1]) / (cdf[k] - cdf[k - 1]).max(1e-300);
        xs[k - 1] + f * (xs[k] - xs[k - 1])
    };
    let shmf = |ln_psi: f64| {
        let x = m::exp(ln_psi) / sp.shmf_alpha;
        sp.shmf_n0 * m::pow(x, -sp.shmf_alpha) * m::exp(-sp.shmf_k * x * x * x)
    };
    let decades = -m::log10(sp.psi_min);
    let n_bins = m::ceil(decades * f64::from(sp.bins_per_decade)).max(1.0) as u64;
    let step = decades * m::ln(10.0) / n_bins as f64;
    let ln_min = m::ln(sp.psi_min);
    let mut out = vec![];
    let (mut drawn, mut merged) = (0u64, 0u64);
    for j in 0..n_bins {
        let lo = ln_min + step * j as f64;
        let lambda = sse_cosmos::cosmology::quad(&shmf, lo, lo + step, 8);
        let count = sse_cosmos::haloes::poisson(&d, j * 4096, lambda).min(400);
        for i in 0..count {
            drawn += 1;
            let k0 = j * 4096 + 16 + i * 8;
            let psi = m::exp(lo + step * d.unit(k0));
            let m_sat = psi * m_host;
            // Infall: when the host held a share f of today's mass, f
            // uniform above ψ (accretion follows the host's growth).
            let f = psi + (1.0 - psi) * d.unit(k0 + 1);
            let (mut zl, mut zh) = (z_obs, 20.0_f64);
            for _ in 0..60 {
                let mid = 0.5 * (zl + zh);
                if host.mass_at(mid) > f * m_host {
                    zl = mid;
                } else {
                    zh = mid;
                }
            }
            let z_acc = 0.5 * (zl + zh);
            let a_acc = 1.0 / (1.0 + z_acc);
            let t_acc = c.time_of_scale(a_acc);
            // Dynamical friction since infall (BK08).
            let ratio = (host.mass_at(z_acc) / m_sat).max(1.0);
            let eta = sp.eta_range[0] + (sp.eta_range[1] - sp.eta_range[0]) * d.unit(k0 + 2);
            let rc = sp.rc_range[0] + (sp.rc_range[1] - sp.rc_range[0]) * d.unit(k0 + 3);
            let tau_dyn = 0.1 * c.hubble_time_gyr / c.e(a_acc);
            let tau = tau_dyn * sp.df_a * m::pow(ratio, sp.df_b) / m::ln(1.0 + ratio)
                * m::exp(sp.df_c * eta)
                * m::pow(rc, sp.df_d);
            if t_acc + tau < t_obs {
                merged += 1;
                continue;
            }
            // Where it is now.
            let r_kpc = radius(d.unit(k0 + 4)) * r200_kpc;
            let cos_t = 2.0 * d.unit(k0 + 5) - 1.0;
            let sin_t = m::sqrt((1.0 - cos_t * cos_t).max(0.0));
            let phi = 2.0 * m::PI * d.unit(k0 + 6);
            let pos = [
                r_kpc * 1000.0 * sin_t * m::cos(phi),
                r_kpc * 1000.0 * sin_t * m::sin(phi),
                r_kpc * 1000.0 * cos_t,
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
            // Its galaxy: a halo with the subhalo's mass at infall.
            let mut m0 = m_sat;
            for _ in 0..12 {
                let (al, be) = sse_cosmos::haloes::correa(c, m0, 0.0);
                m0 = m_sat / (m::pow(1.0 + z_acc, al) * m::exp(be * z_acc));
            }
            let halo = hs.median(m0, &format!("sat-{j}-{i}"));
            let g = sse_cosmos::galaxy::evolve(c, gp, &halo);
            let seen = look(
                c,
                st,
                sky,
                ssp,
                Look {
                    rel_pc: rel,
                    g: &g,
                    quench_gyr: Some(t_acc + sp.quench_delay_gyr),
                    address: &halo.address,
                    halo_mass: m_sat,
                },
                pc_m,
                now_gyr,
                t_obs,
                ask,
            )?;
            if let Some((mag, mut v)) = seen {
                v["satellite"] = true.into();
                v["distance_kpc"] = (dist / 1000.0).into();
                v["from_centre_kpc"] = r_kpc.into();
                v["infall_gyr"] = t_acc.into();
                v["infall_halo_mass_msun"] = m_sat.into();
                out.push((mag, v));
            }
        }
    }
    Ok((
        out,
        serde_json::json!({
            "host_mass_msun": m_host, "r200_kpc": r200_kpc,
            "subhaloes": drawn, "merged": merged,
        }),
    ))
}

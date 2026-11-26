//! The glow map (night sky design section 5, stage Y3 part 2): the light of
//! every star the sky draw does not show, integrated along each pixel's
//! sight line and dimmed by the dust in front.
//!
//! No double counting. The draw (Y2) shows a star when it lies inside its
//! class's walk radius and is at least as bright as the limit. So a star's
//! light goes into the glow exactly when it lies beyond its class's radius,
//! or inside it but fainter than the limit. The stars are not drawn here:
//! each step reads the envelope's system density and the place's light mix
//! (`Stars::light_points`, the count tree's own quadrature) and adds the
//! expected light of the members that pass that test.
//!
//! Transfer: I_λ = (1/4π) ∫ n(s) Σ L_λ e^(−τ_λ(s)) ds, with τ_λ from the
//! same dust column the draw uses (`SkyParams`: gas times 10^[Fe/H] per
//! magnitude of A_V) spread over wavelength by `sse.light.dust_extinction`.

use crate::healpix::Grid;
use serde::{Deserialize, Serialize};
use sse_atoms::extinction::ccm_ratio;
use sse_core::math as m;
use sse_core::quantity::consts::{C, L_SUN};
use sse_cosmos::glow_mix::LightPoint;
use sse_cosmos::sky::SkyParams;
use sse_cosmos::stars::Stars;
use sse_light::photometry::{ab_magnitude, photometry_of, Sight, AB_ZERO};
use sse_light::{planck, Passband, SIGMA};
use std::cell::RefCell;
use std::collections::BTreeMap;

/// Magnitudes of extinction per unit optical depth: 2.5 log₁₀ e.
const MAG_PER_TAU: f64 = 2.5 / core::f64::consts::LN_10;

/// What a glow map is asked for.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct GlowSettings {
    /// HEALPix resolution (nside 32 is 12,288 pixels).
    pub nside: u64,
    /// Steps along each sight line, spaced evenly in log distance.
    pub steps: usize,
    pub near_pc: f64,
    pub far_pc: f64,
    /// Width of the radius bins the light mix is cached in, kpc.
    pub mix_bin_kpc: f64,
}

impl Default for GlowSettings {
    fn default() -> Self {
        GlowSettings {
            nside: 32,
            steps: 96,
            near_pc: 1.0,
            far_pc: 30_000.0,
            mix_bin_kpc: 0.25,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GlowMap {
    pub nside: u64,
    pub bands: Vec<String>,
    /// Each class's walk radius the draw used, pc.
    pub radii_pc: Vec<f64>,
    /// Per band, per pixel (RING order): I_λ at the band's effective
    /// wavelength, W m⁻² m⁻¹ sr⁻¹.
    pub intensity: Vec<Vec<f64>>,
    /// Per pixel: dust in front, V magnitudes, out to the far end.
    pub a_v_total: Vec<f64>,
}

/// The observer's sky axes, as the draw uses them: x toward the galaxy's
/// centre in the plane, z toward the galaxy's north, y completing a
/// right-handed set.
pub fn sky_axes(obs: [f64; 3]) -> [[f64; 3]; 3] {
    let n = m::sqrt(obs[0] * obs[0] + obs[1] * obs[1]);
    let ex = if n > 0.0 {
        [-obs[0] / n, -obs[1] / n, 0.0]
    } else {
        [1.0, 0.0, 0.0]
    };
    let ez = [0.0, 0.0, 1.0];
    let ey = [
        ez[1] * ex[2] - ez[2] * ex[1],
        ez[2] * ex[0] - ez[0] * ex[2],
        ez[0] * ex[1] - ez[1] * ex[0],
    ];
    [ex, ey, ez]
}

/// Direction of galactic (l, b) in degrees, in the galaxy's frame.
pub fn from_lb(ax: &[[f64; 3]; 3], l_deg: f64, b_deg: f64) -> [f64; 3] {
    let (l, b) = (l_deg * m::PI / 180.0, b_deg * m::PI / 180.0);
    let (cl, sl, cb, sb) = (m::cos(l), m::sin(l), m::cos(b), m::sin(b));
    let mut v = [0.0; 3];
    for (i, vi) in v.iter_mut().enumerate() {
        *vi = cb * cl * ax[0][i] + cb * sl * ax[1][i] + sb * ax[2][i];
    }
    v
}

/// The pixel grid's axes are (l, b): pixel direction (x, y, z) in sky axes
/// is (cos b cos l, cos b sin l, sin b).
pub fn pixel_of_lb(g: &Grid, l_deg: f64, b_deg: f64) -> u64 {
    let (l, b) = (l_deg * m::PI / 180.0, b_deg * m::PI / 180.0);
    g.pixel_vec([m::cos(b) * m::cos(l), m::cos(b) * m::sin(l), m::sin(b)])
}

/// Each class's walk radius for a limit, exactly as the draw (Y2) finds it:
/// the dust-free distance at which a blackbody of the class's top
/// luminosity, at whichever temperature is brightest in the limit band,
/// reaches the limit plus the margin, capped.
pub fn class_radii(
    st: &Stars,
    sky: &SkyParams,
    limit_mag: f64,
    band: &Passband,
    pc_m: f64,
) -> Vec<f64> {
    let probe = |l_lsun: f64| -> f64 {
        let mut best = f64::INFINITY;
        for i in 0..24 {
            let tk = 2000.0 * m::pow(25.0, i as f64 / 23.0);
            let t2 = tk * tk;
            let r = m::sqrt(l_lsun * L_SUN / (4.0 * m::PI * SIGMA * t2 * t2));
            let dil = (r / (10.0 * pc_m)) * (r / (10.0 * pc_m));
            let p = photometry_of(
                |lam| m::PI * planck(lam, tk) * dil,
                &Sight {
                    distance_m: 10.0 * pc_m,
                    a_v: 0.0,
                    r_v: 3.1,
                },
                &[band],
            );
            best = best.min(p.magnitudes[0].m_vega);
        }
        best
    };
    let classes = st.params.class_edges_lsun.len() + 1;
    (0..classes)
        .map(|c| st.class_top_lsun(c, sky))
        .map(|top| {
            (10.0 * m::pow(10.0, (limit_mag + sky.radius_margin_mag - probe(top)) / 5.0))
                .min(sky.max_radius_pc)
        })
        .collect()
}

/// One class's light at a place, sorted by absolute magnitude in the limit
/// band, with the light of all members at or fainter than each entry.
struct ClassLight {
    /// Absolute Vega magnitude in the limit band, brightest first.
    abs_mag: Vec<f64>,
    /// `faint[b][k]`: Σ weight L_λ (W/m per system) of entries k and on.
    faint: Vec<Vec<f64>>,
}

struct Mix {
    classes: Vec<ClassLight>,
}

fn mix_of(
    points: &[LightPoint],
    classes: usize,
    bands: &[&Passband],
    limit: &Passband,
    pc_m: f64,
) -> Mix {
    let mut per: Vec<Vec<(f64, Vec<f64>)>> = vec![vec![]; classes];
    let d10 = 10.0 * pc_m;
    for q in points {
        if q.luminosity_lsun <= 0.0 || q.teff_k <= 0.0 || q.weight <= 0.0 {
            continue;
        }
        let t2 = q.teff_k * q.teff_k;
        let r2 = q.luminosity_lsun * L_SUN / (4.0 * m::PI * SIGMA * t2 * t2);
        // L_λ = 4π R² π B_λ(T).
        let l_at = |lam: f64| 4.0 * m::PI * r2 * m::PI * planck(lam, q.teff_k);
        let f10 = l_at(limit.effective_m) / (4.0 * m::PI * d10 * d10);
        let abs = ab_magnitude(f10, limit.effective_m) - limit.ab_minus_vega;
        let light: Vec<f64> = bands
            .iter()
            .map(|b| q.weight * l_at(b.effective_m))
            .collect();
        per[q.class.min(classes - 1)].push((abs, light));
    }
    let classes = per
        .into_iter()
        .map(|mut v| {
            v.sort_by(|a, b| a.0.total_cmp(&b.0));
            let n = v.len();
            let mut faint = vec![vec![0.0; n + 1]; bands.len()];
            for k in (0..n).rev() {
                for (b, f) in faint.iter_mut().enumerate() {
                    f[k] = f[k + 1] + v[k].1[b];
                }
            }
            ClassLight {
                abs_mag: v.into_iter().map(|e| e.0).collect(),
                faint,
            }
        })
        .collect();
    Mix { classes }
}

/// The glow map seen from `observer_pc` (galactic, pc) at `t_myr`, for a
/// draw to `limit_mag` in band `limit`. `bands` are the bands kept.
#[allow(clippy::too_many_arguments)]
pub fn glow(
    st: &Stars,
    sky: &SkyParams,
    set: &GlowSettings,
    observer_pc: [f64; 3],
    t_myr: f64,
    limit_mag: f64,
    limit: &Passband,
    bands: &[&Passband],
    pc_m: f64,
) -> Result<GlowMap, String> {
    let g = Grid::new(set.nside)?;
    if set.steps < 4 || set.far_pc <= set.near_pc || set.far_pc.is_nan() || set.near_pc <= 0.0 {
        return Err("glow needs at least 4 steps and far beyond near".into());
    }
    let radii = class_radii(st, sky, limit_mag, limit, pc_m);
    let classes = radii.len();
    let ax = sky_axes(observer_pc);
    let mixes: RefCell<BTreeMap<i64, Mix>> = RefCell::new(BTreeMap::new());
    // The bulge's stars: one light mix for the whole bulge.
    let bulge_mix = mix_of(&st.bulge_light_points(t_myr), classes, bands, limit, pc_m);
    let lim_ext = ccm_ratio(limit.effective_m, 3.1);
    let band_ext: Vec<f64> = bands
        .iter()
        .map(|b| ccm_ratio(b.effective_m, 3.1))
        .collect();
    // Log-spaced shells.
    let n = set.steps;
    let ln_ratio = m::ln(set.far_pc / set.near_pc);
    let edge = |k: usize| set.near_pc * m::exp(ln_ratio * k as f64 / n as f64);
    let mut intensity = vec![vec![0.0; g.npix() as usize]; bands.len()];
    let mut a_v_total = vec![0.0; g.npix() as usize];
    for p in 0..g.npix() {
        let v = g.centre_vec(p);
        let mut d = [0.0; 3];
        for (i, di) in d.iter_mut().enumerate() {
            *di = v[0] * ax[0][i] + v[1] * ax[1][i] + v[2] * ax[2][i];
        }
        let mut a_v = 0.0;
        let mut sum = vec![0.0; bands.len()];
        for k in 0..n {
            let (s0, s1) = (edge(k), edge(k + 1));
            let s = m::sqrt(s0 * s1);
            let ds = s1 - s0;
            let pos = [
                observer_pc[0] + s * d[0],
                observer_pc[1] + s * d[1],
                observer_pc[2] + s * d[2],
            ];
            let place = st.place(pos, t_myr);
            let (gas, feh) = st.env.gas_pc3(place);
            let da = gas * m::pow(10.0, feh) * ds / sky.gas_per_av_msun_pc2;
            let a_mid = a_v + 0.5 * da;
            a_v += da;
            // GH4: the remains of torn-apart galaxies glow too (with the
            // disc's light mix at that radius: fudge F49).
            let dens = st.env.systems_pc3(place)
                + st.env.remains_pc3(place) / st.env.params.mean_star_msun;
            let dens_bulge = st.env.bulge_pc3(place) / st.env.params.mean_star_msun;
            if dens <= 0.0 && dens_bulge <= 0.0 {
                continue;
            }
            let bin = m::round(place.r_kpc / set.mix_bin_kpc) as i64;
            let mut cache = mixes.borrow_mut();
            let mix = cache.entry(bin).or_insert_with(|| {
                let pts = st.light_points(bin as f64 * set.mix_bin_kpc, t_myr);
                mix_of(&pts, classes, bands, limit, pc_m)
            });
            // Faintest absolute magnitude still drawn at this distance.
            let m_star = limit_mag - 5.0 * m::log10(s / 10.0) - a_mid * lim_ext;
            for (n, mx) in [(dens, &*mix), (dens_bulge, &bulge_mix)] {
                if n <= 0.0 {
                    continue;
                }
                for (c, cl) in mx.classes.iter().enumerate() {
                    let from = if s > radii[c] {
                        0
                    } else {
                        cl.abs_mag.partition_point(|x| *x <= m_star)
                    };
                    for (b, acc) in sum.iter_mut().enumerate() {
                        let tau = a_mid * band_ext[b] / MAG_PER_TAU;
                        *acc += n * ds * cl.faint[b][from] * m::exp(-tau);
                    }
                }
            }
        }
        for (b, acc) in sum.iter().enumerate() {
            intensity[b][p as usize] = acc / (4.0 * m::PI * pc_m * pc_m);
        }
        a_v_total[p as usize] = a_v;
    }
    Ok(GlowMap {
        nside: set.nside,
        bands: bands.iter().map(|b| b.id.clone()).collect(),
        radii_pc: radii,
        intensity,
        a_v_total,
    })
}

/// Surface brightness, magnitudes per square arcsecond (AB), of I_λ at λ.
pub fn mag_per_arcsec2_ab(i_lambda: f64, lambda_m: f64) -> f64 {
    let arcsec = m::PI / (180.0 * 3600.0);
    let i_nu = i_lambda * lambda_m * lambda_m / C * arcsec * arcsec;
    if i_nu <= 0.0 {
        return f64::INFINITY;
    }
    -2.5 * m::log10(i_nu / AB_ZERO)
}

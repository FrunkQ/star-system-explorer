//! W: the cosmic web as a function, not a grid (galaxy design section 3, stage G1).
//!
//! The initial density contrast delta0(x) is a sum of band-limited noise
//! octaves. Octave j has lattice spacing L0 / 2^j; each lattice value is a
//! keyed normal draw (stream `cosmos.web.o<j>.v1`, counter from the lattice
//! index); values between lattice points come from tricubic Catmull-Rom
//! interpolation (64 lattice values per octave). Octave amplitudes are set so
//! the variance added by octave j is sigma^2(R_j) - sigma^2(R_j-1) from the
//! cosmology (R_j the top-hat radius of one lattice cell's volume). Nothing is
//! stored: a lattice cache lives only for one batch of samples.
//!
//! Linear evolution delta = D(a) delta0; a lognormal transform for density;
//! the potential of octave j is -delta_j / k_j^2 with k_j = pi / spacing_j (the
//! band's centre), so one Zel'dovich step moves sample points down its
//! gradient; the tidal tensor (its Hessian, by central differences) classifies
//! each point as void, sheet, filament or knot by how many eigenvalues exceed
//! a threshold (the T-web, Hahn et al. 2007).

use crate::cosmology::Cosmology;
use serde::{Deserialize, Serialize};
use sse_core::draw::{Draws, Stream};
use sse_core::math as m;
use sse_core::{Address, Seed};
use std::collections::BTreeMap;

/// Pack settings (`provides.other.cosmology.web`).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct WebParams {
    /// Spacing of the coarsest octave, comoving Mpc.
    pub top_spacing_mpc: f64,
    /// Octaves 0 to octaves - 1 (at most 16).
    pub octaves: u32,
    /// T-web eigenvalue threshold (Hahn et al. 2007 use 0).
    pub tidal_threshold: f64,
}

impl Default for WebParams {
    fn default() -> Self {
        WebParams {
            top_spacing_mpc: 64.0,
            octaves: 8,
            tidal_threshold: 0.0,
        }
    }
}

/// Cosmic-web environment.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Environment {
    Void,
    Sheet,
    Filament,
    Knot,
}

impl Environment {
    pub fn from_count(n: usize) -> Self {
        match n {
            0 => Environment::Void,
            1 => Environment::Sheet,
            2 => Environment::Filament,
            _ => Environment::Knot,
        }
    }
}

/// Equal-volume top-hat radius of a cube of side L: (3 / 4 pi)^(1/3) L.
const CELL_RADIUS: f64 = 0.620_350_490_899_400_1;

/// The web of one universe.
pub struct Web {
    pub params: WebParams,
    /// Amplitude per octave (multiplies unit lattice noise).
    pub amplitude: Vec<f64>,
    /// sigma^2(R_j) of the field summed to octave j (the target).
    pub variance: Vec<f64>,
    draws: Vec<Draws>,
}

/// Catmull-Rom weights for the four lattice points around fraction t.
fn cr_weights(t: f64) -> [f64; 4] {
    let t2 = t * t;
    let t3 = t2 * t;
    [
        0.5 * (-t3 + 2.0 * t2 - t),
        0.5 * (3.0 * t3 - 5.0 * t2 + 2.0),
        0.5 * (-3.0 * t3 + 4.0 * t2 + t),
        0.5 * (t3 - t2),
    ]
}

/// Mean over t in [0, 1) of the sum of squared Catmull-Rom weights: the
/// variance of 1-D interpolated unit noise (fixed quadrature).
fn kernel_variance_1d() -> f64 {
    crate::cosmology::quad(
        &|t| cr_weights(t).iter().map(|w| w * w).sum::<f64>(),
        0.0,
        1.0,
        4,
    )
}

/// Lattice values, cached for one batch of samples.
#[derive(Default)]
pub struct Cache {
    values: BTreeMap<(u32, i64, i64, i64), f64>,
}

impl Web {
    pub fn new(
        cosmo: &Cosmology,
        params: WebParams,
        seed: &Seed,
        universe: &Address,
    ) -> Result<Self, String> {
        if params.octaves == 0 || params.octaves > 16 {
            return Err("octaves must be 1 to 16".into());
        }
        let kv = m::pow(kernel_variance_1d(), 3.0);
        let mut amplitude = vec![];
        let mut variance = vec![];
        let mut prev = 0.0;
        for j in 0..params.octaves {
            let spacing = params.top_spacing_mpc / m::pow(2.0, f64::from(j));
            let s2 = cosmo.sigma2_r(CELL_RADIUS * spacing);
            amplitude.push(m::sqrt((s2 - prev).max(0.0) / kv));
            variance.push(s2);
            prev = s2;
        }
        let draws = (0..params.octaves)
            .map(|j| {
                Stream::new(&format!("cosmos.web.o{j}.v1"))
                    .map(|s| Draws::new(*seed, universe.clone(), s))
                    .map_err(|e| e.0)
            })
            .collect::<Result<_, _>>()?;
        Ok(Web {
            params,
            amplitude,
            variance,
            draws,
        })
    }

    pub fn spacing(&self, j: u32) -> f64 {
        self.params.top_spacing_mpc / m::pow(2.0, f64::from(j))
    }

    fn lattice(&self, cache: &mut Cache, j: u32, ix: i64, iy: i64, iz: i64) -> f64 {
        *cache.values.entry((j, ix, iy, iz)).or_insert_with(|| {
            // 21 bits per axis, offset to be non-negative: about 2 million cells each way.
            let off = 1_i64 << 20;
            let enc = |v: i64| ((v + off) as u64) & 0x1f_ffff;
            let idx = (enc(ix) << 42) | (enc(iy) << 21) | enc(iz);
            self.draws[j as usize].normal(idx >> 1 | (idx & 1) << 62)
        })
    }

    /// One octave of delta0 at x (comoving Mpc), before its amplitude.
    fn octave(&self, cache: &mut Cache, j: u32, x: [f64; 3]) -> f64 {
        let s = self.spacing(j);
        let mut base = [0_i64; 3];
        let mut w = [[0.0; 4]; 3];
        for d in 0..3 {
            let u = x[d] / s;
            let f = m::floor(u);
            base[d] = f as i64;
            w[d] = cr_weights(u - f);
        }
        let mut v = 0.0;
        for a in 0..4 {
            for b in 0..4 {
                for c in 0..4 {
                    let l = self.lattice(
                        cache,
                        j,
                        base[0] + a as i64 - 1,
                        base[1] + b as i64 - 1,
                        base[2] + c as i64 - 1,
                    );
                    v += w[0][a] * w[1][b] * w[2][c] * l;
                }
            }
        }
        v
    }

    /// delta0 at x summed over octaves 0 to `upto` (inclusive).
    pub fn delta0(&self, cache: &mut Cache, x: [f64; 3], upto: u32) -> f64 {
        (0..=upto.min(self.params.octaves - 1))
            .map(|j| self.amplitude[j as usize] * self.octave(cache, j, x))
            .sum()
    }

    /// Potential (Mpc^2): sum of -delta_j / k_j^2, k_j = pi / spacing_j.
    pub fn potential(&self, cache: &mut Cache, x: [f64; 3], upto: u32) -> f64 {
        (0..=upto.min(self.params.octaves - 1))
            .map(|j| {
                let k = m::PI / self.spacing(j);
                -self.amplitude[j as usize] * self.octave(cache, j, x) / (k * k)
            })
            .sum()
    }

    /// Tidal tensor (Hessian of the potential) by central differences with
    /// a step of a quarter of the finest included spacing.
    pub fn tidal(&self, cache: &mut Cache, x: [f64; 3], upto: u32) -> [[f64; 3]; 3] {
        let h = 0.25 * self.spacing(upto.min(self.params.octaves - 1));
        let mut at =
            |d: [f64; 3]| self.potential(cache, [x[0] + d[0], x[1] + d[1], x[2] + d[2]], upto);
        let p0 = at([0.0; 3]);
        let mut t = [[0.0; 3]; 3];
        for i in 0..3 {
            let mut e = [0.0; 3];
            e[i] = h;
            let mut f = [0.0; 3];
            f[i] = -h;
            t[i][i] = (at(e) - 2.0 * p0 + at(f)) / (h * h);
            for j in (i + 1)..3 {
                let mut pp = [0.0; 3];
                pp[i] = h;
                pp[j] = h;
                let mut pm = pp;
                pm[j] = -h;
                let mut mp = pp;
                mp[i] = -h;
                let mut mm = [0.0; 3];
                mm[i] = -h;
                mm[j] = -h;
                let v = (at(pp) - at(pm) - at(mp) + at(mm)) / (4.0 * h * h);
                t[i][j] = v;
                t[j][i] = v;
            }
        }
        t
    }

    /// Eigenvalues of the tidal tensor, largest first.
    pub fn tidal_eigenvalues(&self, cache: &mut Cache, x: [f64; 3], upto: u32) -> [f64; 3] {
        let mut e = sym3_eigenvalues(self.tidal(cache, x, upto));
        e.sort_by(|a, b| b.total_cmp(a));
        e
    }

    /// T-web environment at x, smoothed to octave `upto`, at growth D (the
    /// threshold applies to D times the eigenvalues).
    pub fn environment(
        &self,
        cache: &mut Cache,
        x: [f64; 3],
        upto: u32,
        growth: f64,
    ) -> Environment {
        let ev = self.tidal_eigenvalues(cache, x, upto);
        Environment::from_count(
            ev.iter()
                .filter(|l| growth * **l > self.params.tidal_threshold)
                .count(),
        )
    }

    /// Density in units of the mean, lognormal: exp(D delta0 - D^2 sigma^2 / 2).
    pub fn density(&self, cache: &mut Cache, x: [f64; 3], upto: u32, growth: f64) -> f64 {
        let s2 = self.variance[upto.min(self.params.octaves - 1) as usize];
        m::exp(growth * self.delta0(cache, x, upto) - 0.5 * growth * growth * s2)
    }

    /// One Zel'dovich step: where the matter that started at q is at growth D.
    pub fn displaced(&self, cache: &mut Cache, q: [f64; 3], upto: u32, growth: f64) -> [f64; 3] {
        let h = 0.25 * self.spacing(upto.min(self.params.octaves - 1));
        let mut out = q;
        for d in 0..3 {
            let mut a = q;
            a[d] += h;
            let mut b = q;
            b[d] -= h;
            let g = (self.potential(cache, a, upto) - self.potential(cache, b, upto)) / (2.0 * h);
            out[d] = q[d] - growth * g;
        }
        out
    }

    /// A square slice for the foam view: n by n samples on the plane z, side
    /// `extent` Mpc centred on (cx, cy), at scale factor a.
    pub fn slice(
        &self,
        cosmo: &Cosmology,
        a: f64,
        centre: [f64; 3],
        extent: f64,
        n: u32,
        upto: u32,
    ) -> Slice {
        let mut cache = Cache::default();
        let d = cosmo.growth(a);
        let mut density = vec![];
        let mut env = vec![];
        let mut moved = vec![];
        for iy in 0..n {
            for ix in 0..n {
                let x = [
                    centre[0] + extent * ((f64::from(ix) + 0.5) / f64::from(n) - 0.5),
                    centre[1] + extent * ((f64::from(iy) + 0.5) / f64::from(n) - 0.5),
                    centre[2],
                ];
                density.push(self.density(&mut cache, x, upto, d));
                env.push(self.environment(&mut cache, x, upto, d));
                let p = self.displaced(&mut cache, x, upto, d);
                moved.push([p[0], p[1]]);
            }
        }
        Slice {
            scale_factor: a,
            growth: d,
            centre,
            extent,
            n,
            octaves: upto + 1,
            density,
            environment: env,
            displaced: moved,
        }
    }
}

/// A foam-view slice (data only; V9 draws it).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Slice {
    pub scale_factor: f64,
    pub growth: f64,
    pub centre: [f64; 3],
    pub extent: f64,
    pub n: u32,
    pub octaves: u32,
    /// Row-major, density in units of the mean.
    pub density: Vec<f64>,
    pub environment: Vec<Environment>,
    /// Zel'dovich positions of the samples (x, y), Mpc.
    pub displaced: Vec<[f64; 2]>,
}

/// Eigenvalues of a symmetric 3x3 matrix by 12 fixed cyclic Jacobi sweeps.
pub fn sym3_eigenvalues(mut a: [[f64; 3]; 3]) -> [f64; 3] {
    for _ in 0..12 {
        for (p, q) in [(0, 1), (0, 2), (1, 2)] {
            if a[p][q] == 0.0 {
                continue;
            }
            let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
            let t = theta.signum() / (theta.abs() + m::sqrt(theta * theta + 1.0));
            let t = if theta == 0.0 { 1.0 } else { t };
            let c = 1.0 / m::sqrt(t * t + 1.0);
            let s = t * c;
            let mut b = a;
            for k in 0..3 {
                b[k][p] = c * a[k][p] - s * a[k][q];
                b[k][q] = s * a[k][p] + c * a[k][q];
            }
            let mut d = b;
            for k in 0..3 {
                d[p][k] = c * b[p][k] - s * b[q][k];
                d[q][k] = s * b[p][k] + c * b[q][k];
            }
            a = d;
        }
    }
    [a[0][0], a[1][1], a[2][2]]
}

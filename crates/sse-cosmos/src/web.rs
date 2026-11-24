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
    /// T-web eigenvalue threshold (Hahn et al. 2007 use 0), used when
    /// `void_fraction` is not set.
    pub tidal_threshold: f64,
    /// The share of space that is void: when set, the T-web threshold is
    /// the one that gives this share of space for the smoothed Gaussian
    /// field (Doroshkevich's distribution of the tidal tensor's eigenvalues,
    /// each weighted by its Zel'dovich room today), in units of the field's
    /// own spread at the smoothing used, so the share holds at every scale
    /// and time. Measured: about 0.77 (Cautun et al.
    /// 2014). Absent: `tidal_threshold` as given.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub void_fraction: Option<f64>,
}

/// Truncated Zel'dovich approximation (Coles, Melott and Shandarin 1993,
/// MNRAS 260, 765): matter is moved by the field smoothed where its linear
/// spread reaches about one, so filaments stay sharp instead of smearing
/// once orbits cross. Melott, Pellman and Shandarin (1994, MNRAS 269, 626)
/// find the best match to N-body at sigma near 1.
pub const REF_TZA_SIGMA: f64 = 1.0;

/// Draws of a Gaussian field's tidal tensor used to find the T-web
/// threshold for a void share (more draws, a finer quantile).
const TWEB_DRAWS: u64 = 20_000;

impl Default for WebParams {
    fn default() -> Self {
        WebParams {
            top_spacing_mpc: 64.0,
            octaves: 8,
            tidal_threshold: 0.0,
            void_fraction: None,
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
    /// The T-web threshold in units of the smoothed field's spread that
    /// gives `params.void_fraction` (None: the fixed `tidal_threshold`).
    void_t: Option<(f64, Vec<[f64; 3]>)>,
}

/// Draws of a unit Gaussian field's tidal tensor, for the T-web threshold.
/// Doroshkevich (1970): for an isotropic Gaussian field of spread sigma the
/// tensor's diagonal terms have variance sigma^2 / 5 and covariance
/// sigma^2 / 15, its off-diagonal terms variance sigma^2 / 15. Keyed draws
/// (stream `cosmos.web.tweb.v1`), eigenvalues sorted by the largest: the
/// same answer every time.
pub fn tweb_draws(seed: &Seed, universe: &Address) -> Result<Vec<[f64; 3]>, String> {
    let d = Stream::new("cosmos.web.tweb.v1")
        .map(|s| Draws::new(*seed, universe.clone(), s))
        .map_err(|e| e.0)?;
    let a = m::sqrt(2.0 / 15.0);
    let b = m::sqrt(1.0 / 15.0);
    let mut out: Vec<[f64; 3]> = (0..TWEB_DRAWS)
        .map(|i| {
            let g = |k: u64| d.normal(i * 8 + k);
            let common = b * g(0);
            let t = [
                [a * g(1) + common, b * g(4), b * g(5)],
                [b * g(4), a * g(2) + common, b * g(6)],
                [b * g(5), b * g(6), a * g(3) + common],
            ];
            let mut e = sym3_eigenvalues(t);
            e.sort_by(|x, y| y.total_cmp(x));
            e
        })
        .collect();
    out.sort_by(|x, y| x[0].total_cmp(&y[0]));
    Ok(out)
}

/// The Zel'dovich volume of a parcel whose tidal eigenvalues, times the
/// growth, are `l`: |(1 - l1)(1 - l2)(1 - l3)| of the volume it started in
/// (Zel'dovich 1970). Past orbit crossing the absolute value counts the
/// streams' volume once; the truncation keeps that rare.
pub fn zeldovich_volume(l: [f64; 3]) -> f64 {
    ((1.0 - l[0]) * (1.0 - l[1]) * (1.0 - l[2])).abs()
}

/// The T-web threshold, in units of the field's spread, below which voids
/// (all three eigenvalues under it) fill a share `f` of space today: of
/// the space the matter occupies after the Zel'dovich step at linear
/// spread `s` (D sigma), not of the space it started in. Voids have spread
/// out, so they take more room than their matter's share; Cautun et al.
/// (2014) measure the share of space.
pub fn void_threshold(f: f64, s: f64, draws: &[[f64; 3]]) -> Result<f64, String> {
    if !(f > 0.0 && f < 1.0) {
        return Err("void_fraction must be between 0 and 1".into());
    }
    if draws.is_empty() {
        return Err("no tidal draws".into());
    }
    let w = |e: &[f64; 3]| zeldovich_volume([s * e[0], s * e[1], s * e[2]]);
    let total: f64 = draws.iter().map(w).sum();
    let mut run = 0.0;
    for e in draws {
        run += w(e);
        if run >= f * total {
            return Ok(e[0]);
        }
    }
    Ok(draws[draws.len() - 1][0])
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
        let void_t = match params.void_fraction {
            Some(f) => {
                let d = tweb_draws(seed, universe)?;
                void_threshold(f, REF_TZA_SIGMA, &d)?;
                Some((f, d))
            }
            None => None,
        };
        Ok(Web {
            params,
            amplitude,
            variance,
            draws,
            void_t,
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
    /// threshold applies to D times the eigenvalues). For many samples, find
    /// the threshold once and use `environment_with`.
    pub fn environment(
        &self,
        cache: &mut Cache,
        x: [f64; 3],
        upto: u32,
        growth: f64,
    ) -> Environment {
        let th = self.threshold(upto, growth);
        self.environment_with(cache, x, upto, growth, th)
    }

    /// T-web environment at x against a threshold from `threshold`.
    pub fn environment_with(
        &self,
        cache: &mut Cache,
        x: [f64; 3],
        upto: u32,
        growth: f64,
        th: f64,
    ) -> Environment {
        let ev = self.tidal_eigenvalues(cache, x, upto);
        Environment::from_count(ev.iter().filter(|l| growth * **l > th).count())
    }

    /// The T-web threshold at a smoothing and growth: the fixed one, or the
    /// one giving the pack's void share of space at the field's linear
    /// spread there, D sigma(R_upto) (in units of D times eigenvalues).
    pub fn threshold(&self, upto: u32, growth: f64) -> f64 {
        match &self.void_t {
            Some((f, d)) => {
                let s = growth * m::sqrt(self.variance[upto.min(self.params.octaves - 1) as usize]);
                void_threshold(*f, s, d).map_or(self.params.tidal_threshold, |t| t * s)
            }
            None => self.params.tidal_threshold,
        }
    }

    /// The octave the truncated Zel'dovich approximation smooths to at
    /// growth D: the finest whose linear spread D sigma(R_j) is still at
    /// most `REF_TZA_SIGMA` (never coarser than the top octave).
    pub fn truncation(&self, growth: f64) -> u32 {
        let lim = REF_TZA_SIGMA * REF_TZA_SIGMA;
        let mut j = 0;
        for (i, v) in self.variance.iter().enumerate() {
            if growth * growth * v <= lim {
                j = i as u32;
            }
        }
        j
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
        // Where matter has gone and what kind of place it is: read from the
        // field smoothed where it is still about linear (truncated
        // Zel'dovich); the shade keeps the finer detail.
        let tza = self.truncation(d).min(upto);
        let th = self.threshold(tza, d);
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
                env.push(self.environment_with(&mut cache, x, tza, d, th));
                let p = self.displaced(&mut cache, x, tza, d);
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

impl Web {
    /// A cube of the web for the 3D view: n^3 parcels of matter on a
    /// comoving grid of side `extent` Mpc round `centre`, each moved by the
    /// truncated Zel'dovich approximation at scale factor a, with the kind
    /// of place it started in (T-web at the same smoothing) and the density
    /// there. The potential is read once per grid point (and a ring of one
    /// more round the cube) and differenced on the grid, so a cube costs
    /// about one potential per parcel.
    pub fn volume(
        &self,
        cosmo: &Cosmology,
        a: f64,
        centre: [f64; 3],
        extent: f64,
        n: u32,
        upto: u32,
    ) -> Volume {
        let mut cache = Cache::default();
        let d = cosmo.growth(a);
        let t = self.truncation(d).min(upto);
        let g = n as usize + 2;
        let h = extent / f64::from(n);
        let at = |i: usize, d: usize| centre[d] + h * (i as f64 - 0.5) - 0.5 * extent;
        let mut phi = vec![0.0; g * g * g];
        let mut den = vec![0.0; g * g * g];
        for k in 0..g {
            for j in 0..g {
                for i in 0..g {
                    let x = [at(i, 0), at(j, 1), at(k, 2)];
                    phi[(k * g + j) * g + i] = self.potential(&mut cache, x, t);
                    den[(k * g + j) * g + i] = self.density(&mut cache, x, t, d);
                }
            }
        }
        let p = |i: usize, j: usize, k: usize| phi[(k * g + j) * g + i];
        let th = self.threshold(t, d);
        let nn = n as usize;
        let mut position = Vec::with_capacity(nn * nn * nn * 3);
        let mut density = Vec::with_capacity(nn * nn * nn);
        let mut environment = Vec::with_capacity(nn * nn * nn);
        let mut room = Vec::with_capacity(nn * nn * nn);
        for k in 1..=nn {
            for j in 1..=nn {
                for i in 1..=nn {
                    let c = p(i, j, k);
                    let grad = [
                        (p(i + 1, j, k) - p(i - 1, j, k)) / (2.0 * h),
                        (p(i, j + 1, k) - p(i, j - 1, k)) / (2.0 * h),
                        (p(i, j, k + 1) - p(i, j, k - 1)) / (2.0 * h),
                    ];
                    let xx = (p(i + 1, j, k) - 2.0 * c + p(i - 1, j, k)) / (h * h);
                    let yy = (p(i, j + 1, k) - 2.0 * c + p(i, j - 1, k)) / (h * h);
                    let zz = (p(i, j, k + 1) - 2.0 * c + p(i, j, k - 1)) / (h * h);
                    let xy = (p(i + 1, j + 1, k) - p(i + 1, j - 1, k) - p(i - 1, j + 1, k)
                        + p(i - 1, j - 1, k))
                        / (4.0 * h * h);
                    let xz = (p(i + 1, j, k + 1) - p(i + 1, j, k - 1) - p(i - 1, j, k + 1)
                        + p(i - 1, j, k - 1))
                        / (4.0 * h * h);
                    let yz = (p(i, j + 1, k + 1) - p(i, j + 1, k - 1) - p(i, j - 1, k + 1)
                        + p(i, j - 1, k - 1))
                        / (4.0 * h * h);
                    let ev = sym3_eigenvalues([[xx, xy, xz], [xy, yy, yz], [xz, yz, zz]]);
                    let q = [at(i, 0), at(j, 1), at(k, 2)];
                    for dd in 0..3 {
                        position.push(q[dd] - d * grad[dd]);
                    }
                    density.push(den[(k * g + j) * g + i]);
                    environment.push(Environment::from_count(
                        ev.iter().filter(|l| d * **l > th).count(),
                    ));
                    room.push(zeldovich_volume([d * ev[0], d * ev[1], d * ev[2]]));
                }
            }
        }
        Volume {
            scale_factor: a,
            growth: d,
            centre,
            extent,
            n,
            smoothing_octave: t,
            smoothing_mpc: self.spacing(t),
            position,
            density,
            environment,
            room,
        }
    }
}

/// A cube of the web for the 3D view (data only; V9 draws it).
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Volume {
    pub scale_factor: f64,
    pub growth: f64,
    pub centre: [f64; 3],
    pub extent: f64,
    pub n: u32,
    /// The octave the matter's motion and the environments were read at
    /// (truncated Zel'dovich), and its lattice spacing, Mpc.
    pub smoothing_octave: u32,
    pub smoothing_mpc: f64,
    /// Where each parcel is now, comoving Mpc: x, y, z for each, x fastest
    /// then y then z over the grid it started on.
    pub position: Vec<f64>,
    /// Density where it started, in units of the mean (lognormal, linear).
    pub density: Vec<f64>,
    pub environment: Vec<Environment>,
    /// The room each parcel takes today, as a share of the room it started
    /// in (Zel'dovich): over 1 in voids, under 1 where matter gathered.
    pub room: Vec<f64>,
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

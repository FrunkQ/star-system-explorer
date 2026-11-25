//! How a particle scatters light (visual polish K-e): Mie theory for a
//! sphere of any size and refractive index, the one rule behind cloud,
//! haze, dust, plume and ring phase tables. Written fresh from Bohren and
//! Huffman's account of the series (1983, chapter 4 and appendix A): the
//! logarithmic derivative by downward recurrence, the Riccati-Bessel
//! functions upward, the coefficients aₙ and bₙ, and from them the
//! efficiencies, the asymmetry and the amplitudes S₁ and S₂ at each angle.

use crate::common::*;
use sse_core::math as m;
use sse_core::Dimension as D;
use sse_flow::{Atom, AtomError, CostClass};

const BH83: &str = "Bohren, C. F. and Huffman, D. R. 1983, Absorption and Scattering of Light by Small Particles, Wiley, chapter 4 (equations 4.53, 4.61, 4.62, 4.74) and appendix A (BHMIE): the Mie series for a homogeneous sphere, its efficiencies, asymmetry and amplitude functions. doi:10.1002/9783527618156";
const WIS80: &str = "Wiscombe, W. J. 1980, Improved Mie scattering algorithms, Applied Optics 19, 1505: how many terms the series needs (x + 4 x^⅓ + 2) and test values for checking a Mie code. doi:10.1364/AO.19.001505";

/// Largest size parameter worked by the series; beyond it geometric optics
/// (`sse.light.rainbow`, `sse.light.halo`) is the right tool.
pub const MAX_SIZE_PARAMETER: f64 = 20_000.0;

#[derive(Clone, Copy, Debug, PartialEq)]
struct C {
    re: f64,
    im: f64,
}

impl C {
    fn new(re: f64, im: f64) -> C {
        C { re, im }
    }
    fn add(self, o: C) -> C {
        C::new(self.re + o.re, self.im + o.im)
    }
    fn sub(self, o: C) -> C {
        C::new(self.re - o.re, self.im - o.im)
    }
    fn mul(self, o: C) -> C {
        C::new(
            self.re * o.re - self.im * o.im,
            self.re * o.im + self.im * o.re,
        )
    }
    fn scale(self, s: f64) -> C {
        C::new(self.re * s, self.im * s)
    }
    fn div(self, o: C) -> C {
        let d = o.re * o.re + o.im * o.im;
        C::new(
            (self.re * o.re + self.im * o.im) / d,
            (self.im * o.re - self.re * o.im) / d,
        )
    }
    fn conj(self) -> C {
        C::new(self.re, -self.im)
    }
    fn norm2(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
}

/// What one sphere does to light.
#[derive(Clone, Debug, PartialEq)]
pub struct Mie {
    /// 2π r / λ in the surrounding medium.
    pub size_parameter: f64,
    pub q_ext: f64,
    pub q_sca: f64,
    /// Mean cosine of the scattering angle.
    pub g: f64,
    /// Phase function at each asked angle, normalised so that its mean over
    /// all directions is 1 (4π over the sphere).
    pub phase: Vec<f64>,
}

/// The Mie series for a sphere of size parameter `x` and relative
/// refractive index `n + i k`, with the phase function at `angles_rad`.
pub fn mie(x: f64, n: f64, k: f64, angles_rad: &[f64]) -> Result<Mie, String> {
    if !(x > 0.0 && x.is_finite()) {
        return Err(format!("size parameter must be positive, got {x}"));
    }
    if x > MAX_SIZE_PARAMETER {
        return Err(format!(
            "size parameter {x:.0} is past {MAX_SIZE_PARAMETER:.0}: use geometric optics (sse.light.rainbow, sse.light.halo) for drops this large"
        ));
    }
    if !(n > 0.0 && k >= 0.0 && n.is_finite() && k.is_finite()) {
        return Err(format!(
            "refractive index needs n above 0 and k of 0 or more, got {n} + {k}i"
        ));
    }
    let mr = C::new(n, k);
    let y = mr.scale(x);
    let xstop = x + 4.0 * m::cbrt(x) + 2.0;
    let nstop = xstop as usize;
    let ymod = m::sqrt(y.norm2());
    let nmx = (xstop.max(ymod) as usize) + 15;
    // Logarithmic derivative D_n(y), downward.
    let mut d = vec![C::new(0.0, 0.0); nmx + 1];
    for i in (2..=nmx).rev() {
        let en = i as f64;
        let ey = C::new(en, 0.0).div(y);
        d[i - 1] = ey.sub(C::new(1.0, 0.0).div(d[i].add(ey)));
    }
    let mus: Vec<f64> = angles_rad.iter().map(|a| m::cos(*a)).collect();
    let mut pi0 = vec![0.0; mus.len()];
    let mut pi1 = vec![1.0; mus.len()];
    let mut s1 = vec![C::new(0.0, 0.0); mus.len()];
    let mut s2 = vec![C::new(0.0, 0.0); mus.len()];
    let (mut psi0, mut psi1) = (m::cos(x), m::sin(x));
    let (mut chi0, mut chi1) = (-m::sin(x), m::cos(x));
    let mut xi1 = C::new(psi1, -chi1);
    let (mut qsca, mut qext, mut gsum) = (0.0, 0.0, 0.0);
    let mut prev: Option<(C, C)> = None;
    for i in 1..=nstop {
        let en = i as f64;
        let fnn = (2.0 * en + 1.0) / (en * (en + 1.0));
        let psi = (2.0 * en - 1.0) * psi1 / x - psi0;
        let chi = (2.0 * en - 1.0) * chi1 / x - chi0;
        let xi = C::new(psi, -chi);
        let nx = C::new(en / x, 0.0);
        let da = d[i].div(mr).add(nx);
        let db = d[i].mul(mr).add(nx);
        let a = da
            .scale(psi)
            .sub(C::new(psi1, 0.0))
            .div(da.mul(xi).sub(xi1));
        let b = db
            .scale(psi)
            .sub(C::new(psi1, 0.0))
            .div(db.mul(xi).sub(xi1));
        qsca += (2.0 * en + 1.0) * (a.norm2() + b.norm2());
        qext += (2.0 * en + 1.0) * (a.re + b.re);
        gsum += fnn * a.mul(b.conj()).re;
        if let Some((ap, bp)) = prev {
            gsum += (en - 1.0) * (en + 1.0) / en * (ap.mul(a.conj()).re + bp.mul(b.conj()).re);
        }
        prev = Some((a, b));
        for j in 0..mus.len() {
            let p = pi1[j];
            let t = en * mus[j] * p - (en + 1.0) * pi0[j];
            s1[j] = s1[j].add(a.scale(fnn * p).add(b.scale(fnn * t)));
            s2[j] = s2[j].add(a.scale(fnn * t).add(b.scale(fnn * p)));
            let next = ((2.0 * en + 1.0) * mus[j] * p - (en + 1.0) * pi0[j]) / en;
            pi0[j] = p;
            pi1[j] = next;
        }
        psi0 = psi1;
        psi1 = psi;
        chi0 = chi1;
        chi1 = chi;
        xi1 = C::new(psi1, -chi1);
    }
    let x2 = x * x;
    let q_sca = 2.0 * qsca / x2;
    let q_ext = 2.0 * qext / x2;
    let g = if q_sca > 0.0 {
        4.0 * gsum / (x2 * q_sca)
    } else {
        0.0
    };
    let phase = s1
        .iter()
        .zip(&s2)
        .map(|(a, b)| {
            if q_sca > 0.0 {
                2.0 * (a.norm2() + b.norm2()) / (x2 * q_sca)
            } else {
                0.0
            }
        })
        .collect();
    Ok(Mie {
        size_parameter: x,
        q_ext,
        q_sca,
        g,
        phase,
    })
}

pub fn atoms() -> Vec<Box<dyn Atom>> {
    vec![Def {
        id: "sse.light.mie_scattering",
        version: "1.0.0",
        inputs: vec![
            port("radius", "particle.radius", D::LENGTH, Some((1e-10, 1.0))),
            port("wavelength", "light.wavelength", D::LENGTH, Some((1e-8, 1e-1))),
            port("index", "light.refractive_index", D::NONE, None),
            port("absorption", "light.absorption_index", D::NONE, None),
        ],
        outputs: vec![
            port("extinction", "particle.extinction_efficiency", D::NONE, None),
            port("scattering", "particle.scattering_efficiency", D::NONE, None),
            port("asymmetry", "particle.asymmetry", D::NONE, None),
        ],
        params: vec![],
        cost: CostClass::Light,
        references: &[BH83, WIS80],
        curious: "Tiny specks scatter blue light more than red, which is why the sky is blue. Particles as big as the light's wavelength throw it forward, which is why clouds are white and a dusty world's sky turns a different colour from ours.",
        science: "Mie theory: the exact solution for a plane wave on a homogeneous sphere. The series in aₙ and bₙ gives the extinction and scattering efficiencies and the asymmetry parameter g from the size parameter 2πr/λ and the complex refractive index n + ik relative to the medium.",
        formula: "Q_ext = (2/x²) Σ (2n+1) Re(aₙ + bₙ),  Q_sca = (2/x²) Σ (2n+1)(|aₙ|² + |bₙ|²)",
        f: |i, _| {
            let r = positive(i[0], "radius")?;
            let lam = positive(i[1], "wavelength")?;
            let x = 2.0 * m::PI * r / lam;
            let o = mie(x, i[2], i[3].max(0.0), &[]).map_err(AtomError)?;
            Ok(vec![o.q_ext, o.q_sca, o.g])
        },
    }
    .build()]
}

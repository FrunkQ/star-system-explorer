//! Secular orbit changes for quiet stretches: Laplace-Lagrange theory
//! (Murray and Dermott 1999, sections 7.2 to 7.4, eqs. 7.9 to 7.10).
//!
//! Eccentricity and pericentre (h = e sin varpi, k = e cos varpi) and
//! inclination and node (p = I sin Omega, q = I cos Omega) each follow
//! linear equations dh/dt = A k, dk/dt = -A h (B for p and q). Weighted by
//! sqrt(m sqrt(a)) the matrices are symmetric; a fixed-sweep Jacobi
//! rotation diagonalises them, and the solution at any time is a rotation
//! of each mode by its frequency. No step size: any interval in one call.

use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// Laplace coefficient b_s^(j)(alpha) = (1/pi) integral over 0..2pi of
/// cos(j psi) / (1 - 2 alpha cos psi + alpha^2)^s, by the trapezoid rule on
/// 1024 points (exponentially convergent for a periodic integrand).
pub fn laplace(s: f64, j: f64, alpha: f64) -> f64 {
    let n = 1024;
    let mut sum = 0.0;
    for k in 0..n {
        let psi = 2.0 * m::PI * k as f64 / n as f64;
        sum += m::cos(j * psi) / m::pow(1.0 - 2.0 * alpha * m::cos(psi) + alpha * alpha, s);
    }
    2.0 * sum / n as f64
}

/// One planet for secular theory.
#[derive(Clone, Copy, PartialEq, Debug, Serialize, Deserialize)]
pub struct Planet {
    /// Solar masses.
    pub mass: f64,
    pub a: f64,
    pub e: f64,
    /// Longitude of pericentre, radians.
    pub varpi: f64,
    /// Inclination to the reference plane, radians.
    pub i: f64,
    pub node: f64,
}

/// Symmetric eigen-decomposition by cyclic Jacobi rotations, a fixed 60
/// sweeps (far more than an 8 by 8 needs). Returns (values, vectors by column).
pub fn jacobi(mut a: Vec<Vec<f64>>) -> (Vec<f64>, Vec<Vec<f64>>) {
    let n = a.len();
    let mut v = vec![vec![0.0; n]; n];
    for (i, row) in v.iter_mut().enumerate() {
        row[i] = 1.0;
    }
    for _ in 0..60 {
        for p in 0..n {
            for q in (p + 1)..n {
                if a[p][q] == 0.0 {
                    continue;
                }
                let theta = (a[q][q] - a[p][p]) / (2.0 * a[p][q]);
                let t = if theta >= 0.0 { 1.0 } else { -1.0 }
                    / (theta.abs() + m::sqrt(theta * theta + 1.0));
                let c = 1.0 / m::sqrt(t * t + 1.0);
                let s = t * c;
                for k in 0..n {
                    let (akp, akq) = (a[k][p], a[k][q]);
                    a[k][p] = c * akp - s * akq;
                    a[k][q] = s * akp + c * akq;
                }
                for k in 0..n {
                    let (apk, aqk) = (a[p][k], a[q][k]);
                    a[p][k] = c * apk - s * aqk;
                    a[q][k] = s * apk + c * aqk;
                }
                for row in v.iter_mut() {
                    let (vp, vq) = (row[p], row[q]);
                    row[p] = c * vp - s * vq;
                    row[q] = s * vp + c * vq;
                }
            }
        }
    }
    ((0..n).map(|i| a[i][i]).collect(), v)
}

/// The secular system: frequencies (rad/yr) and modes.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Secular {
    /// Eccentricity frequencies g and inclination frequencies f, rad/yr.
    pub g: Vec<f64>,
    pub f: Vec<f64>,
    weights: Vec<f64>,
    ve: Vec<Vec<f64>>,
    vi: Vec<Vec<f64>>,
}

impl Secular {
    /// Matrices for planets around a star of `star_mass` (solar masses).
    pub fn new(star_mass: f64, ps: &[Planet]) -> Self {
        let n = ps.len();
        let mut a = vec![vec![0.0; n]; n];
        let mut b = vec![vec![0.0; n]; n];
        for j in 0..n {
            let nj = m::sqrt(
                crate::kepler::G * (star_mass + ps[j].mass) / (ps[j].a * ps[j].a * ps[j].a),
            );
            for k in 0..n {
                if j == k {
                    continue;
                }
                let (al, albar) = if ps[j].a < ps[k].a {
                    let x = ps[j].a / ps[k].a;
                    (x, x)
                } else {
                    (ps[k].a / ps[j].a, 1.0)
                };
                let f = nj * 0.25 * ps[k].mass / (star_mass + ps[j].mass) * al * albar;
                let b1 = laplace(1.5, 1.0, al);
                let b2 = laplace(1.5, 2.0, al);
                a[j][j] += f * b1;
                a[j][k] = -f * b2;
                b[j][j] -= f * b1;
                b[j][k] = f * b1;
            }
        }
        // Symmetrise with w_j = sqrt(m_j sqrt(a_j)): S = W A W^-1.
        let w: Vec<f64> = ps.iter().map(|p| m::sqrt(p.mass * m::sqrt(p.a))).collect();
        let sym = |x: &Vec<Vec<f64>>| -> Vec<Vec<f64>> {
            (0..n)
                .map(|j| {
                    (0..n)
                        .map(|k| 0.5 * (w[j] * x[j][k] / w[k] + w[k] * x[k][j] / w[j]))
                        .collect()
                })
                .collect()
        };
        let (g, ve) = jacobi(sym(&a));
        let (f, vi) = jacobi(sym(&b));
        Secular {
            g,
            f,
            weights: w,
            ve,
            vi,
        }
    }

    /// Rotate (k, h) pairs by the modes for `dt` years.
    fn evolve(
        w: &[f64],
        vals: &[f64],
        vecs: &[Vec<f64>],
        kh: &[(f64, f64)],
        dt: f64,
    ) -> Vec<(f64, f64)> {
        let n = w.len();
        // c = k + i h, in weighted coordinates; modes z = V^T c; z(t) = exp(i g t) z.
        let cw: Vec<(f64, f64)> = (0..n).map(|j| (w[j] * kh[j].0, w[j] * kh[j].1)).collect();
        let mut out = vec![(0.0, 0.0); n];
        for i in 0..n {
            let (mut zr, mut zi) = (0.0, 0.0);
            for j in 0..n {
                zr += vecs[j][i] * cw[j].0;
                zi += vecs[j][i] * cw[j].1;
            }
            let (c, s) = (m::cos(vals[i] * dt), m::sin(vals[i] * dt));
            let (nr, ni) = (zr * c - zi * s, zr * s + zi * c);
            for j in 0..n {
                out[j].0 += vecs[j][i] * nr;
                out[j].1 += vecs[j][i] * ni;
            }
        }
        (0..n).map(|j| (out[j].0 / w[j], out[j].1 / w[j])).collect()
    }

    /// The planets after `dt` years (a and mass unchanged).
    pub fn advance(&self, ps: &[Planet], dt: f64) -> Vec<Planet> {
        let ekh: Vec<(f64, f64)> = ps
            .iter()
            .map(|p| (p.e * m::cos(p.varpi), p.e * m::sin(p.varpi)))
            .collect();
        let iqp: Vec<(f64, f64)> = ps
            .iter()
            .map(|p| (p.i * m::cos(p.node), p.i * m::sin(p.node)))
            .collect();
        let e2 = Self::evolve(&self.weights, &self.g, &self.ve, &ekh, dt);
        let i2 = Self::evolve(&self.weights, &self.f, &self.vi, &iqp, dt);
        ps.iter()
            .zip(e2.iter().zip(&i2))
            .map(|(p, ((k, h), (q, pp)))| Planet {
                e: m::hypot(*k, *h),
                varpi: m::atan2(*h, *k),
                i: m::hypot(*q, *pp),
                node: m::atan2(*pp, *q),
                ..*p
            })
            .collect()
    }
}

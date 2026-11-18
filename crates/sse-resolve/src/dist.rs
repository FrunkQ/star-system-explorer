//! Distributions a cloud declares for its children (H6.1 `distributions`).
//!
//! Every distribution is sampled through its quantile function from one keyed
//! draw, so sampling, the cumulative distribution and the location-first
//! outlook (H6.4) all read the same definition. Maths is pinned: `sse_core::math`
//! plus `erfc` from the same pinned `libm`.

use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// What a sample is.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Sample {
    Int(u64),
    Num(f64),
    Cat(String),
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum Distribution {
    Fixed {
        value: f64,
    },
    Uniform {
        lo: f64,
        hi: f64,
    },
    /// Normal, optionally truncated to `[lo, hi]` (sampled inside the range,
    /// never clamped, so the shape inside is honest).
    Normal {
        mean: f64,
        sd: f64,
        #[serde(default)]
        lo: Option<f64>,
        #[serde(default)]
        hi: Option<f64>,
    },
    /// Log-normal in base 10: `log10(x) ~ Normal(log10(median), sigma_dex)`.
    LogNormal {
        median: f64,
        sigma_dex: f64,
    },
    /// `p(x) ∝ x^-alpha` on `[lo, hi]`, `0 < lo < hi` (initial mass functions,
    /// size spectra).
    PowerLaw {
        alpha: f64,
        lo: f64,
        hi: f64,
    },
    /// A continuous broken power law: `p(x) ∝ x^-alphas[i]` on
    /// `[edges[i], edges[i+1]]`, joined without a jump (the Kroupa 2001
    /// initial mass function is three pieces). `0 < edges` ascending,
    /// `alphas.len() == edges.len() - 1`.
    BrokenPowerLaw {
        edges: Vec<f64>,
        alphas: Vec<f64>,
    },
    /// A labelled category with relative weights.
    Choice {
        options: Vec<(String, f64)>,
    },
    /// A count.
    Poisson {
        mean: f64,
    },
}

#[derive(Clone, Debug, PartialEq)]
pub struct DistError(pub String);
impl std::fmt::Display for DistError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

const SQRT2: f64 = core::f64::consts::SQRT_2;

/// Standard normal cumulative distribution.
pub fn std_normal_cdf(x: f64) -> f64 {
    0.5 * libm::erfc(-x / SQRT2)
}

/// Standard normal quantile: Acklam's rational approximation, then one Halley
/// step against `erfc`, all in pinned maths. `p` must be in (0, 1).
pub fn std_normal_quantile(p: f64) -> f64 {
    const A: [f64; 6] = [
        -3.969_683_028_665_376e1,
        2.209_460_984_245_205e2,
        -2.759_285_104_469_687e2,
        1.383_577_518_672_69e2,
        -3.066_479_806_614_716e1,
        2.506_628_277_459_239,
    ];
    const B: [f64; 5] = [
        -5.447_609_879_822_406e1,
        1.615_858_368_580_409e2,
        -1.556_989_798_598_866e2,
        6.680_131_188_771_972e1,
        -1.328_068_155_288_572e1,
    ];
    const C: [f64; 6] = [
        -7.784_894_002_430_293e-3,
        -3.223_964_580_411_365e-1,
        -2.400_758_277_161_838,
        -2.549_732_539_343_734,
        4.374_664_141_464_968,
        2.938_163_982_698_783,
    ];
    const D: [f64; 4] = [
        7.784_695_709_041_462e-3,
        3.224_671_290_700_398e-1,
        2.445_134_137_142_996,
        3.754_408_661_907_416,
    ];
    const P_LOW: f64 = 0.02425;
    let tail = |q: f64| {
        (((((C[0] * q + C[1]) * q + C[2]) * q + C[3]) * q + C[4]) * q + C[5])
            / ((((D[0] * q + D[1]) * q + D[2]) * q + D[3]) * q + 1.0)
    };
    let mut x = if p < P_LOW {
        tail(m::sqrt(-2.0 * m::ln(p)))
    } else if p <= 1.0 - P_LOW {
        let q = p - 0.5;
        let r = q * q;
        (((((A[0] * r + A[1]) * r + A[2]) * r + A[3]) * r + A[4]) * r + A[5]) * q
            / (((((B[0] * r + B[1]) * r + B[2]) * r + B[3]) * r + B[4]) * r + 1.0)
    } else {
        -tail(m::sqrt(-2.0 * m::ln(1.0 - p)))
    };
    // One Halley refinement.
    let e = std_normal_cdf(x) - p;
    let u = e * m::sqrt(m::TAU) * m::exp(x * x / 2.0);
    x -= u / (1.0 + x * u / 2.0);
    x
}

impl Distribution {
    pub fn validate(&self) -> Result<(), DistError> {
        let bad = |s: &str| Err(DistError(s.into()));
        let fin = |xs: &[f64]| xs.iter().all(|x| x.is_finite());
        match self {
            Distribution::Fixed { value } if !value.is_finite() => {
                bad("fixed value must be finite")
            }
            Distribution::Uniform { lo, hi } if !fin(&[*lo, *hi]) || lo > hi => {
                bad("uniform needs lo <= hi")
            }
            Distribution::Normal { mean, sd, lo, hi } => {
                if !fin(&[*mean, *sd]) || *sd <= 0.0 {
                    return bad("normal needs sd > 0");
                }
                if let (Some(l), Some(h)) = (lo, hi) {
                    if l >= h {
                        return bad("normal truncation needs lo < hi");
                    }
                }
                Ok(())
            }
            Distribution::LogNormal { median, sigma_dex }
                if !fin(&[*median, *sigma_dex]) || *median <= 0.0 || *sigma_dex <= 0.0 =>
            {
                bad("log-normal needs median > 0 and sigma_dex > 0")
            }
            Distribution::PowerLaw { alpha, lo, hi }
                if !fin(&[*alpha, *lo, *hi]) || *lo <= 0.0 || lo >= hi =>
            {
                bad("power law needs 0 < lo < hi")
            }
            Distribution::BrokenPowerLaw { edges, alphas } => {
                if alphas.is_empty()
                    || edges.len() != alphas.len() + 1
                    || !fin(edges)
                    || !fin(alphas)
                    || edges[0] <= 0.0
                    || edges.windows(2).any(|w| w[0] >= w[1])
                {
                    return bad(
                        "broken power law needs ascending edges > 0 and one alpha per piece",
                    );
                }
                Ok(())
            }
            Distribution::Choice { options }
                if options.is_empty()
                    || options.iter().any(|(_, w)| !w.is_finite() || *w < 0.0)
                    || options.iter().all(|(_, w)| *w == 0.0) =>
            {
                bad("choice needs at least one positive weight")
            }
            Distribution::Poisson { mean } if !mean.is_finite() || *mean < 0.0 => {
                bad("poisson needs mean >= 0")
            }
            _ => Ok(()),
        }
    }

    /// The sample at probability `u` in (0, 1).
    pub fn quantile(&self, u: f64) -> Sample {
        match self {
            Distribution::Fixed { value } => Sample::Num(*value),
            Distribution::Uniform { lo, hi } => Sample::Num(lo + (hi - lo) * u),
            Distribution::Normal { mean, sd, lo, hi } => {
                let fl = lo.map_or(0.0, |l| std_normal_cdf((l - mean) / sd));
                let fh = hi.map_or(1.0, |h| std_normal_cdf((h - mean) / sd));
                let p = (fl + (fh - fl) * u).clamp(f64::MIN_POSITIVE, 1.0 - f64::EPSILON / 2.0);
                let mut x = mean + sd * std_normal_quantile(p);
                if let Some(l) = lo {
                    x = x.max(*l);
                }
                if let Some(h) = hi {
                    x = x.min(*h);
                }
                Sample::Num(x)
            }
            Distribution::LogNormal { median, sigma_dex } => Sample::Num(m::pow(
                10.0,
                m::log10(*median) + sigma_dex * std_normal_quantile(u),
            )),
            Distribution::PowerLaw { alpha, lo, hi } => {
                Sample::Num(power_law_quantile(*alpha, *lo, *hi, u))
            }
            Distribution::BrokenPowerLaw { edges, alphas } => {
                Sample::Num(broken_quantile(edges, alphas, u))
            }
            Distribution::Choice { options } => {
                let total: f64 = options.iter().map(|(_, w)| w).sum();
                let target = u * total;
                let mut acc = 0.0;
                for (label, w) in options {
                    acc += w;
                    if target < acc {
                        return Sample::Cat(label.clone());
                    }
                }
                let last = options
                    .iter()
                    .rev()
                    .find(|(_, w)| *w > 0.0)
                    .expect("validated");
                Sample::Cat(last.0.clone())
            }
            Distribution::Poisson { mean } => Sample::Int(poisson_quantile(*mean, u)),
        }
    }

    /// `P(X <= x)` for numeric distributions; `None` for categories.
    pub fn cdf(&self, x: f64) -> Option<f64> {
        Some(match self {
            Distribution::Fixed { value } => (x >= *value) as u8 as f64,
            Distribution::Uniform { lo, hi } => {
                if x < *lo {
                    0.0
                } else if x >= *hi {
                    1.0
                } else {
                    (x - lo) / (hi - lo)
                }
            }
            Distribution::Normal { mean, sd, lo, hi } => {
                let fl = lo.map_or(0.0, |l| std_normal_cdf((l - mean) / sd));
                let fh = hi.map_or(1.0, |h| std_normal_cdf((h - mean) / sd));
                let f = std_normal_cdf((x - mean) / sd);
                ((f - fl) / (fh - fl)).clamp(0.0, 1.0)
            }
            Distribution::LogNormal { median, sigma_dex } => {
                if x <= 0.0 {
                    0.0
                } else {
                    std_normal_cdf((m::log10(x) - m::log10(*median)) / sigma_dex)
                }
            }
            Distribution::PowerLaw { alpha, lo, hi } => {
                if x <= *lo {
                    0.0
                } else if x >= *hi {
                    1.0
                } else {
                    power_law_cdf(*alpha, *lo, *hi, x)
                }
            }
            Distribution::BrokenPowerLaw { edges, alphas } => broken_cdf(edges, alphas, x),
            Distribution::Choice { .. } => return None,
            Distribution::Poisson { mean } => {
                if x < 0.0 {
                    0.0
                } else {
                    poisson_cdf(*mean, m::floor(x) as u64)
                }
            }
        })
    }

    /// `P(lo <= X <= hi)` for numeric distributions (closed interval; for
    /// continuous ones the ends carry no mass).
    pub fn prob_between(&self, lo: f64, hi: f64) -> Option<f64> {
        if lo > hi {
            return Some(0.0);
        }
        let below_lo = match self {
            Distribution::Poisson { .. } => {
                if lo <= 0.0 {
                    0.0
                } else {
                    self.cdf(m::ceil(lo) - 1.0)?
                }
            }
            Distribution::Fixed { value } => (*value < lo) as u8 as f64,
            _ => self.cdf(lo)?,
        };
        Some((self.cdf(hi)? - below_lo).clamp(0.0, 1.0))
    }

    /// `P(X == label)` for a choice.
    pub fn prob_is(&self, label: &str) -> Option<f64> {
        match self {
            Distribution::Choice { options } => {
                let total: f64 = options.iter().map(|(_, w)| w).sum();
                Some(
                    options
                        .iter()
                        .filter(|(l, _)| l == label)
                        .map(|(_, w)| w)
                        .sum::<f64>()
                        / total,
                )
            }
            _ => None,
        }
    }

    /// Mean, where it is closed-form (Poisson, uniform, fixed, normal untruncated,
    /// log-normal, power law); `None` for categories and truncated normals.
    pub fn mean(&self) -> Option<f64> {
        Some(match self {
            Distribution::Fixed { value } => *value,
            Distribution::Uniform { lo, hi } => (lo + hi) / 2.0,
            Distribution::Normal {
                mean,
                lo: None,
                hi: None,
                ..
            } => *mean,
            Distribution::Normal { .. } => return None,
            Distribution::LogNormal { median, sigma_dex } => {
                let s = sigma_dex * m::ln(10.0);
                median * m::exp(s * s / 2.0)
            }
            Distribution::PowerLaw { alpha, lo, hi } => {
                let a = *alpha;
                let norm = power_law_integral(-a, *lo, *hi);
                power_law_integral(1.0 - a, *lo, *hi) / norm
            }
            Distribution::BrokenPowerLaw { edges, alphas } => {
                let pieces = broken_pieces(edges, alphas);
                let mass: f64 = pieces
                    .iter()
                    .map(|(c, a, lo, hi)| c * power_law_integral(1.0 - a, *lo, *hi))
                    .sum();
                mass / pieces.iter().map(|p| p.0 * p_int(p)).sum::<f64>()
            }
            Distribution::Choice { .. } => return None,
            Distribution::Poisson { mean } => *mean,
        })
    }
}

/// `∫ x^k dx` on [lo, hi].
fn power_law_integral(k: f64, lo: f64, hi: f64) -> f64 {
    if (k + 1.0).abs() < 1e-12 {
        m::ln(hi / lo)
    } else {
        (m::pow(hi, k + 1.0) - m::pow(lo, k + 1.0)) / (k + 1.0)
    }
}

/// Pieces of a continuous broken power law: (coefficient, alpha, lo, hi),
/// coefficients chosen so the density has no jump at the edges.
fn broken_pieces(edges: &[f64], alphas: &[f64]) -> Vec<(f64, f64, f64, f64)> {
    let mut c = 1.0;
    let mut out = Vec::with_capacity(alphas.len());
    for (i, a) in alphas.iter().enumerate() {
        if i > 0 {
            // c_i x^-a_i = c_{i-1} x^-a_{i-1} at x = edges[i].
            c *= m::pow(edges[i], a - alphas[i - 1]);
        }
        out.push((c, *a, edges[i], edges[i + 1]));
    }
    out
}
fn p_int(p: &(f64, f64, f64, f64)) -> f64 {
    power_law_integral(-p.1, p.2, p.3)
}
fn broken_cdf(edges: &[f64], alphas: &[f64], x: f64) -> f64 {
    let pieces = broken_pieces(edges, alphas);
    let total: f64 = pieces.iter().map(|p| p.0 * p_int(p)).sum();
    if x <= edges[0] {
        return 0.0;
    }
    let mut acc = 0.0;
    for p in &pieces {
        if x >= p.3 {
            acc += p.0 * p_int(p);
        } else {
            acc += p.0 * power_law_integral(-p.1, p.2, x);
            break;
        }
    }
    (acc / total).clamp(0.0, 1.0)
}
fn broken_quantile(edges: &[f64], alphas: &[f64], u: f64) -> f64 {
    let pieces = broken_pieces(edges, alphas);
    let w: Vec<f64> = pieces.iter().map(|p| p.0 * p_int(p)).collect();
    let total: f64 = w.iter().sum();
    let mut target = u * total;
    for (p, wi) in pieces.iter().zip(&w) {
        if target < *wi {
            return power_law_quantile(p.1, p.2, p.3, target / wi);
        }
        target -= wi;
    }
    let last = pieces.last().expect("validated");
    last.3
}

fn power_law_cdf(alpha: f64, lo: f64, hi: f64, x: f64) -> f64 {
    power_law_integral(-alpha, lo, x) / power_law_integral(-alpha, lo, hi)
}

fn power_law_quantile(alpha: f64, lo: f64, hi: f64, u: f64) -> f64 {
    let k = 1.0 - alpha;
    if k.abs() < 1e-12 {
        lo * m::exp(u * m::ln(hi / lo))
    } else {
        let a = m::pow(lo, k);
        let b = m::pow(hi, k);
        m::pow(a + u * (b - a), 1.0 / k).clamp(lo, hi)
    }
}

/// Above this mean a Poisson count uses the normal approximation (with
/// continuity correction), so the cost is bounded.
const POISSON_NORMAL_ABOVE: f64 = 500.0;

fn poisson_cdf(mean: f64, k: u64) -> f64 {
    if mean == 0.0 {
        return 1.0;
    }
    if mean > POISSON_NORMAL_ABOVE {
        return std_normal_cdf((k as f64 + 0.5 - mean) / m::sqrt(mean));
    }
    let mut p = m::exp(-mean);
    let mut acc = p;
    for i in 1..=k {
        p *= mean / i as f64;
        acc += p;
        if acc >= 1.0 {
            return 1.0;
        }
    }
    acc.min(1.0)
}

fn poisson_quantile(mean: f64, u: f64) -> u64 {
    if mean == 0.0 {
        return 0;
    }
    if mean > POISSON_NORMAL_ABOVE {
        let x = mean + m::sqrt(mean) * std_normal_quantile(u);
        return m::round(x).max(0.0) as u64;
    }
    let mut p = m::exp(-mean);
    let mut acc = p;
    let mut k = 0u64;
    // Bounded: the tail beyond mean + 40 sd + 40 carries no representable mass.
    let cap = (mean + 40.0 * m::sqrt(mean) + 40.0) as u64;
    while acc <= u && k < cap {
        k += 1;
        p *= mean / k as f64;
        acc += p;
    }
    k
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_quantile_inverts_cdf() {
        for i in 1..1000 {
            let p = i as f64 / 1000.0;
            let x = std_normal_quantile(p);
            assert!((std_normal_cdf(x) - p).abs() < 1e-13, "p={p}");
        }
        assert!((std_normal_quantile(0.975) - 1.959_963_984_540_054).abs() < 1e-12);
    }

    #[test]
    fn quantile_and_cdf_agree() {
        let ds = [
            Distribution::Uniform { lo: 2.0, hi: 5.0 },
            Distribution::Normal {
                mean: 1.0,
                sd: 2.0,
                lo: Some(0.0),
                hi: Some(3.0),
            },
            Distribution::LogNormal {
                median: 3.0,
                sigma_dex: 0.4,
            },
            Distribution::PowerLaw {
                alpha: 2.35,
                lo: 0.08,
                hi: 100.0,
            },
            Distribution::PowerLaw {
                alpha: 1.0,
                lo: 1.0,
                hi: 10.0,
            },
        ];
        for d in ds {
            d.validate().unwrap();
            for i in 1..100 {
                let u = i as f64 / 100.0;
                let Sample::Num(x) = d.quantile(u) else {
                    panic!()
                };
                assert!((d.cdf(x).unwrap() - u).abs() < 1e-9, "{d:?} u={u}");
            }
        }
    }

    #[test]
    fn poisson_and_choice() {
        let p = Distribution::Poisson { mean: 3.0 };
        assert_eq!(p.quantile(0.01), Sample::Int(0));
        assert_eq!(p.quantile(0.5), Sample::Int(3));
        assert!((p.prob_between(3.0, 3.0).unwrap() - 0.224_041_807_655_388_6).abs() < 1e-12);
        let c = Distribution::Choice {
            options: vec![("a".into(), 1.0), ("b".into(), 3.0)],
        };
        assert_eq!(c.quantile(0.2), Sample::Cat("a".into()));
        assert_eq!(c.quantile(0.3), Sample::Cat("b".into()));
        assert_eq!(c.prob_is("b"), Some(0.75));
        assert!(Distribution::Choice {
            options: vec![("a".into(), 0.0)]
        }
        .validate()
        .is_err());
    }
}

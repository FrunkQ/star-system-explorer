//! O12 (kernel side): light as it reaches an observer, for the spectrum chart
//! (nebulae-and-light design 5B). One target may be several components (a
//! double star, a planet beside its star); each is shifted by its own line of
//! sight speed, thinned by distance, and summed. Lines are marked where they
//! arrive: emission lines from the description, absorption lines from the
//! photosphere's temperature. Readings (temperature from the peak, speed from
//! line shifts, doubled lines, flux) carry a "Why?" and are made only from
//! what the observer's bands cover.

use crate::shift::*;
use crate::*;
use serde::{Deserialize, Serialize};

/// Wien's displacement constant for the peak of B_λ, m K (CODATA 2018).
pub const WIEN_B: f64 = sse_core::sheet::B_WIEN;
const R_WIEN: &str = "Wien's displacement law: the peak of B_λ(T) lies at λ T = 2.897771955×10⁻³ m K (CODATA 2018). https://physics.nist.gov/cgi-bin/cuu/Value?bwien";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Component {
    pub light: LightDescription,
    /// Speed along the line of sight, positive when receding, m/s.
    #[serde(default)]
    pub radial_velocity_m_s: f64,
    /// Photosphere temperature, for absorption lines (none for planets and nebulae).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub photosphere_k: Option<f64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Target {
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub distance_m: f64,
    pub components: Vec<Component>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Coverage {
    pub name: String,
    pub from_m: f64,
    pub to_m: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Observer {
    /// Wavelength ranges the instruments cover; empty means every band (the GM's view).
    #[serde(default)]
    pub coverage: Vec<Coverage>,
    /// λ/Δλ of the spectrograph; lines closer than λ/R blur into one.
    #[serde(default = "default_r")]
    pub resolving_power: f64,
}

fn default_r() -> f64 {
    5000.0
}

impl Observer {
    pub fn covers(&self, l: f64) -> bool {
        self.coverage.is_empty() || self.coverage.iter().any(|c| l >= c.from_m && l <= c.to_m)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LineMark {
    pub line: String,
    pub name: String,
    pub species: String,
    /// "emission" or "absorption".
    pub kind: String,
    pub rest_m: f64,
    pub observed_m: f64,
    pub component: usize,
    /// Emission: flux at the observer, W/m². Absorption: strength 0 to 1 (stand-in).
    pub amount: f64,
    pub covered: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Reading {
    pub what: String,
    pub value: f64,
    pub unit: String,
    pub why: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Spectrum {
    pub address: String,
    pub distance_m: f64,
    /// `[wavelength m, flux density W m⁻² m⁻¹, covered 0 or 1]` on the pack grid.
    pub samples: Vec<[f64; 3]>,
    /// Flux per band at the observer, W/m².
    pub bands: Vec<BandPower>,
    pub lines: Vec<LineMark>,
    pub readings: Vec<Reading>,
    pub coverage: Vec<Coverage>,
    pub why: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub stand_ins: Vec<String>,
    pub references: Vec<Reference>,
}

fn shift_for(v: f64) -> Shift {
    if v == 0.0 {
        Shift::none()
    } else {
        // Receding: velocity points away from the observer.
        Shift::doppler(v.abs() / C, if v > 0.0 { -1.0 } else { 1.0 }, false)
    }
}

/// Flux density at the observer, W m⁻² m⁻¹, continuum of all components.
pub fn flux_density(t: &Target, lambda_m: f64, table: &LightTable) -> f64 {
    let dil = 1.0 / (4.0 * m::PI * t.distance_m * t.distance_m);
    t.components
        .iter()
        .map(|c| {
            shifted_spectral_luminosity(
                &c.light,
                lambda_m,
                &shift_for(c.radial_velocity_m_s),
                table,
            )
        })
        .sum::<f64>()
        * dil
}

/// Where the flux density peaks between `a` and `b`: a fine log scan, then golden-section refinement.
fn peak_between(t: &Target, a: f64, b: f64, table: &LightTable) -> f64 {
    let n = 400;
    let (la, lb) = (m::ln(a), m::ln(b));
    let at = |i: usize| m::exp(la + (lb - la) * i as f64 / n as f64);
    let mut best = 0;
    let mut bv = -1.0;
    for i in 0..=n {
        let v = flux_density(t, at(i), table);
        if v > bv {
            bv = v;
            best = i;
        }
    }
    let (mut lo, mut hi) = (
        m::ln(at(best.saturating_sub(1))),
        m::ln(at((best + 1).min(n))),
    );
    let g = 0.618_033_988_749_895;
    for _ in 0..80 {
        let x1 = hi - g * (hi - lo);
        let x2 = lo + g * (hi - lo);
        if flux_density(t, m::exp(x1), table) < flux_density(t, m::exp(x2), table) {
            lo = x1;
        } else {
            hi = x2;
        }
    }
    m::exp(0.5 * (lo + hi))
}

/// The spectrum of `t` as `o` would record it.
pub fn observe(t: &Target, o: &Observer, table: &LightTable) -> Result<Spectrum, String> {
    if t.distance_m.is_nan() || t.distance_m <= 0.0 || t.components.is_empty() {
        return Err(format!(
            "{}: needs a positive distance and at least one component",
            t.address
        ));
    }
    let dil = 1.0 / (4.0 * m::PI * t.distance_m * t.distance_m);
    let mut why = vec![];
    let mut stand_ins = vec![];
    let mut references = vec![Reference::parse(R_WIEN), Reference::parse(R_RL79_SHIFT)];
    let samples: Vec<[f64; 3]> = table
        .grid()
        .into_iter()
        .map(|l| {
            [
                l,
                flux_density(t, l, table),
                if o.covers(l) { 1.0 } else { 0.0 },
            ]
        })
        .collect();
    let bands: Vec<BandPower> = table
        .bands
        .iter()
        .map(|b| BandPower {
            band: b.id.clone(),
            power_w: t
                .components
                .iter()
                .map(|c| {
                    shifted_power(
                        &c.light,
                        b.from_m,
                        b.to_m,
                        &shift_for(c.radial_velocity_m_s),
                        table,
                    )
                })
                .sum::<f64>()
                * dil,
        })
        .collect();
    // Lines.
    let mut lines = vec![];
    for (ci, c) in t.components.iter().enumerate() {
        let s = shift_for(c.radial_velocity_m_s);
        for le in &c.light.lines {
            if let Some(def) = table.line(&le.line) {
                let obs = s.line_at(def.at_m);
                lines.push(LineMark {
                    line: def.id.clone(),
                    name: def.name.clone(),
                    species: def.species.clone(),
                    kind: "emission".into(),
                    rest_m: def.at_m,
                    observed_m: obs,
                    component: ci,
                    amount: le.power_w * s.boost() * dil,
                    covered: o.covers(obs),
                });
            }
        }
        if let (Some(tk), Some(abs)) = (c.photosphere_k, &table.absorption) {
            for (id, strength) in abs.at(tk) {
                if strength < 0.05 {
                    continue;
                }
                let def = table.line(id).expect("checked with the table");
                let obs = s.line_at(def.at_m);
                lines.push(LineMark {
                    line: def.id.clone(),
                    name: def.name.clone(),
                    species: def.species.clone(),
                    kind: "absorption".into(),
                    rest_m: def.at_m,
                    observed_m: obs,
                    component: ci,
                    amount: strength,
                    covered: o.covers(obs),
                });
            }
            if !stand_ins
                .iter()
                .any(|x: &String| x.starts_with("absorption"))
            {
                stand_ins.push("absorption line strengths are qualitative (0 to 1) by temperature after the MK classification criteria, until the O4 line-strength grid".into());
                if let Some(r) = table.references.iter().find(|r| r.text.starts_with("Gray")) {
                    references.push(r.clone());
                }
            }
        }
    }
    lines.sort_by(|a, b| {
        a.observed_m
            .total_cmp(&b.observed_m)
            .then(a.component.cmp(&b.component))
    });
    // Readings, from covered light only.
    let mut readings = vec![];
    let total: f64 = bands.iter().map(|b: &BandPower| b.power_w).sum();
    let covered_flux: f64 = if o.coverage.is_empty() {
        total
    } else {
        o.coverage
            .iter()
            .map(|cv| {
                t.components
                    .iter()
                    .map(|c| {
                        shifted_power(
                            &c.light,
                            cv.from_m,
                            cv.to_m,
                            &shift_for(c.radial_velocity_m_s),
                            table,
                        )
                    })
                    .sum::<f64>()
                    * dil
            })
            .sum()
    };
    readings.push(Reading {
        what: "flux in your bands".into(),
        value: covered_flux,
        unit: "W/m²".into(),
        why: format!(
            "All the light your instruments cover, after {:.3e} m of distance thins it by 4πd².",
            t.distance_m
        ),
    });
    // Temperature from the peak, if a covered range holds an interior peak.
    let ranges: Vec<(f64, f64)> = if o.coverage.is_empty() {
        vec![(table.grid.from_m, table.grid.to_m)]
    } else {
        o.coverage.iter().map(|c| (c.from_m, c.to_m)).collect()
    };
    let mut peak: Option<f64> = None;
    for (a, b) in ranges {
        let p = peak_between(t, a, b, table);
        let interior = p > a * 1.02 && p < b / 1.02;
        if interior && peak.is_none_or(|q| flux_density(t, p, table) > flux_density(t, q, table)) {
            peak = Some(p);
        }
    }
    match peak {
        Some(p) => readings.push(Reading {
            what: "temperature from the peak".into(),
            value: WIEN_B / p,
            unit: "K".into(),
            why: format!("The light is brightest at {:.1} nm; by Wien's law a glowing body peaks at 2.898 mm·K over its temperature, so about {:.0} K.", p * 1e9, WIEN_B / p),
        }),
        None => why.push("The brightest part of this light lies outside your instruments' bands, so its temperature cannot be read from the peak.".into()),
    }
    // Speeds from covered lines, per component.
    let mut speeds: Vec<(usize, f64)> = vec![];
    for ci in 0..t.components.len() {
        if let Some(lm) = lines.iter().find(|l| l.component == ci && l.covered) {
            let v = C * (lm.observed_m / lm.rest_m - 1.0);
            speeds.push((ci, v));
            readings.push(Reading {
                what: format!("line-of-sight speed, component {ci}"),
                value: v,
                unit: "m/s".into(),
                why: format!(
                    "{} sits at {:.4} nm instead of {:.4} nm: {} at {:.1} km/s.",
                    lm.name,
                    lm.observed_m * 1e9,
                    lm.rest_m * 1e9,
                    if v >= 0.0 {
                        "moving away"
                    } else {
                        "coming closer"
                    },
                    v.abs() / 1e3
                ),
            });
        }
    }
    if speeds.len() >= 2 {
        let (lo, hi) = speeds
            .iter()
            .fold((f64::MAX, f64::MIN), |(a, b), &(_, v)| (a.min(v), b.max(v)));
        let split = (hi - lo) / C;
        let resolved = split > 1.0 / o.resolving_power;
        readings.push(Reading {
            what: "lines doubled".into(),
            value: if resolved { 1.0 } else { 0.0 },
            unit: "yes/no".into(),
            why: if resolved {
                format!("Every line appears twice, {:.1} km/s apart: two sources moving differently, such as a double star.", (hi - lo) / 1e3)
            } else {
                format!("Two sources {:.1} km/s apart, but your spectrograph (R = {:.0}) blurs their lines into one.", (hi - lo) / 1e3, o.resolving_power)
            },
        });
    }
    if !o.coverage.is_empty() {
        let names: Vec<&str> = o.coverage.iter().map(|c| c.name.as_str()).collect();
        why.push(format!(
            "Seen through {}; the rest of the spectrum is greyed.",
            names.join(", ")
        ));
    }
    Ok(Spectrum {
        address: t.address.clone(),
        distance_m: t.distance_m,
        samples,
        bands,
        lines,
        readings,
        coverage: o.coverage.clone(),
        why,
        stand_ins,
        references,
    })
}

//! SSE4 remote survey (track O13; nebulae-and-light design section 5B).
//!
//! Whether a planet round a distant star is found depends on where the
//! observer is (geometry is a fact here, not a chance), the instrument, the
//! distance, how long it watches and the star's own noise. Each method is the
//! real arithmetic: the signal, the noise, and a verdict of found, candidate
//! or missed, with what the method measures and its error bar. The biases are
//! the real ones: big planets close to small quiet stars first, an Earth twin
//! only with years of watching and a fine instrument, face-on systems with
//! no transits and little wobble.
//!
//! Deterministic: no draws at all. Transit counts come from the planet's
//! phase at the start of the watch.
#![forbid(unsafe_code)]

#[cfg(test)]
mod tests;

use serde::{Deserialize, Serialize};
use sse_core::math as m;
use sse_flow::Reference;

pub const G: f64 = sse_core::quantity::consts::G;
pub const C: f64 = sse_core::quantity::consts::C;
pub const HOUR: f64 = 3600.0;
pub const DAY: f64 = sse_core::sheet::DAY;
pub const YEAR: f64 = sse_core::quantity::consts::JULIAN_YEAR;
/// Photon energy at 550 nm, J (for photon counting in the visible).
pub const PHOTON_550: f64 = 6.626_070_15e-34 * C / 550e-9;

const SM03: &str = "Seager, S. and Mallén-Ornelas, G. 2003, A unique solution of planet and star parameters from an extrasolar planet transit light curve, ApJ 585, 1038: depth (Rp/R*)², duration, impact parameter. doi:10.1086/346105";
const WINN10: &str = "Winn, J. N. 2010, Transits and occultations, in Exoplanets (ed. S. Seager), University of Arizona Press: transit probability R*/a, total duration. https://arxiv.org/abs/1001.2010";
const LF10: &str = "Lovis, C. and Fischer, D. 2010, Radial velocity techniques for exoplanets, in Exoplanets (ed. S. Seager): K = (2πG/P)^⅓ m sin i / (M* + m)^⅔ / √(1 − e²). https://ui.adsabs.harvard.edu/abs/2010exop.book...27L";
const PERRYMAN: &str = "Perryman, M. 2018, The Exoplanet Handbook, 2nd edn, Cambridge University Press: astrometric signature α = (m/M*)(a/d), imaging contrast and inner working angle, timing. doi:10.1017/9781108304160";
const JENKINS10: &str = "Jenkins, J. M. et al. 2010, Overview of the Kepler science processing pipeline, ApJ 713, L87: the 7.1σ detection threshold. doi:10.1088/2041-8205/713/2/L87";
const WF92: &str = "Wolszczan, A. and Frail, D. A. 1992, A planetary system around the millisecond pulsar PSR1257+12, Nature 355, 145. doi:10.1038/355145a0";

fn r(text: &str) -> Reference {
    Reference::parse(text)
}

/// The star being watched.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Star {
    pub address: String,
    pub mass_kg: f64,
    pub radius_m: f64,
    /// Visible-band luminosity, W (from the star's light description).
    pub visible_w: f64,
    /// The star's own brightness flicker over an hour (spots, granulation), ppm.
    /// The Sun is about 10 to 20 (stellar activity, O10, will supply it).
    #[serde(default = "default_activity")]
    pub activity_ppm_hr: f64,
    /// Line-shift jitter from the star's surface, m/s (Sun about 1).
    #[serde(default = "default_jitter")]
    pub rv_jitter_m_s: f64,
    /// A pulsar's timing precision, s (none for ordinary stars).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timing_precision_s: Option<f64>,
}
fn default_activity() -> f64 {
    15.0
}
fn default_jitter() -> f64 {
    1.0
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Planet {
    pub address: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub mass_kg: f64,
    pub radius_m: f64,
    pub a_m: f64,
    #[serde(default)]
    pub e: f64,
    /// Orbit tilt and node in the system frame, radians.
    pub i_rad: f64,
    pub node_rad: f64,
    /// Mean anomaly at the start of the watch, radians (sets when transits fall).
    #[serde(default)]
    pub phase_rad: f64,
    #[serde(default = "default_albedo")]
    pub geometric_albedo: f64,
}
fn default_albedo() -> f64 {
    0.4
}

/// What the observer brings.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Instrument {
    pub name: String,
    pub aperture_m: f64,
    /// Share of arriving photons recorded.
    #[serde(default = "default_eff")]
    pub efficiency: f64,
    /// Systematic photometric floor per hour, ppm (0 = none: photon-limited).
    #[serde(default)]
    pub photometric_floor_ppm_hr: f64,
    /// Line-shift precision per measurement, m/s (none = no spectrograph).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rv_precision_m_s: Option<f64>,
    /// Position precision per measurement, micro-arcseconds (none = no astrometry).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub astrometric_precision_uas: Option<f64>,
    /// Faintest planet-to-star brightness ratio it can separate (none = no imaging).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contrast_floor: Option<f64>,
    /// Imaging wavelength, m (sets the inner working angle 3λ/D).
    #[serde(default = "default_lambda")]
    pub imaging_wavelength_m: f64,
    /// Measurements per day for line-shift, position and timing methods.
    #[serde(default = "default_cadence")]
    pub visits_per_day: f64,
    /// Whether it stares continuously (photometry duty cycle).
    #[serde(default = "default_duty")]
    pub duty_cycle: f64,
}
fn default_eff() -> f64 {
    0.5
}
fn default_lambda() -> f64 {
    1.6e-6
}
fn default_cadence() -> f64 {
    0.2
}
fn default_duty() -> f64 {
    0.9
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Observer {
    /// Unit vector from the system towards the observer, system frame.
    pub direction: [f64; 3],
    pub distance_m: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Method {
    Transit,
    Wobble,
    PositionWobble,
    DirectImaging,
    Timing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Verdict {
    Found,
    Candidate,
    Missed,
}

/// One measured value with its one-sigma error.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Measured {
    pub what: String,
    pub value: f64,
    pub error: f64,
    pub unit: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Finding {
    pub planet: String,
    pub method: Method,
    pub verdict: Verdict,
    /// Signal over noise.
    pub snr: f64,
    pub measured: Vec<Measured>,
    pub why: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Settings {
    /// Signal over noise to count as found (Kepler's 7.1σ) and as a candidate.
    pub found_snr: f64,
    pub candidate_snr: f64,
    /// Transits needed to fix a period.
    pub transits_needed: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            found_snr: 7.1,
            candidate_snr: 5.0,
            transits_needed: 3,
        }
    }
}

pub fn period(star: &Star, p: &Planet) -> f64 {
    m::TAU * m::sqrt(p.a_m * p.a_m * p.a_m / (G * (star.mass_kg + p.mass_kg)))
}

/// Cosine of the orbit's tilt to the line of sight: 0 is edge-on, 1 face-on.
pub fn cos_inclination(p: &Planet, o: &Observer) -> f64 {
    let h = [
        m::sin(p.i_rad) * m::sin(p.node_rad),
        -m::sin(p.i_rad) * m::cos(p.node_rad),
        m::cos(p.i_rad),
    ];
    let n = o.direction;
    let len = m::sqrt(n[0] * n[0] + n[1] * n[1] + n[2] * n[2]);
    (h[0] * n[0] + h[1] * n[1] + h[2] * n[2]) / len
}

fn verdict(snr: f64, enough_time: bool, s: &Settings) -> Verdict {
    if snr >= s.found_snr && enough_time {
        Verdict::Found
    } else if snr >= s.candidate_snr {
        Verdict::Candidate
    } else {
        Verdict::Missed
    }
}

/// Photometric noise over `hours` of staring, ppm: photon noise from the
/// star's visible light through the aperture, the instrument's floor and the
/// star's flicker, added in quadrature, all falling as 1/√hours.
pub fn photometric_noise_ppm(star: &Star, ins: &Instrument, o: &Observer, hours: f64) -> f64 {
    let flux = star.visible_w / (4.0 * m::PI * o.distance_m * o.distance_m);
    let area = m::PI * 0.25 * ins.aperture_m * ins.aperture_m;
    let photons_hr = flux * area * ins.efficiency / PHOTON_550 * HOUR;
    let shot = if photons_hr > 0.0 {
        1e6 / m::sqrt(photons_hr)
    } else {
        f64::INFINITY
    };
    let per_hr = m::sqrt(
        shot * shot
            + ins.photometric_floor_ppm_hr * ins.photometric_floor_ppm_hr
            + star.activity_ppm_hr * star.activity_ppm_hr,
    );
    per_hr / m::sqrt(hours.max(1e-9))
}

/// Transit: the orbit must cross the star's face from where you are.
pub fn transit(
    star: &Star,
    p: &Planet,
    o: &Observer,
    ins: &Instrument,
    watch_s: f64,
    s: &Settings,
) -> Finding {
    let ci = cos_inclination(p, o).abs();
    let b = p.a_m * ci / star.radius_m;
    let k = p.radius_m / star.radius_m;
    let per = period(star, p);
    let name = p.name.clone().unwrap_or_else(|| p.address.clone());
    if b >= 1.0 + k {
        return Finding {
            planet: p.address.clone(),
            method: Method::Transit,
            verdict: Verdict::Missed,
            snr: 0.0,
            measured: vec![],
            why: format!(
                "From here {name}'s orbit is tilted {:.3}° from edge-on; it would need to be within {:.3}° to cross the star's face, so it never transits.",
                m::asin(ci.min(1.0)) * 180.0 / m::PI,
                m::asin(((star.radius_m + p.radius_m) / p.a_m).min(1.0)) * 180.0 / m::PI
            ),
        };
    }
    let depth = k * k;
    let si = m::sqrt(1.0 - ci * ci);
    let chord = m::sqrt(((1.0 + k) * (1.0 + k) - b * b).max(0.0));
    let duration = per / m::PI * m::asin((star.radius_m / p.a_m * chord / si).min(1.0));
    // Transits in the watch: the first comes when the mean anomaly reaches the
    // crossing point (taken as phase zero for a circular orbit).
    let first = per * (1.0 - (p.phase_rad.rem_euclid(m::TAU)) / m::TAU);
    let n = if first > watch_s * ins.duty_cycle {
        0
    } else {
        1 + m::floor((watch_s * ins.duty_cycle - first) / per) as u32
    };
    let noise = photometric_noise_ppm(star, ins, o, n as f64 * duration / HOUR);
    let snr = if n == 0 { 0.0 } else { depth * 1e6 / noise };
    let v = verdict(snr, n >= s.transits_needed, s);
    let depth_err = depth / snr.max(1e-9);
    let mut measured = vec![];
    if v != Verdict::Missed {
        let rp = star.radius_m * m::sqrt(depth);
        measured.push(Measured {
            what: "radius".into(),
            value: rp,
            error: 0.5 * rp * depth_err / depth,
            unit: "m".into(),
        });
        if n >= 2 {
            measured.push(Measured {
                what: "period".into(),
                value: per,
                error: duration / (n as f64 * snr.max(1.0)),
                unit: "s".into(),
            });
        }
    }
    Finding {
        planet: p.address.clone(),
        method: Method::Transit,
        verdict: v,
        snr,
        measured,
        why: format!(
            "{name} crosses the star every {:.1} days, dimming it by {:.0} ppm for {:.1} hours; {n} crossing{} in the watch against {:.0} ppm of noise over that time gives {:.1}σ{}.",
            per / DAY,
            depth * 1e6,
            duration / HOUR,
            if n == 1 { "" } else { "s" },
            noise,
            snr,
            if n < s.transits_needed && snr >= s.found_snr {
                format!(", but {} crossings are needed to fix the period", s.transits_needed)
            } else {
                String::new()
            }
        ),
    }
}

/// Wobble (radial velocity): the star's line shift swings by K.
pub fn wobble(
    star: &Star,
    p: &Planet,
    o: &Observer,
    ins: &Instrument,
    watch_s: f64,
    s: &Settings,
) -> Option<Finding> {
    let prec = ins.rv_precision_m_s?;
    let per = period(star, p);
    let si = m::sqrt(
        (1.0 - {
            let c = cos_inclination(p, o);
            c * c
        })
        .max(0.0),
    );
    let k_true = m::cbrt(m::TAU * G / per) * p.mass_kg * si
        / m::pow(star.mass_kg + p.mass_kg, 2.0 / 3.0)
        / m::sqrt(1.0 - p.e * p.e);
    let visits = (watch_s / DAY * ins.visits_per_day).max(0.0);
    let sigma = m::sqrt(prec * prec + star.rv_jitter_m_s * star.rv_jitter_m_s);
    let snr = k_true / sigma * m::sqrt(visits / 2.0);
    let v = verdict(snr, watch_s >= per, s);
    let name = p.name.clone().unwrap_or_else(|| p.address.clone());
    let mut measured = vec![];
    if v != Verdict::Missed {
        let k_unit = m::cbrt(m::TAU * G / per)
            / m::pow(star.mass_kg + p.mass_kg, 2.0 / 3.0)
            / m::sqrt(1.0 - p.e * p.e);
        let msini = k_true / k_unit;
        measured.push(Measured {
            what: "minimum mass (m sin i)".into(),
            value: msini,
            error: msini / snr,
            unit: "kg".into(),
        });
        measured.push(Measured {
            what: "period".into(),
            value: per,
            error: per / (snr * (watch_s / per).max(1.0)),
            unit: "s".into(),
        });
    }
    Some(Finding {
        planet: p.address.clone(),
        method: Method::Wobble,
        verdict: v,
        snr,
        measured,
        why: format!(
            "{name} swings the star towards and away from here by {:.3} m/s every {:.1} days; {:.0} measurements at {:.2} m/s (with the star's own {:.1} m/s jitter) give {:.1}σ{}. Only the minimum mass is measured: the tilt is unknown.",
            k_true,
            per / DAY,
            visits,
            prec,
            star.rv_jitter_m_s,
            snr,
            if watch_s < per { ", but less than one full orbit was watched" } else { "" }
        ),
    })
}

/// Position wobble (astrometry): the star's circle on the sky.
pub fn position_wobble(
    star: &Star,
    p: &Planet,
    o: &Observer,
    ins: &Instrument,
    watch_s: f64,
    s: &Settings,
) -> Option<Finding> {
    let prec_uas = ins.astrometric_precision_uas?;
    let per = period(star, p);
    let alpha_rad = p.mass_kg / star.mass_kg * p.a_m / o.distance_m;
    let alpha_uas = alpha_rad * 180.0 / m::PI * 3600.0 * 1e6;
    let visits = (watch_s / DAY * ins.visits_per_day).max(0.0);
    let snr = alpha_uas / prec_uas * m::sqrt(visits / 2.0);
    let v = verdict(snr, watch_s >= per, s);
    let name = p.name.clone().unwrap_or_else(|| p.address.clone());
    let ci = cos_inclination(p, o);
    let mut measured = vec![];
    if v == Verdict::Found {
        measured.push(Measured {
            what: "mass".into(),
            value: p.mass_kg,
            error: p.mass_kg / snr,
            unit: "kg".into(),
        });
        measured.push(Measured {
            what: "tilt from edge-on".into(),
            value: m::asin(ci.abs().min(1.0)),
            error: 1.0 / snr,
            unit: "rad".into(),
        });
    }
    Some(Finding {
        planet: p.address.clone(),
        method: Method::PositionWobble,
        verdict: v,
        snr,
        measured,
        why: format!(
            "{name} drags the star round a circle {:.1} µas across on the sky every {:.1} years; {:.0} positions at {:.0} µas give {:.1}σ{}. This gives the true mass and the tilt.",
            2.0 * alpha_uas,
            per / YEAR,
            visits,
            prec_uas,
            snr,
            if watch_s < per { ", but less than one full orbit was watched" } else { "" }
        ),
    })
}

/// Direct imaging: far enough from the star for the telescope's sharpness,
/// bright enough against its glare (reflected light at quadrature).
pub fn direct_imaging(_star: &Star, p: &Planet, o: &Observer, ins: &Instrument) -> Option<Finding> {
    let floor = ins.contrast_floor?;
    let sep = p.a_m / o.distance_m;
    let iwa = 3.0 * ins.imaging_wavelength_m / ins.aperture_m;
    let contrast = p.geometric_albedo * (p.radius_m / p.a_m) * (p.radius_m / p.a_m) / m::PI;
    let snr = if sep > iwa {
        10.0 * contrast / floor
    } else {
        0.0
    };
    let v = if sep > iwa && contrast >= floor {
        Verdict::Found
    } else {
        Verdict::Missed
    };
    let name = p.name.clone().unwrap_or_else(|| p.address.clone());
    let mas = |x: f64| x * 180.0 / m::PI * 3600.0 * 1e3;
    let mut measured = vec![];
    if v == Verdict::Found {
        measured.push(Measured {
            what: "separation".into(),
            value: p.a_m,
            error: 0.05 * p.a_m,
            unit: "m".into(),
        });
    }
    Some(Finding {
        planet: p.address.clone(),
        method: Method::DirectImaging,
        verdict: v,
        snr,
        measured,
        why: if sep <= iwa {
            format!(
                "{name} is at most {:.1} mas from its star, inside this telescope's {:.1} mas reach (3λ/D): lost in the glare.",
                mas(sep),
                mas(iwa)
            )
        } else {
            format!(
                "{name} is {:.1} mas from its star and {:.1e} times as bright in reflected light; the instrument separates down to {:.1e}{}.",
                mas(sep),
                contrast,
                floor,
                if contrast >= floor { ", so it shows as its own point of light" } else { ", so it is too faint" }
            )
        },
    })
}

/// Timing: a pulsar's ticks arrive early and late as it wobbles.
pub fn timing(
    star: &Star,
    p: &Planet,
    o: &Observer,
    ins: &Instrument,
    watch_s: f64,
    s: &Settings,
) -> Option<Finding> {
    let prec = star.timing_precision_s?;
    let per = period(star, p);
    let si = m::sqrt(
        (1.0 - {
            let c = cos_inclination(p, o);
            c * c
        })
        .max(0.0),
    );
    let tau = p.mass_kg / star.mass_kg * p.a_m * si / C;
    let visits = (watch_s / DAY * ins.visits_per_day).max(0.0);
    let snr = tau / prec * m::sqrt(visits / 2.0);
    let v = verdict(snr, watch_s >= per, s);
    let name = p.name.clone().unwrap_or_else(|| p.address.clone());
    Some(Finding {
        planet: p.address.clone(),
        method: Method::Timing,
        verdict: v,
        snr,
        measured: if v == Verdict::Missed {
            vec![]
        } else {
            vec![Measured {
                what: "minimum mass (m sin i)".into(),
                value: p.mass_kg * si,
                error: p.mass_kg * si / snr,
                unit: "kg".into(),
            }]
        },
        why: format!(
            "{name} moves the pulsar by {:.2e} light-seconds, so its pulses arrive {:.2e} s early and late every {:.1} days; at {:.1e} s per measurement that is {:.1}σ.",
            tau,
            tau,
            per / DAY,
            prec,
            snr
        ),
    })
}

/// A survey of one system: every method the instrument allows, on every planet.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Survey {
    pub star: Star,
    pub planets: Vec<Planet>,
    pub observer: Observer,
    pub instrument: Instrument,
    pub watch_s: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<Settings>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SurveyResult {
    pub findings: Vec<Finding>,
    /// Planets found by at least one method.
    pub found: Vec<String>,
    pub references: Vec<Reference>,
}

pub fn run(sv: &Survey) -> SurveyResult {
    let s = sv.settings.clone().unwrap_or_default();
    let mut findings = vec![];
    for p in &sv.planets {
        findings.push(transit(
            &sv.star,
            p,
            &sv.observer,
            &sv.instrument,
            sv.watch_s,
            &s,
        ));
        findings.extend(wobble(
            &sv.star,
            p,
            &sv.observer,
            &sv.instrument,
            sv.watch_s,
            &s,
        ));
        findings.extend(position_wobble(
            &sv.star,
            p,
            &sv.observer,
            &sv.instrument,
            sv.watch_s,
            &s,
        ));
        findings.extend(direct_imaging(&sv.star, p, &sv.observer, &sv.instrument));
        findings.extend(timing(
            &sv.star,
            p,
            &sv.observer,
            &sv.instrument,
            sv.watch_s,
            &s,
        ));
    }
    let mut found: Vec<String> = findings
        .iter()
        .filter(|f| f.verdict == Verdict::Found)
        .map(|f| f.planet.clone())
        .collect();
    found.sort();
    found.dedup();
    SurveyResult {
        findings,
        found,
        references: [SM03, WINN10, LF10, PERRYMAN, JENKINS10, WF92]
            .iter()
            .map(|t| r(t))
            .collect(),
    }
}

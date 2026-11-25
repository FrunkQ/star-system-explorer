//! Light through gas (ND2, new-physics demo section 2.6): one mechanism for
//! a planet's air, a giant's cloud tops, a brown dwarf, a star's
//! atmosphere and a nebula between a source and its observer. Nothing here
//! names a kind of body: what differs is only where the light comes from
//! and how dense, hot and deep the gas is, and those are inputs.
//!
//! A gas's features are rows on the master data sheet
//! (`substances[].optical.absorption`, shape agreed with the data sheet
//! thread on 23 Nov), each one of three kinds:
//! - `table`: a measured band, cross-section per molecule by wavelength;
//! - `line`: a line widened by heat and by crowding, from its oscillator
//!   strength, the energy of its lower level and its damping;
//! - `pair`: absorption by two colliding molecules, growing with both
//!   densities.
//!
//! The physics is the formula blocks `sse.gas.*` (`packs/base-physics/
//! formulas/gas.json`); this module only reads the rows, walks the
//! wavelengths and adds the optical depths up. An empty list of features
//! means "none known here", which is said; a missing list means "not yet
//! looked up", also said; and no table is ever stretched past the
//! wavelengths or temperatures it was measured at.

use crate::colour::run;
use crate::Reference;
use serde::{Deserialize, Serialize};
use sse_core::math as m;

/// A list of numbers with its unit beside it, as the sheet writes arrays.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Values {
    pub v: Vec<f64>,
    pub unit: String,
}

/// Rows of numbers (one per temperature) with their unit.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rows {
    pub v: Vec<Vec<f64>>,
    pub unit: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scalar {
    pub v: f64,
    pub unit: String,
}

/// How a line is widened by the particles around it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Broadening {
    /// A sheet substance id, or `any` for every particle in the gas.
    pub perturber: String,
    /// Damping added per perturber per cubic metre at `at_k`, m³ s⁻¹.
    pub gamma_per_n: Scalar,
    #[serde(default)]
    pub t_exponent: f64,
    pub at_k: f64,
}

/// One absorption feature of a gas, as its sheet row gives it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum Feature {
    Table {
        id: String,
        lambda_nm: Vec<f64>,
        sigma: Values,
        at_k: f64,
        #[serde(default)]
        t_exponent: f64,
        #[serde(default)]
        valid_k: Option<[f64; 2]>,
        src: String,
        #[serde(default)]
        note: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        check: Option<String>,
    },
    Line {
        id: String,
        lambda_nm: f64,
        f: f64,
        #[serde(default)]
        lower_ev: f64,
        gamma_rad_s: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        broadening: Option<Broadening>,
        #[serde(default)]
        valid_k: Option<[f64; 2]>,
        src: String,
        #[serde(default)]
        note: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        check: Option<String>,
    },
    Pair {
        id: String,
        partner: String,
        lambda_nm: Vec<f64>,
        k: Rows,
        at_k: Vec<f64>,
        #[serde(default)]
        valid_k: Option<[f64; 2]>,
        src: String,
        #[serde(default)]
        note: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        check: Option<String>,
    },
}

impl Feature {
    pub fn id(&self) -> &str {
        match self {
            Feature::Table { id, .. } | Feature::Line { id, .. } | Feature::Pair { id, .. } => id,
        }
    }
    pub fn kind(&self) -> &'static str {
        match self {
            Feature::Table { .. } => "table",
            Feature::Line { .. } => "line",
            Feature::Pair { .. } => "pair",
        }
    }
    pub fn src(&self) -> &str {
        match self {
            Feature::Table { src, .. } | Feature::Line { src, .. } | Feature::Pair { src, .. } => {
                src
            }
        }
    }
    fn valid_k(&self) -> Option<[f64; 2]> {
        match self {
            Feature::Table { valid_k, .. }
            | Feature::Line { valid_k, .. }
            | Feature::Pair { valid_k, .. } => *valid_k,
        }
    }

    /// Shape checks: units as agreed, wavelengths rising, rows the right length.
    pub fn check(&self) -> Result<(), String> {
        let rising = |l: &[f64]| l.len() >= 2 && l.windows(2).all(|w| w[1] > w[0]) && l[0] > 0.0;
        match self {
            Feature::Table {
                id,
                lambda_nm,
                sigma,
                at_k,
                ..
            } => {
                if sigma.unit != "m2" {
                    return Err(format!("{id}: sigma is in {}, expected m2", sigma.unit));
                }
                if !rising(lambda_nm) || sigma.v.len() != lambda_nm.len() {
                    return Err(format!(
                        "{id}: lambda_nm must rise (two or more) and sigma match it"
                    ));
                }
                if sigma.v.iter().any(|s| !(s.is_finite() && *s >= 0.0)) || *at_k <= 0.0 {
                    return Err(format!(
                        "{id}: cross-sections must be zero or more and at_k positive"
                    ));
                }
            }
            Feature::Line {
                id,
                lambda_nm,
                f,
                gamma_rad_s,
                broadening,
                ..
            } => {
                if !(*lambda_nm > 0.0 && *f > 0.0 && *gamma_rad_s >= 0.0) {
                    return Err(format!("{id}: a line needs a wavelength, a positive f and a natural damping of zero or more"));
                }
                if let Some(b) = broadening {
                    if b.gamma_per_n.unit != "m3 s-1" || b.at_k <= 0.0 {
                        return Err(format!(
                            "{id}: gamma_per_n must be in m3 s-1 with a positive at_k"
                        ));
                    }
                }
            }
            Feature::Pair {
                id,
                lambda_nm,
                k,
                at_k,
                ..
            } => {
                if k.unit != "m5" {
                    return Err(format!("{id}: k is in {}, expected m5", k.unit));
                }
                if !rising(lambda_nm)
                    || at_k.is_empty()
                    || at_k.windows(2).any(|w| w[1] <= w[0])
                    || k.v.len() != at_k.len()
                    || k.v.iter().any(|r| r.len() != lambda_nm.len())
                {
                    return Err(format!("{id}: lambda_nm and at_k must rise, with one row of k per temperature as long as lambda_nm"));
                }
            }
        }
        Ok(())
    }
}

/// A gas's absorption data, read from its sheet row.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GasOptics {
    pub gas: String,
    pub particle_mass_kg: f64,
    /// `None`: the sheet row has no absorption list yet (not looked up).
    /// `Some(empty)`: none known.
    pub features: Option<Vec<Feature>>,
    /// Polarisability volume, m³: how this gas widens others' lines when
    /// it crowds them (van der Waals, ND2 part 2a).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub polarisability_m3: Option<f64>,
    /// Ionisation energy of the absorbing atom, J: what its own lines'
    /// van der Waals widening reads.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ionisation_j: Option<f64>,
}

impl GasOptics {
    pub fn from_sheet(sheet: &sse_reference::Reference, gas: &str) -> Result<Self, String> {
        let row = sheet
            .row("substances", gas)
            .ok_or_else(|| format!("gas {gas}: the data sheet has no substance row {gas}"))?;
        let n_a = sheet
            .constant("N_A")
            .ok_or("the data sheet has no Avogadro constant")?;
        let particle_mass_kg = sheet
            .molar_mass_kg_mol(gas)
            .ok_or_else(|| format!("gas {gas}: no molar mass on the sheet"))?
            / n_a;
        let features = match row.get("optical").and_then(|o| o.get("absorption")) {
            None => None,
            Some(v) => {
                let f: Vec<Feature> = serde_json::from_value(v.clone())
                    .map_err(|e| format!("gas {gas}: absorption rows: {e}"))?;
                for x in &f {
                    x.check().map_err(|e| format!("gas {gas}: {e}"))?;
                }
                Some(f)
            }
        };
        let fig = |k: &str| {
            row.get("optical")
                .and_then(|o| o.get(k))
                .and_then(|v| v.get("v"))
                .and_then(|v| v.as_f64())
        };
        let polarisability_m3 = fig("polarisability");
        let ionisation_j = match fig("ionisation_energy") {
            Some(ev) => Some(
                ev * sheet
                    .constant("e")
                    .ok_or("the data sheet has no elementary charge")?,
            ),
            None => None,
        };
        Ok(GasOptics {
            gas: gas.into(),
            particle_mass_kg,
            features,
            polarisability_m3,
            ionisation_j,
        })
    }
}

/// How much of one gas there is in a layer.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Amount {
    pub gas: String,
    pub number_density_m3: f64,
}

/// A slab of gas of one temperature and make-up, `path_m` deep along the
/// straight path (vertical for an air, the line of sight for a nebula).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Layer {
    pub t_k: f64,
    pub path_m: f64,
    pub gases: Vec<Amount>,
}

fn one() -> f64 {
    1.0
}

/// Gas between a light and whoever sees it.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Passage {
    #[serde(default)]
    pub address: String,
    pub layers: Vec<Layer>,
    /// How many straight paths the light takes: 1 for light from below
    /// seen straight on (a star's own glow), 2 for light down to a cloud
    /// deck and back up, 1/μ₀ + 1/μ when slanted.
    #[serde(default = "one")]
    pub path_factor: f64,
}

/// What one feature did.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FeatureRead {
    pub gas: String,
    pub feature: String,
    pub kind: String,
    /// Its largest optical depth on the path (path factor included).
    pub peak_optical_depth: f64,
    pub at_m: f64,
    pub src: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Passed {
    pub address: String,
    pub wavelengths_m: Vec<f64>,
    pub light_in: Vec<f64>,
    pub light_out: Vec<f64>,
    /// Optical depth on the whole path, path factor included.
    pub optical_depth: Vec<f64>,
    pub features: Vec<FeatureRead>,
    /// Features used where they were not measured: wavelengths past a
    /// table (no absorption counted there) or temperatures past its rows
    /// (the nearest row used).
    pub outside_measured: Vec<String>,
    pub none_known: Vec<String>,
    pub not_looked_up: Vec<String>,
    pub why: Vec<String>,
    pub stand_ins: Vec<String>,
    pub references: Vec<Reference>,
}

/// Linear interpolation in a rising table; `None` outside it.
fn interp(x: &[f64], y: &[f64], at: f64) -> Option<f64> {
    if at < x[0] || at > x[x.len() - 1] {
        return None;
    }
    let j = x
        .iter()
        .position(|&a| a >= at)
        .unwrap_or(x.len() - 1)
        .max(1);
    let (x0, x1) = (x[j - 1], x[j]);
    let f = if x1 > x0 { (at - x0) / (x1 - x0) } else { 0.0 };
    Some(y[j - 1] + f * (y[j] - y[j - 1]))
}

fn note(list: &mut Vec<String>, s: String) {
    if !list.contains(&s) {
        list.push(s);
    }
}

/// Pass light through gas. `light_in` is the light at each of
/// `wavelengths_m` (any units per wavelength: the result is in the same).
pub fn pass(
    passage: &Passage,
    gases: &[GasOptics],
    wavelengths_m: &[f64],
    light_in: &[f64],
    sheet: &sse_reference::Reference,
) -> Result<Passed, String> {
    if light_in.len() != wavelengths_m.len() {
        return Err("light_in must give one value per wavelength".into());
    }
    let c = sheet
        .constant("c")
        .ok_or("the data sheet has no speed of light")?;
    let ev = sheet
        .constant("e")
        .ok_or("the data sheet has no elementary charge")?;
    let mut tau = vec![0.0; wavelengths_m.len()];
    let mut reads: Vec<FeatureRead> = vec![];
    let mut outside = vec![];
    let mut none_known = vec![];
    let mut not_looked_up = vec![];
    let mut why = vec![];
    let mut extra_stand_ins = vec![];
    let mut used_vdw = false;
    let x = passage.path_factor;
    if !(x.is_finite() && x >= 0.0) {
        return Err(format!("path factor must be zero or more, got {x}"));
    }
    for (li, layer) in passage.layers.iter().enumerate() {
        if !(layer.t_k > 0.0 && layer.path_m >= 0.0) {
            return Err(format!(
                "layer {li}: needs a positive temperature and a depth of zero or more"
            ));
        }
        let total_n: f64 = layer.gases.iter().map(|a| a.number_density_m3).sum();
        let density_of = |id: &str| -> f64 {
            if id == "any" {
                return total_n;
            }
            layer
                .gases
                .iter()
                .filter(|a| a.gas == id)
                .map(|a| a.number_density_m3)
                .sum()
        };
        for a in &layer.gases {
            if !(a.number_density_m3.is_finite() && a.number_density_m3 >= 0.0) {
                return Err(format!(
                    "layer {li}: {} needs a number density of zero or more",
                    a.gas
                ));
            }
            let g = gases
                .iter()
                .find(|g| g.gas == a.gas)
                .ok_or_else(|| format!("no absorption data read for {}", a.gas))?;
            let Some(features) = &g.features else {
                note(&mut not_looked_up, format!("{}: the data sheet has no absorption rows for this gas yet, so it is read as clear", a.gas));
                continue;
            };
            if features.is_empty() {
                note(
                    &mut none_known,
                    format!("{}: no absorption known in this range", a.gas),
                );
                continue;
            }
            let column = a.number_density_m3 * layer.path_m;
            let t = layer.t_k;
            for f in features {
                if let Some([lo, hi]) = f.valid_k() {
                    if t < lo || t > hi {
                        note(
                            &mut outside,
                            format!(
                                "{} {}: measured from {lo} to {hi} K, used at {t} K",
                                a.gas,
                                f.id()
                            ),
                        );
                    }
                }
                let mut peak = (0.0_f64, 0.0_f64);
                match f {
                    Feature::Table {
                        id,
                        lambda_nm,
                        sigma,
                        at_k,
                        t_exponent,
                        ..
                    } => {
                        for (k, l) in wavelengths_m.iter().enumerate() {
                            let Some(s0) = interp(lambda_nm, &sigma.v, l * 1e9) else {
                                note(&mut outside, format!("{} {id}: measured from {} to {} nm; no absorption counted outside that", a.gas, lambda_nm[0], lambda_nm[lambda_nm.len() - 1]));
                                continue;
                            };
                            let s =
                                run("sse.gas.band_cross_section", &[s0, t, *at_k, *t_exponent])?[0];
                            let d = run("sse.gas.optical_depth", &[s, column])?[0];
                            tau[k] += d;
                            if d > peak.0 {
                                peak = (d, *l);
                            }
                        }
                    }
                    Feature::Line {
                        lambda_nm,
                        f: osc,
                        lower_ev,
                        gamma_rad_s,
                        broadening,
                        ..
                    } => {
                        let l0 = lambda_nm * 1e-9;
                        let s = run("sse.gas.line_strength", &[*osc, lower_ev * ev, l0, t])?[0];
                        let (cw, np, xe, t0) = match broadening {
                            Some(b) => (
                                b.gamma_per_n.v,
                                density_of(&b.perturber),
                                b.t_exponent,
                                b.at_k,
                            ),
                            // No measured widening: van der Waals from the
                            // atom's ionisation energy and each neighbour's
                            // polarisability (Unsöld), summed over the layer.
                            None => {
                                let mut sum = 0.0;
                                let mut by = vec![];
                                if let Some(ion) = g.ionisation_j {
                                    for p in &layer.gases {
                                        let Some(pg) = gases.iter().find(|x| x.gas == p.gas) else {
                                            continue;
                                        };
                                        let Some(ap) = pg.polarisability_m3 else {
                                            continue;
                                        };
                                        if p.number_density_m3 <= 0.0 {
                                            continue;
                                        }
                                        let w = run(
                                            "sse.gas.van_der_waals_widening",
                                            &[
                                                ion,
                                                lower_ev * ev,
                                                l0,
                                                t,
                                                g.particle_mass_kg,
                                                pg.particle_mass_kg,
                                                ap,
                                            ],
                                        )?;
                                        sum += w[0] * p.number_density_m3;
                                        by.push(p.gas.clone());
                                    }
                                }
                                if by.is_empty() {
                                    note(&mut extra_stand_ins, format!("{} {}: no widening row and nothing to work one out from (an ionisation energy for it, a polarisability for its neighbours), so only heat and its own lifetime widen it", a.gas, f.id()));
                                } else {
                                    note(&mut why, format!("{} {}: widened by its neighbours ({}) by van der Waals forces, worked out from the sheet (Unsöld's estimate, often about half the measured width)", a.gas, f.id(), by.join(", ")));
                                    used_vdw = true;
                                }
                                (sum, 1.0, 0.0, t)
                            }
                        };
                        let w = run(
                            "sse.gas.line_widths",
                            &[l0, t, g.particle_mass_kg, *gamma_rad_s, cw, np, xe, t0],
                        )?;
                        let v = run("sse.gas.voigt_width", &[w[0], w[1]])?;
                        let nu0 = c / l0;
                        for (k, l) in wavelengths_m.iter().enumerate() {
                            let phi = run("sse.gas.line_shape", &[c / l - nu0, v[0], v[1]])?[0];
                            let sg = run("sse.gas.line_cross_section", &[s, phi])?[0];
                            let d = run("sse.gas.optical_depth", &[sg, column])?[0];
                            tau[k] += d;
                            if d > peak.0 {
                                peak = (d, *l);
                            }
                        }
                    }
                    Feature::Pair {
                        id,
                        partner,
                        lambda_nm,
                        k: rows,
                        at_k,
                        ..
                    } => {
                        let n2 = density_of(partner);
                        if n2 <= 0.0 {
                            continue;
                        }
                        // The row at this temperature, between measured rows;
                        // the nearest row past the ends, and said.
                        let row: Vec<f64> = if t <= at_k[0] || at_k.len() == 1 {
                            if t < at_k[0] {
                                note(&mut outside, format!("{} {id}: measured from {} K, used at {t} K with the coldest row", a.gas, at_k[0]));
                            }
                            rows.v[0].clone()
                        } else if t >= at_k[at_k.len() - 1] {
                            if t > at_k[at_k.len() - 1] {
                                note(&mut outside, format!("{} {id}: measured to {} K, used at {t} K with the hottest row", a.gas, at_k[at_k.len() - 1]));
                            }
                            rows.v[at_k.len() - 1].clone()
                        } else {
                            (0..lambda_nm.len())
                                .map(|i| {
                                    let col: Vec<f64> = rows.v.iter().map(|r| r[i]).collect();
                                    interp(at_k, &col, t).unwrap_or(0.0)
                                })
                                .collect()
                        };
                        for (k, l) in wavelengths_m.iter().enumerate() {
                            let Some(kp) = interp(lambda_nm, &row, l * 1e9) else {
                                note(&mut outside, format!("{} {id}: measured from {} to {} nm; no absorption counted outside that", a.gas, lambda_nm[0], lambda_nm[lambda_nm.len() - 1]));
                                continue;
                            };
                            let d = run("sse.gas.pair_optical_depth", &[kp, n2, column])?[0];
                            tau[k] += d;
                            if d > peak.0 {
                                peak = (d, *l);
                            }
                        }
                    }
                }
                match reads
                    .iter_mut()
                    .find(|r| r.gas == a.gas && r.feature == f.id())
                {
                    Some(r) => {
                        if peak.0 * x > r.peak_optical_depth {
                            r.peak_optical_depth = peak.0 * x;
                            r.at_m = peak.1;
                        }
                    }
                    None => reads.push(FeatureRead {
                        gas: a.gas.clone(),
                        feature: f.id().into(),
                        kind: f.kind().into(),
                        peak_optical_depth: peak.0 * x,
                        at_m: peak.1,
                        src: f.src().into(),
                    }),
                }
            }
        }
    }
    let mut light_out = Vec::with_capacity(tau.len());
    for (i, t) in light_in.iter().zip(&tau) {
        light_out.push(run("sse.gas.transmission", &[*i, *t, x])?[0]);
    }
    let optical_depth: Vec<f64> = tau.iter().map(|t| t * x).collect();
    reads.sort_by(|a, b| b.peak_optical_depth.total_cmp(&a.peak_optical_depth));
    if let Some(r) = reads.first().filter(|r| r.peak_optical_depth > 0.0) {
        why.push(format!(
            "Strongest absorber: {} ({}, {}), optical depth {:.3} at {:.0} nm, which leaves {:.1} % of the light there",
            r.gas,
            r.feature,
            r.kind,
            r.peak_optical_depth,
            r.at_m * 1e9,
            100.0 * m::exp(-r.peak_optical_depth)
        ));
    }
    why.push(format!(
        "{} layer(s), the light crossing them {} time(s) over",
        passage.layers.len(),
        x
    ));
    let mut ids = vec![
        "sse.gas.line_strength",
        "sse.gas.line_widths",
        "sse.gas.voigt_width",
        "sse.gas.band_cross_section",
        "sse.gas.pair_optical_depth",
        "sse.gas.transmission",
    ];
    if used_vdw {
        ids.push("sse.gas.van_der_waals_widening");
    }
    let mut references: Vec<Reference> = sse_atoms::decls_of(&ids)
        .into_iter()
        .flat_map(|d| d.references)
        .collect();
    references.dedup_by(|a, b| a.text == b.text);
    Ok(Passed {
        address: passage.address.clone(),
        wavelengths_m: wavelengths_m.to_vec(),
        light_in: light_in.to_vec(),
        light_out,
        optical_depth,
        features: reads,
        outside_measured: outside,
        none_known,
        not_looked_up,
        why,
        stand_ins: vec![
            "each layer is one temperature and make-up (a stack of layers stands in for a smooth air)".into(),
            "absorption only: light scattered out of or into the beam is not counted here (Rayleigh scattering is in the air optics)".into(),
            "a line's lower level is filled by the Boltzmann factor alone (no partition function), and its wings follow the pseudo-Voigt shape to any distance (real far wings fall off faster)".into(),
        ]
        .into_iter()
        .chain(extra_stand_ins)
        .collect(),
        references,
    })
}

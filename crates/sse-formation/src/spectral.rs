//! A star's formal class (Morgan-Keenan type and luminosity class; brown
//! dwarf L, T, Y; white dwarf D types; neutron stars and black holes), read
//! off the life line's results: surface temperature, mass and radius (so
//! surface gravity) and the phase. A class is a label read off results and
//! never feeds physics (processes, not classes): nothing in the kernel reads
//! this back.
//!
//! - Spectral letter and subclass: the surface temperature against a
//!   temperature scale per subclass. Dwarfs and subgiants read the dwarf
//!   scale (Pecaut and Mamajek 2013, ApJS 208, 9, and its online update;
//!   brown dwarfs L, T, Y from the same table and Kirkpatrick et al. 2019,
//!   ApJS 240, 19). G, K and M giants and supergiants are cooler at the
//!   same type, and read a giant scale (van Belle et al. 1999, AJ 117, 521).
//!   Values quoted from memory, not yet checked against the papers (paper
//!   search out until 1 Dec 2026): unverified, docs/citations.md.
//! - Luminosity class: surface gravity, log g, against the loci of the
//!   luminosity classes at the star's temperature (Straizys and Kuriliene
//!   1981, Ap&SS 80, 353; Gray and Corbally 2009 appendix B). The class is
//!   the locus nearest in log g. Unverified, as above.
//! - White dwarfs: "D", a letter for the atmosphere and the temperature
//!   index 50400 / T (Sion et al. 1983, ApJ 269, 253). Cooler than about
//!   5000 K a white dwarf shows no lines whatever its atmosphere (DC). Hotter,
//!   the letter (DA hydrogen, DB helium, DO ionised helium) needs the
//!   hydrogen layer left on the star, which the life line does not track: the
//!   class says X (Sion's letter for "cannot be said") and says why.
//! - Neutron stars: "NS". Pulsar, magnetar and the like need spin and field,
//!   which come with the exotic objects packages; until then nothing more.
//! - Black holes: by mass, stellar (under 100 solar masses), intermediate
//!   (to 1e5) or supermassive.

use crate::lifeline::Phase;
use serde::Serialize;
use sse_core::math as m;

/// Families, the atlas "star" scheme's base class ids (packs/base-atlas).
pub const FAMILIES: [&str; 10] = [
    "pre-main-sequence",
    "main-sequence",
    "subgiant",
    "giant",
    "bright-giant",
    "supergiant",
    "brown-dwarf",
    "white-dwarf",
    "neutron-star",
    "black-hole",
];

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct StarClass {
    /// The class as written: "G2 V", "K1 III", "B8 Ia", "T5", "DX 4.5", "NS".
    pub code: String,
    /// O, B, A, F, G, K, M, L, T, Y; "D" for a white dwarf; empty otherwise.
    pub letter: String,
    /// 0 to 9 within the letter (absent for remnants).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subclass: Option<u8>,
    /// Ia, Iab, Ib, II, III, IV, V; empty where there is none.
    pub luminosity: String,
    /// One of [`FAMILIES`].
    pub family: &'static str,
    /// The kind within the family, the atlas child class's key: the letter
    /// for a star or brown dwarf, DX or DC for a white dwarf, BH, IMBH or
    /// SMBH for a black hole, NS for a neutron star.
    pub kind: String,
    /// Why, in words and numbers.
    pub why: String,
    /// What the class cannot say and why.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

/// Dwarf temperature scale, K, for subclass 0 of each letter and every
/// subclass after it: (letter, [T at 0, 1, ..., 9]). Pecaut and Mamajek
/// 2013 (and update), Kirkpatrick et al. 2019 for late T and Y. Unverified.
const DWARF: [(&str, [f64; 10]); 10] = [
    (
        "O",
        [
            60000.0, 55000.0, 50000.0, 44900.0, 42900.0, 41400.0, 38800.0, 36500.0, 34500.0,
            32500.0,
        ],
    ),
    (
        "B",
        [
            31400.0, 26000.0, 20600.0, 17000.0, 16400.0, 15700.0, 14500.0, 14000.0, 12500.0,
            10700.0,
        ],
    ),
    (
        "A",
        [
            9700.0, 9300.0, 8800.0, 8600.0, 8250.0, 8100.0, 7910.0, 7760.0, 7590.0, 7400.0,
        ],
    ),
    (
        "F",
        [
            7220.0, 7020.0, 6820.0, 6750.0, 6670.0, 6550.0, 6350.0, 6280.0, 6180.0, 6050.0,
        ],
    ),
    // G2 V is the Sun: the sheet's solar temperature (5772 K) stands for it.
    (
        "G",
        [
            5930.0,
            5860.0,
            sse_core::sheet::T_SUN,
            5720.0,
            5680.0,
            5660.0,
            5600.0,
            5550.0,
            5480.0,
            5380.0,
        ],
    ),
    (
        "K",
        [
            5270.0, 5170.0, 5100.0, 4830.0, 4600.0, 4440.0, 4300.0, 4100.0, 3990.0, 3930.0,
        ],
    ),
    (
        "M",
        [
            3850.0, 3660.0, 3560.0, 3430.0, 3210.0, 3060.0, 2810.0, 2680.0, 2570.0, 2380.0,
        ],
    ),
    (
        "L",
        [
            2270.0, 2160.0, 2060.0, 1920.0, 1870.0, 1710.0, 1550.0, 1430.0, 1380.0, 1350.0,
        ],
    ),
    (
        "T",
        [
            1260.0, 1230.0, 1200.0, 1160.0, 1130.0, 1060.0, 990.0, 950.0, 760.0, 550.0,
        ],
    ),
    (
        "Y",
        [
            450.0, 400.0, 350.0, 300.0, 250.0, 220.0, 200.0, 180.0, 160.0, 140.0,
        ],
    ),
];

/// Giant scale for G, K and M (luminosity class III and brighter), K, after
/// the giant column of Allen's Astrophysical Quantities (Cox 2000, table
/// 15.7) and van Belle et al. 1999. Giants skip K6 to K9 (K5 is followed by
/// M0), so those subclasses are absent (NaN, never nearest). Unverified.
const GIANT: [(&str, [f64; 10]); 3] = [
    (
        "G",
        [
            5850.0, 5700.0, 5560.0, 5400.0, 5200.0, 5050.0, 4950.0, 4875.0, 4800.0, 4730.0,
        ],
    ),
    (
        "K",
        [
            4660.0,
            4510.0,
            4390.0,
            4250.0,
            4150.0,
            4050.0,
            f64::NAN,
            f64::NAN,
            f64::NAN,
            f64::NAN,
        ],
    ),
    (
        "M",
        [
            3690.0, 3600.0, 3540.0, 3460.0, 3400.0, 3330.0, 3250.0, 3150.0, 3000.0, 2800.0,
        ],
    ),
];

/// Temperatures (K) at which the luminosity-class loci are given.
const LOCUS_T: [f64; 10] = [
    40000.0, 30000.0, 15000.0, 9700.0, 7200.0, 5900.0, 5200.0, 4400.0, 3800.0, 3000.0,
];
/// log g (cgs) of each luminosity class at those temperatures. Straizys and
/// Kuriliene 1981, Gray and Corbally 2009. Unverified.
const LOCI: [(&str, [f64; 10]); 7] = [
    (
        "V",
        [3.90, 3.90, 4.10, 4.20, 4.30, 4.40, 4.50, 4.60, 4.70, 5.00],
    ),
    (
        "IV",
        [3.70, 3.60, 3.80, 3.90, 3.90, 3.90, 3.75, 3.60, 3.50, 3.40],
    ),
    (
        "III",
        [3.50, 3.40, 3.50, 3.60, 3.40, 3.00, 2.70, 2.00, 1.40, 0.90],
    ),
    (
        "II",
        [3.35, 3.10, 2.90, 2.90, 2.60, 2.30, 1.90, 1.00, 0.60, 0.20],
    ),
    (
        "Ib",
        [3.25, 2.90, 2.50, 2.40, 1.90, 1.70, 1.30, 0.60, 0.30, 0.00],
    ),
    (
        "Iab",
        [3.20, 2.80, 2.25, 2.05, 1.60, 1.35, 1.00, 0.40, 0.10, -0.20],
    ),
    (
        "Ia",
        [3.15, 2.70, 2.00, 1.70, 1.30, 1.00, 0.70, 0.20, -0.10, -0.40],
    ),
];

/// Below this a white dwarf's spectrum is featureless whatever its
/// atmosphere (DC). Unverified, as above.
const WD_FEATURELESS_K: f64 = 5000.0;
/// Black holes above this mass (solar) are intermediate, above the next
/// supermassive.
const BH_INTERMEDIATE: f64 = 100.0;
const BH_SUPERMASSIVE: f64 = 1.0e5;
/// log g of the Sun (cgs): log10(G M_sun / R_sun^2) in cm s^-2.
pub fn log_g(mass_msun: f64, radius_rsun: f64) -> f64 {
    let g = sse_core::sheet::G * mass_msun * sse_core::sheet::M_SUN
        / (radius_rsun * sse_core::sheet::R_SUN * radius_rsun * sse_core::sheet::R_SUN);
    m::log10(g * 100.0)
}

/// Linear in log T between the anchor temperatures, clamped at the ends.
fn locus_at(row: &[f64; 10], teff: f64) -> f64 {
    let x = m::log10(teff);
    if teff >= LOCUS_T[0] {
        return row[0];
    }
    for i in 1..LOCUS_T.len() {
        if teff >= LOCUS_T[i] {
            let (a, b) = (m::log10(LOCUS_T[i - 1]), m::log10(LOCUS_T[i]));
            let w = (x - b) / (a - b);
            return row[i] + w * (row[i - 1] - row[i]);
        }
    }
    row[LOCUS_T.len() - 1]
}

/// The luminosity class whose locus is nearest in log g, with that locus.
fn luminosity_class(teff: f64, lg: f64) -> (&'static str, f64) {
    let mut best = ("V", f64::INFINITY, 0.0);
    for (c, row) in &LOCI {
        let at = locus_at(row, teff);
        let d = (lg - at).abs();
        if d < best.1 {
            best = (c, d, at);
        }
    }
    // Above the dwarf locus is still a dwarf (VI and below need metallicity
    // and are not given).
    (best.0, best.2)
}

/// The letter and subclass whose temperature is nearest (in log T) on a scale.
fn on_scale(scale: &[(&'static str, [f64; 10])], teff: f64) -> (&'static str, u8) {
    let mut best = (scale[0].0, 0u8, f64::INFINITY);
    let x = m::log10(teff);
    for (l, row) in scale {
        for (i, t) in row.iter().enumerate() {
            let d = (x - m::log10(*t)).abs();
            if d < best.2 {
                best = (l, i as u8, d);
            }
        }
    }
    (best.0, best.1)
}

fn family_of(lum: &str) -> &'static str {
    match lum {
        "V" => "main-sequence",
        "IV" => "subgiant",
        "III" => "giant",
        "II" => "bright-giant",
        _ => "supergiant",
    }
}

fn kelvin(t: f64) -> String {
    format!("{} K", m::round(t) as i64)
}

/// Half-step rounding of a white dwarf's temperature index (Sion et al. 1983).
fn wd_index(teff: f64) -> f64 {
    m::round(2.0 * 50400.0 / teff) / 2.0
}

fn index_text(x: f64) -> String {
    if x >= 10.0 {
        "10".into()
    } else if m::fmod(x, 1.0) == 0.0 {
        format!("{}", x as i64)
    } else {
        format!("{x:.1}")
    }
}

/// The class of a star from its life-line results.
pub fn classify(teff_k: f64, mass_msun: f64, radius_rsun: f64, phase: Phase) -> StarClass {
    match phase {
        Phase::BlackHole => {
            let (word, fam) = if mass_msun >= BH_SUPERMASSIVE {
                ("supermassive", "SMBH")
            } else if mass_msun >= BH_INTERMEDIATE {
                ("intermediate-mass", "IMBH")
            } else {
                ("stellar-mass", "BH")
            };
            StarClass {
                code: fam.into(),
                kind: fam.into(),
                letter: String::new(),
                subclass: None,
                luminosity: String::new(),
                family: "black-hole",
                why: format!(
                    "A {word} black hole: {:.3} solar masses (stellar under {BH_INTERMEDIATE}, supermassive from {BH_SUPERMASSIVE:e}).",
                    mass_msun
                ),
                notes: vec!["Spin and what it is feeding on are not yet known to the kernel (exotic objects packages).".into()],
            }
        }
        Phase::NeutronStar => StarClass {
            code: "NS".into(),
            kind: "NS".into(),
            letter: String::new(),
            subclass: None,
            luminosity: String::new(),
            family: "neutron-star",
            why: format!("A neutron star of {mass_msun:.2} solar masses: what is left of a star that began between 8 and 25 solar masses."),
            notes: vec!["Pulsar, magnetar and the like need its spin and field, not yet known to the kernel (exotic objects packages).".into()],
        },
        Phase::WhiteDwarf => {
            let ix = wd_index(teff_k);
            let lines = teff_k >= WD_FEATURELESS_K;
            let letter = if lines { "X" } else { "C" };
            let mut notes = vec![];
            if lines {
                notes.push("Whether it shows hydrogen (DA) or helium (DB, DO) needs the hydrogen layer left on it, which the life line does not track; X says it cannot be said.".into());
            }
            StarClass {
                code: format!("D{letter} {}", index_text(ix)),
                kind: format!("D{letter}"),
                letter: "D".into(),
                subclass: None,
                luminosity: String::new(),
                family: "white-dwarf",
                why: format!(
                    "A white dwarf at {}: temperature index 50400 / T = {} (Sion et al. 1983){}.",
                    kelvin(teff_k),
                    index_text(ix),
                    if lines { "" } else { "; below about 5000 K its spectrum shows no lines whatever its atmosphere (DC)" }
                ),
                notes,
            }
        }
        _ if phase != Phase::BrownDwarf && log_g(mass_msun, radius_rsun) > 6.5 => {
            // No phase given and white-dwarf gravity: read as one.
            classify(teff_k, mass_msun, radius_rsun, Phase::WhiteDwarf)
        }
        _ => {
            let lg = log_g(mass_msun, radius_rsun);
            let brown = phase == Phase::BrownDwarf || teff_k < 2300.0;
            let (lum, locus) = if brown { ("", 0.0) } else { luminosity_class(teff_k, lg) };
            let giant_scale = matches!(lum, "III" | "II" | "Ib" | "Iab" | "Ia") && teff_k < 6000.0;
            let (letter, sub) = if giant_scale { on_scale(&GIANT, teff_k) } else { on_scale(&DWARF, teff_k) };
            let mut notes = vec![];
            let family = if phase == Phase::BrownDwarf {
                "brown-dwarf"
            } else if phase == Phase::PreMainSequence {
                notes.push("Still contracting towards the main sequence (a T Tauri or Herbig star).".into());
                "pre-main-sequence"
            } else if brown {
                "main-sequence"
            } else {
                family_of(lum)
            };
            let code = if lum.is_empty() { format!("{letter}{sub}") } else { format!("{letter}{sub} {lum}") };
            let why = if lum.is_empty() {
                format!(
                    "Surface {} reads {letter}{sub} on the dwarf temperature scale{}.",
                    kelvin(teff_k),
                    if phase == Phase::BrownDwarf { "; a brown dwarf, never a hydrogen-burning star" } else { "" }
                )
            } else {
                format!(
                    "Surface {} reads {letter}{sub} on the {} temperature scale; surface gravity log g = {lg:.2} (from {mass_msun:.3} solar masses in {radius_rsun:.3} solar radii) is nearest the class {lum} line, log g {locus:.2} at that temperature.",
                    kelvin(teff_k),
                    if giant_scale { "giant" } else { "dwarf" }
                )
            };
            if teff_k > 60000.0 {
                notes.push("Hotter than the O scale: read as O0.".into());
            }
            StarClass { code, kind: letter.into(), letter: letter.into(), subclass: Some(sub), luminosity: lum.into(), family, why, notes }
        }
    }
}

/// Phase from its life-line label ("main sequence", "white-dwarf", ...).
pub fn phase_of(label: &str) -> Option<Phase> {
    let k = label.replace(['_', ' '], "-").to_lowercase();
    Some(match k.as_str() {
        "pre-main-sequence" => Phase::PreMainSequence,
        "main-sequence" => Phase::MainSequence,
        "subgiant" | "hertzsprung-gap" => Phase::HertzsprungGap,
        "giant" => Phase::Giant,
        "brown-dwarf" => Phase::BrownDwarf,
        "white-dwarf" => Phase::WhiteDwarf,
        "neutron-star" => Phase::NeutronStar,
        "black-hole" => Phase::BlackHole,
        _ => return None,
    })
}

/// The temperature scales and gravity loci as data, for the atlas generator
/// (scripts/build-base-atlas.py reads them from tests/golden/star-classes.json,
/// so the numbers live here once).
pub fn tables() -> serde_json::Value {
    let rows = |t: &[(&str, [f64; 10])]| -> serde_json::Value {
        t.iter()
            .map(|(l, r)| {
                let v: Vec<serde_json::Value> = r
                    .iter()
                    .map(|x| {
                        if x.is_nan() {
                            serde_json::Value::Null
                        } else {
                            serde_json::json!(x)
                        }
                    })
                    .collect();
                (l.to_string(), serde_json::Value::Array(v))
            })
            .collect::<serde_json::Map<_, _>>()
            .into()
    };
    serde_json::json!({
        "dwarf_teff_k": rows(&DWARF),
        "giant_teff_k": rows(&GIANT),
        "locus_teff_k": LOCUS_T,
        "locus_log_g": rows(&LOCI),
        "white_dwarf_featureless_k": WD_FEATURELESS_K,
        "black_hole_msun": [BH_INTERMEDIATE, BH_SUPERMASSIVE],
    })
}

/// One star on the life line that lands in a family and kind: a sample the
/// atlas and the class gallery can draw.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Sample {
    pub family: &'static str,
    pub kind: String,
    pub code: String,
    pub mass_msun: f64,
    pub age_yr: f64,
    pub feh: f64,
}

/// Sweep the life line (initial masses 0.013 to 100 solar, ages 10 kyr to
/// 13.8 Gyr, solar [Fe/H]) and keep, for every family and kind reached, the
/// middle one of the stars that land there (ordered by mass, then age).
pub fn samples() -> Vec<Sample> {
    let mut hits: std::collections::BTreeMap<(usize, String), Vec<Sample>> = Default::default();
    for i in 0..=48 {
        let mass = 0.013 * m::pow(100.0 / 0.013, f64::from(i) / 48.0);
        for j in 0..=96 {
            let age = 1.0e4 * m::pow(1.38e10 / 1.0e4, f64::from(j) / 96.0);
            let Ok(d) = crate::describe::describe(mass, age, 0.0) else {
                continue;
            };
            let n = &d.now;
            let c = classify(n.teff_k, n.mass_msun, n.radius_rsun, n.phase);
            let f = FAMILIES.iter().position(|x| *x == c.family).unwrap_or(0);
            hits.entry((f, c.kind.clone())).or_default().push(Sample {
                family: c.family,
                kind: c.kind,
                code: c.code,
                mass_msun: mass,
                age_yr: age,
                feh: 0.0,
            });
        }
    }
    hits.into_values().map(|v| v[v.len() / 2].clone()).collect()
}

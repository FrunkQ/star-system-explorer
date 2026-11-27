//! Golden cases for every first-wave atom. Reference values are cited inline:
//! "NASA fact sheet" means the NASA GSFC Planetary Fact Sheets
//! (nssdc.gsfc.nasa.gov/planetary/factsheet, values as of 2024).
//! "IAU 2015 B3" is the IAU 2015 Resolution B3 nominal solar values.

use super::*;
use sse_core::math as m;
use sse_core::quantity::consts::*;
use std::collections::BTreeSet;
use std::path::PathBuf;

const M_SUN: f64 = GM_SUN / G;
const M_EARTH: f64 = GM_EARTH / G;
const M_JUP: f64 = GM_JUPITER / G;
const GYR: f64 = 1e9 * JULIAN_YEAR;

fn run(id: &str, inputs: &[f64]) -> Vec<f64> {
    run_p(id, inputs, &[])
}

/// Evaluate an atom with its default params, overriding by name.
fn run_p(id: &str, inputs: &[f64], over: &[(&str, f64)]) -> Vec<f64> {
    let reg = registry();
    let a = reg.get(id).unwrap_or_else(|| panic!("no atom {id}"));
    let d = a.decl();
    assert_eq!(inputs.len(), d.inputs.len(), "{id}: input count");
    let mut p: Vec<f64> = d.params.iter().map(|p| p.default).collect();
    for (n, v) in over {
        p[d.param(n).expect("param").0] = *v;
    }
    let out = a.eval(inputs, &p).unwrap_or_else(|e| panic!("{id}: {e}"));
    assert_eq!(out.len(), d.outputs.len(), "{id}: output count");
    out
}

fn close(got: f64, want: f64, rel: f64, what: &str) {
    let err = ((got - want) / want).abs();
    assert!(
        err <= rel,
        "{what}: got {got}, want {want} within {rel} (off by {err:.4})"
    );
}

// ---- declarations -------------------------------------------------------

#[test]
fn every_atom_is_cited_explained_and_versioned() {
    let reg = registry();
    let mut n = 0;
    for d in reg.decls() {
        n += 1;
        assert!(d.id.starts_with("sse."), "{}", d.id);
        assert!(!d.references.is_empty(), "{} has no citation", d.id);
        // Readers follow a citation to its source (Alex, 18 Nov): every atom
        // needs at least one reference with a DOI or link, except those whose
        // only sources are textbooks without one (listed, so new ones are seen).
        const TEXTBOOK_ONLY: &[&str] = &["sse.ocean.brine_freezing_point"];
        assert!(
            d.references.iter().any(|r| r.linked()) || TEXTBOOK_ONLY.contains(&d.id.as_str()),
            "{} has no reference with a DOI or link",
            d.id
        );
        for r in &d.references {
            if let Some(doi) = &r.doi {
                assert!(
                    doi.starts_with("10.") && doi.contains('/'),
                    "{}: DOI {doi}",
                    d.id
                );
                assert_eq!(
                    r.url.as_deref(),
                    Some(format!("https://doi.org/{doi}").as_str())
                );
            }
            if let Some(u) = &r.url {
                assert!(
                    u.starts_with("https://") && !u.contains(' '),
                    "{}: link {u}",
                    d.id
                );
            }
            assert!(
                !r.text.contains("doi:") && !r.text.contains("https://"),
                "{}: {}",
                d.id,
                r.text
            );
        }
        assert!(
            !d.explain.curious.is_empty(),
            "{} has no curious lens",
            d.id
        );
        assert!(
            !d.explain.science.is_empty(),
            "{} has no science lens",
            d.id
        );
        assert!(!d.explain.formula.is_empty(), "{} has no formula", d.id);
        assert!(d.version.split('.').count() == 3, "{} version", d.id);
        for p in &d.params {
            let (lo, hi) = p.range.expect("params are bounded");
            assert!(
                lo <= p.default && p.default <= hi,
                "{} param {}",
                d.id,
                p.name
            );
        }
    }
    // The count is derived from the declarations themselves, so two pull
    // requests that each add an atom never edit the same line (it was a
    // hand-kept number, and every addition conflicted with every other).
    let declared = declared_atom_ids();
    let registered: BTreeSet<String> = reg.decls().map(|d| d.id.clone()).collect();
    let unregistered: Vec<&String> = declared
        .iter()
        .filter(|id| !registered.contains(*id))
        .collect();
    let undeclared: Vec<&String> = registered
        .iter()
        .filter(|id| !declared.contains(*id))
        .collect();
    assert!(
        unregistered.is_empty() && undeclared.is_empty(),
        "declared but not in atoms(): {unregistered:?}; in atoms() but not found as a declaration: {undeclared:?}"
    );
    assert_eq!(
        atoms().len(),
        n,
        "two atoms share an id (the registry keeps only one of them)"
    );
    assert_eq!(declared.len(), n);
    // A floor, not a count: additions never touch it. It catches an atom lost
    // by accident (a dropped block in a merge). Lower it only when an atom is
    // removed on purpose, and say so in the package log.
    const ATOM_FLOOR: usize = 183;
    assert!(
        n >= ATOM_FLOOR,
        "{n} atoms, fewer than the floor {ATOM_FLOOR}: an atom was lost (lower the floor only for a removal on purpose)"
    );
}

/// Every atom id declared in the sources: each `id: "sse.…"` in the Rust atom
/// modules and each entry of each formula file in `packs/base-physics/formulas/`.
/// Fails on a formula file that `formulas::FILES` does not read, and on an id
/// declared twice.
fn declared_atom_ids() -> BTreeSet<String> {
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut ids: Vec<String> = vec![];
    let mut rust: Vec<PathBuf> = std::fs::read_dir(crate_dir.join("src"))
        .expect("src")
        .map(|e| e.expect("entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .filter(|p| p.file_name().is_some_and(|f| f != "tests.rs"))
        .collect();
    rust.sort();
    for p in &rust {
        let text = std::fs::read_to_string(p).expect("read atom module");
        for line in text.lines() {
            if let Some(rest) = line.trim_start().strip_prefix("id: \"sse.") {
                let end = rest.find('"').expect("closing quote");
                ids.push(format!("sse.{}", &rest[..end]));
            }
        }
    }
    let dir = crate_dir.join("../../packs/base-physics/formulas");
    let mut on_disk: Vec<String> = std::fs::read_dir(&dir)
        .expect("formulas dir")
        .map(|e| e.expect("entry").path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .map(|p| p.file_stem().expect("stem").to_string_lossy().into_owned())
        .collect();
    on_disk.sort();
    let mut read: Vec<String> = formulas::FILES.iter().map(|(f, _)| f.to_string()).collect();
    read.sort();
    assert_eq!(
        on_disk, read,
        "every formula file in packs/base-physics/formulas/ must be listed in formulas::FILES"
    );
    for family in &on_disk {
        let text =
            std::fs::read_to_string(dir.join(format!("{family}.json"))).expect("formula file");
        let doc: serde_json::Value = serde_json::from_str(&text).expect("formula json");
        for a in doc["atoms"].as_array().into_iter().flatten() {
            ids.push(a["id"].as_str().expect("formula atom id").to_string());
        }
    }
    let total = ids.len();
    let set: BTreeSet<String> = ids.into_iter().collect();
    assert_eq!(set.len(), total, "an atom id is declared twice");
    set
}

#[test]
fn every_port_kind_is_described_and_dimensionally_consistent() {
    let desc: BTreeMap<&str, &str> = KIND_DESCRIPTIONS.iter().copied().collect();
    let reg = registry();
    let mut dims: BTreeMap<String, Dimension> = BTreeMap::new();
    for d in reg.decls() {
        for p in d.inputs.iter().chain(d.outputs.iter()) {
            assert!(
                desc.contains_key(p.kind.as_str()),
                "kind {} undescribed",
                p.kind
            );
            if let Some(old) = dims.insert(p.kind.clone(), p.dim) {
                assert_eq!(old, p.dim, "kind {} used with two dimensions", p.kind);
            }
        }
    }
    for (k, _) in KIND_DESCRIPTIONS {
        assert!(dims.contains_key(*k), "described kind {k} is unused");
    }
}

// ---- stars --------------------------------------------------------------

#[test]
fn star_luminosity_reproduces_eker_2018() {
    // Formula reproduction (Eker et al. 2018 table 4) and the Sun.
    let l = |ms: f64| run("sse.star.ms_luminosity", &[ms * M_SUN])[0] / L_SUN;
    close(l(1.0), m::pow(10.0, -0.007), 1e-12, "1 M☉");
    close(
        l(2.0),
        m::pow(10.0, 4.329 * m::log10(2.0) + 0.010),
        1e-12,
        "2 M☉",
    );
    close(
        l(0.3),
        m::pow(10.0, 2.028 * m::log10(0.3) - 0.976),
        1e-12,
        "0.3 M☉",
    );
    close(l(10.0), m::pow(10.0, 2.865 + 1.105), 1e-12, "10 M☉");
    close(l(1.0), 1.0, 0.03, "Sun (IAU 2015 B3 L☉)");
}

#[test]
fn star_radius_reproduces_demircan_kahraman_1991() {
    let r = |ms: f64| run("sse.star.ms_radius", &[ms * M_SUN])[0] / R_SUN;
    close(r(1.0), 1.06, 1e-12, "1 M☉");
    close(r(4.0), 1.33 * m::pow(4.0, 0.555), 1e-12, "4 M☉");
    close(r(1.0), 1.0, 0.07, "Sun");
}

#[test]
fn star_effective_temperature_of_the_sun() {
    // IAU 2015 B3: nominal T_eff of the Sun 5772 K from nominal L and R.
    let t = run("sse.star.effective_temperature", &[L_SUN, R_SUN])[0];
    close(t, 5772.0, 2e-4, "Sun T_eff");
}

#[test]
fn star_lifetime_and_brightening() {
    let life = run("sse.star.ms_lifetime", &[M_SUN, L_SUN])[0];
    close(life / GYR, 10.0, 1e-12, "Sun lifetime");
    // A 2 M☉ star burns out in about 1 Gyr.
    let l2 = run("sse.star.ms_luminosity", &[2.0 * M_SUN])[0];
    let t2 = run("sse.star.ms_lifetime", &[2.0 * M_SUN, l2])[0] / GYR;
    assert!(t2 > 0.7 && t2 < 1.3, "2 M☉ lifetime {t2} Gyr");
    // Gough 1981: the zero-age Sun had about 70 percent of today's light.
    let at = |age: f64| {
        run(
            "sse.star.ms_luminosity_at_age",
            &[L_SUN, age * GYR, 10.0 * GYR],
        )[0]
    };
    close(at(4.57), L_SUN, 1e-12, "today");
    close(at(0.0) / L_SUN, 1.0 / 1.4, 1e-12, "zero age");
    // Out of the main sequence: refused, not guessed.
    let reg = registry();
    let a = reg.get("sse.star.ms_luminosity_at_age").unwrap();
    assert!(a.eval(&[L_SUN, 11.0 * GYR, 10.0 * GYR], &[0.457]).is_err());
}

// ---- orbits -------------------------------------------------------------

#[test]
fn orbit_periods_of_earth_and_jupiter() {
    // NASA fact sheet sidereal periods: Earth 365.256 d, Jupiter 4332.589 d.
    let mt = run("sse.orbit.total_mass", &[M_SUN, M_EARTH])[0];
    let p = run("sse.orbit.period", &[AU * 1.000_001_018, mt])[0] / DAY;
    close(p, 365.256, 2e-5, "Earth year");
    let pj = run("sse.orbit.period", &[778.479e9, M_SUN + M_JUP])[0] / DAY;
    close(pj, 4332.589, 1e-3, "Jupiter year");
    let a = run("sse.orbit.semi_major_axis_from_period", &[p * DAY, mt])[0];
    close(a, AU * 1.000_001_018, 1e-12, "round trip");
}

#[test]
fn orbit_apsides_of_earth() {
    // NASA fact sheet: perihelion 147.095e6 km, aphelion 152.100e6 km.
    let q = run("sse.orbit.apsides", &[149.598e9, 0.0167])[0];
    let big_q = run("sse.orbit.apsides", &[149.598e9, 0.0167])[1];
    close(q, 147.095e9, 2e-4, "perihelion");
    close(big_q, 152.100e9, 2e-4, "aphelion");
}

#[test]
fn kepler_equation_is_solved_to_machine_precision() {
    for ei in 0..100 {
        let e = ei as f64 * 0.01;
        for mi in -40..=40 {
            let mean = mi as f64 * 0.25;
            let out = run("sse.orbit.kepler_position", &[mean, e]);
            let big_e = out[0];
            let mut mm = m::fmod(mean, m::TAU);
            if mm > m::PI {
                mm -= m::TAU;
            } else if mm <= -m::PI {
                mm += m::TAU;
            }
            let resid = big_e - e * m::sin(big_e) - mm;
            assert!(resid.abs() < 1e-12, "e={e} M={mean}: residual {resid}");
            assert!((out[2] - (1.0 - e * m::cos(big_e))).abs() < 1e-15, "r/a");
        }
    }
    let circ = run("sse.orbit.kepler_position", &[1.0, 0.0]);
    close(circ[1], 1.0, 1e-15, "circle: true = mean anomaly");
}

#[test]
fn hill_radius_and_roche_limit() {
    // Earth's Hill sphere is about 1.5 million km (Murray and Dermott 1999).
    let h = run("sse.orbit.hill_radius", &[AU, 0.0, M_EARTH, M_SUN])[0];
    close(h, 1.497e9, 0.01, "Earth Hill radius");
    // Fluid Roche limit of the Moon around Earth: about 18,400 km.
    let d = run("sse.orbit.roche_limit", &[6.371e6, 5514.0, 3344.0])[0];
    close(
        d,
        2.44 * 6.371e6 * m::cbrt(5514.0 / 3344.0),
        1e-12,
        "formula",
    );
    close(d, 1.84e7, 0.01, "Earth-Moon fluid Roche");
}

// ---- planets ------------------------------------------------------------

#[test]
fn chen_kipping_relation() {
    let r = |me: f64| run("sse.planet.radius_from_mass", &[me * M_EARTH])[0] / R_EARTH;
    close(r(1.0), 1.008, 1e-12, "Terran constant");
    // Continuous at every break.
    for (hi, _) in planet::CK_SEGMENTS.iter().take(3) {
        close(
            r(hi * (1.0 + 1e-12)),
            r(hi * (1.0 - 1e-12)),
            1e-9,
            "continuity",
        );
    }
    // Paper's Neptunian constant 0.80811 follows from continuity.
    close(r(10.0) / m::pow(10.0, 0.589), 0.808_11, 2e-3, "Neptunian C");
    // A forecast: Jupiter comes out within 30 percent (Chen and Kipping fig. 4 scatter).
    let rj = r(M_JUP / M_EARTH) * R_EARTH / R_JUPITER;
    assert!(rj > 0.9 && rj < 1.3, "Jupiter forecast {rj} RJ");
}

#[test]
fn bulk_properties_of_earth_mars_and_moon() {
    // NASA fact sheets: (mass kg, mean radius m, density kg/m³, g m/s², v_esc m/s).
    let cases = [
        ("Earth", 5.9722e24, 6.3710e6, 5513.0, 9.820, 11186.0),
        ("Mars", 0.64169e24, 3.3895e6, 3934.0, 3.727, 5027.0),
        ("Moon", 0.07346e24, 1.7374e6, 3344.0, 1.624, 2380.0),
    ];
    for (name, mass, r, rho, g, v) in cases {
        close(
            run("sse.planet.bulk_density", &[mass, r])[0],
            rho,
            2e-3,
            name,
        );
        close(
            run("sse.planet.surface_gravity", &[mass, r])[0],
            g,
            3e-3,
            name,
        );
        close(
            run("sse.planet.escape_velocity", &[mass, r])[0],
            v,
            2e-3,
            name,
        );
    }
}

#[test]
fn rocky_interiors_of_earth() {
    // Earth: CMF 0.325 (Zeng et al. 2016), core radius 3480 km (PREM).
    let r = run("sse.interior.rocky_radius", &[M_EARTH, 0.325])[0];
    close(r, 6.371e6, 0.01, "Earth radius from CMF");
    let cmf = run("sse.interior.core_mass_fraction", &[M_EARTH, 6.371e6])[0];
    assert!((cmf - 0.325).abs() < 0.05, "Earth CMF {cmf}");
    let rc = run("sse.interior.core_radius", &[6.371e6, 0.325])[0];
    close(rc, 3.480e6, 0.05, "Earth core radius");
}

// ---- climate and atmosphere ---------------------------------------------

#[test]
fn insolation_and_equilibrium_temperatures() {
    // Solar constant 1361 W/m² (Kopp and Lean 2011; IAU 2015 B3).
    let s = run("sse.climate.insolation", &[L_SUN, AU])[0];
    close(s, 1361.0, 1e-3, "solar constant");
    // NASA fact sheet black-body temperatures.
    let cases = [
        ("Venus", 0.723_332, 0.77, 226.6),
        ("Earth", 1.0, 0.306, 254.0),
        ("Mars", 1.523_679, 0.25, 209.8),
    ];
    for (name, d, a, t) in cases {
        let s = run("sse.climate.insolation", &[L_SUN, d * AU])[0];
        let teq = run("sse.climate.equilibrium_temperature", &[s, a])[0];
        assert!((teq - t).abs() < 0.6, "{name}: {teq} K vs {t} K");
    }
    // Day-side only redistribution is 2^¼ warmer.
    let full = run("sse.climate.equilibrium_temperature", &[1361.0, 0.3])[0];
    let day = run_p(
        "sse.climate.equilibrium_temperature",
        &[1361.0, 0.3],
        &[("redistribution", 0.5)],
    )[0];
    close(day / full, m::pow(2.0, 0.25), 1e-12, "redistribution");
}

#[test]
fn grey_greenhouse() {
    let t = |tau: f64| run("sse.climate.grey_surface_temperature", &[254.0, tau])[0];
    close(t(0.0), 254.0, 1e-15, "no atmosphere");
    // Earth's 288 K needs τ ≈ 0.83 in this model (Catling and Kasting 2017, ch. 2).
    close(t(0.83), 288.0, 0.005, "Earth");
}

#[test]
fn atmosphere_basics_of_earth() {
    // Scale height about 8.5 km (NASA fact sheet), at 288 K, μ 28.97.
    let h = run("sse.atmosphere.scale_height", &[288.0, 28.97, 9.80665])[0];
    close(h, 8.43e3, 0.01, "scale height");
    // Atmosphere mass 5.15e18 kg (Trenberth and Smith 2005) gives about 1 bar.
    let p = run(
        "sse.atmosphere.surface_pressure",
        &[5.148e18, 9.80665, 6.371e6],
    )[0];
    close(
        p,
        0.985e5,
        0.01,
        "mean surface pressure (Trenberth and Smith 2005: 98.5 kPa)",
    );
    // Hydrogen at a 1000 K exobase 500 km up: λ ≈ 7 (Catling and Kasting 2017, ch. 5).
    let lam = run(
        "sse.atmosphere.jeans_parameter",
        &[M_EARTH, 6.871e6, 1000.0, 1.008],
    )[0];
    assert!(lam > 6.5 && lam < 7.5, "λ_H = {lam}");
}

// ---- tides --------------------------------------------------------------

#[test]
fn io_tidal_heating() {
    // Io: R 1821.6 km, a 421,700 km, e 0.0041; Lainey et al. 2009 k2/Q 0.015
    // and observed heat flow (0.93 ± 0.19) × 10¹⁴ W.
    let p = run(
        "sse.tides.eccentricity_heating",
        &[M_JUP, 1.8216e6, 4.217e8, 0.0041],
    )[0];
    close(p, 0.93e14, 0.2, "Io heating");
}

#[test]
fn maxwell_response_limits() {
    // HF4a: a uniform Maxwell body. Stiff and cold, it is elastic, k2 =
    // 3/2 / (1 + 19μ/2ρgR) with no lag; runny, it is a fluid, k2 = 3/2
    // with no lag; between, Im k2 peaks at ωη/μ = 1/(1 + A) at
    // (3/2) A / (2 (1 + A)). Io: μ 6.5e10 Pa, ρ 3530, g 1.796, R 1821.6 km.
    let (mu, rho, g, r, w) = (6.5e10, 3530.0, 1.796, 1.8216e6, 4.11e-5);
    let a = 19.0 * mu / (2.0 * rho * g * r);
    let at = |eta: f64| run("sse.tides.maxwell_response", &[mu, eta, rho, g, r, w]);
    let stiff = at(1e30);
    close(stiff[1], 1.5 / (1.0 + a), 1e-6, "elastic Re k2");
    assert!(stiff[0] < 1e-9, "elastic Im k2 {}", stiff[0]);
    let runny = at(1e3);
    close(runny[1], 1.5, 1e-6, "fluid Re k2");
    assert!(runny[0] < 1e-9, "fluid Im k2 {}", runny[0]);
    let peak = at(mu / (w * (1.0 + a)));
    close(peak[0], 0.75 * a / (1.0 + a), 1e-9, "peak Im k2");
    // Io's solid k2 is about 0.03 (absolute, not a ratio).
    assert!(
        stiff[1] > 0.02 && stiff[1] < 0.04,
        "Io elastic k2 {}",
        stiff[1]
    );
}

#[test]
fn andrade_response_is_maxwell_with_creep_off() {
    // HF4b.0: with its slow creep switched off the Andrade block is the
    // Maxwell block exactly, at every viscosity; with it on, a stiff Io
    // (1e19 Pa s, where Maxwell gives almost nothing) still lags: Andrade's
    // Im k2 there is above 1e-4 (absolute), Maxwell's below 1e-5.
    let (mu, rho, g, r, w) = (6.5e10, 3530.0, 1.796, 1.8216e6, 4.11e-5);
    for eta in [1e10, 1e13, 1.6e14, 1e16, 1e19, 1e22] {
        let mx = run("sse.tides.maxwell_response", &[mu, eta, rho, g, r, w]);
        let an = run_p(
            "sse.tides.andrade_response",
            &[mu, eta, rho, g, r, w, 0.3],
            &[("transient_creep", 0.0)],
        );
        close(an[0], mx[0], 1e-12, "Im k2, creep off");
        close(an[1], mx[1], 1e-12, "Re k2, creep off");
    }
    let mx = run("sse.tides.maxwell_response", &[mu, 1e19, rho, g, r, w]);
    let an = run("sse.tides.andrade_response", &[mu, 1e19, rho, g, r, w, 0.3]);
    assert!(mx[0] < 1e-5, "Maxwell Im k2 at 1e19 Pa s: {}", mx[0]);
    assert!(an[0] > 1e-4, "Andrade Im k2 at 1e19 Pa s: {}", an[0]);
}

#[test]
fn lid_conduction_without_burial_is_plain_conduction() {
    // HF4b.0: with no lava burying it the lid conducts k dT / D, its own
    // heat split half up and half down; burial chokes the top and feeds the
    // base, and the two still differ by exactly the heat made in the lid
    // plus what the moving lid carries (steady state: base - top = rho c v dT - h).
    let (k, tl, ts, d, h, kap) = (3.0, 1500.0, 220.0, 1.0e5, 0.005, 1.0e-6);
    let q = run(
        "sse.heat.lid_conduction_with_burial",
        &[k, tl, ts, d, h, 0.0, kap],
    );
    close(q[0], k * (tl - ts) / d + h / 2.0, 1e-12, "top, no burial");
    close(q[1], k * (tl - ts) / d - h / 2.0, 1e-12, "base, no burial");
    let v = 3.0e-11;
    let b = run(
        "sse.heat.lid_conduction_with_burial",
        &[k, tl, ts, d, h, v, kap],
    );
    assert!(
        b[0] < q[0] && b[1] > q[1],
        "burial chokes the top and feeds the base"
    );
    close(
        b[1] - b[0],
        k / kap * v * (tl - ts) - h,
        1e-9,
        "advected heat",
    );
}

#[test]
fn melt_heat_split_adds_up() {
    // HF4b.0: the mantle's loss to melt is what the lid receives from
    // intrusions plus what erupted lava gives the surface less the lava's
    // cooling below the lid's base (rho Pe c (Tl - Ts)), to round-off.
    let (rho, pe, pi, l, c, tm, tl, ts) =
        (2800.0, 10.0, 42.0, 4.0e5, 1200.0, 1700.0, 1520.0, 220.0);
    let o = run("sse.melt.heat_carried", &[rho, pe, pi, l, c, tm, tl, ts]);
    close(
        o[0],
        o[1] + o[2] - rho * pe * c * (tl - ts),
        1e-12,
        "melt heat",
    );
    let e = run("sse.melt.erupted_share", &[52.0, 4.17e13]);
    close(e[0] + e[1], 52.0, 1e-12, "erupted plus intruded");
}

#[test]
fn locking_time_formula() {
    let (w, a, mb, r, mh) = (
        2.0 * m::PI / (12.0 * 3600.0),
        3.844e8,
        7.346e22,
        1.7374e6,
        M_EARTH,
    );
    let t = run("sse.tides.locking_time", &[w, a, mb, r, mh])[0];
    let (a6, r5) = (a * a * a * a * a * a, r * r * r * r * r);
    let want = w * a6 * 0.4 * mb * r * r * 100.0 / (3.0 * G * mh * mh * 0.3 * r5);
    close(t, want, 1e-12, "Gladman eq. 9");
    // The Moon at today's distance locks in tens of millions of years.
    let yr = t / JULIAN_YEAR;
    assert!(yr > 1e6 && yr < 1e9, "Moon locking time {yr} yr");
}

// ---- layers ----------------------------------------------------------------

#[test]
fn brine_freezing_points() {
    let f = |w: f64| run("sse.ocean.brine_freezing_point", &[w])[0];
    close(f(0.0), 273.15, 1e-15, "pure water");
    // Seawater (35 g/kg) freezes at -1.92 °C (UNESCO 1983); it is not pure NaCl,
    // so this NaCl model is checked to 15 percent of the depression.
    let d = 273.15 - f(0.035);
    assert!((d - 1.92).abs() < 0.15 * 1.92, "seawater depression {d}");
    // 10 percent NaCl freezes near -6.6 °C (CRC Handbook, aqueous NaCl).
    let d10 = 273.15 - f(0.10);
    assert!((d10 - 6.6).abs() < 0.1 * 6.6, "10% NaCl depression {d10}");
}

#[test]
fn ice_shell_conduction() {
    // Europa-like: 100 K surface, base at 270 K, 0.05 W/m² gives about 13 km,
    // inside the 25 km bound of Ojakangas and Stevenson 1989's conductive shell.
    let d = run("sse.heat.ice_shell_thickness", &[0.05, 100.0, 270.0])[0];
    close(d, 651.0 / 0.05 * m::ln(2.7), 1e-12, "formula");
    assert!(d > 1e4 && d < 2.5e4);
    let q = run("sse.heat.ice_shell_heat_flow", &[d, 100.0, 270.0])[0];
    close(q, 0.05, 1e-12, "round trip");
}

#[test]
fn hydrostatic_pressure_of_deep_water() {
    // 10.9 km of seawater (1025 kg/m³ at the surface) in the Mariana Trench: about
    // 1.086e8 Pa measured (Bartlett 2009); compression is inside the 3 percent.
    let p = run(
        "sse.interior.hydrostatic_pressure",
        &[1025.0, 9.81, 10_900.0],
    )[0];
    close(p, 1.086e8, 0.03, "Challenger Deep");
}

// ---- the base-physics pack ----------------------------------------------

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn build_pack() -> serde_json::Value {
    let dir = root().join("packs/base-physics/flows");
    let mut names: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().is_some_and(|x| x == "json"))
        .collect();
    names.sort();
    let flows: Vec<serde_json::Value> = names
        .iter()
        .map(|p| serde_json::from_str(&std::fs::read_to_string(p).unwrap()).unwrap())
        .collect();
    serde_json::json!({
        "format": "sse.pack/1",
        "id": "sse.base-physics",
        "version": "0.1.0",
        "kind": ["base-physics"],
        "kernel_range": "^0.1",
        "licence": "Apache-2.0",
        "provides": {
            "quantity_kinds": kinds().into_values().collect::<Vec<_>>(),
            "flows": flows,
        }
    })
}

/// packs/base-physics/pack.json is generated from the atoms' kinds and the
/// flow files. Set SSE_BLESS=1 to rewrite it.
#[test]
fn base_physics_pack_is_current() {
    let text = serde_json::to_string_pretty(&build_pack()).unwrap() + "\n";
    let path = root().join("packs/base-physics/pack.json");
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(&path, &text).unwrap();
    }
    let on_disk = std::fs::read_to_string(&path).expect("run with SSE_BLESS=1 once");
    assert!(
        on_disk == text,
        "packs/base-physics/pack.json is stale: run SSE_BLESS=1 cargo test -p sse-atoms"
    );
}

#[test]
fn base_physics_pack_resolves_and_every_flow_passes_its_golden_cases() {
    let text = std::fs::read_to_string(root().join("packs/base-physics/pack.json")).unwrap();
    let pack = sse_pack::PackManifest::from_json(&text).unwrap();
    let stack = sse_pack::resolve(&[pack], "0.1.0").unwrap_or_else(|e| panic!("{e:?}"));
    let atoms = registry();
    let cx = sse_flow::Context {
        atoms: &atoms,
        kinds: &stack.kinds,
        units: &stack.units,
    };
    assert!(stack.flows.len() >= 5);
    let mut cases = 0;
    for f in stack.flows.values() {
        let flow = sse_flow::Flow::compile(f.doc.clone(), &cx)
            .unwrap_or_else(|e| panic!("{}: {e:?}", f.doc.id));
        assert!(!f.doc.tests.is_empty(), "{} has no golden cases", f.doc.id);
        cases += f.doc.tests.len();
        let fails = flow.run_tests(&cx);
        assert!(fails.is_empty(), "{}:\n{}", f.doc.id, fails.join("\n"));
    }
    assert!(cases >= 10, "Solar System fixture has {cases} cases");
}

/// Heat-producing elements: Earth's budget, the decay history and Al-26.
#[test]
fn radiogenic_heat_budget() {
    let gyr = 1e9 * sse_core::quantity::consts::JULIAN_YEAR;
    let myr = 1e-3 * gyr;
    // Bulk silicate Earth today: about 4.9e-12 W/kg (Turcotte and Schubert 2014, table 4.2).
    let h = run("sse.heat.radiogenic_heating", &[4.567 * gyr, 1.0, 1.0])[0];
    close(h, 4.91e-12, 0.01, "BSE heating today");
    // At formation it was several times higher (about 4 to 5 times; Frank et al. 2014, fig. 3).
    let h0 = run("sse.heat.radiogenic_heating", &[0.0, 1.0, 1.0])[0];
    assert!((3.5..5.5).contains(&(h0 / h)), "H(0)/H(now) {}", h0 / h);
    // No K-40 at birth removes about a sixth of today's heat.
    let no_k = run("sse.heat.radiogenic_heating", &[4.567 * gyr, 0.0, 1.0])[0];
    close(no_k / h, 0.833, 0.01, "share without K-40");
    let mantle = run("sse.heat.mantle_mass", &[M_EARTH, 0.325])[0];
    let p = run("sse.heat.radiogenic_power", &[h, mantle])[0];
    close(p, 2.0e13, 0.05, "Earth radiogenic power");
    let q = run("sse.heat.surface_heat_flow", &[p, 6.371e6])[0];
    close(q, 0.0388, 0.02, "Earth radiogenic heat flow");
    let life = |r: f64, mass: f64, cmf: f64, radius: f64| {
        let mm = run("sse.heat.mantle_mass", &[mass, cmf])[0];
        run("sse.heat.active_geology_lifetime", &[1.0, r, mm, radius])[0] / gyr
    };
    let earth = life(1.0, M_EARTH, 0.325, 6.371e6);
    let mars = life(1.0, 0.107 * M_EARTH, 0.24, 3.3895e6);
    let moon = life(1.0, 0.0123 * M_EARTH, 0.02, 1.7374e6);
    assert!(earth > 8.0, "Earth lifetime {earth}");
    assert!((3.5..4.3).contains(&mars), "Mars lifetime {mars}");
    assert!(moon < 2.5, "Moon lifetime {moon}");
    assert!(
        life(2.0, M_EARTH, 0.325, 6.371e6) > 1.5 * earth,
        "thorium-rich"
    );
    // Al-26: about 5.3 MJ/kg at time zero, a factor e per 1.034 Myr.
    let e0 = run("sse.heat.al26_planetesimal_heating", &[1.0, 0.0]);
    close(e0[0], 5.31e6, 0.01, "Al-26 energy at CAI time");
    assert_eq!(e0[1], 0.1);
    let e3 = run("sse.heat.al26_planetesimal_heating", &[1.0, 3.0 * myr]);
    assert_eq!(e3[1], 1.0);
    let mid = run("sse.heat.al26_planetesimal_heating", &[1.0, 1.5 * myr])[1];
    assert!(mid > 0.1 && mid < 1.0, "partly dried {mid}");
}

#[test]
fn references_serialise_as_text_doi_and_link() {
    let r = sse_flow::Reference::parse("A. 2001, J 1, 2: title. doi:10.1000/xyz");
    assert_eq!(
        serde_json::to_string(&r).unwrap(),
        r#"{"text":"A. 2001, J 1, 2: title.","doi":"10.1000/xyz","url":"https://doi.org/10.1000/xyz"}"#
    );
    let back: sse_flow::Reference =
        serde_json::from_str(&serde_json::to_string(&r).unwrap()).unwrap();
    assert_eq!(back, r);
    let bare: sse_flow::Reference = serde_json::from_str(r#""a textbook""#).unwrap();
    assert_eq!(
        serde_json::to_string(&bare).unwrap(),
        r#"{"text":"a textbook"}"#
    );
    let ads = sse_flow::Reference::parse(
        "B 1996, Icarus 122 https://ui.adsabs.harvard.edu/abs/1996Icar..122..166G",
    );
    assert_eq!(ads.doi, None);
    assert_eq!(ads.text, "B 1996, Icarus 122");
}

// ---- nebulae (O3) ----

/// The series photon count agrees with a direct numerical integration of the
/// Planck photon spectrum (midpoint rule, 200,000 steps in frequency).
#[test]
fn ionising_photons_series_matches_numeric() {
    use crate::nebula::*;
    let (h, c, k) = (6.626_070_15e-34, 299_792_458.0, 1.380_649e-23);
    for t in [20_000.0, 40_000.0, 100_000.0] {
        let r = 7e9;
        let nu0 = E_ION_H / h;
        let nu1 = nu0 * 40.0;
        let n = 200_000;
        let dnu = (nu1 - nu0) / n as f64;
        let mut s = 0.0;
        for i in 0..n {
            let nu = nu0 + (i as f64 + 0.5) * dnu;
            s += 2.0 * nu * nu / (c * c) / sse_core::math::expm1(h * nu / (k * t));
        }
        let numeric = 4.0 * sse_core::math::PI * r * r * sse_core::math::PI * s * dnu;
        let series = ionising_photons(t, r, E_ION_H);
        close(series, numeric, 1e-4, "ionising photons");
    }
}

/// A star giving 10⁴⁹ ionising photons a second in gas of 100 per cm³ ionises
/// a sphere about 3.1 pc across in radius (Osterbrock and Ferland 2006, sec 2.3).
#[test]
fn stromgren_sphere() {
    let r = run("sse.nebula.stromgren_radius", &[1e49, 1e8])[0];
    close(r / 3.0857e16, 3.15, 0.02, "Strömgren radius");
    // Case B: L(Hα) = 1.37e-12 erg/s per ionising photon (Osterbrock and Ferland).
    let l = run("sse.nebula.balmer_alpha_power", &[1e49])[0];
    close(l, 1.37e-19 * 1e49, 0.01, "Hα");
}

/// Supernova remnant phases: R grows as t^(2/5) in the Sedov phase, and
/// Cioffi et al.'s snowplough starts at R_PDS = 14 pc for E = 10⁵¹ erg, n = 1.
#[test]
fn remnant_phases() {
    use crate::nebula::remnant_radius;
    let yr = sse_core::quantity::consts::JULIAN_YEAR;
    let pc = 3.085_677_581e16;
    let (e, mej, n) = (1e44, 1.4 * 1.988_41e30, 1e6);
    let (r1, p1) = remnant_radius(e, mej, n, 100.0 * yr);
    assert_eq!(p1, "free expansion");
    let (ra, pa) = remnant_radius(e, mej, n, 2000.0 * yr);
    let (rb, _) = remnant_radius(e, mej, n, 4000.0 * yr);
    assert_eq!(pa, "Sedov-Taylor");
    close(rb / ra, sse_core::math::pow(2.0, 0.4), 1e-9, "t^2/5");
    let (rp, pp) = remnant_radius(e, mej, n, 1.33e4 * yr * 1.000001);
    assert_eq!(pp, "pressure-driven snowplough");
    close(rp / pc, 14.0, 0.01, "R_PDS");
    assert!(r1 < ra);
}

/// O10: the Sun's activity from its mass and spin (Wright et al. 2011).
#[test]
fn sun_activity_from_rossby() {
    use crate::activity::*;
    use sse_core::quantity::consts::*;
    let m_sun = GM_SUN / G;
    let tau = convective_turnover(m_sun) / DAY;
    assert!((tau - 14.45).abs() < 0.05, "τ☉ = {tau} d");
    let ro = 26.09 / tau;
    let rx = xray_fraction(ro, -3.13, 0.13, -2.70);
    let log_rx = sse_core::math::ln(rx) / sse_core::math::ln(10.0);
    assert!((log_rx - SUN_LOG_RX).abs() < 0.1, "log Rx = {log_rx}");
    // A fast spinner is saturated at 10^-3.13 whatever its spin.
    assert_eq!(
        xray_fraction(0.05, -3.13, 0.13, -2.70),
        xray_fraction(0.1, -3.13, 0.13, -2.70)
    );
    // Superflares above 1e34 erg on a Sun-like star: once a century or so.
    let lx = 5.754_399e-7 * L_SUN;
    let per_year = flare_rate_above(lx, E_REF_J, FLARE_K, 2.0) * JULIAN_YEAR;
    assert!((per_year - 8.63e-3).abs() < 1e-6, "{per_year}");
    // Ten times the energy, a tenth as often.
    let r1 = flare_rate_above(lx, 1e25, FLARE_K, 2.0);
    let r2 = flare_rate_above(lx, 1e26, FLARE_K, 2.0);
    assert!((r1 / r2 - 10.0).abs() < 1e-9);
    // Cycle: the inactive branch gives the Sun 6.1 years (Mittag et al. 2023 say so too).
    let cyc = cycle_period(26.09 * DAY, 1.488, 1.324) / JULIAN_YEAR;
    assert!((cyc - 6.1).abs() < 0.3, "{cyc} yr");
}

/// Air optics: Earth's sea-level Rayleigh scattering at 550 nm against
/// Bodhaine et al. 1999 (4.5105e-31 m², τ = 0.0971) and Bucholtz 1995.
#[test]
fn earth_rayleigh_scattering() {
    use crate::air::*;
    let n1 = air_refractivity(550e-9);
    assert!((n1 * 1e8 - 27_784.0).abs() < 30.0, "{n1}");
    // Dry air King factor at 550 nm with 360 ppm CO2 (Bodhaine table 3).
    let sigma = rayleigh_cross_section(
        550e-9,
        n1 * (1.0 + 0.54 * (0.00036 - 0.0003)),
        N_S_288,
        1.04882,
    );
    assert!((sigma / 4.5105e-31 - 1.0).abs() < 2e-3, "{sigma}");
    let beta = N_S_288 * sigma;
    assert!((beta / 11.49e-6 - 1.0).abs() < 0.01, "{beta}");
    // Optical depth: g at the column's mass-weighted height, 45°N (Bodhaine eqs 10-11 give about 9.789).
    let m_air = (15.0556 * 0.00036 + 28.9595) * 1e-3 / 6.022_140_76e23;
    let tau = column_optical_depth(101_325.0, sigma, m_air, 9.789);
    assert!((tau / 0.097_069 - 1.0).abs() < 0.005, "{tau}");
    let h = scale_height(288.15, 28.9595 * AMU, 9.80665);
    assert!((h - 8434.0).abs() < 10.0, "{h}");
}

// ---- chem (E0) -------------------------------------------------------------

#[test]
fn gibbs_blocks_reproduce_tabulated_values() {
    // At 298.15 K the temperature correction changes nothing.
    let g = run(
        "sse.chem.standard_gibbs_at_temperature",
        &[-131.0e3, -253.0e3, 298.15],
    )[0];
    close(g, -131.0e3, 1e-12, "no shift at 298.15 K");
    // Hydrogenotrophic methanogenesis (4 H2 + CO2 -> CH4 + 2 H2O(l), gases):
    // ΔH° = -252.9 kJ, ΔG° = -130.8 kJ. Exergonic reactions with a large
    // entropy loss give up less as it warms.
    let hot = run(
        "sse.chem.standard_gibbs_at_temperature",
        &[-130.8e3, -252.9e3, 358.15],
    )[0];
    close(
        hot,
        -252.9e3 - 358.15 * (-252.9e3 + 130.8e3) / 298.15,
        1e-12,
        "formula",
    );
    assert!(hot > -130.8e3);
    // ΔG = ΔG° + RT ln Q. Ten times the product activity costs RT ln 10.
    let r = run("sse.chem.reaction_gibbs", &[-100.0e3, 298.15, m::ln(10.0)])[0];
    close(
        r,
        -100.0e3 + crate::chem::R_GAS * 298.15 * m::ln(10.0),
        1e-12,
        "RT ln Q",
    );
    // pH 7 as the H+ activity: 7 RT ln 10 at 298.15 K is 39.95 kJ (Thauer's
    // table uses -39.87 for ΔG°f of H+ at pH 7).
    let lnq = crate::chem::ln_quotient(&[(1.0, 1.0e-7)]).unwrap();
    let h = run("sse.chem.reaction_gibbs", &[0.0, 298.15, lnq])[0];
    close(h, -39.95e3, 0.002, "pH 7 hydrogen ion");
    assert!(crate::chem::ln_quotient(&[(1.0, 0.0)]).is_err());
}

#[test]
fn stormer_cutoff_matches_earth() {
    use crate::shield::*;
    // Earth's dipole, 8.0e22 A m² (IGRF 2020), at the ground.
    let eq = cutoff_rigidity(8.0e22, 6.371e6, 0.0) * 1e-9;
    assert!((eq - 14.9).abs() < 0.5, "equator {eq} GV");
    assert!(cutoff_rigidity(8.0e22, 6.371e6, m::PI / 2.0) * 1e-9 < 1e-6);
    // Falls as 1/r².
    let up = cutoff_rigidity(8.0e22, 2.0 * 6.371e6, 0.0) * 1e-9;
    assert!((up * 4.0 - eq).abs() < 1e-9);
    assert!((space_share(0.0, 2.0, 1.7) - 1.0).abs() < 1e-15);
    assert!((ground_share(0.0, 10.0, 1.0) - 1.0).abs() < 1e-15);
    let g = ground_share(15e9, 10.0, 1.0);
    assert!(g > 0.4 && g < 0.6, "ground share at 15 GV {g}");
}

#[test]
fn dust_reddens_as_ccm89() {
    use crate::extinction::ccm_ratio;
    // V is the reference; B about 1 + 1/R_V (E(B−V) = A_V / R_V); K about a tenth.
    close(ccm_ratio(0.549e-6, 3.1), 1.0, 0.01, "A_V/A_V");
    let b = ccm_ratio(0.44e-6, 3.1);
    assert!((b - 1.32).abs() < 0.03, "A_B/A_V {b}");
    let k = ccm_ratio(2.2e-6, 3.1);
    assert!((k - 0.114).abs() < 0.01, "A_K/A_V {k}");
    // The 217.5 nm bump stands above its sides.
    assert!(ccm_ratio(217.5e-9, 3.1) > ccm_ratio(170e-9, 3.1) * 0.95);
    assert!(ccm_ratio(217.5e-9, 3.1) > ccm_ratio(260e-9, 3.1));
}

#[test]
fn aurora_footprints_and_hill_distance() {
    use crate::aurora::*;
    let deg = 180.0 / m::PI;
    // Earth's nose near 10 radii lands near 72°; Io's line (5.9 R_J) near 66°.
    let e = footprint_latitude(10.0, 1.0) * deg;
    assert!((e - 71.6).abs() < 0.1, "{e}");
    let io = footprint_latitude(5.9, 1.0) * deg;
    assert!((io - 65.7).abs() < 0.2, "{io}");
    assert_eq!(footprint_latitude(0.5, 1.0), 0.0);
    // Jupiter, Σ* 0.14 S, Ṁ 1000 kg/s: about 30 R_J (Nichols 2011, after Hill 2001).
    let rj = 7.1373e7;
    let h = hill_distance(0.14, 4.264e-4, rj, 1000.0) / rj;
    assert!((h - 30.0).abs() < 1.5, "Hill distance {h}");
    assert!(hill_distance(0.14, 4.264e-4, rj, 0.0).is_infinite());
}

/// Mauk and Fox 2010: the Earth L = 5 storm spectrum (C 2.38e6, index 0.978,
/// E₀ 1748 keV, 7.04; their Table 1) sits at about 0.6 of the limit for a
/// storm field of 149 nT and D = L (their Figure 5); the limit's cap for the
/// E⁻¹ shape falls in proportion to B / (D R_p) in a dense plasma; Neptune's
/// spectrum (L 7.4) is about 30 times below the limit at 1 MeV.
#[test]
fn belts_reach_the_kennel_petschek_limit() {
    use crate::belts::*;
    // Test values typed here (tests are outside the sheet audit).
    let q = Electron {
        mass_kg: 9.109_383_701_5e-31,
        charge_c: 1.602_176_634e-19,
        eps0: 8.854_187_812_8e-12,
    };
    let re = 6.371e6;
    let earth = Shape {
        index: 0.978,
        break_kev: 1748.0,
        break_index: 7.036,
        anisotropy: 0.3,
    };
    // Sheeley et al. 2001 trough density near 10 per cm³ at L = 5.
    let b = 149e-9;
    let wpe = m::sqrt(10e6 * q.charge_c * q.charge_c / (q.eps0 * q.mass_kg))
        / (q.charge_c * b / q.mass_kg);
    let ratio = (2.38e6 / kp_normalisation(b, 5.0 * re, 200.0, wpe, 3.0, &earth, &q)).max(0.0);
    assert!((0.4..=0.8).contains(&ratio), "Earth L5 Cm/CK {ratio}");
    // Dense-limit cap for the E⁻¹ shape scales as B / D.
    let lim = Shape {
        index: 1.0,
        break_kev: 1500.0,
        break_index: 6.0,
        anisotropy: 0.3,
    };
    let c1 = kp_cap(b, 5.0 * re, 300.0, 3.0, &lim, &q);
    let c2 = kp_cap(4.0 * b, 2.0 * re, 300.0, 3.0, &lim, &q);
    close(c2 / c1, 10.0, 1e-6, "B/D scaling");
    close(c1 / 3.83e6, 1.0, 0.1, "Earth storm cap");
    // Neptune: near the limit at 0.1 MeV, far under at 1 MeV.
    let rn = 2.4764e7;
    let nep = Shape {
        index: 1.56,
        break_kev: 625.0,
        break_index: 2.745,
        anisotropy: 0.3,
    };
    let bn = 1.42e-5 / m::pow(7.4, 3.0);
    let at = |e: f64| 5.49e6 / kp_normalisation(bn, 7.4 * rn, e, 300.0, 3.0, &nep, &q);
    assert!(at(100.0) > 0.5, "Neptune 0.1 MeV {}", at(100.0));
    assert!(at(1000.0) < 0.1, "Neptune 1 MeV {}", at(1000.0));
    // Katz-Penfold: 1 MeV goes about 0.41 g/cm²; 1 g/cm² needs about 2 MeV.
    close(electron_range(1.0), 4.12, 1e-9, "range 1 MeV");
    close(
        penetration_energy(electron_range(1.7)),
        1.7,
        1e-6,
        "inverse",
    );
}

/// Bremsstrahlung (MF7 part 2b-i): the radiation yield grows with energy and
/// atomic number, and the X-ray dose behind a shield falls far more slowly
/// than the electrons' (they stop; the X-rays get through).
#[test]
fn bremsstrahlung_behind_shields() {
    use crate::belts::*;
    // Koch and Motz: about 0.8 per cent at 1 MeV in aluminium.
    let y = radiation_yield(1.0, 13.0, 6e-4);
    assert!((0.006..0.009).contains(&y), "{y}");
    assert!(radiation_yield(10.0, 13.0, 6e-4) > 5.0 * y);
    assert!(radiation_yield(1.0, 82.0, 6e-4) > 5.0 * y);
    // A test table (values typed here): flat coefficients.
    let ph = Photons {
        energy_mev: vec![0.01, 100.0],
        shield_mu_rho_cm2_g: vec![0.05, 0.05],
        tissue_mu_en_rho_cm2_g: vec![0.03, 0.03],
    };
    let b = Brems {
        z: 13.0,
        k_per_mev: 6e-4,
        inward_share: 0.5,
        photons: &ph,
        j_per_mev: 1.602e-13,
    };
    // A belt-like spectrum, E^-1 with an e-fold at 1 MeV.
    let j = |e: f64| 1e6 / e * sse_core::math::exp(-e / 1e3);
    let d10 = bremsstrahlung_dose(100.0, &j, &b);
    let d40 = bremsstrahlung_dose(400.0, &j, &b);
    assert!(d10 > d40 && d40 > 0.0 && d10 / d40 < 10.0, "{d10} {d40}");
    assert_eq!(bremsstrahlung_dose(0.0, &j, &b), 0.0);
}

// ND3b: blocks whose maths moved into formulas.

fn formula_atom(id: &str) -> Box<dyn sse_flow::Atom> {
    crate::formulas::atoms()
        .into_iter()
        .find(|a| a.decl().id == id)
        .unwrap_or_else(|| panic!("{id} is not a formula block"))
}

/// The moved blocks' Rust maths as it was before the move, kept here so the
/// formulas are checked against it bit for bit.
fn old(id: &str, i: &[f64], p: &[f64]) -> Result<Vec<f64>, sse_flow::AtomError> {
    use crate::common::positive;
    use sse_core::math as m;
    let g = sse_core::constants::g();
    Ok(match id {
        "sse.planet.bulk_density" => {
            let r = positive(i[1], "radius")?;
            vec![3.0 * positive(i[0], "mass")? / (4.0 * m::PI * r * r * r)]
        }
        "sse.planet.surface_gravity" => {
            let r = positive(i[1], "radius")?;
            vec![g * positive(i[0], "mass")? / (r * r)]
        }
        "sse.planet.escape_velocity" => {
            let r = positive(i[1], "radius")?;
            vec![m::sqrt(2.0 * g * positive(i[0], "mass")? / r)]
        }
        "sse.orbit.total_mass" => vec![positive(i[0], "host mass")? + positive(i[1], "body mass")?],
        "sse.orbit.period" => {
            let a = positive(i[0], "semi-major axis")?;
            let mt = positive(i[1], "total mass")?;
            vec![m::TAU * m::sqrt(a * a * a / (g * mt))]
        }
        "sse.star.effective_temperature" => {
            let l = positive(i[0], "luminosity")?;
            let r = positive(i[1], "radius")?;
            vec![m::pow(
                l / (4.0 * m::PI * r * r * sse_core::constants::sigma_sb()),
                0.25,
            )]
        }
        // Part 2: cube roots, own refusal words and settings as letters.
        "sse.orbit.semi_major_axis_from_period" => {
            let p = positive(i[0], "period")?;
            let mt = positive(i[1], "total mass")?;
            vec![m::cbrt(g * mt * p * p / (4.0 * m::PI * m::PI))]
        }
        "sse.orbit.apsides" => {
            if !(0.0..1.0).contains(&i[1]) {
                return Err(sse_flow::AtomError(format!(
                    "eccentricity {} is not an ellipse",
                    i[1]
                )));
            }
            vec![i[0] * (1.0 - i[1]), i[0] * (1.0 + i[1])]
        }
        "sse.orbit.hill_radius" => {
            let a = positive(i[0], "semi-major axis")?;
            let mb = positive(i[2], "body mass")?;
            let mh = positive(i[3], "host mass")?;
            vec![a * (1.0 - i[1]) * m::cbrt(mb / (3.0 * mh))]
        }
        "sse.orbit.roche_limit" => {
            let r = positive(i[0], "primary radius")?;
            let rp = positive(i[1], "primary density")?;
            let rs = positive(i[2], "satellite density")?;
            vec![p[0] * r * m::cbrt(rp / rs)]
        }
        "sse.star.ms_lifetime" => {
            let m_sun = sse_core::quantity::consts::GM_SUN / sse_core::quantity::consts::G;
            let ms = positive(i[0], "mass")? / m_sun;
            let ls = positive(i[1], "luminosity")? / sse_core::quantity::consts::L_SUN;
            vec![p[0] * ms / ls]
        }
        _ => unreachable!(),
    })
}

#[test]
fn formula_blocks_give_the_old_bits_and_refusals() {
    let ids = [
        "sse.planet.bulk_density",
        "sse.planet.surface_gravity",
        "sse.planet.escape_velocity",
        "sse.orbit.total_mass",
        "sse.orbit.period",
        "sse.star.effective_temperature",
        "sse.orbit.semi_major_axis_from_period",
        "sse.orbit.apsides",
        "sse.orbit.hill_radius",
        "sse.orbit.roche_limit",
        "sse.star.ms_lifetime",
    ];
    // Values over many decades, and the refusals at zero and below.
    let mut xs = vec![0.0, -1.0, -0.0];
    for e in -30..=40 {
        for k in [1.0, 1.7, 3.3, 7.9] {
            xs.push(k * sse_core::math::pow(10.0, f64::from(e)));
        }
    }
    // Eccentricities, and just outside an ellipse.
    xs.extend([0.25, 0.5, 0.9, 0.99, 1.0, 1.5]);
    let mut n = 0;
    for id in ids {
        let a = formula_atom(id);
        let d = a.decl();
        let ins = d.inputs.len();
        // Each setting at its default and at both ends of its range.
        let mut settings: Vec<Vec<f64>> = vec![d.params.iter().map(|p| p.default).collect()];
        for end in [0, 1] {
            settings.push(
                d.params
                    .iter()
                    .map(|p| {
                        let r = p.range.unwrap();
                        if end == 0 {
                            r.0
                        } else {
                            r.1
                        }
                    })
                    .collect(),
            );
        }
        for (j, x) in xs.iter().enumerate() {
            for p in &settings {
                let i: Vec<f64> = (0..ins)
                    .map(|k| {
                        if k == 0 {
                            *x
                        } else {
                            xs[(j * (7 + 4 * k) + 3 * k) % xs.len()]
                        }
                    })
                    .collect();
                let new = a.eval(&i, p);
                match (old(id, &i, p), new) {
                    (Ok(o), Ok(f)) => {
                        assert_eq!(o.len(), f.len());
                        for (p, q) in o.iter().zip(&f) {
                            assert_eq!(p.to_bits(), q.to_bits(), "{id} at {i:?}: {p} vs {q}");
                        }
                        n += 1;
                    }
                    (Err(o), Err(f))
                        if o.0.contains("must be positive") || o.0.contains("not an ellipse") =>
                    {
                        assert_eq!(o.0, f.0, "{id} at {i:?}");
                    }
                    // The old maths gave an infinity or NaN where a formula
                    // refuses a number too large to hold.
                    (Ok(o), Err(f)) => assert!(
                        o.iter().any(|v| !v.is_finite()) && f.0.contains("too large"),
                        "{id} at {i:?}: {o:?} vs {f:?}"
                    ),
                    (o, f) => panic!("{id} at {i:?}: {o:?} vs {f:?}"),
                }
            }
        }
    }
    assert!(n > 3000, "{n}");
}

#[test]
fn formula_blocks_follow_a_changed_constant() {
    let a = formula_atom("sse.planet.surface_gravity");
    let before = a.eval(&[6.0e24, 6.4e6], &[]).unwrap()[0];
    let mut c = sse_core::constants::Constants::REAL;
    c.g *= 2.0;
    let after = sse_core::constants::with(c, || a.eval(&[6.0e24, 6.4e6], &[]).unwrap()[0]);
    assert_eq!(after, 2.0 * before);
}

/// CM13 part 2: the link rate reads k_B through the handle, so a doubled
/// Boltzmann constant (twice the noise per kelvin) halves the rate.
#[test]
fn the_link_rate_follows_a_changed_boltzmann_constant() {
    let a = formula_atom("sse.ship.link_rate");
    let ins = [20.0, 5.8e4, 3.0e6, 0.0357, 1.0e12, 25.0, 2.5];
    let before = a.eval(&ins, &[]).unwrap()[0];
    let mut c = sse_core::constants::Constants::REAL;
    c.k_b *= 2.0;
    let after = sse_core::constants::with(c, || a.eval(&ins, &[]).unwrap()[0]);
    assert!(
        before > 1.0e4,
        "a 20 W probe at 1e12 m sends {before} bit/s"
    );
    assert_eq!(after, before / 2.0);
}

#[test]
fn the_rust_blocks_list_matches_the_registry() {
    let moved: Vec<String> = crate::formulas::loaded()
        .iter()
        .map(|a| sse_flow::Atom::decl(a).id.clone())
        .collect();
    let mut rust: Vec<String> = crate::atoms()
        .iter()
        .map(|a| a.decl().id.clone())
        .filter(|i| !moved.contains(i))
        .collect();
    rust.sort();
    let text = include_str!("../../../handover/packages/ND3b-rust-atoms.txt");
    let mut listed: Vec<String> = text
        .lines()
        .filter(|l| !l.starts_with('#') && !l.trim().is_empty())
        .map(|l| {
            let (id, why) = l.split_once('\t').expect("id, a tab, then why");
            assert!(!why.trim().is_empty(), "{id} says why it stays");
            id.to_string()
        })
        .collect();
    listed.sort();
    assert_eq!(
        rust, listed,
        "a block moved into a formula takes its line out of ND3b-rust-atoms.txt; a new Rust block adds one with why"
    );
}

#[test]
fn formula_blocks_have_worded_names_and_keep_their_faces() {
    for a in crate::formulas::loaded() {
        let d = sse_flow::Atom::decl(a);
        assert!(
            a.name().split_whitespace().count() >= 3,
            "{}: {}",
            d.id,
            a.name()
        );
        assert!(
            !d.references.is_empty() && !d.explain.curious.is_empty(),
            "{}",
            d.id
        );
        assert!(!d.explain.formula.is_empty(), "{}", d.id);
    }
}

#[test]
fn a_worked_example_reads_constants_from_the_sheet() {
    let text = include_str!("../../../packs/base-physics/formulas/planet.json");
    let file: serde_json::Value = serde_json::from_str(text).unwrap();
    let mut entry = file["atoms"][1].clone();
    assert_eq!(entry["id"], "sse.planet.surface_gravity");
    sse_formula::FormulaAtom::from_json(&entry).unwrap();
    entry["equation"]["examples"][0]["G"] = serde_json::json!(6.6743e-11);
    let e = sse_formula::FormulaAtom::from_json(&entry).unwrap_err();
    assert!(e.contains("reads constants/G from the sheet"), "{e}");
}

/// Every block moved by the generator (ND3b part 3 onward) against its Rust
/// as it was (`src/was/`): the same face (ports, kinds, units, valid ranges,
/// settings, references, words) and the same bits and refusals over many
/// decades of input, at each setting's default and both ends of its range.
#[test]
fn moved_blocks_keep_their_faces_and_bits() {
    let formulas = crate::formulas::atoms();
    let was = crate::was::atoms();
    let mut xs = vec![
        0.0, -1.0, -0.0, 0.25, 0.5, 0.9, 0.99, 1.0, 1.5, -0.3, 2.0, 3.0,
    ];
    for e in -30..=40 {
        for k in [1.0, 1.7, 3.3, 7.9] {
            xs.push(k * sse_core::math::pow(10.0, f64::from(e)));
        }
    }
    let mut n = 0;
    for old in &was {
        let d = old.decl();
        let new = formulas
            .iter()
            .find(|a| a.decl().id == d.id)
            .unwrap_or_else(|| panic!("{} has no formula", d.id));
        assert_eq!(new.decl(), d, "{} keeps its face", d.id);
        let mut settings: Vec<Vec<f64>> = vec![d.params.iter().map(|p| p.default).collect()];
        for end in [0, 1] {
            settings.push(
                d.params
                    .iter()
                    .map(|p| {
                        let r = p.range.unwrap();
                        if end == 0 {
                            r.0
                        } else {
                            r.1
                        }
                    })
                    .collect(),
            );
        }
        let ins = d.inputs.len();
        for (j, x) in xs.iter().enumerate() {
            for p in &settings {
                let i: Vec<f64> = (0..ins)
                    .map(|k| {
                        if k == 0 {
                            *x
                        } else {
                            xs[(j * (7 + 4 * k) + 3 * k) % xs.len()]
                        }
                    })
                    .collect();
                match (old.eval(&i, p), new.eval(&i, p)) {
                    (Ok(o), Ok(f)) => {
                        assert_eq!(o.len(), f.len());
                        for (a, b) in o.iter().zip(&f) {
                            assert!(
                                a.to_bits() == b.to_bits() || (a.is_nan() && b.is_nan()),
                                "{} at {i:?} {p:?}: {a} vs {b}",
                                d.id
                            );
                        }
                        n += 1;
                    }
                    (Err(o), Err(f)) => assert_eq!(o.0, f.0, "{} at {i:?} {p:?}", d.id),
                    // Where the old maths gave an infinity or NaN, a formula
                    // refuses in words that name the step.
                    (Ok(o), Err(_)) if o.iter().any(|v| !v.is_finite()) => {}
                    (o, f) => panic!("{} at {i:?} {p:?}: {o:?} vs {f:?}", d.id),
                }
            }
        }
    }
    assert!(n >= was.len() * 50, "{n}");
}

#[test]
fn tides_of_the_moon_and_the_sun_on_earth() {
    let (me, re) = (5.972e24, 6.371e6);
    let moon = run("sse.tides.ocean_range", &[7.342e22, me, re, 3.844e8])[0];
    let sun = run("sse.tides.ocean_range", &[1.989e30, me, re, 1.496e11])[0];
    // The textbook equilibrium figures: about 0.53 m from the Moon, and the
    // Sun's about 0.46 of it.
    close(moon, 0.535, 0.01, "lunar range");
    close(sun / moon, 0.46, 0.02, "solar to lunar");
    let t = run("sse.tides.period", &[86_164.1, 27.321_66 * 86_400.0])[0];
    close(t / 3600.0, 12.42, 0.001, "lunar semidiurnal period");
    // A backward spin turns under the moon faster: a shorter tide.
    let back = run("sse.tides.period", &[-86_164.1, 27.321_66 * 86_400.0])[0];
    assert!(back < t);
}

#[test]
fn waves_grow_with_wind_and_fetch_until_the_sea_is_full_grown() {
    let h = |u: f64, x: f64, g: f64| run("sse.ocean.wind_waves", &[u, x, g]);
    let a = h(10.0, 1.0e4, 9.81);
    let b = h(10.0, 1.0e5, 9.81);
    let c = h(10.0, 1.0e7, 9.81);
    let d = h(10.0, 1.0e8, 9.81);
    assert!(a[0] < b[0] && b[0] < c[0] && a[1] < b[1]);
    // Fully grown: more fetch adds nothing.
    assert_eq!(c[0], d[0]);
    assert!(h(20.0, 1.0e5, 9.81)[0] > b[0]);
    // A 10 m/s wind over 300 km of sea: near 2.75 m and 7 s (the manual's
    // relations); full grown about 3.1 m, as observed for that wind.
    let e = h(10.0, 3.0e5, 9.81);
    close(e[0], 2.75, 0.01, "wave height");
    close(c[0], 3.12, 0.02, "full grown height");
    // Weaker gravity, taller waves for the same wind.
    assert!(h(2.0, 1.0e5, 1.352)[0] > h(2.0, 1.0e5, 9.81)[0]);
}

#[test]
fn the_glint_spreads_with_wind_and_brightens_with_the_index() {
    let g = |u: f64, n: f64| run("sse.light.sea_glint", &[u, n]);
    // Cox and Munk's clean sea at 10 m/s: 0.0542; calm water 0.003.
    close(g(10.0, 1.333)[0], 0.0542, 1e-9, "slope variance at 10 m/s");
    close(g(0.0, 1.333)[0], 0.003, 1e-12, "calm sea");
    // Water reflects 2.0% straight back; Titan's methane (n about 1.27) 1.4%.
    close(
        g(7.0, 1.333)[1],
        0.020_373,
        1e-4,
        "water's normal reflectance",
    );
    assert!(g(7.0, 1.27)[1] < g(7.0, 1.333)[1]);
}

/// MP6: the composition blocks (Equations) and the sums the composition
/// chain runs (`matter`) agree over a spread of inputs.
#[test]
fn composition_blocks_match_the_chain() {
    use crate::matter as mt;
    let near = |a: f64, b: f64, what: &str| {
        assert!(
            (a - b).abs() <= 1e-12 * a.abs().max(b.abs()).max(1e-30),
            "{what}: {a} against {b}"
        );
    };
    for (ci, r, o, w) in [
        (0.0965, 1.0, 0.6583, 0.43),
        (1e-7, 3.0, 0.0, 0.25),
        (0.0, 1.0, 0.0, 0.4),
        (0.1, 1.0, 0.5, 0.0),
    ] {
        let got = run("sse.composition.share_in_material", &[ci, r, o, w]);
        let (s, b) = mt::share_in_material(ci, r, o, w);
        near(got[0], s, "share");
        near(got[1], b, "bound oxygen");
    }
    let g = sse_core::constants::g();
    for (mass, radius, e) in [
        (5.9722e24, 6.371e6, 0.0),
        (9.38e20, 4.697e5, 0.0),
        (2.59e20, 2.63e5, 1.0e6),
    ] {
        let got = run("sse.composition.separation_strength", &[mass, radius, e]);
        near(
            got[0],
            mt::separation_strength(g, mass, radius, e, 1.8e6),
            "separation",
        );
    }
    for (d, f, s) in [
        (13.66, 0.325, 1.0),
        (500.0, 0.2, 0.4),
        (0.0, 0.3, 1.0),
        (2.0, 0.0, 1.0),
    ] {
        near(
            run("sse.composition.core_share", &[d, f, s])[0],
            mt::core_share(d, f, s),
            "core",
        );
        near(
            run("sse.composition.crust_share", &[d, f / 10.0, s])[0],
            mt::crust_share(d, f / 10.0, s),
            "crust",
        );
    }
    for (radius, sil) in [(6.371e6, 4.03e24), (1.7374e6, 7.0e22), (2.0e5, 1.0e19)] {
        near(
            run("sse.composition.crust_fraction", &[radius, sil])[0],
            mt::crust_fraction(
                radius,
                sil,
                2.0e4,
                sse_core::sheet::mixture_density::CRUST_ROCK,
            ),
            "crust fraction",
        );
    }
    for (a, c, k) in [
        (1.0e13, 6.0e-5, 160.0),
        (3.0e9, 1.5e-9, 3800.0),
        (1.0e6, 0.5, 10.0),
    ] {
        let got = run("sse.composition.deposit_amount", &[a, c, k]);
        let want = mt::deposit_amount(
            a,
            3300.0,
            sse_core::sheet::mixture_density::CRUST_ROCK,
            4.5e-5,
            c,
            k,
        );
        near(got[0], want.0, "deposit mass");
        near(got[1], want.1, "deposit grade");
    }
    for (z, mass) in [(0.014, 1.9e27), (0.02, 1.0e26), (0.014, 1.0e25)] {
        near(
            run("sse.composition.heavy_element_share", &[z, mass])[0],
            mt::heavy_element_share(
                z,
                mass,
                sse_core::sheet::M_JUP,
                mt::GIANT_METAL_COEFFICIENT,
                mt::GIANT_METAL_SLOPE,
            ),
            "heavy share",
        );
    }
}

#[test]
fn redox_blocks_match_the_stage_8_sums() {
    use crate::redox as rx;
    use sse_core::sheet::{atomic_weight as aw, molar_mass as mm};
    let near = |a: f64, b: f64, what: &str| {
        assert!(
            (a - b).abs() <= 1e-12 * a.abs().max(b.abs()).max(1e-30),
            "{what}: {a} against {b}"
        );
    };
    let (fe, c) = (aw::FE / 1000.0, aw::C / 1000.0);
    for (cmf, carbon) in [
        (0.325, 0.0),
        (0.24, 0.0),
        (0.7, 0.0),
        (0.3, 0.02),
        (0.0, 0.0),
    ] {
        let got = run("sse.interior.redox_iw", &[cmf, carbon, fe, mm::FEO, c]);
        let (d, core) = rx::redox_iw(cmf, carbon, fe, mm::FEO, c, &rx::REDOX);
        near(got[0], d, "redox");
        near(got[1], core, "redox at core formation");
    }
    // Earth: about IW-2.2 at core formation (Wade and Wood 2005), IW+0.5 at
    // its magma ocean's surface (Sossi et al. 2020); Mars more oxidised; a
    // Mercury-like core leaves a reduced mantle; carbon reduces.
    let (earth, earth_core) = rx::redox_iw(0.325, 0.0, fe, mm::FEO, c, &rx::REDOX);
    assert!((-2.6..-1.8).contains(&earth_core), "{earth_core}");
    assert!((0.0..1.0).contains(&earth), "{earth}");
    let mars = rx::redox_iw(0.24, 0.0, fe, mm::FEO, c, &rx::REDOX).0;
    let mercury = rx::redox_iw(0.7, 0.0, fe, mm::FEO, c, &rx::REDOX).0;
    let sooty = rx::redox_iw(0.325, 0.01, fe, mm::FEO, c, &rx::REDOX).0;
    assert!(
        mars > earth && mercury < earth - 3.0 && sooty < earth,
        "{mars} {mercury} {sooty}"
    );
    for (d, h, cc, p) in [
        (0.5, 1.0, 0.3, 1.0e7),
        (-2.0, 1.0, 0.3, 1.0e7),
        (3.0, 1.0, 0.0, 1.0e5),
        (-4.0, 5.0e20, 1.0e20, 3.0e8),
        (0.0, 0.0, 0.0, 0.0),
    ] {
        let got = run("sse.atmosphere.outgassed_mix", &[d, h, cc, p]);
        let want = rx::outgassed_mix(d, h, cc, p, &rx::OUTGAS);
        for k in 0..5 {
            near(got[k], want[k], "outgassed mix");
        }
        let sum: f64 = want.iter().sum();
        assert!(
            sum == 0.0 || (sum - 1.0).abs() < 1e-12,
            "fractions sum to {sum}"
        );
    }
    // Oxidised air is water and CO2; reduced air is hydrogen and CO.
    let ox = rx::outgassed_mix(3.0, 1.0, 1.0, 1.0e7, &rx::OUTGAS);
    let red = rx::outgassed_mix(-3.0, 1.0, 1.0, 1.0e7, &rx::OUTGAS);
    assert!(ox[1] > ox[0] && ox[3] > ox[2], "{ox:?}");
    assert!(red[0] > red[1] && red[2] > red[3], "{red:?}");
}

/// K-e: the Mie series against Bohren and Huffman's own worked example
/// (n = 1.55, x = 5.213: Q_ext = Q_sca = 3.1054), the Rayleigh limit for
/// small spheres, the extinction paradox (Q_ext → 2) for big ones, a phase
/// function whose mean over the sphere is 1 and whose mean cosine is g, and
/// water's primary bow showing as a bump near 138° in a big drop.
#[test]
fn mie_scattering_matches_the_textbook() {
    use crate::mie::mie;
    let o = mie(5.213, 1.55, 0.0, &[]).unwrap();
    assert!((o.q_ext - 3.1054).abs() < 2e-3, "{}", o.q_ext);
    assert!((o.q_sca - o.q_ext).abs() < 1e-9);
    let x = 0.01;
    let s = mie(x, 1.5, 0.0, &[]).unwrap();
    let r = (1.5f64 * 1.5 - 1.0) / (1.5 * 1.5 + 2.0);
    let rayleigh = 8.0 / 3.0 * x * x * x * x * r * r;
    assert!(
        (s.q_sca / rayleigh - 1.0).abs() < 1e-3,
        "{} {rayleigh}",
        s.q_sca
    );
    assert!(s.g.abs() < 1e-3);
    let big = mie(2000.0, 1.33, 1e-4, &[]).unwrap();
    assert!((big.q_ext - 2.0).abs() < 0.02, "{}", big.q_ext);
    // Normalisation and asymmetry from a fine table.
    let n = 3601;
    let angles: Vec<f64> = (0..n).map(|i| m::PI * i as f64 / (n - 1) as f64).collect();
    let t = mie(5.213, 1.55, 0.0, &angles).unwrap();
    let (mut norm, mut mean_cos) = (0.0, 0.0);
    for i in 0..n - 1 {
        let (a, b) = (angles[i], angles[i + 1]);
        let w = 0.5 * (m::sin(a) * t.phase[i] + m::sin(b) * t.phase[i + 1]) * (b - a);
        norm += w;
        mean_cos += 0.5 * (m::cos(a) + m::cos(b)) * w;
    }
    assert!((norm / 2.0 - 1.0).abs() < 1e-3, "{norm}");
    assert!(
        (mean_cos / 2.0 - t.g).abs() < 2e-3,
        "{} {}",
        mean_cos / 2.0,
        t.g
    );
    // A 100 µm water drop in green light: the bow's bump near 138°.
    let deg: Vec<f64> = (120..=160).map(|d| d as f64 * m::PI / 180.0).collect();
    let w = mie(2.0 * m::PI * 1e-4 / 5.5e-7, 1.333, 0.0, &deg).unwrap();
    let peak = (0..deg.len())
        .max_by(|&a, &b| w.phase[a].total_cmp(&w.phase[b]))
        .unwrap()
        + 120;
    assert!((136..=141).contains(&peak), "{peak}");
    assert!(mie(3e4, 1.33, 0.0, &[]).unwrap_err().contains("geometric"));
}

#[test]
fn ledger_blocks_match_the_field_sums() {
    use crate::ledger as lg;
    let near = |a: f64, b: f64, what: &str| {
        assert!(
            (a - b).abs() <= 1e-12 * a.abs().max(b.abs()).max(1e-30),
            "{what}: {a} against {b}"
        );
    };
    let mu0 = sse_core::sheet::MU0;
    for (q, g, a, rho, cp, p, d) in [
        (0.02, 7.0, 1.25e-5, 1.1e4, 840.0, 86164.0, 2.26e6),
        (5.4, 23.0, 1.0e-5, 1500.0, 14000.0, 35730.0, 5.0e7),
        (0.0, 7.0, 1.25e-5, 1.1e4, 840.0, 86164.0, 2.26e6),
    ] {
        let got = run("sse.dynamo.flow_speed", &[q, g, a, rho, cp, p, d]);
        let (u, ro) = lg::flow_speed(q, g, a, rho, cp, p, d, lg::FLOW_C, lg::FLOW_B);
        near(got[0], u, "flow speed");
        near(got[1], ro, "local Rossby");
        let got = run("sse.dynamo.reynolds", &[u.max(1e-6), d, 1.0e6]);
        let (rm, tau) = lg::reynolds(u.max(1e-6), d, 1.0e6, mu0);
        near(got[0], rm, "Rm");
        near(got[1], tau, "decay time");
        near(
            run("sse.dynamo.onset", &[rm])[0],
            lg::onset(rm, lg::ONSET_LO, lg::ONSET_HI),
            "onset",
        );
        let got = run("sse.dynamo.rossby", &[p, d, u.max(1e-6)]);
        let (r, f) = lg::rossby(p, d, u.max(1e-6), lg::ROSSBY_SAT, lg::ROSSBY_SLOPE);
        near(got[0], r, "Rossby");
        near(got[1], f, "Rossby factor");
    }
    // Earth's core: about a millimetre a second (secular variation, 0.1 to
    // 1 mm/s), Rm in the hundreds to thousands, a dead core's field gone in
    // tens of thousands of years, and spin never limiting.
    let (u, _) = lg::flow_speed(
        0.02,
        7.0,
        1.25e-5,
        1.1e4,
        840.0,
        86164.0,
        2.26e6,
        lg::FLOW_C,
        lg::FLOW_B,
    );
    assert!((1e-4..3e-3).contains(&u), "Earth core flow {u} m/s");
    let (rm, tau) = lg::reynolds(u, 2.26e6, 1.0e6, mu0);
    assert!((100.0..1e4).contains(&rm), "Earth Rm {rm}");
    let tau_kyr = tau / sse_core::sheet::YR_JULIAN / 1e3;
    assert!((10.0..1000.0).contains(&tau_kyr), "decay {tau_kyr} kyr");
    assert_eq!(lg::onset(rm, lg::ONSET_LO, lg::ONSET_HI), 1.0);
    assert_eq!(
        lg::rossby(86164.0, 2.26e6, u, lg::ROSSBY_SAT, lg::ROSSBY_SLOPE).1,
        1.0
    );
    // Salty water: no dynamo.
    let (rm_sea, _) = lg::reynolds(0.01, 1.0e5, 5.0, mu0);
    assert_eq!(lg::onset(rm_sea, lg::ONSET_LO, lg::ONSET_HI), 0.0);
}

#[test]
fn the_dipole_ratio_constant_is_the_blocks_default() {
    let reg = registry();
    let d = reg
        .get("sse.magnetic.surface_dipole")
        .expect("block")
        .decl();
    let k = d.param("dipole_ratio").expect("param").1.default;
    assert_eq!(k, crate::magnetic::DIPOLE_RATIO);
}

/// The inertia blocks give the same C as `inertia`'s sums; Earth's comes
/// out a few per cent over its measured 0.3307 (no self-compression), the
/// coreless Moon near a uniform ball's 0.4, a giant at the polytrope's 0.261.
#[test]
fn inertia_blocks_match_their_sums() {
    use crate::inertia as ia;
    for (q, rc, r) in [
        (0.325, 3.48e6, 6.371e6),
        (0.24, 1.83e6, 3.39e6),
        (0.0, 0.0, 1.7374e6),
        (0.5, 7.0e6, 6.0e6),
    ] {
        let got = run("sse.interior.inertia_two_layer", &[q, rc, r])[0];
        assert!(
            (got - ia::two_layer(q, rc, r)).abs() < 1e-12,
            "{q} {rc} {r}: {got}"
        );
    }
    let earth = ia::two_layer(0.325, 3.48e6, 6.4e6);
    assert!(earth > 0.3307 && earth < 0.36, "{earth}");
    assert!((ia::two_layer(0.0, 0.0, 1.7e6) - 0.4).abs() < 1e-12);
    for z in [0.0, 0.1, 0.25] {
        let got = run("sse.interior.inertia_giant", &[z])[0];
        assert!(
            (got - ia::giant(z, ia::CORE_SHARE)).abs() < 1e-12,
            "{z}: {got}"
        );
    }
    assert!((ia::giant(0.0, 0.5) - 0.2614).abs() < 1e-4);
}

#[test]
fn weathering_blocks_match_their_sums() {
    use crate::weathering as w;
    for (t, f, o) in [
        (3.2e16, 1.0, 1.0),
        (3.2e15, 1.0, 1.0),
        (3.2e16, 7.0, 0.0),
        (0.0, 1.0, 1.0),
    ] {
        let got = run("sse.surface.space_weathering", &[t, f, o])[0];
        let want = w::maturity(t, f, o > 0.5, w::TAU_1AU_S, w::WIND_SHARE);
        assert!(
            (got - want).abs() < 1e-12,
            "{t} {f} {o}: {got} against {want}"
        );
    }
    assert_eq!(
        w::maturity(0.0, 1.0, true, w::TAU_1AU_S, w::WIND_SHARE),
        0.0
    );
    for m in [0.0, 0.3, 1.0] {
        let got = run("sse.surface.weathered_reflectance", &[m]);
        let want = w::factors(m, w::MATURE_OVER_FRESH);
        for i in 0..3 {
            assert!((got[i] - want[i]).abs() < 1e-12, "{m}: {got:?}");
        }
    }
    // Fresh ground is exactly unchanged; mature ground is redder.
    assert_eq!(w::factors(0.0, w::MATURE_OVER_FRESH), [1.0, 1.0, 1.0]);
    let f = w::factors(1.0, w::MATURE_OVER_FRESH);
    assert!(f[0] > f[1] && f[1] > f[2]);
}

#[test]
fn ice_ground_blocks_match_their_sums() {
    use crate::ice_ground as ig;
    use sse_core::sheet::{density_solid, mixture_density, molar_mass};
    let (rr, ri) = (mixture_density::SURFACE_ROCK, mixture_density::SURFACE_ICE);
    for rho in [3013.0, 1936.0, 3344.0, 5514.0, 800.0] {
        let got = run("sse.surface.ice_share_from_density", &[rho, rr, ri])[0];
        let want = ig::ice_share(rho, rr, ri);
        assert!((got - want).abs() < 1e-12, "{rho}: {got} against {want}");
    }
    // Europa is a little ice, Ganymede about a third, Earth none.
    assert!((ig::ice_share(3013.0, rr, ri) - 0.058).abs() < 0.002);
    assert!(ig::ice_share(5514.0, rr, ri) == 0.0);
    let mw = molar_mass::H2O;
    let tau = sse_core::sheet::photo_lifetime_1au::H2O;
    for (p, t, g, ve, d) in [
        (76.0, 250.0, 1.62, 2380.0, 1.0),
        (2.0e-6, 130.0, 1.24, 2440.0, 5.2),
        (0.0, 100.0, 1.31, 2025.0, 5.2),
    ] {
        let got = run(
            "sse.surface.open_ice_loss",
            &[p, t, mw, g, ve, d * sse_core::sheet::AU, 1.4e17, ri, tau],
        )[0];
        let want = ig::open_loss_m(p, t, mw, g, ve, d, 1.4e17, ri, tau);
        assert!(
            (got - want).abs() <= 1e-9 * want.abs().max(1e-30),
            "{p} {t}: {got} against {want}"
        );
    }
    // The warm Moon loses kilometres; cold Callisto a few hundred metres at most.
    assert!(ig::open_loss_m(76.0, 250.0, mw, 1.62, 2380.0, 1.0, 1.4e17, ri, tau) > 1e6);
    assert!(ig::open_loss_m(2.0e-6, 130.0, mw, 1.24, 2440.0, 5.2, 1.4e17, ri, tau) < 1e3);
    let _ = density_solid::H2O;
    for t in [1.4e17, 6.3e16, 1.9e15, 0.0] {
        let got = run("sse.surface.ice_dust_lag", &[t, 0.95, 0.0572]);
        let d = ig::lag_share(t, ig::LAG_TAU_S);
        assert!((got[0] - d).abs() < 1e-12, "{t}: {got:?}");
        let r = ig::lagged_reflectance(0.95, 0.0572, d);
        assert!((got[1] - r).abs() < 1e-12, "{t}: {got:?}");
    }
    // Callisto's old ground is dark, Europa's young ground bright.
    assert!(ig::lagged_reflectance(0.95, 0.0572, ig::lag_share(1.4e17, ig::LAG_TAU_S)) < 0.25);
    assert!(ig::lagged_reflectance(0.95, 0.0572, ig::lag_share(1.9e15, ig::LAG_TAU_S)) > 0.9);
}

#[test]
fn prominence_blocks_match_their_sums() {
    use crate::activity as a;
    for ro in [0.02, 0.13, 0.5, 1.8, 3.0] {
        let got = run("sse.activity.surface_field", &[ro])[0];
        let want = a::surface_field(ro, 2050.0, -0.11, 199.0, -1.26, 0.13);
        assert!(
            (got - want).abs() <= 1e-12 * want,
            "{ro}: {got} against {want}"
        );
    }
    // The Sun: about a hundred gauss, a corona near 1.7 MK about 2e14 per m³ at its foot.
    let b = a::surface_field(1.8, 2050.0, -0.11, 199.0, -1.26, 0.13);
    assert!((0.008..0.011).contains(&b), "{b}");
    let lx = m::pow(10.0, a::SUN_LOG_RX) * L_SUN;
    let g = GM_SUN / (R_SUN * R_SUN);
    let t = run("sse.activity.corona_temperature", &[lx, R_SUN])[0];
    assert!(
        (t - a::corona_temperature(lx, R_SUN, 0.11, 0.26)).abs() < 1e-6,
        "{t}"
    );
    assert!((1.4e6..2.0e6).contains(&t), "{t}");
    let n = run("sse.activity.corona_base_density", &[lx, R_SUN, t, g])[0];
    let nw = a::corona_base_density(lx, R_SUN, t, g, 1.2e-35, 0.6);
    assert!((n - nw).abs() <= 1e-9 * nw, "{n} against {nw}");
    assert!((5e13..1e15).contains(&n), "{n}");
    // The Sun's co-rotation radius is far outside its closed field; AB Doradus's is inside.
    let day = sse_core::sheet::DAY;
    let rk = run(
        "sse.orbit.semi_major_axis_from_period",
        &[26.09 * day, M_SUN],
    )[0];
    assert!((rk - a::corotation_radius(M_SUN, 26.09 * day)).abs() <= 1e-9 * rk);
    assert!((30.0..45.0).contains(&(rk / R_SUN)), "{}", rk / R_SUN);
    for bf in [b, 0.285] {
        let got = run("sse.activity.closed_corona_radius", &[bf])[0];
        let want = a::closed_corona_radius(bf, 2.5, 95.0, 1.0 / 3.0);
        assert!((got - want).abs() <= 1e-12 * want, "{got} against {want}");
    }
    let rk_abdor = a::corotation_radius(0.86 * M_SUN, 0.514 * day) / (0.96 * R_SUN);
    assert!(rk_abdor < a::closed_corona_radius(0.285, 2.5, 95.0, 1.0 / 3.0));
}

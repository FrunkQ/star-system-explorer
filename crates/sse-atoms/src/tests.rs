//! Golden cases for every first-wave atom. Reference values are cited inline:
//! "NASA fact sheet" means the NASA GSFC Planetary Fact Sheets
//! (nssdc.gsfc.nasa.gov/planetary/factsheet, values as of 2024).
//! "IAU 2015 B3" is the IAU 2015 Resolution B3 nominal solar values.

use super::*;
use sse_core::math as m;
use sse_core::quantity::consts::*;
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
    assert_eq!(
        n, 48,
        "first-wave atom count changed: update handover/packages/P1.md"
    );
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
    // inside the 10 to 30 km range of conductive models (Ojakangas and Stevenson 1989).
    let d = run("sse.heat.ice_shell_thickness", &[0.05, 100.0, 270.0])[0];
    close(d, 651.0 / 0.05 * m::ln(2.7), 1e-12, "formula");
    assert!(d > 1e4 && d < 3e4);
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
    close(e0[0], 5.26e6, 0.01, "Al-26 energy at CAI time");
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

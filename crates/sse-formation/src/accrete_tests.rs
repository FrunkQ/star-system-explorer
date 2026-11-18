//! F2 evidence: the Classic Accrete block against System Lab's own outputs.
use crate::accrete::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    cases: Vec<Case>,
}
#[derive(Deserialize)]
struct Case {
    seed: String,
    star: String,
    luminosity: f64,
    mass_kg: f64,
    controls: Controls,
    iterations: u32,
    exhausted_dust: bool,
    duration_years: f64,
    bodies: Vec<FBody>,
    bands: Vec<DustBand>,
    events: usize,
}
#[derive(Deserialize)]
struct FBody {
    id: String,
    formed_from: Vec<String>,
    a_au: f64,
    e: f64,
    mass_solar: f64,
    gas_giant: bool,
}

fn fixture() -> Fixture {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/fixtures/formation/system-lab-accrete.json"
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn request(c: &Case) -> Request {
    Request {
        master_seed: c.seed.clone(),
        star_mass: c.mass_kg / SOLAR_MASS_KG,
        star_luminosity: c.luminosity,
        controls: c.controls,
        rule: Rule::SystemLab,
    }
}

fn rel(a: f64, b: f64) -> f64 {
    if a == b {
        0.0
    } else {
        (a - b).abs() / a.abs().max(b.abs())
    }
}

#[test]
fn random_streams_match_system_lab() {
    // FNV-1a 32 of "abc" and of System Lab's framed first axis stream.
    assert_eq!(fnv1a32(""), 0x811c_9dc5);
    assert_eq!(fnv1a32("a"), 0xe40c_292c);
    let mut s = Mulberry32::new(1);
    // mulberry32(1) first output, as JavaScript gives it.
    assert_eq!(s.next_float(), 0.6270739405881613);
}

#[test]
fn reproduces_system_lab_for_every_fixture_seed() {
    let f = fixture();
    assert!(f.cases.len() >= 200, "{} cases", f.cases.len());
    let (mut exact, mut worst) = (0usize, 0.0f64);
    let mut bodies = 0;
    for c in &f.cases {
        let r = run(&request(c)).unwrap();
        let what = format!("{} {} {:?}", c.seed, c.star, c.controls);
        assert_eq!(r.iterations, c.iterations, "{what}: iterations");
        assert_eq!(r.exhausted_dust, c.exhausted_dust, "{what}");
        assert!(
            rel(r.duration_years, c.duration_years) < 1e-15,
            "{what}: duration"
        );
        assert_eq!(r.bodies.len(), c.bodies.len(), "{what}: bodies");
        assert_eq!(r.bands.len(), c.bands.len(), "{what}: bands");
        // Events: System Lab counts the same four kinds.
        assert_eq!(r.events.len(), c.events, "{what}: events");
        let mut same = true;
        for (b, w) in r.bodies.iter().zip(&c.bodies) {
            assert_eq!(b.id, w.id, "{what}");
            assert_eq!(b.formed_from, w.formed_from, "{what}");
            assert_eq!(b.gas_giant, w.gas_giant, "{what} {}", b.id);
            for (x, y) in [(b.a_au, w.a_au), (b.e, w.e), (b.mass, w.mass_solar)] {
                worst = worst.max(rel(x, y));
                same &= x == y;
            }
            bodies += 1;
        }
        for (b, w) in r.bands.iter().zip(&c.bands) {
            assert_eq!((b.dust, b.gas), (w.dust, w.gas), "{what}");
            worst = worst
                .max(rel(b.lower_au, w.lower_au))
                .max(rel(b.upper_au, w.upper_au));
            same &= b.lower_au == w.lower_au && b.upper_au == w.upper_au;
        }
        exact += same as usize;
    }
    assert!(worst < 1e-9, "worst relative difference {worst}");
    eprintln!(
        "{} cases, {bodies} bodies, {exact} bit-identical, worst relative difference {worst:e}",
        f.cases.len()
    );
}

#[test]
fn ledger_sums_to_each_bodys_mass() {
    for c in fixture().cases.iter().take(60) {
        let r = run(&request(c)).unwrap();
        for b in &r.bodies {
            assert_eq!(b.ledger.total(), b.mass, "{} {}", c.seed, b.id);
            let check = |l: &Ledger| {
                // Every merged sub-ledger also sums to the mass it was merged with.
                fn walk(l: &Ledger) {
                    for e in &l.entries {
                        if let Entry::Merged { mass, body } = e {
                            assert_eq!(body.total(), *mass, "{}", body.id);
                            walk(body);
                        }
                    }
                }
                walk(l)
            };
            check(&b.ledger);
            let g = b.ledger.gas();
            assert!(g >= 0.0 && g <= b.mass, "gas {g} of {}", b.mass);
            // Rocky bodies took no gas; gas giants took mostly gas.
            if !b.gas_giant
                && !b
                    .ledger
                    .entries
                    .iter()
                    .any(|e| matches!(e, Entry::Merged { .. }))
            {
                assert_eq!(g, 0.0, "{} {}", c.seed, b.id);
            }
        }
    }
}

#[test]
fn hands_over_to_early_worlds() {
    // The default solar case through F6: rocky bodies get no hydrogen, giants are giants.
    let f = fixture();
    let c = &f.cases[0];
    let r = run(&request(c)).unwrap();
    let sys = crate::scenario::system(77);
    let fits = crate::hurley::Fits::new(0.02);
    let host = crate::lifeline::LifeLine::new(&fits, 1.0).unwrap();
    let pp = protoplanets(&r, &sys, None);
    assert_eq!(pp.len(), r.bodies.len());
    for (p, b) in pp.iter().zip(&r.bodies) {
        let total = p.mass + p.gas;
        assert!(
            rel(total, b.earth_mass()) < 1e-12,
            "{} {} vs {}",
            b.id,
            total,
            b.earth_mass()
        );
        let w = crate::early::early_life(p, &fits, &host);
        if b.gas_giant && p.gas > p.mass {
            assert_eq!(w.envelope, crate::early::Envelope::Giant, "{}", b.id);
        }
    }
}

#[test]
fn refuses_controls_outside_system_labs_ranges() {
    let mut req = request(&fixture().cases[0]);
    req.controls.K = 500.0;
    assert!(run(&req).is_err());
}

#[test]
fn accrete_golden() {
    crate::golden::check("formation-accrete.json", &crate::scenario::accrete_golden());
}

fn dole(seed: &str, mass: f64, feh: f64) -> Run {
    let metals = sse_core::math::pow(10.0, feh);
    run(&Request {
        master_seed: seed.into(),
        star_mass: mass,
        star_luminosity: 1.0,
        controls: Controls {
            A: DOLE_A * sse_core::math::sqrt(mass) * metals,
            K: DOLE_K / metals,
            ..Controls::DEFAULT
        },
        rule: Rule::Dole1970,
    })
    .unwrap()
}

#[test]
fn corrected_rule_makes_rocky_inner_planets_and_giants_further_out() {
    // System Lab's rule gives the Sun three giants and no rocky planets (known difference 2);
    // Dole's own reach gives the familiar Accrete systems: small rocky worlds inside, giants outside.
    let (mut rocky_inner, mut giants, mut runs) = (0, 0, 0);
    for k in 0..40 {
        let r = dole(&format!("dole-{k}"), 1.0, 0.0);
        assert_eq!(r.rule_version, RULE_DOLE);
        runs += 1;
        rocky_inner += r
            .bodies
            .iter()
            .filter(|b| b.a_au < 2.0 && !b.gas_giant)
            .count();
        giants += r.bodies.iter().filter(|b| b.gas_giant).count();
        for b in &r.bodies {
            let mut total = 0.0;
            for e in &b.ledger.entries {
                total += match e {
                    Entry::Seed { mass } | Entry::Merged { mass, .. } => *mass,
                    Entry::Sweep { mass, .. } => *mass,
                };
            }
            assert_eq!(total, b.mass, "ledger sums bit for bit");
        }
    }
    let solar_lab = run(&Request {
        master_seed: "dole-0".into(),
        star_mass: 1.0,
        star_luminosity: 1.0,
        controls: Controls::DEFAULT,
        rule: Rule::SystemLab,
    })
    .unwrap();
    assert_eq!(solar_lab.rule_version, RULE_VERSION);
    eprintln!("Dole 1970 rule, 40 Sun runs: {rocky_inner} rocky inside 2 AU, {giants} giants");
    let lab_rocky: usize = (0..40)
        .map(|k| {
            run(&Request {
                master_seed: format!("dole-{k}"),
                star_mass: 1.0,
                star_luminosity: 1.0,
                controls: Controls::DEFAULT,
                rule: Rule::SystemLab,
            })
            .unwrap()
            .bodies
            .iter()
            .filter(|b| b.a_au < 2.0 && !b.gas_giant)
            .count()
        })
        .sum();
    eprintln!("System Lab rule, same seeds: {lab_rocky} rocky inside 2 AU");
    assert!(
        rocky_inner as f64 / runs as f64 >= 1.0,
        "rocky inner planets per run {rocky_inner}/{runs}"
    );
    assert!(
        rocky_inner > 2 * lab_rocky.max(1),
        "corrected {rocky_inner} against System Lab {lab_rocky}"
    );
    assert!(
        giants as f64 / runs as f64 >= 1.0,
        "giants per run {giants}/{runs}"
    );
}

#[test]
fn corrected_rule_follows_the_inheritance() {
    // More metals in the patch: more dust, so more planet mass (same nuclei, same draws).
    let mass = |r: &Run| r.bodies.iter().map(|b| b.earth_mass()).sum::<f64>();
    let (mut rich, mut poor) = (0.0, 0.0);
    for k in 0..20 {
        rich += mass(&dole(&format!("z-{k}"), 1.0, 0.3));
        poor += mass(&dole(&format!("z-{k}"), 1.0, -0.5));
    }
    assert!(rich > 2.0 * poor, "rich {rich} poor {poor}");
    // From G0: every solar-patch system's request carries its own [Fe/H] and star.
    for h in crate::scenario::solar_patch().into_iter().flatten().take(8) {
        let req = Request::from_history(&h, "g0");
        let want = DOLE_A
            * sse_core::math::sqrt(h.stars[0].mass)
            * sse_core::math::pow(10.0, h.inheritance.feh);
        assert_eq!(req.controls.A, want);
        assert_eq!(req.star_mass, h.stars[0].mass);
        assert_eq!(req.rule, Rule::Dole1970);
        let r = run(&req).unwrap();
        // The hand-off reads the same system's disc bands, which F1 built from the same inheritance.
        let p = protoplanets(&r, &h.manifest.system, Some(&h.disc));
        assert_eq!(p.len(), r.bodies.len());
    }
}

#[test]
fn dole_golden() {
    crate::golden::check(
        "formation-accrete-dole.json",
        &crate::scenario::accrete_dole_golden(),
    );
}

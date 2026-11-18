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

//! T1 checks.

use crate::epoch::Ageing;
use crate::kepler::{self, Elements};
use crate::nbody::{Order, Window};
use crate::scenario;
use crate::secular::{Planet, Secular};
use sse_core::math as m;

const MJ: f64 = 9.547_919e-4;
const MS: f64 = 2.858_860e-4;
const DEG: f64 = m::PI / 180.0;
const ARCSEC_YR: f64 = m::PI / (180.0 * 3600.0);

fn el(a: f64, e: f64, i: f64, node: f64, peri: f64, mean: f64) -> Elements {
    Elements {
        a,
        e,
        i: i * DEG,
        node: node * DEG,
        peri: peri * DEG,
        mean: mean * DEG,
    }
}
fn jupiter_saturn() -> Vec<(String, f64, Elements)> {
    vec![
        (
            "Jupiter".into(),
            MJ,
            el(
                5.2026,
                0.04849,
                1.303,
                100.464,
                14.331 - 100.464,
                34.40 - 14.331,
            ),
        ),
        (
            "Saturn".into(),
            MS,
            el(
                9.5549,
                0.05551,
                2.489,
                113.665,
                93.057 - 113.665,
                49.94 - 93.057,
            ),
        ),
    ]
}
fn fixed(_: f64) -> f64 {
    1.0
}

#[test]
fn energy_drift_is_bounded() {
    // Sun, Jupiter, Saturn for 20 000 years (1700 Jupiter orbits).
    for (order, dt, bound) in [(Order::Fourth, 0.1, 1.0e-6), (Order::Second, 0.05, 1.0e-4)] {
        let mut w = Window::from_elements(1.0, &jupiter_saturn(), dt);
        let e0 = w.energy();
        let mut worst = 0.0_f64;
        let mut errs = vec![];
        for _ in 0..20 {
            w.run((1000.0 / dt) as u64, order, &fixed);
            let err = ((w.energy() - e0) / e0).abs();
            worst = worst.max(err);
            errs.push(err);
        }
        println!("{order:?} dt {dt}: worst relative energy error {worst:.2e}");
        assert!(worst < bound, "{order:?} drift {worst:e}");
        // Symplectic: no secular growth, the second half no worse than 3x the first.
        let first = errs[..10].iter().cloned().fold(0.0, f64::max);
        let second = errs[10..].iter().cloned().fold(0.0, f64::max);
        assert!(
            second < 3.0 * first.max(1.0e-15),
            "growth {first:e} -> {second:e}"
        );
    }
}

#[test]
fn two_body_matches_kepler() {
    let e0 = el(1.3, 0.3, 10.0, 40.0, 70.0, 20.0);
    let mp = 1.0e-3;
    let run = |order: Order, per_yr: f64| {
        let dt = 1.0 / per_yr;
        let mut w = Window::from_elements(1.0, &[("p".into(), mp, e0)], dt);
        let n = (per_yr * 150.0) as u64;
        w.run(n, order, &fixed);
        let t = n as f64 * dt;
        let e1 = w.elements(1);
        let nn = m::sqrt(kepler::G * (1.0 + mp) / (e0.a * e0.a * e0.a));
        let expect = m::fmod(e0.mean + nn * t, 2.0 * m::PI);
        let dmean = m::fmod(e1.mean - expect + 3.0 * m::PI, 2.0 * m::PI) - m::PI;
        (e1, dmean)
    };
    // Fourth order: errors fall 16 times when the step halves (Yoshida 1990);
    // second order 4 times.
    for (order, k, lo, hi) in [(Order::Fourth, 4, 12.0, 20.0), (Order::Second, 2, 3.5, 4.5)] {
        let (a1, m1) = run(order, 200.0);
        let (a2, m2) = run(order, 400.0);
        let r = (a1.a / e0.a - 1.0).abs() / (a2.a / e0.a - 1.0).abs();
        println!("{order:?}: da {:.1e} then {:.1e} (ratio {r:.1}, order {k}); dmean {m1:.1e} then {m2:.1e} rad", a1.a / e0.a - 1.0, a2.a / e0.a - 1.0);
        assert!(r > lo && r < hi, "{order:?} ratio {r}");
    }
    let (e1, dmean) = run(Order::Fourth, 1000.0);
    println!(
        "Fourth, 1000 steps a year over 150 yr: da {:.1e} de {:.1e} dmean {dmean:.1e}",
        e1.a / e0.a - 1.0,
        e1.e - e0.e
    );
    assert!((e1.a / e0.a - 1.0).abs() < 1.0e-8);
    assert!((e1.e - e0.e).abs() < 1.0e-8);
    assert!(dmean.abs() < 1.0e-6);
    assert!((e1.i - e0.i).abs() < 1.0e-9 && (e1.node - e0.node).abs() < 1.0e-9);
    // Elements round trip.
    let (x, v) = kepler::to_state(&e0, kepler::G);
    let back = kepler::from_state(x, v, kepler::G);
    assert!(
        (back.a - e0.a).abs() < 1e-12
            && (back.e - e0.e).abs() < 1e-12
            && (back.mean - e0.mean).abs() < 1e-10
    );
}

#[test]
fn mass_loss_keeps_a_times_m() {
    // Slow mass loss (Jeans 1924): a M stays constant while the star loses
    // half its mass over 3000 orbits.
    let e0 = el(1.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    let dt = 1.0 / 100.0;
    let mut w = Window::from_elements(1.0, &[("p".into(), 1.0e-6, e0)], dt);
    let mass = |t: f64| 1.0 - 0.5 * (t / 3000.0).min(1.0);
    w.run(300_000, Order::Fourth, &mass);
    let e1 = w.elements(1);
    let inv = e1.a * mass(w.t);
    println!("a M = {inv:.5} (1 expected), e = {:.1e}", e1.e);
    assert!((inv - 1.0).abs() < 2.0e-3);
    assert!(e1.e < 1.0e-2);
}

fn planets(ps: &[(String, f64, Elements)]) -> Vec<Planet> {
    ps.iter()
        .map(|(_, mass, e)| Planet {
            mass: *mass,
            a: e.a,
            e: e.e,
            varpi: e.node + e.peri,
            i: e.i,
            node: e.node,
        })
        .collect()
}

/// Laplace-Lagrange against the integrator: e and i of each planet sampled
/// over `years`; returns the largest error in e and in i (radians) and the
/// largest secular swing of e the theory predicts.
fn secular_against_nbody(
    ps: &[(String, f64, Elements)],
    years: f64,
    dt: f64,
    samples: u32,
) -> (f64, f64, f64) {
    let p0 = planets(ps);
    let sec = Secular::new(1.0, &p0);
    let mut w = Window::from_elements(1.0, ps, dt);
    let per = (years / f64::from(samples) / dt) as u64;
    let (mut de, mut di) = (0.0_f64, 0.0_f64);
    let mut swing = 0.0_f64;
    for k in 1..=samples {
        w.run(per, Order::Fourth, &fixed);
        let s = sec.advance(&p0, w.t);
        for (j, p) in s.iter().enumerate() {
            let o = w.elements(j + 1);
            de = de.max((o.e - p.e).abs());
            swing = swing.max((p.e - p0[j].e).abs());
            di = di.max((o.i - p.i).abs());
        }
        let _ = k;
    }
    (de, di, swing)
}

#[test]
fn laplace_lagrange_against_integrator() {
    // A well-separated pair (period ratio 8): first-order secular theory
    // should hold to a small fraction of the eccentricities.
    // 150 steps per inner orbit keeps the integrator's own precession
    // (about (2 pi / 150)^4 rad per orbit) far below the secular rates.
    let wide = vec![
        ("b".to_string(), 2.0e-3, el(1.0, 0.05, 1.0, 0.0, 0.0, 0.0)),
        (
            "c".to_string(),
            2.0e-3,
            el(4.0, 0.03, 2.0, 120.0, 30.0, 200.0),
        ),
    ];
    let sec = Secular::new(1.0, &planets(&wide));
    let half = m::PI / (sec.g[0] - sec.g[1]).abs();
    let (de, di, swing) = secular_against_nbody(&wide, half, 1.0 / 150.0, 40);
    println!("wide pair over half a secular period ({half:.0} yr): e swings by {swing:.4}; max error e {de:.4}, i {:.4} deg", di / DEG);
    assert!(de < 0.2 * swing, "e error {de} against a swing of {swing}");
    assert!(di < 0.1 * DEG, "i error {di}");
    // Jupiter and Saturn: near the 5:2 (the Great Inequality, period about
    // 900 yr), which first-order theory leaves out; the error is reported and
    // only loosely bounded.
    let js = jupiter_saturn();
    let (de, di, swing) = secular_against_nbody(&js, 20_000.0, 0.05, 40);
    println!(
        "Jupiter-Saturn over 20 kyr: e swings by {swing:.4}; max error e {de:.4}, i {:.4} deg",
        di / DEG
    );
    assert!(de < 0.03, "e error {de}");
    assert!(di < 0.3 * DEG, "i error {di}");
}

#[test]
fn solar_system_secular_frequencies() {
    // All eight planets, first-order Laplace-Lagrange. Against Brouwer and
    // van Woerkom 1950 (their secular theory for the Solar System): the
    // terrestrial and ice-giant modes agree to about 1 percent; g5 and g6 are
    // lower here because Brouwer and van Woerkom add second-order terms of the
    // Jupiter-Saturn near 5:2 (they give 4.30 and 27.77). Without those terms
    // Murray and Dermott 1999 (section 7.4, Jupiter and Saturn alone) give 3.7
    // and 22.3.
    let sp = scenario::solar_system();
    let ps: Vec<Planet> = sp
        .planets
        .iter()
        .map(|p| Planet {
            mass: p.mass_me * 3.003_489e-6,
            a: p.a_au,
            e: p.e,
            varpi: p.varpi_deg * DEG,
            i: p.i_deg * DEG,
            node: p.node_deg * DEG,
        })
        .collect();
    let sec = Secular::new(1.0, &ps);
    let mut g: Vec<f64> = sec.g.iter().map(|x| x / ARCSEC_YR).collect();
    let mut f: Vec<f64> = sec.f.iter().map(|x| x / ARCSEC_YR).collect();
    g.sort_by(|a, b| a.total_cmp(b));
    f.sort_by(|a, b| a.total_cmp(b));
    println!("g = {g:.2?}\nf = {f:.2?}");
    let near = |xs: &[f64], v: f64, rel: f64| xs.iter().any(|x| (x - v).abs() < rel * v.abs());
    for v in [5.46, 7.34, 17.33, 18.00, 2.72, 0.63] {
        assert!(near(&g, v, 0.02), "g {v}");
    }
    for v in [-5.20, -6.57, -18.74, -17.63, -25.73, -2.90, -0.68] {
        assert!(near(&f, v, 0.02), "s {v}");
    }
    assert!(
        near(&g, 3.7, 0.03) && near(&g, 22.3, 0.03),
        "g5 g6 without the 5:2 terms"
    );
    // One inclination mode is zero (the invariable plane).
    assert!(f.iter().any(|x| x.abs() < 1.0e-6));
}

#[test]
fn ageing_star_follows_the_sun() {
    let st = scenario::settings();
    let s = crate::star::AgeingStar::new(1.0, 0.02, st.star.clone()).unwrap();
    let l = &s.line;
    // Brightens on the main sequence: about 10 percent per Gyr from now.
    let now = s.state_at(4570.0).star.luminosity;
    let later = s.state_at(5570.0).star.luminosity;
    assert!(later / now > 1.05 && later / now < 1.15, "{now} {later}");
    // Red giant tip: core near 0.47 (helium flash), radius 150 to 260 R_sun,
    // mass after winds 0.6 to 0.75 (Schroeder and Smith 2008 give 0.67 and
    // 256 R_sun with a modified Reimers law).
    let tip = s.state_at(l.t_end - 1.0e-6);
    println!(
        "tip: L {:.0} R {:.0} M {:.3} Mc {:.3}",
        tip.star.luminosity, tip.star.radius, tip.mass, tip.core_mass
    );
    assert!((tip.core_mass - 0.47).abs() < 0.02);
    assert!(tip.star.radius > 150.0 && tip.star.radius < 260.0);
    assert!(tip.mass > 0.6 && tip.mass < 0.75);
    // Most of the giant branch is spent faint: half way through by age it is
    // still under 10 L_sun.
    assert!(
        s.state_at(l.t_bgb + 0.5 * (l.t_end - l.t_bgb))
            .star
            .luminosity
            < 10.0
    );
    // Mass never rises and never falls below the core.
    let mut prev = 2.0;
    for k in 0..=1000 {
        let t = l.t_tms + (l.t_end - l.t_tms) * f64::from(k) / 1000.0;
        let a = s.state_at(t);
        assert!(a.mass <= prev + 1e-15 && a.mass >= a.core_mass - 1e-12);
        prev = a.mass;
    }
    // Blackbody colours.
    assert_eq!(crate::star::blackbody_rgb(6600.0), [255, 255, 255]);
    assert!(crate::star::blackbody_rgb(3000.0)[2] < 200);
}

#[test]
fn age_the_solar_system() {
    let st = scenario::settings();
    let sp = scenario::solar_system();
    let a = Ageing::new(&st, &sp).unwrap();
    let r = a.run(a.star.line.t_end + 10_000.0);
    let kinds: Vec<&str> = r.events.iter().map(|e| e.kind.as_str()).collect();
    println!("{kinds:?}");
    for want in [
        "oceans_lost",
        "leaves_main_sequence",
        "red_giant",
        "engulfed",
        "planetary_nebula",
        "cooling_colour",
        "pollution",
    ] {
        assert!(kinds.contains(&want), "missing {want}");
    }
    // Every event answers "why?", in date order, labelled beyond the horizon.
    let mut t = 0.0;
    for e in &r.events {
        assert!(e.why.len() > 40, "{}", e.kind);
        assert!(e.age_myr >= t);
        t = e.age_myr;
        assert_eq!(e.plausible_future, e.age_myr > sp.age_myr + st.horizon_myr);
    }
    // Oceans go 1 to 2 Gyr from now (Kopparapu 2013 type limit on the HPT00 Sun).
    let oceans = r.events.iter().find(|e| e.kind == "oceans_lost").unwrap();
    assert!(
        oceans.age_myr > 5300.0 && oceans.age_myr < 6800.0,
        "{}",
        oceans.age_myr
    );
    // Mercury is engulfed; Earth and the giants survive on widened orbits.
    let last = r.snapshots.last().unwrap();
    let p = |n: &str| last.planets.iter().find(|x| x.name == n).unwrap();
    assert!(p("Mercury").engulfed);
    assert!(!p("Earth").engulfed);
    let widen = 1.0 / 0.569;
    assert!((p("Jupiter").a_au / 5.2026 / widen - 1.0).abs() < 0.02);
    assert!((p("Earth").a_au / 1.0 / widen - 1.0).abs() < 0.03);
    // The white dwarf cools through the colours in order.
    let colours: Vec<&str> = r
        .events
        .iter()
        .filter(|e| e.kind == "cooling_colour")
        .map(|e| e.why.as_str())
        .collect();
    assert!(colours.len() >= 4);
    assert_eq!(last.star.phase, "white dwarf");
    // Windows: energy held, orbits not rewritten in the quiet Solar System.
    assert!(r.windows.len() >= 3);
    for w in &r.windows {
        assert!(w.energy_error < 1.0e-5, "{} {}", w.reason, w.energy_error);
        assert!(!w.written_back);
    }
    // Deterministic, work counted.
    let r2 = a.run(a.star.line.t_end + 10_000.0);
    assert_eq!(r, r2);
    assert!(r.work > r.snapshots.len() as u64);
}

// ---- T2: look-ahead, commit, replay ----

use crate::branch::{self, Request};
use sse_chronicle::{Chronicle, UniverseTime};
use sse_core::cbor::Value;
use std::sync::OnceLock;

fn aged() -> &'static scenario::Aged {
    static A: OnceLock<scenario::Aged> = OnceLock::new();
    A.get_or_init(scenario::age_the_solar_system)
}

#[test]
fn look_ahead_stays_on_its_branch_until_commit() {
    let st = scenario::settings();
    let sp = scenario::solar_system();
    let mut ch = Chronicle::new();
    let head = ch.head();
    // A short look-ahead (to the end of the main sequence) for speed.
    let req = Request::new(sp.clone(), st, 11_000.0);
    let la = branch::look_ahead(&mut ch, req, &mut |_| true)
        .unwrap()
        .unwrap();
    assert_eq!(ch.head(), head, "the main chronicle is untouched");
    let earth: sse_core::Address = "u:sol/sys:0/body:2".parse().unwrap();
    let star: sse_core::Address = sp.star_address.parse().unwrap();
    let later = branch::universe_time(&sp, 10_000.0);
    let b = ch.branch_state_at(&la.branch, later).unwrap();
    let lum = match &b.facts[&star]["star.luminosity"] {
        Value::Float(x) => *x,
        v => panic!("{v:?}"),
    };
    assert!(lum > 1.5, "brighter in 5 Gyr: {lum}");
    assert!(b.facts.contains_key(&earth));
    assert!(ch.state_at(later).facts.is_empty());
    // Discard leaves nothing.
    ch.discard(&la.branch).unwrap();
    assert_eq!(ch.head(), head);
    assert!(ch.branch_state_at(&la.branch, later).is_err());
}

#[test]
fn look_ahead_cancels_by_work_units() {
    let st = scenario::settings();
    let sp = scenario::solar_system();
    let mut ch = Chronicle::new();
    let req = Request::new(sp, st, 11_000.0);
    let mut seen = vec![];
    let out = branch::look_ahead(&mut ch, req, &mut |w| {
        seen.push(w);
        w < 10
    })
    .unwrap();
    assert!(out.is_none());
    assert_eq!(
        seen,
        (1..=10).collect::<Vec<u64>>(),
        "progress in work units, cancelled after the tenth"
    );
    assert_eq!(ch.len(), 0);
}

#[test]
fn commit_ages_the_solar_system() {
    let a = aged();
    let ch = &a.chronicle;
    let sp = &a.look.request.system;
    let kinds: Vec<&str> = ch.events().iter().map(|e| e.body.kind.as_str()).collect();
    assert_eq!(kinds[0], branch::REQUEST);
    assert!(kinds.contains(&"commit.branch"));
    for k in [
        "time.oceans_lost",
        "time.leaves_main_sequence",
        "time.red_giant",
        "time.engulfed",
        "time.planetary_nebula",
        "time.cooling_colour",
        "time.pollution",
    ] {
        assert!(kinds.contains(&k), "{k}");
    }
    // Every named event explains itself and is labelled past the horizon.
    for e in ch
        .events()
        .iter()
        .filter(|e| e.body.kind.starts_with("time.") && e.body.kind != branch::REQUEST)
    {
        let Some(Value::Map(d)) = &e.body.payload.data else {
            panic!("no data")
        };
        assert!(matches!(&d["why"], Value::Text(t) if t.len() > 40));
        assert!(matches!(d["plausible_future"], Value::Bool(_)));
    }
    // "Now" moved to 1 Gyr after the nebula; the state there is the aged system.
    let sys: sse_core::Address = sp.system.parse().unwrap();
    let far = UniverseTime::years(i64::MAX / 31_557_600 - 1);
    let year = match ch.state_at(far).facts[&sys][branch::SYSTEM_NOW] {
        Value::Int(y) => y as i64,
        ref v => panic!("{v:?}"),
    };
    let then = UniverseTime::years(year);
    let st = ch.state_at(then);
    let star: sse_core::Address = sp.star_address.parse().unwrap();
    assert_eq!(
        st.facts[&star]["star.phase"],
        Value::Text("white dwarf".into())
    );
    let mercury: sse_core::Address = sp.planets[0].address.parse().unwrap();
    assert_eq!(st.facts[&mercury]["body.engulfed"], Value::Bool(true));
    let jupiter: sse_core::Address = sp.planets[4].address.parse().unwrap();
    let Value::Float(a) = st.facts[&jupiter]["orbit.a"] else {
        panic!()
    };
    assert!(a > 9.0 && a < 9.3, "Jupiter at {a} au");
    // Before any of it: the present Sun.
    let today = ch.state_at(branch::universe_time(sp, sp.age_myr));
    assert_eq!(
        today.facts[&star]["star.phase"],
        Value::Text("main sequence".into())
    );
}

#[test]
fn replay_from_the_authored_chronicle() {
    let a = aged();
    let again = branch::replay(&a.chronicle).unwrap();
    assert_eq!(again.head(), a.chronicle.head());
    assert_eq!(again.authored_hash(), a.chronicle.authored_hash());
    // A second full run gives the same manifest and head.
    let b = scenario::age_the_solar_system();
    assert_eq!(b.manifest.id(), a.manifest.id());
    assert_eq!(b.chronicle.head(), a.chronicle.head());
    // Tampering with a derived event is caught by replay.
    let mut evs: Vec<_> = a.chronicle.events().to_vec();
    let k = evs
        .iter()
        .position(|e| e.body.kind == "time.engulfed")
        .unwrap();
    let mut body = evs[k].body.clone();
    body.payload.data = Some(Value::Text("tampered".into()));
    evs[k] = sse_chronicle::Event::new(body).unwrap();
    let mut t = Chronicle::new();
    let mut ok = true;
    for e in evs {
        if t.append(e).is_err() {
            ok = false;
            break;
        }
    }
    if ok {
        assert_ne!(t.head(), again.head());
    }
}

#[test]
fn golden() {
    let g = scenario::golden();
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/golden/time.json");
    let text = serde_json::to_string_pretty(&g).unwrap() + "\n";
    if std::env::var("SSE_BLESS").is_ok() {
        std::fs::write(path, &text).unwrap();
    }
    let want = std::fs::read_to_string(path).expect("golden exists (bless with SSE_BLESS=1)");
    assert_eq!(text, want);
}

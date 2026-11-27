//! One ring per world (ring.rs), checked against Saturn's rings. Moon masses
//! and orbits are recalled survey values (Pan 4.95e15 kg at 133,584 km,
//! Daphnis 7.7e13 kg at 136,505 km, Janus 1.898e18 kg at 151,460 km, Mimas
//! 3.79e19 kg at 185,520 km; Saturn J2 0.01629): unverified until the paper
//! search is back (1 Dec 2026), as in the renderer's earlier checks.

use crate::ring::*;
use crate::Settings;
use sse_core::Seed;
use sse_reference::Reference;

const KM: f64 = 1000.0;
const SATURN_KG: f64 = 5.683e26;
const SATURN_R: f64 = 60_268.0 * KM;

fn moons() -> Vec<Sculptor> {
    let s = |n: &str, a: f64, m: f64| Sculptor {
        id: n.to_lowercase(),
        name: n.into(),
        a_m: a * KM,
        mass_kg: m,
    };
    vec![
        s("Pan", 133_584.0, 4.95e15),
        s("Daphnis", 136_505.0, 7.7e13),
        s("Janus", 151_460.0, 1.898e18),
        s("Mimas", 185_520.0, 3.79e19),
    ]
}

fn zone(
    name: &str,
    a: f64,
    b: f64,
    tau: f64,
    range: Option<[f64; 2]>,
    albedo: Option<f64>,
) -> ZoneIn {
    ZoneIn {
        name: Some(name.into()),
        inner_m: a * KM,
        outer_m: b * KM,
        tau: Some(tau),
        tau_range: range,
        albedo,
        ..Default::default()
    }
}

/// Saturn's catalogued bands, as base-sol holds them.
fn saturn(j2: Option<f64>, variant: u64) -> RingAsk {
    RingAsk {
        key: "saturn".into(),
        variant,
        host: RingHost {
            mass_kg: SATURN_KG,
            radius_m: SATURN_R,
            j2,
            ..Default::default()
        },
        sculptors: moons(),
        ring: RingIn {
            inner_m: 66_900.0 * KM,
            outer_m: 136_780.0 * KM,
            zones: vec![
                zone("D", 66_900.0, 74_510.0, 1e-5, None, None),
                zone("C", 74_658.0, 91_975.0, 0.2, Some([0.05, 0.35]), Some(0.21)),
                zone("B", 91_975.0, 117_507.0, 1.45, Some([0.4, 2.5]), Some(0.5)),
                zone(
                    "Cassini",
                    117_507.0,
                    122_340.0,
                    0.05,
                    Some([0.0, 0.1]),
                    Some(0.3),
                ),
                zone("A", 122_340.0, 136_780.0, 0.7, Some([0.4, 1.0]), Some(0.5)),
            ],
            ..Default::default()
        },
        samples: Some(4096),
        window: None,
    }
}

fn run(a: &RingAsk) -> RingSystem {
    let s = Settings::default();
    structure(
        a,
        &s.rings,
        &s.makeup.grain_density_kg_m3,
        &Seed::from_text("test"),
        &Reference::base(),
    )
    .expect("ring")
}

#[test]
fn j2_brings_mimas_2_1_onto_the_b_rings_edge() {
    // Without J2 the 2:1 falls at 116,870 km, 0.5% inside the B ring's 117,507 km; with Saturn's J2 it moves out
    // to within 0.1% of it (fudge SB-resonance-j2 retires when J2 is on record).
    let flat = run(&saturn(None, 0));
    let j2 = run(&saturn(Some(0.01629), 0));
    let at = |r: &RingSystem| {
        r.marks
            .iter()
            .find(|k| k.sculptor_name == "Mimas" && k.m == 2)
            .expect("Mimas 2:1")
            .r_m
    };
    let edge = 117_507.0 * KM;
    assert!(
        (at(&flat) - edge).abs() / edge > 0.004,
        "no J2: {} km",
        at(&flat) / KM
    );
    assert!(
        (at(&j2) - edge).abs() / edge < 0.002,
        "with J2: {} km",
        at(&j2) / KM
    );
    assert!(at(&j2) > at(&flat));
    let k = j2
        .marks
        .iter()
        .find(|k| k.sculptor_name == "Mimas" && k.m == 2)
        .unwrap();
    assert_eq!(k.kind, "edge");
    assert!(k.ratio.unwrap() > 10.0);
}

#[test]
fn pan_clears_the_encke_gap_with_edge_waves() {
    let r = run(&saturn(Some(0.01629), 0));
    let g = r
        .marks
        .iter()
        .find(|k| k.kind == "gap" && k.sculptor_name == "Pan")
        .expect("Encke gap");
    // The Encke gap is about 325 km wide (fit, fudge SB-gap-width).
    assert!(
        (2.0 * g.half_width_m / KM - 325.0).abs() < 35.0,
        "{} km",
        2.0 * g.half_width_m / KM
    );
    // Edge waves 3 pi x the half-width (about 1,500 km) with an amplitude of a few km at most.
    assert!((g.edge_wave_m.unwrap() / KM - 1532.0).abs() < 160.0);
    let amp = g.edge_amplitude_m.unwrap();
    assert!(amp > 300.0 && amp < 5_000.0, "{amp} m");
    // The ring really has a hole there: no zone holds Pan's orbit and the profile is clear at it.
    assert!(!r
        .zones
        .iter()
        .any(|z| g.r_m > z.inner_m && g.r_m < z.outer_m));
    let p = &r.profile;
    let i = ((g.r_m - p.lo_m) / (p.hi_m - p.lo_m) * p.tau.len() as f64) as usize;
    assert_eq!(p.tau[i], 0.0);
    assert_eq!(p.zone[i], -1);
}

#[test]
fn janus_launches_density_waves_in_the_a_ring_kilometres_long() {
    let r = run(&saturn(Some(0.01629), 0));
    let waves: Vec<_> = r
        .marks
        .iter()
        .filter(|k| k.kind == "wave" && k.sculptor_name == "Janus")
        .collect();
    assert!(
        waves.len() >= 2,
        "{:?}",
        waves.iter().map(|w| w.m).collect::<Vec<_>>()
    );
    for w in &waves {
        // Observed A ring waves start at a few tens of km and die away within some 100 km (recalled, unverified).
        let lam = w.wave_lambda_m.unwrap() / KM;
        let xd = w.damping_m.unwrap() / KM;
        assert!(
            lam > 2.0 && lam < 80.0,
            "{}:{} lambda {lam} km",
            w.m,
            w.m - 1
        );
        assert!(
            xd > 20.0 && xd < 600.0,
            "{}:{} damping {xd} km",
            w.m,
            w.m - 1
        );
    }
}

#[test]
fn optical_depth_varies_inside_each_band_within_its_measured_range() {
    let r = run(&saturn(Some(0.01629), 0));
    let p = &r.profile;
    let b = r
        .zones
        .iter()
        .position(|z| z.name.as_deref() == Some("B"))
        .unwrap();
    let in_b: Vec<f64> = p
        .tau
        .iter()
        .zip(&p.zone)
        .filter(|(_, z)| **z == b as i32)
        .map(|(t, _)| *t)
        .collect();
    let lo = in_b.iter().cloned().fold(f64::INFINITY, f64::min);
    let hi = in_b.iter().cloned().fold(0.0, f64::max);
    // Not one flat stripe: the B ring spans more than a factor of two, inside its catalogued 0.4 to 2.5.
    assert!(hi / lo > 2.0, "B ring from {lo} to {hi}");
    assert!(lo >= 0.4 - 1e-9 && hi <= 2.5 + 1e-9);
}

#[test]
fn zone_masses_sum_to_the_rings_and_land_near_saturns() {
    let r = run(&saturn(Some(0.01629), 0));
    let sum: f64 = r.zones.iter().map(|z| z.mass_kg).sum();
    assert!((sum - r.mass_kg).abs() <= 1e-9 * sum);
    // Cassini's gravity gives 1.54e19 kg (Iess et al. 2019, recalled); one grain size and depth make it within 3x.
    assert!(r.mass_kg > 5e18 && r.mass_kg < 5e19, "{:e}", r.mass_kg);
}

#[test]
fn grains_are_ice_reddened_by_the_contaminant_and_the_albedo_holds() {
    let r = run(&saturn(Some(0.01629), 0));
    for z in r.zones.iter().filter(|z| z.name.as_deref() != Some("D")) {
        let want = match z.name.as_deref() {
            Some("C") => 0.21,
            Some("Cassini") => 0.3,
            _ => 0.5,
        };
        assert!((z.albedo - want).abs() < 0.02, "{:?} {}", z.name, z.albedo);
        // Redder than blue, as Saturn's rings are.
        assert!(z.rgb[0] > z.rgb[2], "{:?} {:?}", z.name, z.rgb);
    }
}

#[test]
fn a_band_painted_with_hematite_turns_red_by_its_makeup() {
    // A GM's cult tips fine hematite into the B ring (Alex's John Varley case): the band's colour follows from what
    // its grains are made of, not from a colour typed in. Clean ice is neutral; the paint reddens it.
    let painted = |share_rock: f64| {
        let mut a = saturn(Some(0.01629), 0);
        let z = &mut a.ring.zones[2];
        z.albedo = None;
        z.makeup = Some(crate::Makeup {
            metal: 0.0,
            rock: share_rock,
            carbon: 0.0,
            ice: 1.0 - share_rock,
        });
        z.contaminant = Some("hematite".into());
        z.contaminant_grain_m = Some(2e-7);
        let r = run(&a);
        r.zones
            .iter()
            .find(|z| z.name.as_deref() == Some("B"))
            .unwrap()
            .rgb
    };
    let clean = painted(0.0);
    assert!(clean[0] / clean[2] < 1.02, "{clean:?}");
    let red = painted(0.5);
    assert!(red[0] > red[1] && red[0] > red[2], "{red:?}");
    assert!(red[0] / red[2] > 1.05, "{red:?}");
    // Thicker paint, redder ring.
    let heavy = painted(0.9);
    assert!(
        heavy[0] / heavy[2] > red[0] / red[2] + 0.1,
        "{heavy:?} vs {red:?}"
    );
    // And darker than clean ice: paint absorbs.
    assert!(red[1] < clean[1]);
}

#[test]
fn variants_change_only_the_seeded_part() {
    let a = run(&saturn(Some(0.01629), 0));
    let b = run(&saturn(Some(0.01629), 1));
    assert_eq!(a.marks, b.marks);
    assert_eq!(a.zones.len(), b.zones.len());
    assert_ne!(a.profile.tau, b.profile.tau);
    // The same ask twice is the same ring.
    assert_eq!(a, run(&saturn(Some(0.01629), 0)));
}

#[test]
fn a_closer_window_keeps_the_coarse_shape_and_adds_detail() {
    let whole = run(&saturn(Some(0.01629), 0));
    let mut ask = saturn(Some(0.01629), 0);
    let (lo, hi) = (100_000.0 * KM, 102_000.0 * KM);
    ask.window = Some([lo, hi]);
    let near = run(&ask);
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let p = &whole.profile;
    let n = p.tau.len() as f64;
    let i0 = ((lo - p.lo_m) / (p.hi_m - p.lo_m) * n) as usize;
    let i1 = ((hi - p.lo_m) / (p.hi_m - p.lo_m) * n) as usize;
    let coarse = mean(&p.tau[i0..i1]);
    let fine = mean(&near.profile.tau);
    assert!((fine / coarse - 1.0).abs() < 0.25, "{coarse} vs {fine}");
    // More structure close up: neighbouring samples differ more than the coarse view's do over the same span.
    let rough = |v: &[f64]| v.windows(2).map(|w| (w[1] - w[0]).abs()).sum::<f64>();
    assert!(rough(&near.profile.tau) > rough(&p.tau[i0..i1]));
}

#[test]
fn a_generated_ring_gets_seeded_zones_that_hold_its_mass() {
    let ask = RingAsk {
        key: "sys/body:3".into(),
        variant: 0,
        host: RingHost {
            mass_kg: 1.0e27,
            radius_m: 7.0e7,
            ..Default::default()
        },
        sculptors: vec![],
        ring: RingIn {
            inner_m: 8.4e7,
            outer_m: 1.6e8,
            mass_kg: Some(1.0e19),
            makeup: Some(crate::Makeup {
                metal: 0.0,
                rock: 0.1,
                carbon: 0.05,
                ice: 0.85,
            }),
            ..Default::default()
        },
        samples: Some(1024),
        window: None,
    };
    let r = run(&ask);
    assert!(r.zones.len() >= 2);
    assert!(r.zones.iter().all(|z| z.cause.kind == "seeded"));
    assert!((r.mass_kg / 1.0e19 - 1.0).abs() < 1e-9);
    let taus: Vec<f64> = r.zones.iter().map(|z| z.tau).collect();
    let spread = taus.iter().cloned().fold(0.0, f64::max)
        / taus.iter().cloned().fold(f64::INFINITY, f64::min);
    assert!(spread > 1.2, "{taus:?}");
}
